//! `App::clear_color`: the colour eframe fills the window with before drawing the UI. It is
//! what shows wherever no panel paints a background.
//!
//! `cargo run -p concepts --example app_trait_clear_color`

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("App trait: clear colour")
            .with_inner_size([420.0, 150.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-clear-color",
        native_options,
        Box::new(|creation_context| {
            // Light text on the dark colours below, whatever the system theme.
            creation_context.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(ClearColorApp::default()))
        }),
    )
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Background {
    EframeDefault,
    ThemePanel,
    #[default]
    Fixed,
}

#[derive(Default)]
struct ClearColorApp {
    background: Background,
}

impl eframe::App for ClearColorApp {
    /// Read every frame, after `ui`, so a choice made in `ui` shows in the same frame.
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        let colour = match self.background {
            // What the trait returns if you do not override it.
            Background::EframeDefault => egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180),
            // Follows the theme, like a `CentralPanel` background.
            Background::ThemePanel => visuals.panel_fill,
            Background::Fixed => egui::Color32::from_rgb(20, 40, 80),
        };
        // The renderer takes gamma-space values from 0 to 1.
        colour.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // No `CentralPanel`: nothing paints a background, so the clear colour shows.
        ui.label("The window behind this text is the clear colour.");
        ui.radio_value(
            &mut self.background,
            Background::EframeDefault,
            "eframe's default",
        );
        ui.radio_value(
            &mut self.background,
            Background::ThemePanel,
            "The theme's panel colour",
        );
        ui.radio_value(&mut self.background, Background::Fixed, "A fixed blue");
    }
}
