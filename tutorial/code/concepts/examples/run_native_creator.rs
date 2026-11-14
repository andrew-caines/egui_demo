//! The usual shape: `main` builds the options, and the creator closure hands the
//! `CreationContext` to the app's own `new`.
//!
//! Run with `cargo run -p concepts --example run_native_creator`.

use eframe::egui;

/// The creator closure only forwards to `StartupApp::new`.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("run_native: the creator")
            .with_inner_size([460.0, 200.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-run-native",
        native_options,
        Box::new(|creation_context| Ok(Box::new(StartupApp::new(creation_context)))),
    )
}

/// What the creator found, kept so the window can show it.
struct StartupApp {
    has_storage: bool,
    has_wgpu: bool,
    has_window: bool,
    pixels_per_point: f32,
}

impl StartupApp {
    /// Runs once: after the window and the renderer exist, before the first frame.
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        Self {
            // `None` unless eframe's `persistence` feature is on.
            has_storage: creation_context.storage.is_some(),
            // The GPU device and queue, for your own wgpu drawing.
            has_wgpu: creation_context.wgpu_render_state.is_some(),
            // The window exists already, still hidden.
            has_window: creation_context.winit_window().is_some(),
            // Already known, so sizes can be worked out before the first frame.
            pixels_per_point: creation_context.egui_ctx.pixels_per_point(),
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

impl eframe::App for StartupApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("What the creator was given");
            ui.label(format!("Storage: {}", yes_no(self.has_storage)));
            ui.label(format!("wgpu render state: {}", yes_no(self.has_wgpu)));
            ui.label(format!("Window: {}", yes_no(self.has_window)));
            ui.label(format!("Pixels per point: {}", self.pixels_per_point));
        });
    }
}
