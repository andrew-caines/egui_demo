use eframe::egui;

use crate::settings::Settings;

/// A floating, non-modal settings window.
///
/// Unlike the edit dialog it does not block the rest of the app, and changes take
/// effect immediately, so the user can watch the list restyle as they adjust.
pub struct SettingsWindow<'a> {
    settings: &'a mut Settings,
    open: &'a mut bool,
    /// Where the app's data is saved, shown for reference.
    storage_path: Option<std::path::PathBuf>,
}

impl<'a> SettingsWindow<'a> {
    pub fn new(settings: &'a mut Settings, open: &'a mut bool) -> Self {
        Self {
            settings,
            open,
            storage_path: None,
        }
    }

    pub fn storage_path(mut self, storage_path: Option<std::path::PathBuf>) -> Self {
        self.storage_path = storage_path;
        self
    }

    /// Returns `true` if any setting changed this frame.
    pub fn show(self, ctx: &egui::Context) -> bool {
        // Comparing before and after is simpler and more robust than collecting
        // `changed()` from every widget, and it also catches the reset button.
        let settings_before = self.settings.clone();

        egui::Window::new("Settings")
            // Adds a close button to the title bar and clears the flag when it is used.
            .open(self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("settings_fields")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Theme");
                        theme_picker(ui);
                        ui.end_row();

                        let accent_label = ui.label("Accent colour");
                        ui.color_edit_button_srgb(&mut self.settings.accent_color)
                            .labelled_by(accent_label.id);
                        ui.end_row();

                        let spacing_label = ui.label("Row spacing");
                        ui.add(
                            egui::Slider::new(
                                &mut self.settings.row_spacing,
                                Settings::ROW_SPACING_RANGE,
                            )
                            .step_by(1.0)
                            .suffix(" pt"),
                        )
                        .labelled_by(spacing_label.id);
                        ui.end_row();
                    });

                ui.add_space(4.0);
                // Enabled only while the value differs from `Settings::default()`.
                egui::reset_button(ui, self.settings, "Reset to defaults");

                ui.separator();

                // Collapsed by default; egui remembers whether the user opened it.
                ui.collapsing("Storage", |ui| {
                    ui.label("Todos and settings are saved automatically to:");
                    match &self.storage_path {
                        Some(path) => ui.monospace(path.display().to_string()),
                        None => ui.weak("(no storage location available)"),
                    };
                });
            });

        *self.settings != settings_before
    }
}

/// A System / Light / Dark switch.
///
/// The theme preference is not part of our `Settings`: egui already owns it,
/// applies it, and saves it with its own memory, so we only read and write it.
fn theme_picker(ui: &mut egui::Ui) {
    let current_preference = ui.ctx().options(|options| options.theme_preference);
    let mut chosen_preference = current_preference;

    ui.horizontal(|ui| {
        for (preference, label) in [
            (egui::ThemePreference::System, "System"),
            (egui::ThemePreference::Light, "Light"),
            (egui::ThemePreference::Dark, "Dark"),
        ] {
            ui.selectable_value(&mut chosen_preference, preference, label);
        }
    });

    if chosen_preference != current_preference {
        ui.ctx().set_theme(chosen_preference);
    }
}
