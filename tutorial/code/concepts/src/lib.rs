//! Every code example shown on the concept pages lives in this crate, so each one is compiled,
//! linted and rendered headlessly by the tests.
//!
//! One module per concept page. A module exposes `pub fn examples() -> Vec<Example>`, and
//! [`all`] concatenates them. The gallery binary lists every example.

use eframe::egui;

pub mod app_trait;
pub mod central_panel;
pub mod gallery;
pub mod layout;
pub mod response;
pub mod ui_region;

/// A small piece of UI that can be drawn into any `ui`.
pub trait Demo {
    fn show(&mut self, ui: &mut egui::Ui);
}

/// Any closure over a `ui` is a demo; state lives in what the closure captures.
impl<F: FnMut(&mut egui::Ui)> Demo for F {
    fn show(&mut self, ui: &mut egui::Ui) {
        self(ui);
    }
}

/// One entry in the gallery.
pub struct Example {
    /// The concept page this example belongs to, such as `"central-panel"`.
    pub concept: &'static str,
    /// Unique within the crate; shown in the gallery list.
    pub name: &'static str,
    /// The one-line "use this when" shown above the example.
    pub when: &'static str,
    /// Builds a fresh demo with its initial state.
    pub make: fn() -> Box<dyn Demo>,
}

/// Every example in the crate, in page order.
pub fn all() -> Vec<Example> {
    let mut examples = Vec::new();
    examples.extend(central_panel::examples());
    examples.extend(ui_region::examples());
    examples.extend(app_trait::examples());
    examples.extend(layout::examples());
    examples.extend(response::examples());
    examples
}
