//! Vsync with the default `wgpu` renderer: set at start-up in `NativeOptions`, changed at
//! runtime through `Frame`.
//!
//! The window repaints continuously and counts its frames, so the effect shows as a number:
//! with vsync the frame rate stays at the monitor's refresh rate, without it the frame rate
//! goes as high as the machine can manage.
//!
//! Run with `cargo run -p concepts --example native_options_vsync`.

use eframe::egui;
use eframe::wgpu::PresentMode;

/// Vsync lives in the renderer's options, not in the `ViewportBuilder`.
fn main() -> eframe::Result {
    let mut native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Native options: vsync")
            .with_inner_size([420.0, 180.0]),
        ..Default::default()
    };
    // Start without vsync. The default is `AutoVsync`.
    native_options.wgpu_options.surface.present_mode = PresentMode::AutoNoVsync;

    eframe::run_native(
        "egui-concepts-vsync",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(VsyncDemo::default()))),
    )
}

#[derive(Default)]
struct VsyncDemo {
    /// Frames counted since `counting_since`.
    frames: u32,
    counting_since: f64,
    /// The last full count, in frames per second.
    frames_per_second: f64,
}

impl eframe::App for VsyncDemo {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.count_frame(ui.input(|input| input.time));
        // Keep repainting, so the frame rate is limited only by vsync and the machine.
        ui.ctx().request_repaint();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(format!("{:.0} frames per second", self.frames_per_second));
            // `None` would mean the renderer is not wgpu.
            if let Some(mut surface) = frame.wgpu_surface_config() {
                let mut vsync = surface.present_mode == PresentMode::AutoVsync;
                if ui.checkbox(&mut vsync, "Vsync").changed() {
                    surface.present_mode = if vsync {
                        PresentMode::AutoVsync
                    } else {
                        PresentMode::AutoNoVsync
                    };
                    // Takes effect before the next paint; no restart needed.
                    frame.set_wgpu_surface_config(surface);
                }
            }
            ui.label("With vsync on, the count settles at the monitor's refresh rate.");
        });
    }
}

impl VsyncDemo {
    /// Updates `frames_per_second` once per half second from the frames seen in that time.
    fn count_frame(&mut self, now: f64) {
        self.frames += 1;
        let elapsed = now - self.counting_since;
        if elapsed >= 0.5 {
            self.frames_per_second = f64::from(self.frames) / elapsed;
            self.frames = 0;
            self.counting_since = now;
        }
    }
}
