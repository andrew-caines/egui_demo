//! Checks behind the claims on the `ui` concept page: some against a bare root `Ui`, some
//! against the page's examples as the gallery draws them.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

const WINDOW: egui::Vec2 = egui::vec2(400.0, 300.0);

/// Runs one pass with the root `Ui` of a window of size [`WINDOW`].
fn run_root(draw: impl FnMut(&mut egui::Ui)) {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, WINDOW)),
        ..Default::default()
    };
    ctx.run_ui(input, draw).drop_without_applying_deltas();
}

/// A harness over the page's example called `name`, built the way the gallery builds it.
fn example_harness(name: &str) -> Harness<'static> {
    let example = concepts::all()
        .into_iter()
        .find(|example| example.name == name)
        .unwrap_or_else(|| panic!("no example named {name:?}"));
    let mut demo = (example.make)();
    Harness::new_ui(move |ui| demo.show(ui))
}

/// Two labels, with `add_space(extra)` between them if given. Returns the top of the second
/// label, measured from the top of the first.
fn second_label_top(extra: Option<f32>) -> f32 {
    let mut offset = None;
    run_root(|ui| {
        let first = ui.label("First");
        if let Some(amount) = extra {
            ui.add_space(amount);
        }
        let second = ui.label("Second");
        offset = Some(second.rect.top() - first.rect.top());
    });
    offset.unwrap()
}

#[test]
fn add_space_moves_the_next_widget_down_by_exactly_that_amount() {
    let without = second_label_top(None);
    let with = second_label_top(Some(20.0));
    assert_eq!(with - without, 20.0);
}

#[test]
fn add_space_comes_on_top_of_item_spacing() {
    let mut gaps = None;
    run_root(|ui| {
        let item_spacing = ui.spacing().item_spacing.y;
        let first = ui.label("First");
        ui.add_space(20.0);
        let second = ui.label("Second");
        gaps = Some((second.rect.top() - first.rect.bottom(), item_spacing));
    });
    let (gap, item_spacing) = gaps.unwrap();
    assert_eq!(
        item_spacing, 3.0,
        "the default item spacing is 3 points down"
    );
    assert_eq!(gap, 20.0 + item_spacing);
}

#[test]
fn the_cursor_waits_item_spacing_below_the_last_widget() {
    run_root(|ui| {
        let available_before = ui.available_height();
        let label = ui.label("A widget");
        let cursor_top = ui.cursor().top();
        assert_eq!(
            cursor_top,
            label.rect.bottom() + ui.spacing().item_spacing.y
        );
        // What is left shrinks by the widget and the spacing after it.
        assert_eq!(
            available_before - ui.available_height(),
            cursor_top - label.rect.top()
        );
    });
}

#[test]
fn allocate_exact_size_returns_exactly_the_size_asked_for() {
    run_root(|ui| {
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(120.0, 16.0), egui::Sense::hover());
        assert_eq!(rect.size(), egui::vec2(120.0, 16.0));
    });
}

#[test]
fn allocate_ui_takes_only_the_space_the_contents_use() {
    run_root(|ui| {
        let region = ui.allocate_ui(egui::vec2(220.0, 120.0), |ui| {
            ui.label("Short");
        });
        let next = ui.label("Next");
        let used = region.response.rect;
        assert!(used.height() < 120.0, "used {used:?}");
        assert!(used.width() < 220.0, "used {used:?}");
        // The next widget follows what was used, not what was offered.
        assert_eq!(next.rect.top(), used.bottom() + ui.spacing().item_spacing.y);
    });
}

#[test]
fn set_min_width_widens_and_set_max_width_wraps() {
    run_root(|ui| {
        let wide = ui.scope(|ui| {
            ui.set_min_width(240.0);
            ui.label("Short.");
        });
        assert!(wide.response.rect.width() >= 240.0);

        let one_line = ui.label("Short.").rect.height();
        let narrow = ui.scope(|ui| {
            ui.set_max_width(100.0);
            ui.label("This sentence is far too long to fit on one line of 100 points.")
        });
        assert!(narrow.inner.rect.width() <= 100.0);
        assert!(narrow.inner.rect.height() > one_line, "the label wrapped");
    });
}

#[test]
fn scope_keeps_the_id_scope_but_push_id_changes_it() {
    run_root(|ui| {
        let in_scope = |ui: &mut egui::Ui| ui.scope(|ui| ui.make_persistent_id("Details")).inner;
        assert_eq!(in_scope(ui), in_scope(ui), "two scopes share ids");

        let first = ui.push_id(0, |ui| ui.make_persistent_id("Details")).inner;
        let second = ui.push_id(1, |ui| ui.make_persistent_id("Details")).inner;
        assert_ne!(first, second, "push_id makes them distinct");
    });
}

#[test]
fn extra_space_example_puts_the_next_line_20_points_lower() {
    let harness = example_harness("Extra space");
    let gap = |above: &str, below: &str| {
        harness.get_by_label(below).rect().top() - harness.get_by_label(above).rect().bottom()
    };
    let without = gap("Without add_space", "The next line");
    let with = gap("With add_space(20.0)", "The next line, 20 points lower");
    assert!((with - without - 20.0).abs() < 0.5, "{with} vs {without}");
}

#[test]
fn disabled_button_turns_on_with_its_checkbox() {
    let mut harness = example_harness("Disabled widgets");
    harness.run();
    assert!(
        harness
            .get_by_label("Continue")
            .accesskit_node()
            .is_disabled()
    );
    // A click on the disabled button is ignored.
    harness.get_by_label("Continue").click();
    harness.run();
    assert!(harness.query_by_label("You clicked Continue.").is_none());

    harness.get_by_label("I have read the terms").click();
    harness.run();
    assert!(
        !harness
            .get_by_label("Continue")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Continue").click();
    harness.run();
    harness.get_by_label("You clicked Continue.");
}

#[test]
fn disabled_section_disables_everything_inside() {
    let mut harness = example_harness("Disabled widgets");
    harness.run();
    assert!(harness.get_by_label("Reset").accesskit_node().is_disabled());
    assert!(
        harness
            .get_by_label("Compact rows")
            .accesskit_node()
            .is_disabled()
    );

    harness.get_by_label("Allow changes").click();
    harness.run();
    assert!(!harness.get_by_label("Reset").accesskit_node().is_disabled());
}

#[test]
fn scoped_style_ends_with_the_closure() {
    let harness = example_harness("Scoped style");
    let height = |label: &str| harness.get_by_label(label).rect().height();
    assert!(height("Bigger button") > height("Normal button"));
    assert_eq!(height("Normal again"), height("Normal button"));
}
