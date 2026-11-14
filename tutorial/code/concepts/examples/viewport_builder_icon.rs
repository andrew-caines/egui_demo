//! The window icon: set at start-up with `with_icon`, changed later with `ViewportCommand::Icon`.
//!
//! Run with `cargo run -p concepts --example viewport_builder_icon`.

use std::sync::Arc;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Viewport builder: icon")
            .with_inner_size([420.0, 160.0])
            // Without this, eframe uses its own icon: a white "e" on black.
            .with_icon(round_icon(RED)),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-icon",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(IconDemo { red: true }))),
    )
}

const RED: egui::Color32 = egui::Color32::from_rgb(200, 40, 40);
const BLUE: egui::Color32 = egui::Color32::from_rgb(40, 90, 200);

/// Width and height of the icon in pixels. egui asks for a multiple of 4.
const ICON_SIZE: u32 = 32;

/// An icon made in code: a disc of one colour on a transparent square. A real app would more
/// often load a PNG with `eframe::icon_data::from_png_bytes(include_bytes!("icon.png"))`.
fn round_icon(colour: egui::Color32) -> egui::IconData {
    let centre = ICON_SIZE as f32 / 2.0;
    let mut rgba = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let distance = egui::vec2(x as f32 + 0.5 - centre, y as f32 + 0.5 - centre).length();
            let alpha = if distance <= centre - 1.0 { 255 } else { 0 };
            // Four bytes per pixel, red, green, blue, alpha, alpha not premultiplied.
            rgba.extend_from_slice(&[colour.r(), colour.g(), colour.b(), alpha]);
        }
    }
    egui::IconData {
        rgba,
        width: ICON_SIZE,
        height: ICON_SIZE,
    }
}

struct IconDemo {
    red: bool,
}

impl eframe::App for IconDemo {
    /// Changing the icon while the app runs.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("The icon in the title bar is a disc drawn in code.");
            if ui.button("Change the icon").clicked() {
                self.red = !self.red;
                let colour = if self.red { RED } else { BLUE };
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(round_icon(
                        colour,
                    )))));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(icon: &egui::IconData, x: u32, y: u32) -> [u8; 4] {
        let start = ((y * icon.width + x) * 4) as usize;
        icon.rgba[start..start + 4].try_into().unwrap()
    }

    #[test]
    fn icon_has_four_bytes_per_pixel() {
        let icon = round_icon(RED);

        assert_eq!(icon.width, ICON_SIZE);
        assert_eq!(icon.height, ICON_SIZE);
        assert_eq!(icon.width % 4, 0, "egui asks for a multiple of 4");
        assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
    }

    #[test]
    fn icon_is_a_coloured_disc_on_transparency() {
        let icon = round_icon(RED);
        let centre = ICON_SIZE / 2;

        assert_eq!(pixel(&icon, centre, centre), [200, 40, 40, 255]);
        assert_eq!(pixel(&icon, 0, 0)[3], 0, "the corners are transparent");
    }

    #[test]
    fn default_icon_data_is_empty() {
        // eframe treats an empty `IconData` as "use the operating system's default icon".
        assert!(egui::IconData::default().is_empty());
    }
}
