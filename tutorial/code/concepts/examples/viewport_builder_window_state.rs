//! Maximized, minimized and fullscreen: start-up values in `ViewportBuilder`, changes with
//! `ViewportCommand`, and the current state read back from the viewport's info.
//!
//! Run with `cargo run -p concepts --example viewport_builder_window_state`.

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Viewport builder: window state")
            .with_inner_size([440.0, 200.0])
            // `true` here would open the window maximized, or fullscreen.
            .with_maximized(false)
            .with_fullscreen(false),
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-window-state",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(WindowStateDemo))),
    )
}

struct WindowStateDemo;

impl eframe::App for WindowStateDemo {
    /// Read the window's state, then ask for a new one
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // What the operating system reports. `None` means "not known yet".
        let (maximized, fullscreen) = ui.input(|input| {
            let viewport = input.viewport();
            (
                viewport.maximized.unwrap_or(false),
                viewport.fullscreen.unwrap_or(false),
            )
        });
        // A fullscreen window has no title bar, so always give a way out from the keyboard.
        if fullscreen && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(format!("Maximized: {maximized}, fullscreen: {fullscreen}"));
            ui.horizontal(|ui| {
                let maximize = if maximized { "Restore" } else { "Maximize" };
                if ui.button(maximize).clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                }
                let full = if fullscreen {
                    "Leave fullscreen (Esc)"
                } else {
                    "Fullscreen"
                };
                if ui.button(full).clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                }
                if ui.button("Minimize").clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            });
        });
    }
}
