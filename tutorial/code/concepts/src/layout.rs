//! Examples for the `layout` concept page.
//!
//! Each example draws straight into the `ui` it is given, as code inside a panel would.

use eframe::egui;

use crate::{Demo, Example};

pub fn examples() -> Vec<Example> {
    vec![
        Example {
            concept: "layout",
            name: "Rows and columns",
            when: "Use this when widgets should sit side by side, or a row should hold columns.",
            make: || Box::new(rows_and_columns),
        },
        Example {
            concept: "layout",
            name: "Buttons on the right",
            when: "Use this when some widgets belong at the left of a row and others at the right.",
            make: || Box::new(buttons_on_the_right),
        },
        Example {
            concept: "layout",
            name: "Title left, buttons right",
            when: "Use this when a row has a title at the left edge and buttons at the right edge.",
            make: || Box::new(title_left_buttons_right),
        },
        Example {
            concept: "layout",
            name: "Wrapping row",
            when: "Use this when a row holds more items than fit, and they should flow onto new lines.",
            make: || Box::new(wrapping_row),
        },
        Example {
            concept: "layout",
            name: "Fixed-height bar",
            when: "Use this when a strip should have a set height with its contents centred in it.",
            make: || Box::new(fixed_height_bar),
        },
        Example {
            concept: "layout",
            name: "Full-width buttons",
            when: "Use this when every widget in a column should be as wide as the column.",
            make: || Box::new(full_width_buttons),
        },
        Example {
            concept: "layout",
            name: "Equal columns",
            when: "Use this when the space should be split into columns of the same width.",
            make: || Box::new(equal_columns),
        },
        Example {
            concept: "layout",
            name: "Form grid",
            when: "Use this when labels and fields should line up in columns, as in a form.",
            make: || Box::new(FormGrid::default()),
        },
        Example {
            concept: "layout",
            name: "Centre content of unknown width",
            when: "Use this when a group of widgets should be centred, and its width is not known until it is drawn.",
            make: || Box::new(centre_unknown_width),
        },
    ]
}

/// A row is a child `Ui` with a left-to-right layout; a column one with a top-down layout.
fn rows_and_columns(ui: &mut egui::Ui) {
    // A row: widgets left to right, centred vertically.
    ui.horizontal(|ui| {
        ui.label("A row:");
        let _ = ui.button("One");
        let _ = ui.button("Two");
        let _ = ui.button("Three");
    });
    // A row of two columns. `horizontal_top` lines the columns up at the top.
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label("Column one");
            let _ = ui.button("Above");
            let _ = ui.button("Below");
        });
        ui.vertical(|ui| {
            ui.label("Column two");
            let _ = ui.button("Only one");
        });
    });
}

/// A label on the left, buttons pushed to the right edge. A right-to-left layout starts at
/// the right edge, so the first widget added ends up furthest right.
fn buttons_on_the_right(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("3 items selected");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = ui.button("Delete"); // added first: at the right edge
            let _ = ui.button("Archive"); // added second: to the left of Delete
        });
    });
}

/// A title pinned to the left edge and buttons pinned to the right edge of one row. The right
/// side is laid out right to left, so its first widget ends up furthest right.
fn title_left_buttons_right(ui: &mut egui::Ui) {
    egui::Sides::new().show(
        ui,
        |ui| {
            ui.heading("Inbox");
        },
        |ui| {
            let _ = ui.button("Delete"); // added first: at the right edge
            let _ = ui.button("Archive"); // added second: to the left of Delete
        },
    );
}

/// A row that starts a new line when the next item would cross the right edge. The gap
/// between items is the item spacing, changed here for this section only.
fn wrapping_row(ui: &mut egui::Ui) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
        ui.horizontal_wrapped(|ui| {
            let tags = "rust gui egui eframe immediate-mode layout flexbox wrapping \
                        desktop web tutorial widgets";
            for tag in tags.split_whitespace() {
                let _ = ui.button(tag);
            }
        });
    });
}

/// Ask for a region of a set size with a layout of your choice. The left-to-right layout
/// centres its widgets vertically in the full 48 points.
fn fixed_height_bar(ui: &mut egui::Ui) {
    let size = egui::vec2(ui.available_width(), 48.0);
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    let bar = ui.allocate_ui_with_layout(size, layout, |ui| {
        ui.heading("Inbox");
        let _ = ui.button("Refresh");
    });
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter()
        .rect_stroke(bar.response.rect, 0.0, stroke, egui::StrokeKind::Inside);
    ui.label(format!(
        "The bar above is {:.0} points tall.",
        bar.response.rect.height()
    ));
}

/// A justified top-down layout stretches every widget across the column.
fn full_width_buttons(ui: &mut egui::Ui) {
    ui.scope(|ui| {
        ui.set_max_width(200.0);
        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                let _ = ui.button("New");
                let _ = ui.button("Open recent");
                let _ = ui.button("Settings");
            },
        );
    });
}

/// `columns` splits the width into equal parts, each a top-down justified `Ui`.
fn equal_columns(ui: &mut egui::Ui) {
    ui.columns(3, |columns| {
        columns[0].label("To do");
        let _ = columns[0].button("Write the page");
        let _ = columns[0].button("Take screenshots");
        columns[1].label("Doing");
        let _ = columns[1].button("Write the tests");
        columns[2].label("Done");
    });
}

/// A grid lines its cells up in columns as wide as their widest cell.
#[derive(Default)]
struct FormGrid {
    name: String,
    email: String,
    newsletter: bool,
}

impl Demo for FormGrid {
    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("form")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                let name_label = ui.label("Name");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
                ui.end_row();

                let email_label = ui.label("Email address");
                ui.text_edit_singleline(&mut self.email)
                    .labelled_by(email_label.id);
                ui.end_row();

                ui.label("Newsletter");
                ui.checkbox(&mut self.newsletter, "Send me the newsletter");
                ui.end_row();
            });
    }
}

/// A centred column centres each widget by its own size, known when it is added. A row is
/// one region as wide as the space, so it stays at the left. `Aligned` measures the row on a
/// first, invisible pass and centres it by that size.
fn centre_unknown_width(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("A centred heading");
        ui.label(
            "Each widget in a centred column is centred on its own. Text that is too long \
             for one line, like this sentence, wraps, and each of its lines is centred.",
        );
        let _ = ui.button("A centred button");
        // Not centred: the row takes the full width, and its widgets start at the left.
        ui.horizontal(|ui| {
            let _ = ui.button("First");
            ui.label("Not centred");
            let _ = ui.button("Last");
        });
    });
    ui.separator();
    egui::Aligned::new(egui::Align2::CENTER_TOP).show(ui, |ui| {
        ui.horizontal(|ui| {
            let _ = ui.button("Previous");
            ui.label("Page 3 of 7");
            let _ = ui.button("Next");
        });
    });
}
