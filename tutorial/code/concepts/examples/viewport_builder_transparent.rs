//! A transparent window with no decorations: only what egui paints is visible, here a rounded
//! card.
//!
//! Run with `cargo run -p concepts --example viewport_builder_transparent`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Viewport builder: transparent")
            .with_inner_size([380.0, 180.0])
            // Both are needed for a window of any shape: no border, and no background.
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-transparent",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(TransparentDemo))),
    )
}

struct TransparentDemo;

impl eframe::App for TransparentDemo {
    /// The colour eframe clears the window to before each frame
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // eframe's default is a dark grey that is partly see-through. Clear to nothing instead.
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The usual central panel would paint the whole window. Paint a rounded card instead,
        // and leave the corners outside it empty.
        let card = egui::Frame::NONE
            .fill(ui.visuals().window_fill())
            .stroke(ui.visuals().window_stroke())
            .corner_radius(24.0)
            .inner_margin(20.0);
        egui::CentralPanel::no_frame().show(ui, |ui| {
            card.show(ui, |ui| {
                // The card is the drag region: there is no title bar to move the window by.
                let background = ui.interact(
                    ui.max_rect(),
                    ui.scope_id().with("drag_region"),
                    egui::Sense::drag(),
                );
                if background.drag_started_by(egui::PointerButton::Primary) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.heading("A window with rounded corners");
                ui.label("Drag the card to move it.");
                if ui.button("Close").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                // Stretch the card to fill the window.
                ui.allocate_space(ui.available_size());
            });
        });
    }
}
