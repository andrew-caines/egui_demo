//! Examples for the `response` concept page.
//!
//! Every widget call returns a `Response`. Each example asks it what happened on the same line
//! the widget is drawn, and keeps what it learns in its own fields.

use eframe::egui;

use crate::{Demo, Example};

pub fn examples() -> Vec<Example> {
    vec![
        Example {
            concept: "response",
            name: "Click counter",
            when: "Use this when something should happen when a button is clicked.",
            make: || Box::new(ClickCounter::default()),
        },
        Example {
            concept: "response",
            name: "Clickable label",
            when: "Use this when a label, or a container around several widgets, should react to a click.",
            make: || Box::new(ClickableLabel::default()),
        },
        Example {
            concept: "response",
            name: "Double-click and right-click",
            when: "Use this when an item should react to a double-click, or offer a menu on a right-click.",
            make: || Box::new(DoubleAndRightClick::default()),
        },
        Example {
            concept: "response",
            name: "Hover and tooltip",
            when: "Use this when a widget should explain itself on hover, or you need its size and position.",
            make: || Box::new(hover_and_tooltip),
        },
        Example {
            concept: "response",
            name: "Typing and Enter",
            when: "Use this when you react to edits in a text field, or submit it with Enter.",
            make: || Box::new(TypingAndEnter::default()),
        },
        Example {
            concept: "response",
            name: "Focus",
            when: "Use this when a button or a shortcut should put the keyboard focus in a field.",
            make: || Box::new(Focus::default()),
        },
        Example {
            concept: "response",
            name: "Drag",
            when: "Use this when something should follow the pointer while it is dragged.",
            make: || Box::new(Drag::default()),
        },
        Example {
            concept: "response",
            name: "Several widgets as one",
            when: "Use this when a group of widgets should be asked one question, such as whether any of them changed.",
            make: || Box::new(SeveralAsOne::default()),
        },
    ]
}

/// Counts clicks on a button.
#[derive(Default)]
struct ClickCounter {
    clicks: u32,
}

impl Demo for ClickCounter {
    fn show(&mut self, ui: &mut egui::Ui) {
        // The button is drawn, and its response asked, in one expression.
        if ui.button("Add one").clicked() {
            self.clicks += 1;
        }
        ui.label(format!("Clicks: {}", self.clicks));
    }
}

/// A label that asks for clicks when it is built, and a card whose response has click
/// sensing added after it was drawn.
#[derive(Default)]
struct ClickableLabel {
    opened: u32,
    archived: u32,
}

impl Demo for ClickableLabel {
    fn show(&mut self, ui: &mut egui::Ui) {
        // `sense` asks the label for clicks, and gives it a hover effect like a button's.
        let open = egui::Label::new("Open the report").sense(egui::Sense::click());
        if ui.add(open).clicked() {
            self.opened += 1;
        }
        ui.label(format!("Opened: {}", self.opened));

        // A container senses nothing. `interact` adds a sense to the response it returned,
        // so a click anywhere on the card counts. It adds no hover effect.
        let card = egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label("Archive this card");
        });
        if card.response.interact(egui::Sense::click()).clicked() {
            self.archived += 1;
        }
        ui.label(format!("Archived: {}", self.archived));
    }
}

/// One item that reports clicks, double-clicks and right-clicks, and has a context menu.
#[derive(Default)]
struct DoubleAndRightClick {
    clicks: u32,
    last: Option<&'static str>,
}

impl Demo for DoubleAndRightClick {
    fn show(&mut self, ui: &mut egui::Ui) {
        let item = ui.add(egui::Label::new("Report.txt").sense(egui::Sense::click()));
        if item.clicked() {
            self.clicks += 1;
            self.last = Some("clicked");
        }
        // A double-click is two clicks: the second one is also `clicked()`.
        if item.double_clicked() {
            self.last = Some("double-clicked");
        }
        if item.secondary_clicked() {
            self.last = Some("right-clicked");
        }
        // Opens on a right-click (or a long press on a touch screen).
        item.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                self.last = Some("chose Rename");
                ui.close();
            }
            if ui.button("Delete").clicked() {
                self.last = Some("chose Delete");
                ui.close();
            }
        });
        ui.label(format!("Clicks: {}", self.clicks));
        ui.label(format!("Last: {}", self.last.unwrap_or("nothing yet")));
    }
}

