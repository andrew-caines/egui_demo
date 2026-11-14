//! Fonts, style and theme, set once in the creator, before the first frame is drawn.
//!
//! Run with `cargo run -p concepts --example run_native_style`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("run_native: fonts, style and theme")
            .with_inner_size([460.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-run-native-style",
        native_options,
        Box::new(|creation_context| {
            set_up_look(&creation_context.egui_ctx);
            Ok(Box::new(StyleDemo { checked: true }))
        }),
    )
}

/// Everything about the look, done once on the context. The first frame already uses it.
fn set_up_look(ctx: &egui::Context) {
    // Fonts: put the bundled monospace font first for ordinary text too.
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Hack".to_owned());
    ctx.set_fonts(fonts);

    // Style: larger text and more space between widgets, in both the dark and light style.
    ctx.all_styles_mut(|style| {
        for font_id in style.text_styles.values_mut() {
            font_id.size *= 1.2;
        }
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    });

    // Theme: light, whatever the operating system uses.
    ctx.set_theme(egui::Theme::Light);
}

struct StyleDemo {
    checked: bool,
}

impl eframe::App for StyleDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Set up in the creator");
            ui.label("Monospace text, 20 % larger, wider spacing, light theme.");
            ui.checkbox(&mut self.checked, "Every widget uses it");
            let _ = ui.button("No per-frame code needed");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_is_light() {
        let ctx = egui::Context::default();
        set_up_look(&ctx);

        assert_eq!(ctx.theme(), egui::Theme::Light);
    }

    #[test]
    fn both_styles_get_the_larger_text() {
        let default_size = egui::Style::default().text_styles[&egui::TextStyle::Body].size;
        let ctx = egui::Context::default();
        set_up_look(&ctx);

        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let size = ctx.style_of(theme).text_styles[&egui::TextStyle::Body].size;
            assert_eq!(size, default_size * 1.2);
        }
    }

    #[test]
    fn monospace_font_comes_first_for_ordinary_text() {
        let ctx = egui::Context::default();
        set_up_look(&ctx);
        // Font changes are applied at the start of a pass. A headless pass has no renderer, so
        // drop the font texture it uploads.
        let mut output = ctx.run_ui(egui::RawInput::default(), |_ui| {});
        output.textures_delta.clear();

        let first = ctx.fonts(|fonts| {
            fonts.definitions().families[&egui::FontFamily::Proportional][0].clone()
        });
        assert_eq!(first, "Hack");
    }
}
