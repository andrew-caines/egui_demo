use std::f32::consts::TAU;

use eframe::egui;

/// A circular progress indicator, drawn entirely with the painter.
pub struct ProgressRing {
    fraction: f32,
    diameter: f32,
}

impl ProgressRing {
    /// `fraction` is the completed share, from 0.0 to 1.0.
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            diameter: 64.0,
        }
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }
}

impl egui::Widget for ProgressRing {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(self.diameter), egui::Sense::hover());

        response.widget_info(|| {
            let percent = (self.fraction * 100.0).round();
            egui::WidgetInfo::labeled(
                egui::Role::ProgressIndicator,
                true,
                format!("{percent}% completed"),
            )
        });

        // egui remembers the previous value under this ID and eases towards the new
        // one. While the animation runs it also requests repaints for us.
        let animated_fraction = ui
            .ctx()
            .animate_value_with_time(response.id, self.fraction, 0.4);

        if ui.is_rect_visible(rect) {
            let visuals = ui.visuals();
            let ring_width = self.diameter * 0.12;
            let radius = (self.diameter - ring_width) / 2.0;
            let center = rect.center();

            // The full circle underneath, as the "empty" track.
            ui.painter().circle_stroke(
                center,
                radius,
                egui::Stroke::new(ring_width, visuals.widgets.inactive.bg_fill),
            );

            // The completed part: a polyline along the arc, starting at 12 o'clock
            // and running clockwise.
            if animated_fraction > 0.0 {
                let segment_count = (64.0 * animated_fraction).ceil().max(2.0) as usize;
                let arc_points = (0..=segment_count)
                    .map(|segment| {
                        let progress = segment as f32 / segment_count as f32;
                        let angle = -TAU / 4.0 + TAU * animated_fraction * progress;
                        center + radius * egui::Vec2::angled(angle)
                    })
                    .collect();
                ui.painter().line(
                    arc_points,
                    egui::Stroke::new(ring_width, visuals.selection.bg_fill),
                );
            }

            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{:.0}%", animated_fraction * 100.0),
                egui::FontId::proportional(self.diameter * 0.24),
                visuals.text_color(),
            );
        }

        response
    }
}
