#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod model;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 560.0])
            .with_min_inner_size([360.0, 280.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Todo List",
        native_options,
        Box::new(|creation_context| Ok(Box::new(app::TodoApp::new(creation_context)))),
    )
}
