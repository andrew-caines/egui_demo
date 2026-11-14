// Release builds on Windows would otherwise open a console window next to the GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use concepts::gallery::Gallery;
use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([820.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui concepts",
        native_options,
        Box::new(|_creation_context| {
            let mut app = App::default();
            // `cargo run -p concepts -- "Panel order"` opens the gallery at that example.
            if let Some(name) = std::env::args().nth(1) {
                app.gallery.select(&name);
            }
            Ok(Box::new(app))
        }),
    )
}

#[derive(Default)]
struct App {
    gallery: Gallery,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.gallery.show(ui);
    }
}
