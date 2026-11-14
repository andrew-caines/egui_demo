//! `run_ui_native`: no creator and no `App` type, only a closure called every frame. With no
//! title set, the app name becomes the window title.
//!
//! Run with `cargo run -p concepts --example run_native_ui_only`.

use eframe::egui;

/// The closure owns the state; `move` takes it in.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        // No `with_title`: the title bar shows the app name.
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 150.0]),
        ..Default::default()
    };
    let mut clicks = 0;
    eframe::run_ui_native(
        "egui-concepts-ui-only",
        native_options,
        move |ui, _frame| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.heading("No creator, no App type");
                if ui.button("Click").clicked() {
                    clicks += 1;
                }
                ui.label(format!("Clicked {clicks} times"));
            });
        },
    )
}
