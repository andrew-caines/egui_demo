use eframe::egui;

/// What the dialog wants the app to do this frame.
pub enum ImportOutcome {
    /// Still open; nothing to do.
    Open,
    /// The user dismissed the dialog.
    Closed,
    /// Start downloading from this URL.
    Start(String),
    /// Stop the download that is running.
    Stop,
}

/// Where the import currently stands. The dialog only *displays* this; the app
/// decides when it changes, because the app is what talks to the worker.
#[derive(Debug, PartialEq)]
enum Progress {
    Idle,
    Fetching,
    Failed(String),
}

/// A modal that asks for a URL and shows the progress of the download.
///
/// It knows nothing about HTTP or threads. It reports what the user asked for and
/// is told what happened, which is why it can be tested with no network at all.
pub struct ImportDialog {
    url: String,
    progress: Progress,
    /// True until the first frame has been shown, to focus the URL box once.
    focus_url: bool,
}

impl ImportDialog {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            progress: Progress::Idle,
            focus_url: true,
        }
    }

    pub fn set_fetching(&mut self) {
        self.progress = Progress::Fetching;
    }

    pub fn set_idle(&mut self) {
        self.progress = Progress::Idle;
    }

    pub fn set_failed(&mut self, message: String) {
        self.progress = Progress::Failed(message);
    }

    pub fn is_fetching(&self) -> bool {
        self.progress == Progress::Fetching
    }

    pub fn show(&mut self, ctx: &egui::Context) -> ImportOutcome {
        let fetching = self.is_fetching();
        let can_start = !fetching && !self.url.trim().is_empty();
        let mut start_requested = false;
        let mut stop_requested = false;
        let mut close_requested = false;

        let modal = egui::Modal::new(egui::Id::unique("import_todos_dialog")).show(ctx, |ui| {
            ui.set_width(420.0);
            ui.heading("Import todos");
            ui.add_space(8.0);

            let url_label = ui.label("URL of a JSON list of todos");
            // The URL cannot be changed while a request for it is in flight.
            let url_response = ui
                .add_enabled(
                    !fetching,
                    egui::TextEdit::singleline(&mut self.url).desired_width(f32::INFINITY),
                )
                .labelled_by(url_label.id);
            if std::mem::take(&mut self.focus_url) {
                url_response.request_focus();
            }
            if url_response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                start_requested = true;
            }

            ui.add_space(4.0);
            match &self.progress {
                Progress::Idle => {
                    ui.weak("Each item needs a \"title\" and a \"completed\" field.");
                }
                Progress::Fetching => {
                    ui.horizontal(|ui| {
                        // The spinner animates by asking for a repaint every frame, so
                        // the window stays live for as long as it is on screen.
                        ui.spinner();
                        ui.label("Fetching…");
                    });
                }
                Progress::Failed(message) => {
                    ui.colored_label(ui.visuals().error_fg_color, message);
                }
            }

            ui.separator();

            egui::Sides::new().show(
                ui,
                |_left| {},
                |ui| {
                    if fetching {
                        if ui.button("Stop").clicked() {
                            stop_requested = true;
                        }
                    } else if ui
                        .add_enabled(can_start, egui::Button::new("Import"))
                        .clicked()
                    {
                        start_requested = true;
                    }
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                },
            );
        });

        if start_requested && can_start {
            ImportOutcome::Start(self.url.trim().to_owned())
        } else if stop_requested {
            ImportOutcome::Stop
        } else if close_requested || modal.should_close() {
            ImportOutcome::Closed
        } else {
            ImportOutcome::Open
        }
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    use super::*;

    /// Shows the dialog on its own and records the last thing it asked for.
    fn dialog_harness(dialog: ImportDialog) -> Harness<'static, (ImportDialog, Option<String>)> {
        Harness::builder()
            .with_size(egui::vec2(520.0, 260.0))
            .build_ui_state(
                |ui, (dialog, started_url): &mut (ImportDialog, Option<String>)| {
                    if let ImportOutcome::Start(url) = dialog.show(ui.ctx()) {
                        *started_url = Some(url);
                    }
                },
                (dialog, None),
            )
    }

    #[test]
    fn import_reports_the_trimmed_url() {
        let mut harness = dialog_harness(ImportDialog::new("  http://example.test/todos  "));
        harness.run();

        harness.get_by_label("Import").click();
        harness.run();

        assert_eq!(
            harness.state().1.as_deref(),
            Some("http://example.test/todos")
        );
    }

    #[test]
    fn a_failure_is_shown_and_the_import_button_comes_back() {
        let mut dialog = ImportDialog::new("http://example.test/todos");
        dialog.set_failed("The server answered with HTTP 500.".to_owned());
        let mut harness = dialog_harness(dialog);
        harness.run();

        harness.get_by_label("The server answered with HTTP 500.");
        harness.get_by_label("Import");
        assert!(harness.query_by_label("Stop").is_none());
    }
}
