//! A window that stays above other windows, with a switch to turn that off and on.
//!
//! Run with `cargo run -p concepts --example viewport_builder_always_on_top`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Viewport builder: always on top")
            .with_inner_size([400.0, 140.0])
            // Short for `.with_window_level(egui::WindowLevel::AlwaysOnTop)`.
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-always-on-top",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(OnTopDemo { on_top: true }))),
    )
}

struct OnTopDemo {
    /// egui cannot ask the window for its level, so the app keeps track of it.
    on_top: bool,
}

impl eframe::App for OnTopDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Click another window: this one stays in front of it.");
            if ui.checkbox(&mut self.on_top, "Always on top").changed() {
                let level = if self.on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                };
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
            }
        });
    }
}
