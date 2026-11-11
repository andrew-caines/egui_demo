use eframe::egui;
use serde::{Deserialize, Serialize};

/// User preferences that change how the app looks.
///
/// `#[serde(default)]` fills in any field missing from an older save file with its
/// default, so adding a setting later never invalidates what users already saved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Colour of selections, the progress ring and links, as sRGB bytes.
    pub accent_color: [u8; 3],
    /// Vertical gap between neighbouring widgets, in points.
    pub row_spacing: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            accent_color: [60, 130, 200],
            row_spacing: 4.0,
        }
    }
}

impl Settings {
    pub const ROW_SPACING_RANGE: std::ops::RangeInclusive<f32> = 2.0..=12.0;

    /// Pushes the settings into egui's style. Call once at startup and again
    /// whenever a setting changes; the style then stays in effect for every frame.
    pub fn apply(&self, ctx: &egui::Context) {
        let [red, green, blue] = self.accent_color;
        let accent_color = egui::Color32::from_rgb(red, green, blue);
        let text_on_accent = readable_text_color(self.accent_color);

        // egui keeps one style for the light theme and one for the dark theme.
        // `all_styles_mut` edits both, so the settings survive a theme switch.
        ctx.all_styles_mut(|style| {
            style.visuals.selection.bg_fill = accent_color;
            style.visuals.selection.stroke.color = text_on_accent;
            style.visuals.hyperlink_color = accent_color;
            style.spacing.item_spacing.y = self.row_spacing;
        });
    }
}

/// Picks black or white, whichever contrasts better with the given background.
fn readable_text_color([red, green, blue]: [u8; 3]) -> egui::Color32 {
    // Perceived brightness: the eye is most sensitive to green and least to blue.
    let brightness = 0.299 * f32::from(red) + 0.587 * f32::from(green) + 0.114 * f32::from(blue);
    if brightness > 150.0 {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_on_the_accent_colour_stays_readable() {
        assert_eq!(readable_text_color([250, 240, 120]), egui::Color32::BLACK);
        assert_eq!(readable_text_color([20, 40, 120]), egui::Color32::WHITE);
    }

    #[test]
    fn apply_writes_the_settings_into_both_themes() {
        let ctx = egui::Context::default();
        let settings = Settings {
            accent_color: [200, 30, 30],
            row_spacing: 9.0,
        };

        settings.apply(&ctx);

        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let style = ctx.style_of(theme);
            assert_eq!(
                style.visuals.selection.bg_fill,
                egui::Color32::from_rgb(200, 30, 30)
            );
            assert_eq!(style.spacing.item_spacing.y, 9.0);
        }
    }
}
