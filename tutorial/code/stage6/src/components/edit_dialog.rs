use eframe::egui;

use crate::model::{Priority, Todo, TodoAction, TodoId};

/// What happened to the dialog this frame.
pub enum DialogOutcome {
    /// Still open; keep showing it.
    Open,
    /// Closed without saving.
    Cancelled,
    /// Closed with changes to apply.
    Saved(TodoAction),
}

/// A modal editor for one todo.
///
/// The dialog edits its *own copy* of the fields. The real todo is untouched until
/// the user presses Save, which makes Cancel trivial: just drop the dialog.
pub struct EditDialog {
    id: TodoId,
    title: String,
    notes: String,
    priority: Priority,
    /// True until the first frame has been shown, to focus the title box once.
    focus_title: bool,
}

impl EditDialog {
    pub fn new(todo: &Todo) -> Self {
        Self {
            id: todo.id,
            title: todo.title.clone(),
            notes: todo.notes.clone(),
            priority: todo.priority,
            focus_title: true,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> DialogOutcome {
        let can_save = !self.title.trim().is_empty();
        let mut save_requested = false;
        let mut cancel_requested = false;

        // A `Modal` is drawn above everything else and blocks input to the rest of
        // the window. It takes the `Context` because it is not placed inside a parent
        // `Ui`; it floats over the whole viewport.
        let modal = egui::Modal::new(egui::Id::unique("edit_todo_dialog")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading("Edit todo");
            ui.add_space(8.0);

            egui::Grid::new("edit_todo_fields")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    let title_label = ui.label("Title");
                    let title_response = ui
                        .add(egui::TextEdit::singleline(&mut self.title).desired_width(280.0))
                        .labelled_by(title_label.id);
                    if std::mem::take(&mut self.focus_title) {
                        title_response.request_focus();
                    }
                    // Enter in the title box is a shortcut for Save.
                    if title_response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        save_requested = true;
                    }
                    ui.end_row();

                    ui.label("Priority");
                    ui.horizontal(|ui| {
                        for priority in Priority::ALL {
                            ui.selectable_value(&mut self.priority, priority, priority.label());
                        }
                    });
                    ui.end_row();

                    let notes_label = ui.label("Notes");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.notes)
                            .desired_rows(4)
                            .desired_width(280.0),
                    )
                    .labelled_by(notes_label.id);
                    ui.end_row();
                });

            ui.separator();

            egui::Sides::new().show(
                ui,
                |_left| {},
                |ui| {
                    if ui
                        .add_enabled(can_save, egui::Button::new("Save"))
                        .clicked()
                    {
                        save_requested = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel_requested = true;
                    }
                },
            );
        });

        if save_requested && can_save {
            DialogOutcome::Saved(TodoAction::Update {
                id: self.id,
                title: self.title.clone(),
                notes: self.notes.clone(),
                priority: self.priority,
            })
        } else if cancel_requested || modal.should_close() {
            // `should_close` is true when Escape is pressed or the backdrop is clicked.
            DialogOutcome::Cancelled
        } else {
            DialogOutcome::Open
        }
    }
}
