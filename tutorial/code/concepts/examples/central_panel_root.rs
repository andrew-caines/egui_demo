//! The root `Ui` with and without a `CentralPanel`.
//!
//! This is a whole program rather than a gallery example because the difference only shows at
//! the root of a real window: inside the gallery, the gallery's own central panel has already
//! painted a background behind every example.
//!
//! Run with `cargo run -p concepts --example central_panel_root`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 160.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Root Ui without a CentralPanel",
        native_options,
        Box::new(|creation_context| {
            // In the light theme the missing background is obvious: dark text on the dark
            // colour eframe clears the window to.
            creation_context.egui_ctx.set_theme(egui::Theme::Light);
            Ok(Box::new(RootDemo::default()))
        }),
    )
}

#[derive(Default)]
struct RootDemo {
    use_central_panel: bool,
}

impl eframe::App for RootDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.use_central_panel {
            egui::CentralPanel::default().show(ui, |ui| self.contents(ui));
        } else {
            // Straight into the root `Ui`: no margin, and no background behind the text.
            self.contents(ui);
        }
    }
}

impl RootDemo {
    fn contents(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hello from the root Ui");
        ui.label("Without a CentralPanel the text touches the window edge.");
        ui.checkbox(&mut self.use_central_panel, "Wrap in a CentralPanel");
    }
}
