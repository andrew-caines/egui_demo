//! `App::save` on a timer of your choosing, what is saved, and `App::on_exit`.
//!
//! Needs eframe's `persistence` feature, which this crate turns on only through its own
//! `persistence` feature:
//! `cargo run -p concepts --example app_trait_save_and_exit --features persistence`.

use std::time::Duration;

use eframe::egui;

/// Names the save folder, `%APPDATA%\egui-concepts-save-and-exit`.
const APP_NAME: &str = "egui-concepts-save-and-exit";
const SAVES_KEY: &str = "saves";

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("App trait: save and exit")
            .with_inner_size([460.0, 170.0]),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|_creation_context| Ok(Box::new(SaveApp::default()))),
    )
}

#[derive(Default)]
struct SaveApp {
    /// How many times eframe has called `save` in this run.
    saves: u32,
}

impl eframe::App for SaveApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(format!("Calls to save so far: {}", self.saves));
            ui.label("eframe calls it every 5 seconds, and once more when the window closes.");
        });
        // eframe checks the save timer after each frame, so keep frames coming while idle.
        ui.ctx().request_repaint_after(Duration::from_secs(1));
    }

    /// How often eframe calls `save` while the app runs. The default is 30 seconds.
    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(5)
    }

    /// Called on the timer and when the window closes. Write your state here.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.saves += 1;
        storage.set_string(SAVES_KEY, self.saves.to_string());
        eprintln!("save: call number {}", self.saves);
    }

    /// Leave egui's memory (scroll positions, collapsed headers, the theme choice) out of the
    /// save file. The window position and size are still saved.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    /// Called once, after the last `save`.
    fn on_exit(&mut self) {
        eprintln!("on_exit: after {} saves", self.saves);
    }
}
