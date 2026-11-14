//! `App::raw_input_hook`: change or drop input before egui sees it.
//!
//! `cargo run -p concepts --example app_trait_input_hook`

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("App trait: input hook")
            .with_inner_size([440.0, 200.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-input-hook",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(HookApp::default()))),
    )
}

#[derive(Default)]
struct HookApp {
    text: String,
    read_only: bool,
}

/// Removes typed text and pasted text, so no text box can change. Keys such as the arrows
/// still arrive, so the cursor can still move.
fn block_text_input(raw_input: &mut egui::RawInput) {
    raw_input
        .events
        .retain(|event| !matches!(event, egui::Event::Text(_) | egui::Event::Paste(_)));
}

impl eframe::App for HookApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if self.read_only {
            block_text_input(raw_input);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.checkbox(&mut self.read_only, "Block typing and pasting");
            ui.add(egui::TextEdit::singleline(&mut self.text).hint_text("Type here"));
            ui.label(format!("{} characters", self.text.chars().count()));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_and_pasting_are_removed_and_other_keys_kept() {
        let arrow = egui::Event::Key {
            key: egui::Key::ArrowLeft,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        let mut raw_input = egui::RawInput {
            events: vec![
                egui::Event::Text("a".to_owned()),
                arrow.clone(),
                egui::Event::Paste("pasted".to_owned()),
            ],
            ..Default::default()
        };

        block_text_input(&mut raw_input);

        assert_eq!(raw_input.events, vec![arrow]);
    }
}
