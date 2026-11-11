// Release builds on Windows would otherwise open a console window next to the GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result {
    // `NativeOptions` configures the OS window; `viewport` is egui's word for "native window".
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([360.0, 160.0]),
        ..Default::default()
    };

    // The closure runs once, after the window and renderer exist, and returns our app state.
    eframe::run_native(
        "Hello egui",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(HelloApp::default()))),
    )
}

/// All state lives in a plain struct that we own; egui keeps no copy of it.
#[derive(Default)]
struct HelloApp {
    name: String,
    click_count: u32,
}

impl eframe::App for HelloApp {
    /// Called every time the window needs repainting. The whole UI is rebuilt from `self`.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The root `ui` has no margin or background; `CentralPanel` adds both.
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Hello, egui!");

            // `horizontal` lays its children out left-to-right instead of top-down.
            ui.horizontal(|ui| {
                ui.label("Your name:");
                // The widget edits our field in place through the `&mut` borrow.
                ui.text_edit_singleline(&mut self.name);
            });

            // A widget is drawn and its interaction reported in the same call.
            if ui.button("Click me").clicked() {
                self.click_count += 1;
            }

            ui.label(format!(
                "Hello '{}', you clicked {} times.",
                self.name, self.click_count
            ));
        });
    }
}
