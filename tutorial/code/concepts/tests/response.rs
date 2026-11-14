//! Checks behind the claims on the `response` concept page: some against small widgets drawn
//! here, most against the page's examples as the gallery draws them.

use eframe::egui;
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, HarnessBuilder};

/// A harness over the page's example called `name`, built the way the gallery builds it.
fn example_harness(name: &str) -> Harness<'static> {
    example_harness_with(Harness::builder(), name)
}

/// [`example_harness`] from a builder, for tests that need other settings.
fn example_harness_with(builder: HarnessBuilder, name: &str) -> Harness<'static> {
    let example = concepts::all()
        .into_iter()
        .find(|example| example.name == name)
        .unwrap_or_else(|| panic!("no example named {name:?}"));
    let mut demo = (example.make)();
    builder.build_ui(move |ui| demo.show(ui))
}

/// Drags from the centre of the widget labelled `label` by `by`, a few points at a time.
fn drag(harness: &mut Harness<'static>, label: &str, by: egui::Vec2) {
    let start = harness.get_by_label(label).rect().center();
    harness.hover_at(start);
    harness.run();
    harness.drag_at(start);
    harness.run();
    for step in 1..=4 {
        harness.hover_at(start + by * (step as f32 / 4.0));
        harness.run();
    }
    harness.drop_at(start + by);
    harness.run();
}

#[test]
fn clicking_the_button_increments_the_visible_counter() {
    let mut harness = example_harness("Click counter");
    harness.run();
    harness.get_by_label("Clicks: 0");

    harness.get_by_label("Add one").click();
    harness.run();
    harness.get_by_label("Clicks: 1");

    harness.get_by_label("Add one").click();
    harness.run();
    harness.get_by_label("Clicks: 2");
}

#[test]
fn a_label_with_sense_click_responds_to_a_click() {
    let mut harness = example_harness("Clickable label");
    harness.run();
    harness.get_by_label("Opened: 0");

    harness.get_by_label("Open the report").click();
    harness.run();
    harness.get_by_label("Opened: 1");
}

#[test]
fn a_label_with_sense_click_can_be_clicked_from_the_keyboard() {
    let mut harness = example_harness("Clickable label");
    harness.run();
    // `Sense::click()` makes the label focusable, so Tab reaches it and Enter clicks it.
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert!(harness.get_by_label("Open the report").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.get_by_label("Opened: 1");
}

#[test]
fn interact_makes_a_whole_container_clickable() {
    let mut harness = example_harness("Clickable label");
    harness.run();
    harness.get_by_label("Archived: 0");

    // Click in the card's margin, beside its label rather than on it.
    let beside_label = harness
        .get_by_label("Archive this card")
        .rect()
        .left_center()
        - egui::vec2(3.0, 0.0);
    harness.hover_at(beside_label);
    harness.drag_at(beside_label);
    harness.drop_at(beside_label);
    harness.run();
    harness.get_by_label("Archived: 1");

    // The card was registered after its label, so it is on top of the label as well.
    harness.get_by_label("Archive this card").click();
    harness.run();
    harness.get_by_label("Archived: 2");
}

#[test]
fn a_button_does_not_report_changed() {
    let mut harness = Harness::new_ui_state(
        |ui, changed: &mut bool| {
            if ui.button("Press").changed() {
                *changed = true;
            }
        },
        false,
    );
    harness.run();
    harness.get_by_label("Press").click();
    harness.run();
    assert!(!*harness.state());
}

#[test]
fn right_click_reports_secondary_clicked_and_opens_the_context_menu() {
    let mut harness = example_harness("Double-click and right-click");
    harness.run();
    assert!(harness.query_by_label("Rename").is_none());

    harness.get_by_label("Report.txt").click_secondary();
    harness.run();
    harness.get_by_label("Last: right-clicked");
    harness.get_by_label("Rename");
}

#[test]
fn a_double_click_also_reports_two_clicks() {
    // The harness moves its clock on a quarter of a second per frame by default, and a few
    // frames per `run`, which is longer than the 0.3 second double-click limit.
    let builder = Harness::builder().with_step_dt(1.0 / 60.0);
    let mut harness = example_harness_with(builder, "Double-click and right-click");
    harness.run();
    // Two clicks a few frames apart.
    harness.get_by_label("Report.txt").click();
    harness.run();
    harness.get_by_label("Report.txt").click();
    harness.run();
    harness.get_by_label("Last: double-clicked");
    harness.get_by_label("Clicks: 2");
}

#[test]
fn hovering_the_button_is_reported_by_hovered() {
    let mut harness = example_harness("Hover and tooltip");
    harness.run();
    harness.get_by_label("The pointer is elsewhere.");

    harness.get_by_label("Hover over me").hover();
    harness.run();
    harness.get_by_label("The pointer is over the button.");
}

#[test]
fn typing_is_changed_and_enter_submits() {
    let mut harness = example_harness("Typing and Enter");
    harness.run();
    harness.get_by_label("Search").focus();
    harness.run();
    harness.get_by_label("Search").type_text("egui");
    harness.run();
    harness.get_by_label("Frames with a change: 1");
    assert!(harness.query_by_label("Searched for: egui").is_none());

    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.get_by_label("Searched for: egui");
}

#[test]
fn losing_focus_without_enter_does_not_submit() {
    let mut harness = example_harness("Typing and Enter");
    harness.run();
    harness.get_by_label("Search").focus();
    harness.run();
    harness.get_by_label("Search").type_text("egui");
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.get_by_label("Search").is_focused());
    assert!(harness.query_by_label("Searched for: egui").is_none());
}

