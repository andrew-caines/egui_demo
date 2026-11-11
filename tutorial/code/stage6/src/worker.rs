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
use crate::model::{NewTodo, Todo, TodoId, TodoList};
use crate::storage::{DbLocation, Repository, StorageResult};

/// Work the UI can ask for.
#[derive(Debug)]
pub enum Command {
    /// Download todos from `url`. `request_id` comes back with the result so the UI
    /// can tell which request an answer belongs to.
    ImportTodos {
        request_id: u64,
        url: String,
    },
    /// Stop the import that is running, if any.
    CancelImport,

    // Database writes. Each one is answered with an `Event::WriteFinished`.
    InsertTodo(Todo),
    UpdateTodo(Todo),
    DeleteTodo(TodoId),
    DeleteCompletedTodos,
    /// Store the list order: the first ID comes first.
    SetTodoOrder(Vec<TodoId>),
}

/// What the worker reports back. Errors cross the channel as text, because all the
/// UI does with them is show them.
#[derive(Debug)]
pub enum Event {
    ImportFinished {
        request_id: u64,
        result: Result<Vec<NewTodo>, ImportError>,
    },
    /// Sent once, shortly after start-up: the todos read from the database.
    TodosLoaded(Result<Vec<Todo>, String>),
    WriteFinished(Result<(), String>),
}

