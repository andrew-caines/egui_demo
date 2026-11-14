//! Examples for the `ui` concept page.
//!
//! The module is not called `ui` because every function here already has a parameter of that
//! name. Each example draws straight into the `ui` it is given, as code inside a panel would.

use eframe::egui;

use crate::{Demo, Example};

pub fn examples() -> Vec<Example> {
    vec![
        Example {
            concept: "ui",
            name: "The cursor",
            when: "Use this when you want to see where the next widget will go, and why.",
            make: || Box::new(the_cursor),
        },
        Example {
            concept: "ui",
            name: "Fill the available width",
            when: "Use this when you draw something yourself and it should be as wide as the space left.",
            make: || Box::new(fill_available_width),
        },
        Example {
            concept: "ui",
            name: "Reserve a region",
            when: "Use this when a group of widgets should get at most a given size, and take only what it uses.",
            make: || Box::new(reserve_region),
        },
        Example {
            concept: "ui",
            name: "Extra space",
            when: "Use this when two widgets need more room between them than the usual spacing.",
            make: || Box::new(extra_space),
        },
        Example {
            concept: "ui",
            name: "Width limits",
            when: "Use this when a box must be at least, or at most, a given width.",
            make: || Box::new(width_limits),
        },
        Example {
            concept: "ui",
            name: "Disabled widgets",
            when: "Use this when a widget or a whole section should be visible but not usable yet.",
            make: || Box::new(DisabledWidgets::default()),
        },
        Example {
            concept: "ui",
            name: "Scoped style",
            when: "Use this when a few widgets need different spacing or colours from the rest.",
            make: || Box::new(scoped_style),
        },
        Example {
            concept: "ui",
            name: "Repeated widgets",
            when: "Use this when the same stateful widget appears more than once, such as in a loop.",
            make: || Box::new(repeated_widgets),
        },
    ]
}

/// Widgets go one below the other. After each one, the cursor (drawn as a line) waits
/// `item_spacing.y` below it, and the next widget starts there.
fn the_cursor(ui: &mut egui::Ui) {
    ui.heading("A heading");
    mark_cursor(ui);
    ui.label("A label. It is placed at the cursor, and the cursor moves below it.");
    mark_cursor(ui);
    let _ = ui.button("A button");
    mark_cursor(ui);
    let left = ui.available_size();
    ui.label(format!("Space left: {:.0} by {:.0}.", left.x, left.y));
}

/// Paints a line across the `Ui` where the next widget will start. Painting allocates no
/// space, so it does not move the cursor.
fn mark_cursor(ui: &mut egui::Ui) {
    let y = ui.cursor().top();
    let stroke = egui::Stroke::new(1.0, ui.visuals().warn_fg_color);
    ui.painter().hline(ui.max_rect().x_range(), y, stroke);
    ui.painter().text(
        egui::pos2(ui.max_rect().right(), y),
        egui::Align2::RIGHT_BOTTOM,
        "cursor",
        egui::TextStyle::Small.resolve(ui.style()),
        ui.visuals().warn_fg_color,
    );
}

/// Read how much width is left, ask for exactly that, then paint into the rectangle you get.
fn fill_available_width(ui: &mut egui::Ui) {
    ui.label("A bar as wide as the space left:");
    let size = egui::vec2(ui.available_width(), 16.0);
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
    ui.label(format!("available_width() was {:.0} points.", size.x));
}

/// `allocate_ui` offers its contents a region of at most this size. Afterwards it takes only
/// the space the contents used, and the next widget follows that.
fn reserve_region(ui: &mut egui::Ui) {
    let offered_size = egui::vec2(220.0, 120.0);
    let offered = egui::Rect::from_min_size(ui.cursor().min, offered_size);
    let region = ui.allocate_ui(offered_size, |ui| {
        ui.label("This child Ui was offered 220 by 120 points.");
        let _ = ui.button("A button inside");
    });
    let used = region.response.rect;
    ui.label(format!(
        "It used {:.0} by {:.0}, so this label comes straight after.",
        used.width(),
        used.height()
    ));
    outline_offered_and_used(ui, offered, used);
}

/// Outlines what was offered (thin) and what was used (strong). Inside strokes, so the edges
/// against the pane are not clipped.
fn outline_offered_and_used(ui: &egui::Ui, offered: egui::Rect, used: egui::Rect) {
    let painter = ui.painter();
    let thin = ui.visuals().widgets.noninteractive.bg_stroke;
    let strong = egui::Stroke::new(1.5, ui.visuals().warn_fg_color);
    painter.rect_stroke(offered, 0.0, thin, egui::StrokeKind::Inside);
    painter.rect_stroke(used, 0.0, strong, egui::StrokeKind::Inside);
}

/// `add_space` moves the cursor on by that many points, on top of the item spacing that
/// already follows every widget.
fn extra_space(ui: &mut egui::Ui) {
    // `horizontal_top`, so both columns start at the top and the gaps line up.
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label("Without add_space");
            ui.label("The next line");
        });
        // In a horizontal layout the space is horizontal.
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.label("With add_space(20.0)");
            ui.add_space(20.0);
            ui.label("The next line, 20 points lower");
        });
    });
}

/// `set_min_width` makes a `Ui` at least that wide. `set_max_width` narrows the space that
/// widgets added after it may use, so text wraps.
fn width_limits(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.set_min_width(240.0);
        ui.label("Short text.");
    });
    ui.group(|ui| {
        ui.set_max_width(240.0);
        ui.label(
            "This text is too long for 240 points, so it wraps inside the group \
             instead of running on across the pane.",
        );
    });
}

/// One button and one section, each enabled only when its checkbox is ticked.
#[derive(Default)]
struct DisabledWidgets {
    read_terms: bool,
    continued: bool,
    allow_changes: bool,
    compact_rows: bool,
}

impl Demo for DisabledWidgets {
    fn show(&mut self, ui: &mut egui::Ui) {
        // One widget: `add_enabled` greys it out and ignores clicks.
        ui.checkbox(&mut self.read_terms, "I have read the terms");
        if ui
            .add_enabled(self.read_terms, egui::Button::new("Continue"))
            .clicked()
        {
            self.continued = true;
        }
        if self.continued {
            ui.label("You clicked Continue.");
        }
        ui.separator();
        // A whole section: everything added inside the closure is disabled.
        ui.checkbox(&mut self.allow_changes, "Allow changes");
        ui.add_enabled_ui(self.allow_changes, |ui| {
            ui.checkbox(&mut self.compact_rows, "Compact rows");
            if ui.button("Reset").clicked() {
                self.compact_rows = false;
            }
        });
    }
}

/// Style changes inside `scope` apply to that child `Ui` only, and end with the closure.
fn scoped_style(ui: &mut egui::Ui) {
    let _ = ui.button("Normal button");
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(16.0, 8.0);
        ui.visuals_mut().override_text_color = Some(ui.visuals().warn_fg_color);
        let _ = ui.button("Bigger button");
        ui.label("Text in the warning colour");
    });
    let _ = ui.button("Normal again");
}

/// Each collapsing header keeps its open state under an id made from its title. `push_id`
/// gives every row its own id scope, so three headers called "Details" do not clash.
fn repeated_widgets(ui: &mut egui::Ui) {
    for folder in ["Inbox", "Archive", "Spam"] {
        // Salt with something that stays with the item, not its position in the list.
        ui.push_id(folder, |ui| {
            ui.label(folder);
            ui.collapsing("Details", |ui| {
                ui.label(format!("Details of {folder}."));
            });
        });
    }
}
