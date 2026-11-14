//! The everyday `NativeOptions`: a window with a title, a size, a minimum size, centred on the
//! screen.
//!
//! Run with `cargo run -p concepts --example native_options_window`.

use eframe::egui;

/// The usual start of `main`: what the window looks like, then where it opens.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        // Everything about the window itself goes in the `ViewportBuilder`.
        viewport: egui::ViewportBuilder::default()
            .with_title("Native options: window")
            .with_inner_size([480.0, 220.0])
            .with_min_inner_size([320.0, 160.0]),
        // Open in the middle of the primary monitor instead of where the OS puts it.
        centered: true,
        // Everything else keeps eframe's defaults.
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-window",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(WindowDemo))),
    )
}

struct WindowDemo;

impl eframe::App for WindowDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("A 480 by 220 window, centred");
            ui.label("Try to shrink it: it stops at 320 by 160 points.");
            // The size the window has now, in points (not physical pixels).
            let size = ui.ctx().viewport_rect().size();
            ui.label(format!(
                "Inner size now: {:.0} x {:.0} points",
                size.x, size.y
            ));
        });
    }
}
