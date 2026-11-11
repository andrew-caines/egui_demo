use eframe::egui;

use super::{PriorityBadge, todo_context_menu};
use crate::model::{Todo, TodoAction, TodoId};

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

        // Make the row itself clickable. Giving the *scope* a sense registers it
        // before its children, so the checkbox and buttons stay on top and still
        // receive their own clicks; the row only gets what they do not use.
        let row_builder = egui::UiBuilder::new().sense(egui::Sense::click());
        let row_response = ui
            .scope_builder(row_builder, |ui| {
                ui.horizontal(|ui| {
                    // --- Drag handle ---------------------------------------------
                    let handle = drag_handle(ui);
                    // Stores the ID as the drag payload when a drag starts here.
                    handle.dnd_set_drag_payload(todo.id);

                    // `checkbox` needs a `&mut bool`, so give it a copy and report
                    // the change.
                    let mut done = todo.done;
                    let checkbox = ui
                        .checkbox(&mut done, "")
                        // Nothing visible names this checkbox, so name it for
                        // screen readers and UI tests.
                        .accessible_name(format!("Done: {}", todo.title));
                    if checkbox.changed() {
                        action = Some(TodoAction::Toggle(todo.id));
                    }

                    ui.add(PriorityBadge::new(todo.priority));

                    // Buttons first, in a right-to-left layout, so the title can
                    // then take whatever width is left over.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Every row has these two buttons, so their accessible
                        // names say which todo they belong to.
                        let delete_button = ui
                            .small_button("Delete")
                            .accessible_name(format!("Delete: {}", todo.title));
                        if delete_button.clicked() {
                            action = Some(TodoAction::Delete(todo.id));
                        }
                        let edit_button = ui
                            .small_button("Edit")
                            .accessible_name(format!("Edit: {}", todo.title));
                        if edit_button.clicked() {
                            action = Some(TodoAction::StartEditing(todo.id));
                        }

                        // Back to left-to-right for the text itself.
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            let mut title = egui::RichText::new(&todo.title);
                            if todo.done {
                                title = title.strikethrough().weak();
                            }
                            // Not selectable, so that clicks on the text fall
                            // through to the row (double-click, context menu).
                            let title_label = egui::Label::new(title).truncate().selectable(false);
                            let title_response = ui.add(title_label);
                            if !todo.notes.is_empty() {
                                title_response.on_hover_text(&todo.notes);
                                ui.weak("(notes)");
                            }
                        });
                    });
                });
            })
            .response;

        if row_response.double_clicked() {
            action = Some(TodoAction::StartEditing(todo.id));
        }

        row_response.context_menu(|ui| {
            if let Some(menu_action) = todo_context_menu(ui, todo) {
                action = Some(menu_action);
            }
        });

        // --- Drop target ---------------------------------------------------------
        // While another todo is dragged over this row, outline it as the target.
        if let Some(dragged_id) = row_response.dnd_hover_payload::<TodoId>()
            && *dragged_id != todo.id
        {
            ui.painter().rect_stroke(
                row_response.rect.expand(2.0),
                4.0,
                ui.visuals().selection.stroke,
                egui::StrokeKind::Outside,
            );
        }

        // The payload is handed over on the frame the pointer is released here.
        if let Some(dragged_id) = row_response.dnd_release_payload::<TodoId>()
            && *dragged_id != todo.id
        {
            action = Some(TodoAction::Reorder {
                moved: *dragged_id,
                target: todo.id,
            });
        }

        action
    }
}

/// A six-dot "grip" that can be dragged. Painted by hand rather than with a text
/// glyph, so it does not depend on what the current font happens to contain.
fn drag_handle(ui: &mut egui::Ui) -> egui::Response {
    let size = egui::vec2(10.0, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());

    if ui.is_rect_visible(rect) {
        // `interact` picks the colours for the widget's current state (idle,
        // hovered, dragged), so the grip lights up like any built-in widget.
        let dot_color = ui.style().interact(&response).fg_stroke.color;
        for column in [-1.0, 1.0] {
            for row in [-1.0, 0.0, 1.0] {
                let dot_center = rect.center() + egui::vec2(column * 2.5, row * 4.0);
                ui.painter().circle_filled(dot_center, 1.2, dot_color);
            }
        }
    }

    response.on_hover_text("Drag to reorder")
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    use super::*;
    use crate::model::{Priority, TodoList};

    /// A component that returns actions can be tested on its own: no app, no
    /// window, just "given this todo and this click, expect this action".
    #[test]
    fn clicking_delete_returns_a_delete_action_and_changes_nothing() {
        let mut todos = TodoList::default();
        let id = todos.add("Water the plants", Priority::Low).unwrap();
        let todo = todos.get(id).unwrap();

        let mut harness = Harness::new_ui_state(
            |ui, actions: &mut Vec<TodoAction>| actions.extend(TodoRow::new(todo).show(ui)),
            Vec::new(),
        );

        harness.get_by_label("Delete: Water the plants").click();
        harness.run();

        assert_eq!(harness.state(), &[TodoAction::Delete(id)]);
        assert_eq!(todos.len(), 1);
    }
}
