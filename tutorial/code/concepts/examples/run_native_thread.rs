//! `run_native` keeps the main thread until the window closes. Work that should run alongside
//! the UI goes on another thread, started in the creator with a clone of the context.
//!
//! Run with `cargo run -p concepts --example run_native_thread`, close the window, and read
//! the terminal.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use eframe::egui;

/// A background thread that ticks once a second and asks for a repaint.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("run_native: a background thread")
            .with_inner_size([440.0, 150.0]),
        ..Default::default()
    };
    println!("main: calling run_native");
    let result = eframe::run_native(
        "egui-concepts-run-native-thread",
        native_options,
        Box::new(|creation_context| {
            let seconds = Arc::new(AtomicU32::new(0));
            let ctx = creation_context.egui_ctx.clone();
            let thread_seconds = Arc::clone(&seconds);
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    thread_seconds.fetch_add(1, Ordering::Relaxed);
                    // Without this, nothing is redrawn until the next mouse move or key.
                    ctx.request_repaint();
                }
            });
            Ok(Box::new(ThreadDemo { seconds }))
        }),
    );
    // Only reached once the window has closed.
    println!("main: run_native returned");
    result
}

struct ThreadDemo {
    seconds: Arc<AtomicU32>,
}

impl eframe::App for ThreadDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let seconds = self.seconds.load(Ordering::Relaxed);
            ui.heading(format!("The thread has counted {seconds} seconds"));
            ui.label("The main thread is inside run_native, drawing this window.");
        });
    }
}
