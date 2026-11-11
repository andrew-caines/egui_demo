use eframe::egui;
use egui_extras::{Column, TableBuilder};

use super::{PriorityBadge, todo_context_menu};
use crate::model::{Todo, TodoAction};

/// A denser, spreadsheet-like view of the same todos.
///
/// It reuses `PriorityBadge` and `todo_context_menu`, and returns the same
/// `TodoAction`s as `TodoRow`, so the app can swap views without any other change.
pub struct TodoTable<'a> {
    todos: &'a [&'a Todo],
}

impl<'a> TodoTable<'a> {
    pub fn new(todos: &'a [&'a Todo]) -> Self {
        Self { todos }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<TodoAction> {
        let todos = self.todos;
        let mut action = None;
        let row_height = ui.spacing().interact_size.y + 4.0;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            // Lets whole rows report clicks, for double-click and the context menu.
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto().at_least(64.0))
            .column(Column::initial(220.0).at_least(80.0).clip(true))
            .column(Column::remainder().clip(true))
            .header(row_height, |mut header| {
                for column_title in ["Done", "Priority", "Title", "Notes"] {
                    header.col(|ui| {
                        ui.strong(column_title);
                    });
                }
            })
            .body(|body| {
                // `rows` only runs the closure for rows that are scrolled into view,
                // so the table stays fast with thousands of todos.
                body.rows(row_height, todos.len(), |mut row| {
                    let todo = todos[row.index()];

                    row.col(|ui| {
                        let mut done = todo.done;
                        let checkbox = ui
                            .checkbox(&mut done, "")
                            .accessible_name(format!("Done: {}", todo.title));
                        if checkbox.changed() {
                            action = Some(TodoAction::Toggle(todo.id));
                        }
                    });
                    row.col(|ui| {
                        ui.add(PriorityBadge::new(todo.priority));
                    });
                    row.col(|ui| {
                        let mut title = egui::RichText::new(&todo.title);
                        if todo.done {
                            title = title.strikethrough().weak();
                        }
                        ui.add(egui::Label::new(title).selectable(false));
                    });
                    row.col(|ui| {
                        // Show only the first line; the edit dialog has the rest.
                        let first_line = todo.notes.lines().next().unwrap_or_default();
                        let notes = egui::RichText::new(first_line).weak();
                        ui.add(egui::Label::new(notes).selectable(false));
                    });

                    let row_response = row.response();
                    if row_response.double_clicked() {
                        action = Some(TodoAction::StartEditing(todo.id));
                    }
                    row_response.context_menu(|ui| {
                        if let Some(menu_action) = todo_context_menu(ui, todo) {
                            action = Some(menu_action);
                        }
                    });
                });
            });

        action
    }
}
