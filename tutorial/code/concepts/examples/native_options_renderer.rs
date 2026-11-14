//! Which renderer eframe uses, and which graphics adapter it picked.
//!
//! With eframe's default features only `wgpu` is compiled in, so `Renderer::Wgpu` is the only
//! choice. `Renderer::Glow` exists only when the `glow` cargo feature of eframe is turned on.
//!
//! Run with `cargo run -p concepts --example native_options_renderer`.

use eframe::egui;

/// Choosing the renderer, and the settings that only matter if you draw 3D yourself.
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Native options: renderer")
            .with_inner_size([460.0, 200.0]),
        // The default already; spelled out because it is the point of this example.
        renderer: eframe::Renderer::Wgpu,
        // egui needs neither of these. They are for your own 3D drawing.
        multisampling: 0,
        depth_buffer: 0,
        ..Default::default()
    };
    eframe::run_native(
        "egui-concepts-renderer",
        native_options,
        Box::new(|creation_context| {
            // The render state exists by the time the creator closure runs.
            let adapter = creation_context
                .wgpu_render_state
                .as_ref()
                .map(|render_state| render_state.adapter.get_info());
            Ok(Box::new(RendererDemo { adapter }))
        }),
    )
}

struct RendererDemo {
    adapter: Option<eframe::wgpu::AdapterInfo>,
}

impl eframe::App for RendererDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Renderer: wgpu");
            match &self.adapter {
                Some(info) => {
                    // `info.name` is the GPU's model name. It is left out here so the
                    // screenshot on the page carries no machine-specific detail.
                    ui.label(format!("Graphics API: {:?}", info.backend));
                    ui.label(format!("Device type: {:?}", info.device_type));
                }
                None => {
                    ui.label("No wgpu render state: another renderer is in use.");
                }
            }
        });
    }
}