/// A tooltip, a hover check and the button's rectangle, all from one response.
fn hover_and_tooltip(ui: &mut egui::Ui) {
    let button = ui
        .button("Hover over me")
        .on_hover_text("A tooltip appears after the pointer rests here.");
    if button.hovered() {
        ui.label("The pointer is over the button.");
    } else {
        ui.label("The pointer is elsewhere.");
    }
    let rect = button.rect;
    ui.label(format!(
        "The button is {:.0} by {:.0} points, its top left at ({:.0}, {:.0}).",
        rect.width(),
        rect.height(),
        rect.left(),
        rect.top()
    ));
}

/// Counts the frames on which the text changed, and submits it when Enter is pressed.
#[derive(Default)]
struct TypingAndEnter {
    query: String,
    frames_changed: u32,
    submitted: Option<String>,
}

impl Demo for TypingAndEnter {
    fn show(&mut self, ui: &mut egui::Ui) {
        let field = ui
            .text_edit_singleline(&mut self.query)
            .accessible_name("Search");
        // `changed()` is set by the widget that edited the text, on that frame only.
        if field.changed() {
            self.frames_changed += 1;
        }
        // A single-line field gives up focus on Enter, so check which key it was.
        if field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            self.submitted = Some(self.query.clone());
        }
        ui.label(format!("Frames with a change: {}", self.frames_changed));
        if let Some(query) = &self.submitted {
            ui.label(format!("Searched for: {query}"));
        }
    }
}

/// A button that moves the keyboard focus into a text field.
#[derive(Default)]
struct Focus {
    name: String,
}

impl Demo for Focus {
    fn show(&mut self, ui: &mut egui::Ui) {
        let focus_clicked = ui.button("Focus the name field").clicked();
        let field = ui
            .add(egui::TextEdit::singleline(&mut self.name).hint_text("Your name"))
            .accessible_name("Name");
        if focus_clicked {
            field.request_focus();
        }
        if field.has_focus() {
            ui.label("The name field has focus.");
        } else {
            ui.label("The name field does not have focus.");
        }
    }
}

/// A square that follows the pointer while it is dragged, inside a fixed area.
#[derive(Default)]
struct Drag {
    offset: egui::Vec2,
}

impl Demo for Drag {
    fn show(&mut self, ui: &mut egui::Ui) {
        let square_size = egui::vec2(40.0, 40.0);
        let (area, _) = ui.allocate_exact_size(egui::vec2(280.0, 120.0), egui::Sense::hover());
        let square = egui::Rect::from_min_size(area.min + self.offset, square_size);
        let response = ui
            .interact(square, ui.make_persistent_id("square"), egui::Sense::drag())
            .accessible_name("Draggable square");
        // `drag_delta` is how far the pointer moved since last frame, zero when not dragged.
        if response.dragged() {
            let largest = area.size() - square_size;
            self.offset = (self.offset + response.drag_delta()).clamp(egui::Vec2::ZERO, largest);
        }

        let visuals = ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect_stroke(
            area,
            4.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let moved = egui::Rect::from_min_size(area.min + self.offset, square_size);
        painter.rect_filled(moved, 4.0, visuals.bg_fill);
        painter.rect_stroke(moved, 4.0, visuals.fg_stroke, egui::StrokeKind::Inside);
        ui.label(format!(
            "Offset: {:.0}, {:.0}. Dragged: {}.",
            self.offset.x,
            self.offset.y,
            response.dragged()
        ));
    }
}

/// Two number fields asked as one with `|`, inside a row whose closure returns the result.
#[derive(Default)]
struct SeveralAsOne {
    x: f32,
    y: f32,
    frames_changed: u32,
}

impl Demo for SeveralAsOne {
    fn show(&mut self, ui: &mut egui::Ui) {
        // The closure's value comes back as `inner`; `response` covers the whole row.
        let point = ui.horizontal(|ui| {
            ui.label("Point:");
            let x = ui
                .add(egui::DragValue::new(&mut self.x))
                .accessible_name("x");
            let y = ui
                .add(egui::DragValue::new(&mut self.y))
                .accessible_name("y");
            x | y
        });
        if point.inner.changed() {
            self.frames_changed += 1;
        }
        point
            .response
            .on_hover_text("Drag a number sideways, or click it to type.");
        ui.label(format!("Frames with a change: {}", self.frames_changed));
    }
}
