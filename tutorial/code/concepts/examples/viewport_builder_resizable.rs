//! A fixed-size window, like a dialog: it cannot be resized or maximized until the switch is
//! turned on.
//!
//! Run with `cargo run -p concepts --example viewport_builder_resizable`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Viewport builder: resizable")
            .with_inner_size([400.0, 140.0])
            // No resizing by the edges, and no maximize button in the title bar.
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-resizable",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(ResizableDemo { resizable: false }))),
    )
}

struct ResizableDemo {
    resizable: bool,
}

impl eframe::App for ResizableDemo {
    /// Turning resizing on and off while the app runs
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Try to drag an edge of this window.");
            if ui.checkbox(&mut self.resizable, "Resizable").changed() {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Resizable(self.resizable));
                // The title bar buttons are one command, all three at once.
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::EnableButtons {
                        close: true,
                        minimized: true,
                        maximize: self.resizable,
                    });
            }
        });
    }
}
