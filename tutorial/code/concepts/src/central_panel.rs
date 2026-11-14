//! Examples for the `central-panel` concept page.
//!
//! Each example draws into a `ui` exactly as it would in `App::ui`. The gallery shows them in
//! a fixed-size stand-in for a window (see [`mock_window`]), because panels arrange themselves
//! inside whatever `Ui` they are given.

use eframe::egui;

use crate::{Demo, Example};

pub fn examples() -> Vec<Example> {
    vec![
        Example {
            concept: "central-panel",
            name: "Default central panel",
            when: "Use this when the main content should fill whatever space the other panels leave.",
            make: || Box::new(|ui: &mut egui::Ui| mock_window(ui, default_panel)),
        },
        Example {
            concept: "central-panel",
            name: "App shell",
            when: "Use this when the window has a header, a sidebar and a status bar around the main area.",
            make: || Box::new(|ui: &mut egui::Ui| mock_window(ui, app_shell)),
        },
        Example {
            concept: "central-panel",
            name: "Panel order",
            when: "Use this when deciding which panel gets the corners, or when a panel has vanished.",
            make: || Box::new(PanelOrder::default()),
        },
        Example {
            concept: "central-panel",
            name: "Full-bleed canvas",
            when: "Use this when you paint the whole area yourself: a canvas, a map, an image viewer.",
            make: || Box::new(|ui: &mut egui::Ui| mock_window(ui, full_bleed_canvas)),
        },
        Example {
            concept: "central-panel",
            name: "Custom background and margin",
            when: "Use this when the main area needs its own colour or more room around the content.",
            make: || Box::new(|ui: &mut egui::Ui| mock_window(ui, custom_frame)),
        },
        Example {
            concept: "central-panel",
            name: "Content taller than the window",
            when: "Use this when the content can be longer than the space the panel gets.",
            make: || Box::new(|ui: &mut egui::Ui| mock_window(ui, scrolling_content)),
        },
    ]
}

/// The size of the stand-in window the gallery draws each example into.
const MOCK_WINDOW_SIZE: egui::Vec2 = egui::vec2(360.0, 200.0);

/// Gives `add_contents` a fixed-size region with a border, clipped like a real window, so the
/// panels inside it take the same shape they would take in a window of that size.
fn mock_window(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    let (rect, _response) = ui.allocate_exact_size(MOCK_WINDOW_SIZE, egui::Sense::hover());
    let mut window_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    window_ui.shrink_clip_rect(rect);
    add_contents(&mut window_ui);
    ui.painter().rect_stroke(
        rect,
        0.0,
        ui.visuals().window_stroke(),
        egui::StrokeKind::Outside,
    );
}

/// The Hello World form: one central panel, nothing else.
fn default_panel(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Central panel");
        ui.label("It fills the space left over and adds a margin and a background.");
    });
}

/// The shape of most desktop apps. Every `Panel` takes a strip off the edge of what is left;
/// the central panel, declared last, gets the rest.
fn app_shell(ui: &mut egui::Ui) {
    egui::Panel::top("header").show(ui, |ui| {
        ui.label("Header");
    });
    egui::Panel::bottom("status_bar").show(ui, |ui| {
        ui.label("Status bar");
    });
    egui::Panel::left("sidebar")
        .default_size(110.0)
        .show(ui, |ui| {
            ui.label("Sidebar");
        });
    egui::CentralPanel::default().show(ui, |ui| {
        ui.label("Central panel: everything that is left.");
    });
}

/// Which panel is declared first.
#[derive(Clone, Copy, Default, PartialEq)]
enum First {
    #[default]
    Header,
    Sidebar,
    CentralPanel,
}

/// Draws the same three panels in an order picked with radio buttons.
#[derive(Default)]
struct PanelOrder {
    first: First,
}

impl Demo for PanelOrder {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.first, First::Header, "Header first");
            ui.radio_value(&mut self.first, First::Sidebar, "Sidebar first");
            ui.radio_value(&mut self.first, First::CentralPanel, "Central panel first");
        });
        let first = self.first;
        mock_window(ui, |ui| panels_in_order(ui, first));
    }
}

/// The panel declared first is the outermost, so it runs the full length of its edge.
fn panels_in_order(ui: &mut egui::Ui, first: First) {
    let header = |ui: &mut egui::Ui| {
        egui::Panel::top("header").show(ui, |ui| {
            ui.label("Header");
        });
    };
    let sidebar = |ui: &mut egui::Ui| {
        egui::Panel::left("sidebar")
            .default_size(110.0)
            .show(ui, |ui| {
                ui.label("Sidebar");
            });
    };
    let central = |ui: &mut egui::Ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Central panel");
        });
    };
    match first {
        First::Header => {
            header(ui);
            sidebar(ui);
            central(ui);
        }
        First::Sidebar => {
            sidebar(ui);
            header(ui);
            central(ui);
        }
        // Wrong: the central panel takes everything, and the two panels after it get no space.
        First::CentralPanel => {
            central(ui);
            header(ui);
            sidebar(ui);
        }
    }
}

/// No frame means no margin and no background: the painting reaches every edge.
fn full_bleed_canvas(ui: &mut egui::Ui) {
    egui::CentralPanel::no_frame().show(ui, |ui| {
        let rect = ui.max_rect();
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
        let grid_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        let spacing = 20.0;
        let mut x = rect.left();
        while x < rect.right() {
            painter.vline(x, rect.y_range(), grid_stroke);
            x += spacing;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.hline(rect.x_range(), y, grid_stroke);
            y += spacing;
        }
        ui.label("The grid starts at the very edge.");
    });
}

/// Start from the default frame and change only what you need, so the rest still follows the
/// theme.
fn custom_frame(ui: &mut egui::Ui) {
    let frame = egui::Frame::central_panel(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(24);
    egui::CentralPanel::default().frame(frame).show(ui, |ui| {
        ui.heading("Document");
        ui.label("A different background, and a 24-point margin instead of 8.");
    });
}

/// A central panel clips what does not fit. A `ScrollArea` inside it makes the rest reachable.
fn scrolling_content(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            // Fill the panel, so the scroll bar sits at its right edge rather than next to
            // the text.
            .auto_shrink(false)
            .show(ui, |ui| {
                for line in 1..=40 {
                    ui.label(format!("Line {line}"));
                }
            });
    });
}
