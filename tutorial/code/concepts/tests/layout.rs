//! Checks behind the claims on the `layout` concept page: some against a bare root `Ui`, some
//! against the page's examples as the gallery draws them.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

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
fn example_harness(name: &str, size: egui::Vec2) -> Harness<'static> {
    let example = concepts::all()
        .into_iter()
        .find(|example| example.name == name)
        .unwrap_or_else(|| panic!("no example named {name:?}"));
    let mut demo = (example.make)();
    let mut harness = Harness::builder()
        .with_size(size)
        .build_ui(move |ui| demo.show(ui));
    harness.run();
    harness
}

#[test]
fn right_to_left_puts_the_first_widget_furthest_right() {
    run_root(|ui| {
        let right_edge = ui.max_rect().right();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let first = ui.button("First").rect;
            let second = ui.button("Second").rect;
            let third = ui.button("Third").rect;
            assert!(first.right() > second.right(), "{first:?} {second:?}");
            assert!(second.right() > third.right(), "{second:?} {third:?}");
            assert_eq!(
                first.right(),
                right_edge,
                "the first starts at the right edge"
            );
            // The gap between neighbours is the item spacing, as in a left-to-right row.
            assert_eq!(first.left() - second.right(), ui.spacing().item_spacing.x);
        });
    });
}

#[test]
fn a_horizontal_inside_right_to_left_also_runs_right_to_left() {
    run_root(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                let first = ui.label("First").rect;
                let second = ui.label("Second").rect;
                assert!(first.right() > second.right());
            });
        });
    });
}

#[test]
fn buttons_on_the_right_example_pushes_buttons_to_the_edge() {
    let harness = example_harness("Buttons on the right", egui::vec2(500.0, 100.0));
    let rect = |label: &str| harness.get_by_label(label).rect();
    assert!(rect("Delete").right() > rect("Archive").right());
    // The label stays at the left; the buttons sit against the right edge.
    assert!(rect("3 items selected").left() < 10.0);
    assert!(rect("Delete").right() > 490.0, "{:?}", rect("Delete"));
}

#[test]
fn sides_pins_the_title_left_and_the_buttons_to_the_right_edge() {
    let example = concepts::all()
        .into_iter()
        .find(|example| example.name == "Title left, buttons right")
        .expect("no example named \"Title left, buttons right\"");
    let mut demo = (example.make)();
    // The state records the right edge of the space the example is given.
    let mut harness = Harness::builder()
        .with_size(egui::vec2(500.0, 100.0))
        .build_ui_state(
            move |ui, right_edge: &mut f32| {
                *right_edge = ui.available_rect_before_wrap().right();
                demo.show(ui);
            },
            0.0,
        );
    harness.run();
    let right_edge = *harness.state();
    let rect = |label: &str| harness.get_by_label(label).rect();

    // The title is at the left, the buttons are to its right.
    assert!(rect("Inbox").right() < rect("Archive").left());
    // The right group runs right to left: the first button added is furthest right.
    assert!(rect("Archive").right() < rect("Delete").left());
    // And it sits against the right edge of the available space.
    assert!(
        (rect("Delete").right() - right_edge).abs() <= 1.0,
        "{:?} {right_edge}",
        rect("Delete")
    );
}

#[test]
fn horizontal_wrapped_starts_new_rows_at_the_left_edge() {
    let harness = example_harness("Wrapping row", egui::vec2(260.0, 300.0));
    let first = harness.get_by_label("rust").rect();
    let last = harness.get_by_label("widgets").rect();
    assert!(last.top() > first.bottom(), "the row wrapped");
    // Every button stays inside the width.
    for tag in ["rust", "immediate-mode", "tutorial", "widgets"] {
        assert!(harness.get_by_label(tag).rect().right() <= 260.0, "{tag}");
    }
}

#[test]
fn item_spacing_sets_the_gap_between_items() {
    let harness = example_harness("Wrapping row", egui::vec2(800.0, 300.0));
    let first = harness.get_by_label("rust").rect();
    let second = harness.get_by_label("gui").rect();
    assert_eq!(second.left() - first.right(), 4.0);
}

#[test]
fn fixed_height_bar_is_48_points_with_contents_centred() {
    run_root(|ui| {
        let size = egui::vec2(ui.available_width(), 48.0);
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        let bar = ui.allocate_ui_with_layout(size, layout, |ui| ui.button("Refresh").rect);
        let bar_rect = bar.response.rect;
        let button = bar.inner;
        assert_eq!(bar_rect.height(), 48.0);
        assert!((button.center().y - bar_rect.center().y).abs() < 1.0);
    });
}

#[test]
fn justified_buttons_are_all_as_wide_as_the_column() {
    let harness = example_harness("Full-width buttons", egui::vec2(400.0, 300.0));
    for label in ["New", "Open recent", "Settings"] {
        assert_eq!(harness.get_by_label(label).rect().width(), 200.0, "{label}");
    }
}

#[test]
fn columns_are_equal_and_spaced_by_item_spacing() {
    run_root(|ui| {
        let width = ui.available_width();
        let spacing = ui.spacing().item_spacing.x;
        ui.columns(3, |columns| {
            let rects: Vec<egui::Rect> = columns.iter().map(|column| column.max_rect()).collect();
            let expected = (width - 2.0 * spacing) / 3.0;
            for rect in &rects {
                assert!((rect.width() - expected).abs() < 0.01, "{rect:?}");
            }
            assert!((rects[1].left() - rects[0].right() - spacing).abs() < 0.01);
            // Each column is justified: a button fills its width.
            let button = columns[0].button("Fills the column").rect;
            assert!((button.width() - expected).abs() < 1.0, "{button:?}");
        });
    });
}

#[test]
fn grid_lines_up_the_second_column() {
    let harness = example_harness("Form grid", egui::vec2(600.0, 300.0));
    let left = |label: &str| harness.get_by_label(label).rect().left();
    // The text fields are labelled by the label in their row.
    let fields: Vec<f32> = harness
        .get_all_by_role(egui::accesskit::Role::TextInput)
        .map(|node| node.rect().left())
        .collect();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0], fields[1]);
    // The checkbox in the third row starts where the fields do.
    assert!((left("Send me the newsletter") - fields[0]).abs() < 1.0);
    // The second column starts after the widest label in the first. (The text field is
    // labelled "Email address" too, so ask for the label by its role.)
    let email = harness.get_by_role_and_label(egui::accesskit::Role::Label, "Email address");
    let name = harness.get_by_role_and_label(egui::accesskit::Role::Label, "Name");
    assert!(fields[0] > email.rect().right());
    assert!(email.rect().width() > name.rect().width());
}

#[test]
fn aligned_centres_a_row_but_a_centred_column_does_not() {
    let width = 600.0;
    let harness = example_harness("Centre content of unknown width", egui::vec2(width, 400.0));
    let pane_centre = width / 2.0;

    // A single widget in a centred column is centred.
    let button = harness.get_by_label("A centred button").rect();
    assert!((button.center().x - pane_centre).abs() < 1.0, "{button:?}");

    // A row inside a centred column starts at the left.
    assert!(harness.get_by_label("First").rect().left() < 20.0);

    // `Aligned` centres the row as a whole.
    let row_left = harness.get_by_label("Previous").rect().left();
    let row_right = harness.get_by_label("Next").rect().right();
    let row_centre = (row_left + row_right) / 2.0;
    assert!(
        (row_centre - pane_centre).abs() < 1.0,
        "{row_left} {row_right}"
    );
}