#[test]
fn request_focus_moves_the_keyboard_focus_to_the_field() {
    let mut harness = example_harness("Focus");
    harness.run();
    harness.get_by_label("The name field does not have focus.");

    harness.get_by_label("Focus the name field").click();
    harness.run();
    assert!(harness.get_by_label("Name").is_focused());
    harness.get_by_label("The name field has focus.");
}

#[test]
fn dragging_the_square_moves_it_by_the_drag() {
    let mut harness = example_harness("Drag");
    harness.run();
    let before = harness.get_by_label("Draggable square").rect().min;

    drag(&mut harness, "Draggable square", egui::vec2(40.0, 20.0));
    let after = harness.get_by_label("Draggable square").rect().min;
    assert_eq!(after - before, egui::vec2(40.0, 20.0));
}

/// The number at the end of the label that starts "Frames with a change: ".
fn frames_changed(harness: &Harness<'static>) -> u32 {
    let label = harness
        .get_by_label_contains("Frames with a change: ")
        .value()
        .expect("a label keeps its text in its value");
    label["Frames with a change: ".len()..].parse().unwrap()
}

#[test]
fn union_reports_a_change_in_either_value() {
    let mut harness = example_harness("Several widgets as one");
    harness.run();
    assert_eq!(frames_changed(&harness), 0);

    drag(&mut harness, "x", egui::vec2(30.0, 0.0));
    let after_x = frames_changed(&harness);
    assert!(after_x > 0);
    drag(&mut harness, "y", egui::vec2(30.0, 0.0));
    assert!(frames_changed(&harness) > after_x);
}

#[test]
fn union_takes_the_rect_around_both_and_the_first_id() {
    let mut harness = Harness::new_ui(|ui| {
        let a = ui.button("A");
        let b = ui.button("B");
        let both = a.clone() | b.clone();
        assert_eq!(both.rect, a.rect.union(b.rect));
        assert_eq!(both.id, a.id);
    });
    harness.run();
}

#[test]
fn a_container_returns_its_closure_value_as_inner() {
    let mut harness = Harness::new_ui(|ui| {
        let row = ui.horizontal(|ui| {
            ui.label("Inside");
            42
        });
        assert_eq!(row.inner, 42);
        assert!(row.response.rect.width() > 0.0);
    });
    harness.run();
}

/// Clicks a plain `ui.label` and returns whether its response reported `clicked()`.
fn plain_label_clicked(selectable_labels: bool) -> bool {
    let mut harness = Harness::new_ui_state(
        move |ui, clicked: &mut bool| {
            ui.style_mut().interaction.selectable_labels = selectable_labels;
            if ui.label("Plain label").clicked() {
                *clicked = true;
            }
        },
        false,
    );
    harness.run();
    harness.get_by_label("Plain label").click();
    harness.run();
    *harness.state()
}

#[test]
fn a_plain_label_reports_clicks_only_because_its_text_is_selectable() {
    // Selectable text senses clicks, and labels are selectable by default.
    assert!(plain_label_clicked(true));
    assert!(!plain_label_clicked(false));
}

#[test]
fn lost_focus_is_reported_on_two_frames() {
    let mut harness = Harness::new_ui_state(
        |ui, (text, frames): &mut (String, u32)| {
            let field = ui.text_edit_singleline(text).accessible_name("Field");
            if field.lost_focus() {
                *frames += 1;
            }
        },
        (String::new(), 0),
    );
    harness.get_by_label("Field").focus();
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run_steps(5);
    // So pair it with the key that was pressed, which is reported on one frame only.
    assert_eq!(harness.state().1, 2);
}
