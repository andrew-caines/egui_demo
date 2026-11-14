//! Checks behind the claims on the `native-options` concept page. The examples are whole
//! programs that open real windows, so these tests check the defaults and paths the page
//! states rather than drawing anything. The persistence example's own set-up function is
//! tested inside `examples/native_options_persist.rs`.

use eframe::wgpu::PresentMode;

#[test]
fn defaults_are_the_ones_the_page_lists() {
    let options = eframe::NativeOptions::default();

    assert!(!options.centered);
    assert!(options.persist_window);
    assert!(options.run_and_return);
    assert_eq!(options.multisampling, 0);
    assert_eq!(options.depth_buffer, 0);
    assert_eq!(options.stencil_buffer, 0);
    assert!(options.persistence_path.is_none());
}

#[test]
fn default_viewport_sets_nothing() {
    // eframe fills in the title (and app id) from the app name only when these are unset.
    let viewport = eframe::NativeOptions::default().viewport;

    assert!(viewport.title.is_none());
    assert!(viewport.app_id.is_none());
    assert!(viewport.inner_size.is_none());
}

#[test]
fn wgpu_is_the_default_renderer() {
    // With eframe's default features, wgpu is the only renderer compiled in.
    assert_eq!(
        eframe::NativeOptions::default().renderer,
        eframe::Renderer::Wgpu
    );
}

#[test]
fn vsync_is_on_by_default() {
    let surface = eframe::NativeOptions::default().wgpu_options.surface;

    assert_eq!(surface.present_mode, PresentMode::AutoVsync);
}

#[test]
fn hooks_are_dropped_when_options_are_cloned() {
    let options = eframe::NativeOptions {
        window_builder: Some(Box::new(|builder| builder)),
        ..Default::default()
    };

    assert!(options.clone().window_builder.is_none());
}

#[cfg(all(feature = "persistence", windows))]
#[test]
fn save_folder_is_named_after_the_app() {
    let folder = eframe::storage_dir("egui-concepts-persist").expect("a folder on Windows");
    let appdata = std::env::var_os("APPDATA").expect("APPDATA is set on Windows");

    assert_eq!(
        folder,
        std::path::Path::new(&appdata)
            .join("egui-concepts-persist")
            .join("data")
    );
}
