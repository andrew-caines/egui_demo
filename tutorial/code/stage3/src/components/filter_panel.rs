use eframe::egui;

use crate::model::{Filter, Priority, StatusFilter, TodoAction, TodoList};

/// The sidebar: filter controls on top, progress underneath.
///
/// The filter is passed as `&mut` and edited in place, exactly like the built-in
/// widgets edit a `&mut String` or `&mut bool`. That is appropriate here because the
/// filter is view state. Anything that touches the todos still goes out as an action.
pub struct FilterPanel<'a> {
    filter: &'a mut Filter,
    total_count: usize,
    completed_count: usize,
    focus_search: bool,
}

impl<'a> FilterPanel<'a> {
    pub fn new(filter: &'a mut Filter, todos: &TodoList) -> Self {
        Self {
            filter,
            total_count: todos.len(),
            completed_count: todos.completed_count(),
            focus_search: false,
        }
    }

    /// Builder-style option, following the convention of egui's own widgets.
    pub fn focus_search(mut self, focus_search: bool) -> Self {
        self.focus_search = focus_search;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<TodoAction> {
        let mut action = None;

        ui.add_space(4.0);
        let search_label = ui.strong("Search");
        let search_response = ui
            .add(
                egui::TextEdit::singleline(&mut self.filter.search)
                    .hint_text("Title or notes…")
                    .desired_width(f32::INFINITY),
            )
            // Ties the box to the visible label next to it, for screen readers.
            .labelled_by(search_label.id);
        if self.focus_search {
            search_response.request_focus();
        }

        ui.add_space(8.0);
        ui.strong("Status");
        for status in StatusFilter::ALL {
            ui.radio_value(&mut self.filter.status, status, status.label());
        }

        ui.add_space(8.0);
        let priority_label = ui.strong("Priority");
        egui::ComboBox::from_id_salt("priority_filter")
            .selected_text(self.filter.priority.map_or("Any", Priority::label))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.filter.priority, None, "Any");
                for priority in Priority::ALL {
                    ui.selectable_value(
                        &mut self.filter.priority,
                        Some(priority),
                        priority.label(),
                    );
                }
            })
            .response
            .labelled_by(priority_label.id);

        ui.add_space(8.0);
        // `add_enabled` greys the button out and ignores clicks when the flag is false.
        let reset_button = egui::Button::new("Reset filters");
        if ui
            .add_enabled(self.filter.is_active(), reset_button)
            .clicked()
        {
            *self.filter = Filter::default();
        }

        ui.separator();

        ui.strong("Progress");
        let completed_fraction = if self.total_count == 0 {
            0.0
        } else {
            self.completed_count as f32 / self.total_count as f32
        };
        ui.add(egui::ProgressBar::new(completed_fraction).show_percentage());

        // `Grid` aligns its cells into columns; `end_row` starts the next line.
        egui::Grid::new("todo_statistics")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Total");
                ui.label(self.total_count.to_string());
                ui.end_row();

                ui.label("Active");
                ui.label((self.total_count - self.completed_count).to_string());
                ui.end_row();

                ui.label("Completed");
                ui.label(self.completed_count.to_string());
                ui.end_row();
            });

        ui.add_space(8.0);
        let clear_button = egui::Button::new("Clear completed");
        if ui
            .add_enabled(self.completed_count > 0, clear_button)
            .clicked()
        {
            action = Some(TodoAction::ClearCompleted);
        }

        action
    }
}
