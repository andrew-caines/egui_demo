use eframe::egui;

use super::PriorityBadge;
use crate::model::{Todo, TodoAction};

/// One line in the list.
///
/// This is a *composite* component: it arranges other widgets and translates what
/// the user did into a `TodoAction`. It borrows the todo immutably, so the compiler
/// guarantees that drawing a row can never modify the data.
pub struct TodoRow<'a> {
    todo: &'a Todo,
}

impl<'a> TodoRow<'a> {
    pub fn new(todo: &'a Todo) -> Self {
        Self { todo }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<TodoAction> {
        let todo = self.todo;
        let mut action = None;

        ui.horizontal(|ui| {
            // `checkbox` needs a `&mut bool`, so give it a copy and report the change.
            let mut done = todo.done;
            let checkbox = ui
                .checkbox(&mut done, "")
                // Nothing visible names this checkbox, so name it for screen
                // readers and UI tests.
                .accessible_name(format!("Done: {}", todo.title));
            if checkbox.changed() {
                action = Some(TodoAction::Toggle(todo.id));
            }

            ui.add(PriorityBadge::new(todo.priority));

            let mut title = egui::RichText::new(&todo.title);
            if todo.done {
                title = title.strikethrough().weak();
            }
            // Long titles are cut off with "…" rather than pushing the row wider.
            ui.add(egui::Label::new(title).truncate());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let delete_button = ui
                    .small_button("Delete")
                    // Every row has a "Delete" button; say which todo this one is for.
                    .accessible_name(format!("Delete: {}", todo.title));
                if delete_button.clicked() {
                    action = Some(TodoAction::Delete(todo.id));
                }
            });
        });

        action
    }
}
