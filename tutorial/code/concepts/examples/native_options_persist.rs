//! Persistence: eframe saves the window's position and size, and anything the app writes in
//! `App::save`, under a folder named after the app.
//!
//! Needs eframe's `persistence` feature, which this crate turns on only through its own
//! `persistence` feature:
//! `cargo run -p concepts --example native_options_persist --features persistence`.

use eframe::egui;

/// The save folder is named after this, so it must not change between releases.
const APP_NAME: &str = "egui-concepts-persist";
const LAUNCHES_KEY: &str = "launches";

/// The window settings are saved for you; your own state you save in `App::save`.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Native options: persistence")
            .with_inner_size([520.0, 180.0]),
        // The default: remember where the window was and how big.
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|creation_context| {
            // `storage` is `None` without the `persistence` feature.
            let stored = creation_context
                .storage
                .and_then(|storage| storage.get_string(LAUNCHES_KEY));
            Ok(Box::new(PersistDemo {
                launches: next_launch_count(stored.as_deref()),
            }))
        }),
    )
}

/// One more than the stored count; a missing or unreadable value counts as no launches yet.
fn next_launch_count(stored: Option<&str>) -> u32 {
    let previous: u32 = stored.and_then(|text| text.parse().ok()).unwrap_or(0);
    previous + 1
}

/// Shows `%APPDATA%` in place of the user's own folder on Windows, so screenshots do not carry
/// a user name.
fn shorten_home(folder: &std::path::Path) -> String {
    let appdata = std::env::var_os("APPDATA").map(std::path::PathBuf::from);
    match appdata.and_then(|appdata| {
        folder
            .strip_prefix(appdata)
            .ok()
            .map(|rest| rest.to_owned())
    }) {
        Some(rest) => format!("%APPDATA%\\{}", rest.display()),
        None => folder.display().to_string(),
    }
}

struct PersistDemo {
    launches: u32,
}

impl eframe::App for PersistDemo {
    /// Called when the window closes, and every 30 seconds by default.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(LAUNCHES_KEY, self.launches.to_string());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(format!("Launch number {}", self.launches));
            ui.label("Move or resize the window, close it, run it again.");
            if let Some(folder) = eframe::storage_dir(APP_NAME) {
                ui.label(format!("Saved in {}", shorten_home(&folder)));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::next_launch_count;

    #[test]
    fn first_launch_is_number_one() {
        assert_eq!(next_launch_count(None), 1);
    }

    #[test]
    fn counts_on_from_the_stored_value() {
        assert_eq!(next_launch_count(Some("4")), 5);
    }

    #[test]
    fn unreadable_value_starts_again() {
        assert_eq!(next_launch_count(Some("not a number")), 1);
    }
}
