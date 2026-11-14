//! Checks behind the claims on the `central-panel` concept page, run against a real root `Ui`.

use eframe::egui;

const WINDOW: egui::Vec2 = egui::vec2(400.0, 300.0);

/// Runs one pass with the root `Ui` of a window of size [`WINDOW`], as eframe hands it to
/// `App::ui`.
fn run_root(draw: impl FnMut(&mut egui::Ui)) {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, WINDOW)),
        ..Default::default()
    };
    ctx.run_ui(input, draw).drop_without_applying_deltas();
}

/// A top panel of fixed height followed by a central panel; returns both rects.
fn top_then_central(ui: &mut egui::Ui) -> (egui::Rect, egui::Rect) {
    let top = egui::Panel::top("top")
        .exact_size(40.0)
        .show(ui, |ui| ui.label("Top"))
        .response
        .rect;
    let central = egui::CentralPanel::default()
        .show(ui, |ui| ui.label("Central"))
        .response
        .rect;
    (top, central)
}

#[test]
fn central_panel_fills_remaining_space() {
    let mut rects = None;
    run_root(|ui| rects = Some(top_then_central(ui)));
    let (top, central) = rects.unwrap();

    assert!((central.height() - (WINDOW.y - top.height())).abs() <= 1.0);
    assert!((central.width() - WINDOW.x).abs() <= 1.0);
    assert!((central.top() - top.bottom()).abs() <= 1.0);
}

#[test]
fn panels_behave_the_same_in_a_child_ui() {
    // The gallery shows examples in a region of its pane, not at the root of a window.
    let region = egui::Rect::from_min_size(egui::pos2(50.0, 20.0), egui::vec2(300.0, 200.0));
    let mut rects = None;
    run_root(|ui| {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(region));
        rects = Some(top_then_central(&mut child));
    });
    let (top, central) = rects.unwrap();

    assert!((central.height() - (region.height() - top.height())).abs() <= 1.0);
    assert!((central.width() - region.width()).abs() <= 1.0);
    assert!((central.bottom() - region.bottom()).abs() <= 1.0);
}

#[test]
fn nothing_after_the_central_panel_gets_space() {
    let mut after = Vec::new();
    run_root(|ui| {
        egui::CentralPanel::default().show(ui, |ui| ui.label("Central"));
        let sidebar = egui::Panel::left("sidebar").show(ui, |ui| ui.label("Sidebar"));
        let second = egui::CentralPanel::default().show(ui, |ui| ui.label("Second"));
        after = vec![sidebar.response.rect, second.response.rect];
    });

    // Both are placed below the first central panel, outside the window, where nothing is
    // drawn.
    for rect in after {
        assert!(rect.top() >= WINDOW.y, "{rect:?} starts inside the window");
    }
}

#[test]
fn central_panel_clips_content_that_does_not_fit() {
    let mut clip = None;
    let mut panel = None;
    run_root(|ui| {
        let response = egui::CentralPanel::default().show(ui, |ui| {
            for line in 0..100 {
                ui.label(format!("Line {line}"));
            }
            ui.clip_rect()
        });
        clip = Some(response.inner);
        panel = Some(response.response.rect);
    });

    // The content runs far past the window, but drawing stops at the window's edge.
    assert!(panel.unwrap().bottom() > WINDOW.y);
    assert!(clip.unwrap().bottom() <= WINDOW.y);
}
