//! The everyday `ViewportBuilder`: a title, a starting size, limits on resizing and a starting
//! position.
//!
//! Run with `cargo run -p concepts --example viewport_builder_size`.

use eframe::egui;

/// Size and place the window
fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Viewport builder: size")
        // The area your `ui` draws in, in points. The title bar and border are extra.
        .with_inner_size([460.0, 220.0])
        // The user can resize the window, but only between these two sizes.
        .with_min_inner_size([340.0, 160.0])
        .with_max_inner_size([800.0, 400.0])
        // The top-left corner of the whole window, title bar included, on the screen.
        .with_position([120.0, 120.0]);
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-size",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(SizeDemo))),
    )
}

struct SizeDemo;

impl eframe::App for SizeDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Resize me: 340 x 160 up to 800 x 400");
            // What the window is now, as the operating system reports it, in points.
            let (inner, outer) = ui.input(|input| {
                let viewport = input.viewport();
                (viewport.inner_rect, viewport.outer_rect)
            });
            if let Some(inner) = inner {
                ui.label(format!(
                    "Inner size: {:.0} x {:.0}",
                    inner.width(),
                    inner.height()
                ));
            }
            if let Some(outer) = outer {
                ui.label(format!(
                    "Outer size: {:.0} x {:.0}, top-left corner at {:.0}, {:.0}",
                    outer.width(),
                    outer.height(),
                    outer.min.x,
                    outer.min.y
                ));
            }
        });
    }
}
