use eframe::egui;

use crate::model::Priority;

/// A small coloured pill showing a priority.
///
/// This is a *leaf* widget: it draws itself, reports a `Response`, and changes
/// nothing. Implementing `egui::Widget` lets callers write `ui.add(PriorityBadge::new(..))`
/// exactly as they would for a built-in widget.
pub struct PriorityBadge {
    priority: Priority,
}

impl PriorityBadge {
    pub fn new(priority: Priority) -> Self {
        Self { priority }
    }

    /// Chosen per theme so the badge stays readable on both light and dark backgrounds.
    fn accent_color(&self, dark_mode: bool) -> egui::Color32 {
        match (self.priority, dark_mode) {
            (Priority::Low, true) => egui::Color32::from_rgb(120, 190, 130),
            (Priority::Low, false) => egui::Color32::from_rgb(40, 120, 60),
            (Priority::Medium, true) => egui::Color32::from_rgb(230, 190, 90),
            (Priority::Medium, false) => egui::Color32::from_rgb(150, 100, 0),
            (Priority::High, true) => egui::Color32::from_rgb(240, 120, 110),
            (Priority::High, false) => egui::Color32::from_rgb(180, 40, 40),
        }
    }
}

impl egui::Widget for PriorityBadge {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let label = self.priority.label();
        let accent_color = self.accent_color(ui.visuals().dark_mode);
        let padding = egui::vec2(6.0, 2.0);

        // 1. Measure. Lay the text out first so we know how much space to ask for.
        let font_id = egui::TextStyle::Small.resolve(ui.style());
        let galley = ui
            .painter()
            .layout_no_wrap(label.to_owned(), font_id, accent_color);

        // 2. Allocate. This reserves the rectangle, advances the layout cursor and
        //    tells us how the pointer is interacting with that rectangle.
        let (rect, response) =
            ui.allocate_exact_size(galley.size() + 2.0 * padding, egui::Sense::hover());

        // Tell screen readers (and UI tests) what this rectangle of pixels means.
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Label, true, format!("{label} priority"))
        });

        // 3. Paint. Skipped when the widget is scrolled out of view.
        if ui.is_rect_visible(rect) {
            let corner_radius = rect.height() / 2.0;
            ui.painter()
                .rect_filled(rect, corner_radius, accent_color.gamma_multiply(0.2));
            ui.painter()
                .galley(rect.min + padding, galley, accent_color);
        }

        response
    }
}
