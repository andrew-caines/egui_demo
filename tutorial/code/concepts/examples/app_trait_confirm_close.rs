//! Asking before closing with unsaved changes, and `App::on_exit`.
//!
//! `cargo run -p concepts --example app_trait_confirm_close`

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("App trait: confirm close")
            .with_inner_size([440.0, 260.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-confirm-close",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(EditorApp::default()))),
    )
}

#[derive(Default)]
struct EditorApp {
    text: String,
    saved_text: String,
    /// The close dialog is open.
    asking: bool,
    /// The user chose to close anyway, so the next close request goes through.
    close_confirmed: bool,
}

impl EditorApp {
    fn has_unsaved_changes(&self) -> bool {
        self.text != self.saved_text
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        // The close button sets this for one frame. Unless the app answers with
        // `CancelClose` in that same frame, the window closes after it.
        let close_requested = ui.input(|input| input.viewport().close_requested());
        if close_requested && self.has_unsaved_changes() && !self.close_confirmed {
            ui.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.asking = true;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.saved_text = self.text.clone();
                }
                ui.label(if self.has_unsaved_changes() {
                    "Unsaved changes"
                } else {
                    "Saved"
                });
            });
            ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .hint_text("Type something, then close the window."),
            );
        });

        if self.asking {
            egui::Modal::new(egui::Id::unique("confirm_close")).show(ui.ctx(), |ui| {
                ui.label("Close without saving?");
                ui.horizontal(|ui| {
                    if ui.button("Close without saving").clicked() {
                        self.close_confirmed = true;
                        // Arrives as a close request next frame, which now goes through.
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Cancel").clicked() {
                        self.asking = false;
                    }
                });
            });
        }
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    /// Called once, after `save`, when the app is really closing. Here it only reports.
    fn on_exit(&mut self) {
        if self.has_unsaved_changes() {
            eprintln!("on_exit: closing without saving");
        } else {
            eprintln!("on_exit: closing, everything saved");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs one frame in which the user has just clicked the window's close button, and
    /// returns the commands the app sent to the window.
    fn close_clicked(app: &mut EditorApp) -> Vec<egui::ViewportCommand> {
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .events
            .push(egui::ViewportEvent::Close);
        let mut output = ctx.run_ui(input, |ui| app.show(ui));
        let commands = output
            .viewport_output
            .remove(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.commands)
            .unwrap_or_default();
        output.drop_without_applying_deltas();
        commands
    }

    #[test]
    fn closes_when_everything_is_saved() {
        let mut app = EditorApp::default();

        let commands = close_clicked(&mut app);

        assert!(!commands.contains(&egui::ViewportCommand::CancelClose));
        assert!(!app.asking);
    }

    #[test]
    fn cancels_the_close_and_asks_when_there_are_unsaved_changes() {
        let mut app = EditorApp {
            text: "draft".to_owned(),
            ..Default::default()
        };

        let commands = close_clicked(&mut app);

        assert!(commands.contains(&egui::ViewportCommand::CancelClose));
        assert!(app.asking);
    }

    #[test]
    fn closes_once_the_user_has_confirmed() {
        let mut app = EditorApp {
            text: "draft".to_owned(),
            close_confirmed: true,
            ..Default::default()
        };

        let commands = close_clicked(&mut app);

        assert!(!commands.contains(&egui::ViewportCommand::CancelClose));
    }
}
