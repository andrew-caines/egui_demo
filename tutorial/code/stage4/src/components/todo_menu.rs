use eframe::egui;

use crate::model::{Priority, Todo, TodoAction};

/// The contents of a todo's right-click menu.
///
/// A plain function is the lightest kind of component. It is shared by the list
/// row and the table row, so both views offer exactly the same commands.
pub fn todo_context_menu(ui: &mut egui::Ui, todo: &Todo) -> Option<TodoAction> {
    let mut action = None;

    if ui.button("Edit…").clicked() {
        action = Some(TodoAction::StartEditing(todo.id));
    }

    // A `menu_button` inside a menu becomes a sub-menu.
    ui.menu_button("Priority", |ui| {
        for priority in Priority::ALL {
            if ui
                .radio(todo.priority == priority, priority.label())
                .clicked()
            {
                action = Some(TodoAction::SetPriority(todo.id, priority));
            }
        }
    });

    ui.separator();

    if ui.button("Delete").clicked() {
        action = Some(TodoAction::Delete(todo.id));
    }

    // No explicit "close" call is needed: menus close when something in them is clicked.
    action
}
