//! With `run_and_return: true` (the default), `run_native` returns when the window closes and
//! `main` carries on.
//!
//! Run with `cargo run -p concepts --example native_options_run_and_return`, close the window,
//! and read the terminal.

use eframe::egui;

/// Code after `run_native` runs once the window has closed.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Native options: run and return")
            .with_inner_size([420.0, 140.0]),
        // The default. With `false`, eframe ends the process when the window closes and
        // nothing below `run_native` runs.
        run_and_return: true,
        ..Default::default()
    };
    let result = eframe::run_native(
        "egui-concepts-run-and-return",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(ReturnDemo))),
    );
    println!("run_native returned {result:?}; main is still running.");
    result
}

struct ReturnDemo;

impl eframe::App for ReturnDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Close this window, then look at the terminal.");
        });
    }
}
