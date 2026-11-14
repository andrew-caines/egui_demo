//! Checks behind the claims on the `run-native` concept page. The examples are whole programs
//! that open real windows, so these tests check the facts the page states rather than drawing
//! anything. The examples' own set-up functions are tested inside `examples/run_native_*.rs`.

#[test]
fn the_context_can_be_sent_to_another_thread() {
    // The thread example moves a clone of `egui::Context` into a thread.
    fn send_sync_clone<T: Send + Sync + Clone>() {}
    send_sync_clone::<eframe::egui::Context>();
}

#[test]
fn without_set_theme_egui_follows_the_system() {
    let ctx = eframe::egui::Context::default();

    ctx.options(|options| {
        assert_eq!(
            options.theme_preference,
            eframe::egui::ThemePreference::System
        );
        // Used when the system theme cannot be detected.
        assert_eq!(options.fallback_theme, eframe::egui::Theme::Dark);
    });
}

#[test]
fn creator_errors_are_reported_as_app_creation() {
    let error = eframe::Error::AppCreation(Box::from("no config file"));

    assert_eq!(error.to_string(), "app creation error: no config file");
}

#[cfg(all(feature = "persistence", windows))]
#[test]
fn restore_example_saves_under_its_app_name() {
    let folder =
        eframe::storage_dir("egui-concepts-run-native-restore").expect("a folder on Windows");
    let appdata = std::env::var_os("APPDATA").expect("APPDATA is set on Windows");

    assert_eq!(
        folder,
        std::path::Path::new(&appdata)
            .join("egui-concepts-run-native-restore")
            .join("data")
    );
}
