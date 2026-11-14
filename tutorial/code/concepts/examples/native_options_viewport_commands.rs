//! `NativeOptions` is read once, when the window is created. To change the window after that,
//! send a `ViewportCommand`.
//!
//! Run with `cargo run -p concepts --example native_options_viewport_commands`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        // The starting values. Changing this struct later would do nothing: eframe has
        // already used it.
        viewport: egui::ViewportBuilder::default()
            .with_title("Native options: viewport commands")
            .with_inner_size([460.0, 200.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-viewport-commands",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(CommandsDemo { clicks: 0 }))),
    )
}

struct CommandsDemo {
    clicks: u32,
}

impl eframe::App for CommandsDemo {
    /// Changing the window while the app runs.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("The title and size of this window were set at start-up.");
            if ui.button("Change the title").clicked() {
                self.clicks += 1;
                let title = format!("Title changed {} times", self.clicks);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Title(title));
            }
            ui.horizontal(|ui| {
                if ui.button("Small").clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            360.0, 160.0,
                        )));
                }
                if ui.button("Large").clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            640.0, 320.0,
                        )));
                }
            });
        });
    }
}
