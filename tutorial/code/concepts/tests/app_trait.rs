//! Checks behind the claims on the `app-trait` concept page: the trait's defaults, when egui
//! asks for another frame, and that an example drawn at the root of a window lays out the same
//! as one nested in a region (which is how the gallery shows it).

use std::time::Duration;

use concepts::Demo as _;
use concepts::app_trait::{FrameCounter, TickingClock};
use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

const WINDOW: egui::Vec2 = egui::vec2(400.0, 300.0);

/// An app that implements only the one required method.
struct OnlyUi;

impl eframe::App for OnlyUi {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}
}

#[test]
fn the_provided_methods_have_the_documented_defaults() {
    let app = OnlyUi;
    let visuals = egui::Visuals::light();

    assert_eq!(
        eframe::App::auto_save_interval(&app),
        Duration::from_secs(30)
    );
    assert!(eframe::App::persist_egui_memory(&app));
    // A near-black, partly transparent grey, whatever the theme.
    assert_eq!(
        eframe::App::clear_color(&app, &visuals),
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()
    );
}

/// Runs one pass of a window of size [`WINDOW`] at the given time and returns how long egui
/// asks the integration to wait before the next frame.
fn pass_at(ctx: &egui::Context, time: f64, draw: impl FnMut(&mut egui::Ui)) -> Duration {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, WINDOW)),
        time: Some(time),
        ..Default::default()
    };
    let output = ctx.run_ui(input, draw);
    let delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
    // There is no renderer to hand the font texture to.
    output.drop_without_applying_deltas();
    delay
}

#[test]
fn an_idle_ui_asks_for_no_more_frames() {
    let ctx = egui::Context::default();
    let mut counter = FrameCounter::default();
    let mut delay = Duration::ZERO;
    for pass in 0..5 {
        delay = pass_at(&ctx, f64::from(pass), |ui| counter.show(ui));
    }

    // No input and no request: eframe would wait for the next event.
    assert_eq!(delay, Duration::MAX);
    assert_eq!(counter.frames(), 5);
}

#[test]
fn ticking_the_checkbox_asks_for_a_frame_straight_away() {
    let mut harness = Harness::new_ui_state(
        |ui, counter: &mut FrameCounter| counter.show(ui),
        FrameCounter::default(),
    );
    harness.run();
    harness.get_by_label("Repaint every frame").click();
    harness.step();
    harness.step();

    let delay = harness.output().viewport_output[&egui::ViewportId::ROOT].repaint_delay;
    assert_eq!(delay, Duration::ZERO);
}

#[test]
fn the_clock_asks_for_a_frame_at_the_next_whole_second() {
    let ctx = egui::Context::default();
    let mut clock = TickingClock::default();
    // egui asks for a few frames of its own while a new ui settles; let them pass.
    for _ in 0..3 {
        pass_at(&ctx, 10.0, |ui| clock.show(ui));
    }
    let delay = pass_at(&ctx, 12.25, |ui| clock.show(ui));

    assert_eq!(clock.whole_seconds(), 2);
    let expected = Duration::from_millis(750);
    assert!(
        delay.abs_diff(expected) < Duration::from_millis(5),
        "asked to wait {delay:?}"
    );
}

/// Draws a fresh frame counter and returns the region it was given and the area its widgets
/// used.
fn counter_rects(nested_in: Option<egui::Rect>) -> (egui::Rect, egui::Rect) {
    let ctx = egui::Context::default();
    let mut counter = FrameCounter::default();
    let mut rects = None;
    pass_at(&ctx, 0.0, |ui| match nested_in {
        None => {
            counter.show(ui);
            rects = Some((ui.max_rect(), ui.min_rect()));
        }
        Some(region) => {
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(region));
            counter.show(&mut child);
            rects = Some((child.max_rect(), child.min_rect()));
        }
    });
    rects.unwrap()
}

#[test]
fn the_root_ui_is_the_whole_window_with_no_margin() {
    let (region, used) = counter_rects(None);

    assert_eq!(region, egui::Rect::from_min_size(egui::Pos2::ZERO, WINDOW));
    // The first widget starts at the window's corner.
    assert_eq!(used.min, egui::Pos2::ZERO);
}

#[test]
fn an_example_lays_out_the_same_nested_as_at_the_root() {
    let region = egui::Rect::from_min_size(egui::pos2(50.0, 20.0), egui::vec2(300.0, 200.0));
    let (_, at_root) = counter_rects(None);
    let (given, nested) = counter_rects(Some(region));

    assert_eq!(given, region);
    // Same size, moved by where the region starts.
    assert_eq!(nested.min, region.min);
    assert_eq!(nested.size(), at_root.size());
}
