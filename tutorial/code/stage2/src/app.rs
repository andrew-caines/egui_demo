use eframe::egui;

use crate::components::{AddBar, TodoRow};
use crate::model::{Priority, TodoAction, TodoList};

pub struct TodoApp {
    todos: TodoList,
    add_bar: AddBar,
}

impl TodoApp {
    pub fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        let mut todos = TodoList::default();
        todos.add("Read the egui tutorial", Priority::High);
        todos.add("Split the UI into components", Priority::Medium);
        todos.add("Water the plants", Priority::Low);

        let mut add_bar = AddBar::default();
        // Start with the cursor in the input so the user can type straight away.
        add_bar.request_focus();

        Self { todos, add_bar }
    }

    /// The only place where app state changes in response to the UI.
    fn apply(&mut self, action: TodoAction) {
        match action {
            TodoAction::Add { title, priority } => {
                self.todos.add(&title, priority);
            }
            TodoAction::Toggle(id) => self.todos.toggle(id),
            TodoAction::Delete(id) => self.todos.remove(id),
        }
    }
}

impl eframe::App for TodoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Phase 1: draw the UI from the current state and collect what the user did.
        let mut actions: Vec<TodoAction> = Vec::new();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Todo List");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(format!(
                        "{} of {} done",
                        self.todos.completed_count(),
                        self.todos.len()
                    ));
                });
            });
            ui.add_space(8.0);

            // `Option` is iterable, so `extend` adds zero or one actions.
            actions.extend(self.add_bar.show(ui));

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.todos.is_empty() {
                        ui.weak("Nothing to do. Add your first item above.");
                    }

                    for todo in self.todos.iter() {
                        // Scope each row's widget IDs to the todo, not to its position.
                        ui.push_id(todo.id, |ui| {
                            actions.extend(TodoRow::new(todo).show(ui));
                        });
                    }
                });
        });

        // Phase 2: mutate. Nothing is borrowed any more, so this is always allowed.
        for action in actions {
            self.apply(action);
        }
    }
}
