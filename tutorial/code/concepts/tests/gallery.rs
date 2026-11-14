use std::collections::HashSet;
use std::path::Path;

use concepts::gallery::Gallery;
use eframe::egui;
use eframe::egui::accesskit::Toggled;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

#[test]
fn every_example_renders() {
    for example in concepts::all() {
        let mut demo = (example.make)();
        let mut harness = Harness::new_ui(move |ui| demo.show(ui));
        harness.run_steps(3);
    }
}

#[test]
fn names_are_unique() {
    let mut seen = HashSet::new();
    for example in concepts::all() {
        assert!(
            seen.insert(example.name),
            "duplicate example name: {}",
            example.name
        );
    }
}

#[test]
fn every_module_is_registered() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let concepts_with_examples: HashSet<&str> = concepts::all()
        .iter()
        .map(|example| example.concept)
        .collect();

    for entry in std::fs::read_dir(src).expect("src directory is readable") {
        let path = entry.expect("directory entry is readable").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap();
        if ["lib", "main", "gallery"].contains(&stem) {
            continue;
        }
        // The `ui_region` module documents the concept page called `ui`.
        let concept = if stem == "ui_region" {
            "ui".to_owned()
        } else {
            stem.replace('_', "-")
        };
        assert!(
            concepts_with_examples.contains(concept.as_str()),
            "src/{stem}.rs has no example registered under concept `{concept}`"
        );
    }
}

fn gallery_harness(size: egui::Vec2) -> Harness<'static, Gallery> {
    Harness::builder().with_size(size).build_ui_state(
        |ui, gallery: &mut Gallery| gallery.show(ui),
        Gallery::default(),
    )
}

#[test]
fn gallery_shows_every_example() {
    let mut harness = gallery_harness(egui::vec2(900.0, 600.0));
    for example in concepts::all() {
        harness.state_mut().select(example.name);
        harness.run();
        harness.get_by_label(example.when);
    }
}

#[test]
fn switching_examples_resets_state() {
    let mut harness = gallery_harness(egui::vec2(900.0, 600.0));
    harness.state_mut().select("Panel order");
    harness.run();
    harness.get_by_label("Sidebar first").click();
    harness.run();
    assert_eq!(
        harness
            .get_by_label("Sidebar first")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );

    harness.state_mut().select("Default central panel");
    harness.run();
    harness.state_mut().select("Panel order");
    harness.run();

    // A fresh demo was built, so the choice is back to its initial value.
    assert_eq!(
        harness
            .get_by_label("Header first")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
}

#[test]
fn switching_examples_resets_scroll_position() {
    // Narrow enough that the example pane scrolls sideways.
    let mut harness = gallery_harness(egui::vec2(400.0, 300.0));
    harness.state_mut().select("Default central panel");
    harness.run();
    let unscrolled_left = harness.get_by_label("Central panel").rect().left();

    harness.state_mut().select("App shell");
    harness.run();
    // The right-hand end of this label is off the side of the pane; bring it into view.
    harness
        .get_by_label("Central panel: everything that is left.")
        .scroll_to_me();
    harness.run();

    harness.state_mut().select("Default central panel");
    harness.run();
    assert_eq!(
        harness.get_by_label("Central panel").rect().left(),
        unscrolled_left
    );
}

#[test]
fn gallery_fits_small_window() {
    let mut harness = gallery_harness(egui::vec2(400.0, 300.0));
    for example in concepts::all() {
        harness.state_mut().select(example.name);
        harness.run();
        // The list panel still shows this example's entry in the scrolling list.
        harness.get_by_label(example.name);
    }
}
