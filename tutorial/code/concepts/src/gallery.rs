//! The gallery: a list of every example on the left, the chosen one on the right.

use eframe::egui;

use crate::{Demo, Example, all};

pub struct Gallery {
    examples: Vec<Example>,
    /// Index into `examples` of the example being shown, with its live state.
    selected: Option<(usize, Box<dyn Demo>)>,
}

impl Default for Gallery {
    fn default() -> Self {
        let mut gallery = Self {
            examples: all(),
            selected: None,
        };
        if let Some(first) = gallery.examples.first().map(|example| example.name) {
            gallery.select(first);
        }
        gallery
    }
}

impl Gallery {
    /// Shows the named example. The demo is built once here, so its state survives repaints;
    /// choosing a different example and coming back builds a fresh one.
    pub fn select(&mut self, name: &str) {
        let Some(index) = self
            .examples
            .iter()
            .position(|example| example.name == name)
        else {
            return;
        };
        if self
            .selected
            .as_ref()
            .is_some_and(|(current, _)| *current == index)
        {
            return;
        }
        self.selected = Some((index, (self.examples[index].make)()));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;

        egui::Panel::left("example_list")
            .resizable(true)
            .default_size(200.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut last_concept = None;
                    for (index, example) in self.examples.iter().enumerate() {
                        if last_concept != Some(example.concept) {
                            last_concept = Some(example.concept);
                            ui.heading(example.concept);
                        }
                        let is_selected = self
                            .selected
                            .as_ref()
                            .is_some_and(|(current, _)| *current == index);
                        if ui.selectable_label(is_selected, example.name).clicked() {
                            chosen = Some(example.name);
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some((index, demo)) = self.selected.as_mut() else {
                ui.label("No examples.");
                return;
            };
            let example = &self.examples[*index];
            ui.label(example.when);
            ui.separator();
            // The name salts every id, the scroll area's included, so two examples never share
            // state (a scroll position, a text box) by accident.
            egui::ScrollArea::both()
                .id_salt(example.name)
                .show(ui, |ui| {
                    ui.push_id(example.name, |ui| demo.show(ui));
                });
        });

        if let Some(name) = chosen {
            self.select(name);
        }
    }
}
