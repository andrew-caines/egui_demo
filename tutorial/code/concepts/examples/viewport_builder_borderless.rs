//! A window with no title bar or border from the operating system, and a title bar drawn by the
//! app instead.
//!
//! Run with `cargo run -p concepts --example viewport_builder_borderless`.

use eframe::egui;

const TITLE: &str = "Viewport builder: borderless";

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_decorations(false)
            .with_inner_size([440.0, 220.0])
            .with_min_inner_size([320.0, 150.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-borderless",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(BorderlessDemo))),
    )
}

struct BorderlessDemo;

impl eframe::App for BorderlessDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        title_bar(ui, TITLE);
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("No border and no title bar from the operating system.");
        });
    }
}

/// A title bar drawn by the app. With decorations off, this is the only way to move, maximize,
/// minimize or close the window.
fn title_bar(ui: &mut egui::Ui, title: &str) {
    egui::Panel::top("title_bar")
        .exact_size(32.0)
        .show(ui, |ui| {
            // The whole bar is the drag region. It is added first, so the buttons added after it
            // sit on top and get their own clicks.
            let bar = ui.interact(
                ui.max_rect(),
                ui.scope_id().with("drag_region"),
                egui::Sense::click_and_drag(),
            );
            if bar.double_clicked() {
                let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            } else if bar.drag_started_by(egui::PointerButton::Primary) {
                // Hand the drag to the operating system, which moves the window.
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.horizontal_centered(|ui| {
                // Not selectable, so a press on the title drags the window instead of
                // selecting the text.
                ui.add(egui::Label::new(egui::RichText::new(title).strong()).selectable(false));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Minimize").clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{ViewportCommand, ViewportId};
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    /// The viewport commands the app sent in the last frame the harness ran.
    fn commands(harness: &Harness<'_>) -> Vec<ViewportCommand> {
        harness.output().viewport_output[&ViewportId::ROOT]
            .commands
            .clone()
    }

    fn harness() -> Harness<'static> {
        let mut harness = Harness::new_ui(|ui| title_bar(ui, "Borderless"));
        harness.run();
        harness
    }

    #[test]
    fn close_button_closes_the_window() {
        let mut harness = harness();
        harness.get_by_label("Close").click();
        harness.step();

        assert!(commands(&harness).contains(&ViewportCommand::Close));
    }

    #[test]
    fn minimize_button_minimizes_the_window() {
        let mut harness = harness();
        harness.get_by_label("Minimize").click();
        harness.step();

        assert!(commands(&harness).contains(&ViewportCommand::Minimized(true)));
    }

    #[test]
    fn dragging_the_title_bar_moves_the_window() {
        let mut harness = harness();
        let start = harness.get_by_label("Borderless").rect().center();
        harness.hover_at(start);
        harness.drag_at(start);
        harness.hover_at(start + egui::vec2(30.0, 0.0));
        harness.step();

        assert!(commands(&harness).contains(&ViewportCommand::StartDrag));
    }

    #[test]
    fn double_clicking_the_title_bar_maximizes_the_window() {
        // A double click must fit in 0.3 seconds. The harness's default frame time is longer
        // than that, so step at 60 frames per second instead.
        let mut harness = Harness::builder()
            .with_step_dt(1.0 / 60.0)
            .build_ui(|ui| title_bar(ui, "Borderless"));
        harness.run();
        let title = harness.get_by_label("Borderless");
        title.click();
        title.click();
        harness.step();

        assert!(commands(&harness).contains(&ViewportCommand::Maximized(true)));
    }

    #[test]
    fn a_plain_click_on_the_title_bar_sends_nothing() {
        let mut harness = harness();
        harness.get_by_label("Borderless").click();
        harness.step();

        assert!(commands(&harness).is_empty());
    }
}
