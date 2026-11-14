//! Restoring state in the creator: read what `App::save` wrote last time, before the first
//! frame, so the window opens with it.
//!
//! Needs eframe's `persistence` feature:
//! `cargo run -p concepts --example run_native_restore --features persistence`.

use eframe::egui;

/// Also the name of the save folder.
const APP_NAME: &str = "egui-concepts-run-native-restore";
const DRAFT_KEY: &str = "draft";

/// Restore in the creator, save in `App::save`.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("run_native: restoring state")
            .with_inner_size([460.0, 180.0]),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|creation_context| {
            Ok(Box::new(DraftApp {
                draft: restore_draft(creation_context.storage),
            }))
        }),
    )
}

/// The saved draft, or an empty one on the first run (or without persistence).
fn restore_draft(storage: Option<&dyn eframe::Storage>) -> String {
    storage
        .and_then(|storage| storage.get_string(DRAFT_KEY))
        .unwrap_or_default()
}

struct DraftApp {
    draft: String,
}

impl eframe::App for DraftApp {
    /// Called when the window closes, and every 30 seconds.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(DRAFT_KEY, self.draft.clone());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Type, close, run again");
            let label = ui.label("Draft:");
            ui.text_edit_multiline(&mut self.draft)
                .labelled_by(label.id);
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// Stands in for eframe's file storage.
    #[derive(Default)]
    struct MemoryStorage(HashMap<String, String>);

    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_owned(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }

    #[test]
    fn no_storage_means_an_empty_draft() {
        assert_eq!(restore_draft(None), "");
    }

    #[test]
    fn nothing_saved_yet_means_an_empty_draft() {
        let storage = MemoryStorage::default();

        assert_eq!(restore_draft(Some(&storage)), "");
    }

    #[test]
    fn what_save_wrote_is_restored() {
        let mut storage = MemoryStorage::default();
        let mut app = DraftApp {
            draft: "buy milk".to_owned(),
        };
        eframe::App::save(&mut app, &mut storage);

        assert_eq!(restore_draft(Some(&storage)), "buy milk");
    }
}
