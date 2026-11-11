//! The background worker: one thread that owns an async runtime and does every job
//! that would otherwise make the UI wait.
//!
//! The UI thread and the worker never share state. They exchange messages instead:
//! [`Command`]s go in, [`Event`]s come out. That keeps locks out of the UI code and
//! makes the worker testable without a window.

use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::thread;

use tokio::sync::mpsc as tokio_mpsc;
use tokio::task::JoinHandle;

use crate::import::{self, ImportError};
use crate::model::NewTodo;

/// Work the UI can ask for.
#[derive(Debug)]
pub enum Command {
    /// Download todos from `url`. `request_id` comes back with the result so the UI
    /// can tell which request an answer belongs to.
    ImportTodos { request_id: u64, url: String },
    /// Stop the import that is running, if any.
    CancelImport,
}

/// What the worker reports back.
#[derive(Debug)]
pub enum Event {
    ImportFinished {
        request_id: u64,
        result: Result<Vec<NewTodo>, ImportError>,
    },
}

/// The UI's handle to the worker thread.
pub struct Worker {
    /// `Option` so that `Drop` can close the channel before joining the thread.
    commands: Option<tokio_mpsc::UnboundedSender<Command>>,
    events: std_mpsc::Receiver<Event>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Starts the worker thread. `wake` is called after every event is queued; the
    /// app passes a closure that asks egui to repaint, so a result is shown even
    /// while the window is idle.
    pub fn spawn(wake: impl Fn() + Send + Sync + 'static) -> Self {
        // Two different channels, because the two ends live in different worlds: the
        // worker *awaits* commands (tokio), the UI *polls* for events (std).
        let (command_sender, command_receiver) = tokio_mpsc::unbounded_channel();
        let (event_sender, event_receiver) = std_mpsc::channel();

        let thread = thread::Builder::new()
            .name("todo-worker".to_owned())
            .spawn(move || run(command_receiver, event_sender, Arc::new(wake)))
            .expect("the operating system refused to start a thread");

        Self {
            commands: Some(command_sender),
            events: event_receiver,
            thread: Some(thread),
        }
    }

    /// Queues a command. Never blocks, so it is safe to call while drawing.
    pub fn send(&self, command: Command) {
        if let Some(commands) = &self.commands {
            // Sending only fails if the worker thread has died, and then there is
            // nobody left to do the work anyway.
            let _ = commands.send(command);
        }
    }

    /// Returns the events that have arrived since the last call, without waiting.
    pub fn poll(&self) -> Vec<Event> {
        self.events.try_iter().collect()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Closing the command channel is the shutdown signal: the worker's loop ends
        // once it has handled the commands already queued.
        self.commands = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The body of the worker thread.
fn run(
    mut commands: tokio_mpsc::UnboundedReceiver<Command>,
    events: std_mpsc::Sender<Event>,
    wake: Arc<dyn Fn() + Send + Sync>,
) {
    // A current-thread runtime runs every task on this one thread. That is plenty for
    // I/O-bound work and avoids starting a thread pool the app does not need.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to start the async runtime");
    let client =
        import::build_client(import::REQUEST_TIMEOUT).expect("failed to build the HTTP client");

    runtime.block_on(async move {
        let mut running_import: Option<JoinHandle<()>> = None;

        // `recv` returns `None` once the `Worker` handle has been dropped.
        while let Some(command) = commands.recv().await {
            match command {
                Command::ImportTodos { request_id, url } => {
                    // Only one import at a time: a new one replaces the old one.
                    if let Some(previous_import) = running_import.take() {
                        previous_import.abort();
                    }

                    let client = client.clone();
                    let events = events.clone();
                    let wake = Arc::clone(&wake);
                    // Spawning, rather than awaiting here, keeps the loop free to
                    // receive a `CancelImport` while the request is in flight.
                    running_import = Some(tokio::spawn(async move {
                        let result = import::fetch_todos(&client, &url).await;
                        let _ = events.send(Event::ImportFinished { request_id, result });
                        wake();
                    }));
                }
                Command::CancelImport => {
                    // Aborting drops the request future, which closes the connection.
                    if let Some(running_import) = running_import.take() {
                        running_import.abort();
                    }
                }
            }
        }
    });
    // Dropping the runtime here cancels any request still in flight, so quitting the
    // app never waits for the network.
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;

    #[test]
    fn a_command_produces_an_event_and_a_wake_up() {
        let wake_count = Arc::new(AtomicUsize::new(0));
        let worker = Worker::spawn({
            let wake_count = Arc::clone(&wake_count);
            move || {
                wake_count.fetch_add(1, Ordering::SeqCst);
            }
        });

        // An invalid URL fails without any network access, which keeps this test
        // about the plumbing rather than about HTTP.
        worker.send(Command::ImportTodos {
            request_id: 7,
            url: "not a url".to_owned(),
        });

        // `recv_timeout` blocks, which a test may do and the UI never should.
        let event = worker
            .events
            .recv_timeout(Duration::from_secs(5))
            .expect("the worker answers");
        let Event::ImportFinished { request_id, result } = event;
        assert_eq!(request_id, 7);
        assert_eq!(result, Err(ImportError::InvalidUrl));

        // Dropping the handle joins the thread, so the wake-up has happened by now.
        drop(worker);
        assert_eq!(wake_count.load(Ordering::SeqCst), 1);
    }
}
