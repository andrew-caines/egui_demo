use eframe::egui;

use crate::model::{Priority, TodoAction};

/// The "new todo" input row.
///
/// This is a *stateful* component: it owns the half-typed draft, which is nobody
/// else's business, and only tells the outside world when the user commits it.
#[derive(Default)]
pub struct AddBar {
    draft_title: String,
    draft_priority: Priority,
    /// Set by `request_focus`, consumed the next time the bar is shown.
    focus_requested: bool,
}

impl AddBar {
    /// Moves keyboard focus to the title box on the next frame.
    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    /// Draws the bar. Returns `TodoAction::Add` on the frame the user submits.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<TodoAction> {
        // `Sides` pins one group of widgets to each edge. With `shrink_left` the right
        // group is laid out first and the left group gets whatever width remains,
        // which is how the text box stretches without us hard-coding any widths.
        let (title_response, add_clicked) = egui::Sides::new().shrink_left().show(
            ui,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.draft_title)
                        .hint_text("What needs doing?")
                        .desired_width(f32::INFINITY),
                )
                // The hint disappears once the user types, so name the box explicitly
                // for screen readers and UI tests.
                .accessible_name("New todo title")
            },
            |ui| {
                // The right side is filled right-to-left, so the button comes first.
                let add_clicked = ui.button("Add").clicked();

                // The salt only has to be unique among this widget's siblings.
                egui::ComboBox::from_id_salt("draft_priority")
                    .selected_text(self.draft_priority.label())
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for priority in Priority::ALL {
                            ui.selectable_value(
                                &mut self.draft_priority,
                                priority,
                                priority.label(),
                            );
                        }
                    })
                    .response
                    .accessible_name("Priority of the new todo");

                add_clicked
            },
        );

        if std::mem::take(&mut self.focus_requested) {
            title_response.request_focus();
        }

        // A single-line `TextEdit` surrenders focus when Enter is pressed.
        let submitted_with_enter =
            title_response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

        if !(add_clicked || submitted_with_enter) || self.draft_title.trim().is_empty() {
            return None;
        }

        // Keep the cursor in the box so several todos can be typed in a row.
        title_response.request_focus();

        Some(TodoAction::Add {
            // `take` moves the text out and leaves the draft empty for the next todo.
            title: std::mem::take(&mut self.draft_title),
            priority: self.draft_priority,
        })
    }
}
