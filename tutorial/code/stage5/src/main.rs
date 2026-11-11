#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod import;
mod model;
mod settings;
mod worker;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Todo List")
            .with_inner_size([760.0, 520.0])
            .with_min_inner_size([520.0, 320.0]),
        ..Default::default()
    };

    // The first argument names the storage folder; the window title is set above.
    eframe::run_native(
        app::APP_ID,
        native_options,
        Box::new(|creation_context| Ok(Box::new(app::TodoApp::new(creation_context)))),
    )
}