/// The UI's handle to the worker thread.
pub struct Worker {
    /// `Option` so that `Drop` can close the channel before joining the thread.
    commands: Option<tokio_mpsc::UnboundedSender<Command>>,
    events: std_mpsc::Receiver<Event>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Starts the worker thread, which opens the database at `db_location` and
    /// reports its contents. `wake` is called after every event is queued; the app
    /// passes a closure that asks egui to repaint, so a result is shown even while
    /// the window is idle.
    pub fn spawn(db_location: DbLocation, wake: impl Fn() + Send + Sync + 'static) -> Self {
        // Two different channels, because the two ends live in different worlds: the
        // worker *awaits* commands (tokio), the UI *polls* for events (std).
        let (command_sender, command_receiver) = tokio_mpsc::unbounded_channel();
        let (event_sender, event_receiver) = std_mpsc::channel();

        let thread = thread::Builder::new()
            .name("todo-worker".to_owned())
            .spawn(move || run(db_location, command_receiver, event_sender, Arc::new(wake)))
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
        // once it has handled the commands already queued. Joining then waits for
        // that, so a todo added a moment before quitting is still written.
        self.commands = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Sends events to the UI and wakes it up.
#[derive(Clone)]
struct Reporter {
    events: std_mpsc::Sender<Event>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl Reporter {
    fn report(&self, event: Event) {
        // Sending only fails if the UI has gone away, and then nobody is listening.
        let _ = self.events.send(event);
        (self.wake)();
    }
}

/// The body of the worker thread.
fn run(
    db_location: DbLocation,
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
    let reporter = Reporter { events, wake };

    runtime.block_on(async move {
        let repository = match open_and_load(&db_location).await {
            Ok((repository, todos)) => {
                reporter.report(Event::TodosLoaded(Ok(todos)));
                Some(repository)
            }
            Err(error) => {
                // Without a database the app still runs; it just cannot save.
                reporter.report(Event::TodosLoaded(Err(error.to_string())));
                None
            }
        };
        let mut running_import: Option<JoinHandle<()>> = None;

        // `recv` returns `None` once the `Worker` handle has been dropped *and* the
        // queue is empty, so nothing that was sent is ever skipped.
        while let Some(command) = commands.recv().await {
            match command {
                Command::ImportTodos { request_id, url } => {
                    // Only one import at a time: a new one replaces the old one.
                    if let Some(previous_import) = running_import.take() {
                        previous_import.abort();
                    }

                    let client = client.clone();
                    let reporter = reporter.clone();
                    // Spawning, rather than awaiting here, keeps the loop free to
                    // receive a `CancelImport` while the request is in flight.
                    running_import = Some(tokio::spawn(async move {
                        let result = import::fetch_todos(&client, &url).await;
                        reporter.report(Event::ImportFinished { request_id, result });
                    }));
                }
                Command::CancelImport => {
                    // Aborting drops the request future, which closes the connection.
                    if let Some(running_import) = running_import.take() {
                        running_import.abort();
                    }
                }
                database_write => {
                    // Awaited right here, unlike the import. Writes take milliseconds,
                    // and finishing one before starting the next keeps them in the
                    // order the user made them: an insert can never overtake the
                    // update that follows it.
                    let result = match &repository {
                        Some(repository) => write(repository, database_write)
                            .await
                            .map_err(|error| error.to_string()),
                        None => Err("The database is not open.".to_owned()),
                    };
                    reporter.report(Event::WriteFinished(result));
                }
            }
        }

        if let Some(repository) = repository {
            repository.close().await;
        }
    });
    // Dropping the runtime here cancels any request still in flight, so quitting the
    // app never waits for the network.
}

/// Opens the database and reads the todos. A database that did not exist before is
/// first filled with the sample todos, so a new user does not start on an empty list.
async fn open_and_load(db_location: &DbLocation) -> StorageResult<(Repository, Vec<Todo>)> {
    let is_new_database = match db_location {
        DbLocation::File(path) => {
            if let Some(folder) = path.parent() {
                // SQLite creates the file but not the folders above it.
                std::fs::create_dir_all(folder)?;
            }
            !path.exists()
        }
        DbLocation::InMemory => true,
    };

    let repository = Repository::open(db_location).await?;
    if is_new_database {
        for todo in TodoList::with_sample_todos().iter() {
            repository.insert(todo).await?;
        }
    }

    let todos = repository.load_all().await?;
    Ok((repository, todos))
}

/// Carries out one database command.
async fn write(repository: &Repository, command: Command) -> StorageResult<()> {
    match command {
        Command::InsertTodo(todo) => repository.insert(&todo).await,
        Command::UpdateTodo(todo) => repository.update(&todo).await,
        Command::DeleteTodo(id) => repository.delete(id).await,
        Command::DeleteCompletedTodos => repository.delete_completed().await,
        Command::SetTodoOrder(ordered_ids) => repository.set_order(&ordered_ids).await,
        // The two import commands are handled by the caller and never reach here.
        Command::ImportTodos { .. } | Command::CancelImport => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;
    use crate::model::Priority;

    /// Waits for the worker's next event. `recv_timeout` blocks, which a test may do
    /// and the UI never should.
    fn next_event(worker: &Worker) -> Event {
        worker
            .events
            .recv_timeout(Duration::from_secs(5))
            .expect("the worker answers")
    }

    #[test]
    fn a_new_database_is_seeded_with_the_sample_todos_and_loaded() {
        let worker = Worker::spawn(DbLocation::InMemory, || {});

        let Event::TodosLoaded(Ok(todos)) = next_event(&worker) else {
            panic!("the first event is the loaded todos");
        };

        let titles: Vec<&str> = todos.iter().map(|todo| todo.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Read the egui tutorial",
                "Split the UI into components",
                "Water the plants"
            ]
        );
    }

    #[test]
    fn a_command_produces_an_event_and_a_wake_up() {
        let wake_count = Arc::new(AtomicUsize::new(0));
        let worker = Worker::spawn(DbLocation::InMemory, {
            let wake_count = Arc::clone(&wake_count);
            move || {
                wake_count.fetch_add(1, Ordering::SeqCst);
            }
        });
        assert!(matches!(next_event(&worker), Event::TodosLoaded(Ok(_))));

        // An invalid URL fails without any network access, which keeps this test
        // about the plumbing rather than about HTTP.
        worker.send(Command::ImportTodos {
            request_id: 7,
            url: "not a url".to_owned(),
        });

        let Event::ImportFinished { request_id, result } = next_event(&worker) else {
            panic!("the import is answered");
        };
        assert_eq!(request_id, 7);
        assert_eq!(result, Err(ImportError::InvalidUrl));

        // Dropping the handle joins the thread, so both wake-ups have happened by
        // now: one for the loaded todos and one for the import.
        drop(worker);
        assert_eq!(wake_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn writes_queued_just_before_shutdown_still_reach_the_file() {
        let folder = tempfile::tempdir().unwrap();
        let location = DbLocation::File(folder.path().join("nested").join("todos.db"));
        // Start from the samples so the new ID does not clash with theirs.
        let mut todos = TodoList::with_sample_todos();
        let id = todos
            .add("Written at the last moment", Priority::High)
            .unwrap();
        let todo = todos.get(id).unwrap().clone();

        // Send and drop immediately, without waiting for any answer.
        let worker = Worker::spawn(location.clone(), || {});
        worker.send(Command::InsertTodo(todo.clone()));
        drop(worker);

        // A second worker on the same file plays the part of the next launch.
        let worker = Worker::spawn(location, || {});
        let Event::TodosLoaded(Ok(loaded_todos)) = next_event(&worker) else {
            panic!("the first event is the loaded todos");
        };
        assert_eq!(loaded_todos.last(), Some(&todo));
        // Three sample todos plus ours: the file was seeded once, not twice.
        assert_eq!(loaded_todos.len(), 4);
    }
}
