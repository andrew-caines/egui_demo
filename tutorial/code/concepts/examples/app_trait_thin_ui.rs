//! The tutorial's pattern: `App::ui` only forwards to the app's own `show(ui)`, which a test
//! can drive headlessly without a window.
//!
//! `cargo run -p concepts --example app_trait_thin_ui`

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("App trait: thin ui")
            .with_inner_size([420.0, 160.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-thin-ui",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(CounterApp::default()))),
    )
}

/// All the state. egui keeps none of it.
#[derive(Default)]
struct CounterApp {
    count: u32,
}

impl CounterApp {
    /// The whole UI. It takes any `Ui`, so it does not know or care whether it is the root of
    /// a window, a region of the gallery, or a test harness.
    fn show(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(format!("Count: {}", self.count));
            if ui.button("Add one").clicked() {
                self.count += 1;
            }
        });
    }
}

impl eframe::App for CounterApp {
    /// The only required method, kept to one line.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    use super::*;

    #[test]
    fn clicking_the_button_counts() {
        let mut harness = Harness::new_ui_state(
            |ui, app: &mut CounterApp| app.show(ui),
            CounterApp::default(),
        );
        harness.run();
        harness.get_by_label("Add one").click();
        harness.run();

        assert_eq!(harness.state().count, 1);
        harness.get_by_label("Count: 1");
    }

    /// Runs `show` once, in the root `Ui` of an 400 by 300 window or in a region of it, and
    /// returns the region and the rect of the central panel.
    fn panel_rect(region: Option<egui::Rect>) -> (egui::Rect, egui::Rect) {
        let window = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(window),
            ..Default::default()
        };
        let mut app = CounterApp::default();
        let mut rects = None;
        ctx.run_ui(input, |ui| match region {
            // Exactly what `App::ui` does with the root `Ui`.
            None => {
                let given = ui.max_rect();
                app.show(ui);
                rects = Some((given, ui.min_rect()));
            }
            Some(region) => {
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(region));
                app.show(&mut child);
                rects = Some((region, child.min_rect()));
            }
        })
        .drop_without_applying_deltas();
        rects.unwrap()
    }

    #[test]
    fn show_fills_whatever_it_is_given() {
        // At the root, the panel is the whole window.
        let (window, at_root) = panel_rect(None);
        assert_eq!(at_root, window);

        // Nested, as in the gallery or a test, it is exactly the region.
        let region = egui::Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(250.0, 150.0));
        let (given, nested) = panel_rect(Some(region));
        assert_eq!(given, region);
        assert_eq!(nested, region);
    }
}
