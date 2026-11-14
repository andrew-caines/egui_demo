//! Checks behind the claims on the `viewport-builder` concept page. The examples are whole
//! programs that open real windows, so these tests check what the builder holds and how it
//! turns into commands. The custom title bar is tested inside
//! `examples/viewport_builder_borderless.rs`, the icon inside `examples/viewport_builder_icon.rs`.

use eframe::egui::{self, ViewportBuilder, ViewportCommand, WindowLevel};

#[test]
fn default_builder_sets_nothing() {
    // Every field is an `Option`; `None` means "use the platform's default".
    let builder = ViewportBuilder::default();

    assert!(builder.title.is_none());
    assert!(builder.inner_size.is_none());
    assert!(builder.position.is_none());
    assert!(builder.decorations.is_none());
    assert!(builder.transparent.is_none());
    assert!(builder.resizable.is_none());
    assert!(builder.window_level.is_none());
    assert!(builder.icon.is_none());
}

#[test]
fn builder_methods_fill_in_the_fields() {
    let builder = ViewportBuilder::default()
        .with_title("Title")
        .with_inner_size([440.0, 220.0])
        .with_min_inner_size([320.0, 150.0])
        .with_max_inner_size([800.0, 400.0])
        .with_position([120.0, 120.0])
        .with_decorations(false)
        .with_resizable(false);

    assert_eq!(builder.title.as_deref(), Some("Title"));
    assert_eq!(builder.inner_size, Some(egui::vec2(440.0, 220.0)));
    assert_eq!(builder.min_inner_size, Some(egui::vec2(320.0, 150.0)));
    assert_eq!(builder.max_inner_size, Some(egui::vec2(800.0, 400.0)));
    assert_eq!(builder.position, Some(egui::pos2(120.0, 120.0)));
    assert_eq!(builder.decorations, Some(false));
    assert_eq!(builder.resizable, Some(false));
}

#[test]
fn always_on_top_is_a_window_level() {
    let builder = ViewportBuilder::default().with_always_on_top();

    assert_eq!(builder.window_level, Some(WindowLevel::AlwaysOnTop));
    assert_eq!(WindowLevel::default(), WindowLevel::Normal);
}

#[test]
fn a_changed_builder_becomes_commands() {
    // `ViewportCommand` is a diff of two builders. This is how egui updates an extra window
    // whose builder changes between frames.
    let mut current = ViewportBuilder::default()
        .with_title("Old")
        .with_inner_size([400.0, 200.0]);
    let wanted = ViewportBuilder::default()
        .with_title("New")
        .with_inner_size([400.0, 200.0])
        .with_always_on_top();

    let (commands, recreate) = current.patch(wanted);

    assert_eq!(
        commands,
        vec![
            ViewportCommand::Title("New".to_owned()),
            ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop),
        ]
    );
    assert!(!recreate, "both have commands, so the window is kept");
}

#[test]
fn some_settings_have_no_command_and_rebuild_the_window() {
    let mut current = ViewportBuilder::default();

    let (commands, recreate) = current.patch(ViewportBuilder::default().with_close_button(false));

    assert!(commands.is_empty());
    assert!(
        recreate,
        "the close button can only be set when the window is created"
    );
}
