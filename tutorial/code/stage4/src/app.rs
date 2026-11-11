use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::components::{
    AddBar, DialogOutcome, EditDialog, FilterPanel, SettingsWindow, TodoRow, TodoTable,
};
use crate::model::{Filter, Priority, Todo, TodoAction, TodoList};
use crate::settings::Settings;

/// Names the folder the app's data is saved in. Kept separate from the window
/// title so that renaming the window never orphans existing save files.
pub const APP_ID: &str = "egui-todo-tutorial";

// `COMMAND` means Ctrl on Windows and Linux and ⌘ on macOS.
const FOCUS_NEW_TODO: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::N);
const FOCUS_SEARCH: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::F);

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
enum ViewMode {
    #[default]
    List,
    Table,
}

/// Everything that should survive a restart, and nothing else.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedState {
    todos: TodoList,
    view_mode: ViewMode,
    sidebar_open: bool,
    settings: Settings,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            todos: TodoList::default(),
            view_mode: ViewMode::default(),
            sidebar_open: true,
            settings: Settings::default(),
        }
    }
}

impl PersistedState {
    /// What a new user sees the first time the app starts.
    pub fn with_sample_todos() -> Self {
        let mut todos = TodoList::default();
        todos.add("Read the egui tutorial", Priority::High);
        if let Some(id) = todos.add("Split the UI into components", Priority::Medium) {
            todos.update(
                id,
                "Split the UI into components",
                "Leaf widgets implement egui::Widget.\nComposite components return actions.",
                Priority::Medium,
            );
        }
        todos.add("Water the plants", Priority::Low);

        Self {
            todos,
            ..Self::default()
        }
    }
}

pub struct TodoApp {
    /// Saved to disk.
    state: PersistedState,

    // Everything below is transient: it is rebuilt from scratch on every start.
    add_bar: AddBar,
    filter: Filter,
    /// Set by the Ctrl+F shortcut, consumed when the sidebar is next drawn.
    search_focus_requested: bool,
    /// `Some` while the edit dialog is open.
    edit_dialog: Option<EditDialog>,
    settings_open: bool,
}

impl TodoApp {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        // `storage` is `None` if the "persistence" feature is off, and `get_value`
        // returns `None` on first launch or if the saved data cannot be parsed.
        let state = creation_context
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_else(PersistedState::with_sample_todos);

        state.settings.apply(&creation_context.egui_ctx);
        Self::from_state(state)
    }

    /// Builds the app without a window, which is what the UI tests use.
    pub fn from_state(state: PersistedState) -> Self {
        let mut add_bar = AddBar::default();
        // Start with the cursor in the input so the user can type straight away.
        add_bar.request_focus();

        Self {
            state,
            add_bar,
            filter: Filter::default(),
            search_focus_requested: false,
            edit_dialog: None,
            settings_open: false,
        }
    }

    /// The only place where app state changes in response to the UI.
    fn apply(&mut self, action: TodoAction) {
        let todos = &mut self.state.todos;
        match action {
            TodoAction::Add { title, priority } => {
                todos.add(&title, priority);
            }
            TodoAction::Toggle(id) => todos.toggle(id),
            TodoAction::Delete(id) => todos.remove(id),
            TodoAction::StartEditing(id) => {
                self.edit_dialog = todos.get(id).map(EditDialog::new);
            }
            TodoAction::Update {
                id,
                title,
                notes,
                priority,
            } => todos.update(id, &title, &notes, priority),
            TodoAction::SetPriority(id, priority) => todos.set_priority(id, priority),
            TodoAction::Reorder { moved, target } => todos.reorder(moved, target),
            TodoAction::ClearCompleted => todos.clear_completed(),
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // While the modal is open the rest of the window is blocked, so the
        // shortcuts that target it should be too.
        if self.edit_dialog.is_some() {
            return;
        }

        // `consume_shortcut` removes the key press from this frame's input, so a
        // focused text box will not also see it as typing.
        if ctx.input_mut(|input| input.consume_shortcut(&FOCUS_NEW_TODO)) {
            self.add_bar.request_focus();
        }
        if ctx.input_mut(|input| input.consume_shortcut(&FOCUS_SEARCH)) {
            self.state.sidebar_open = true;
            self.search_focus_requested = true;
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, actions: &mut Vec<TodoAction>) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                let new_todo_button = egui::Button::new("New todo")
                    .shortcut_text(ui.ctx().format_shortcut(&FOCUS_NEW_TODO));
                if ui.add(new_todo_button).clicked() {
                    self.add_bar.request_focus();
                }

                let clear_button = egui::Button::new("Clear completed");
                if ui
                    .add_enabled(self.state.todos.completed_count() > 0, clear_button)
                    .clicked()
                {
                    actions.push(TodoAction::ClearCompleted);
                }

                ui.separator();

                if ui.button("Settings…").clicked() {
                    self.settings_open = true;
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    // Window-level requests go to the viewport, not to a widget.
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.state.sidebar_open, "Sidebar");
                let find_button = egui::Button::new("Find…")
                    .shortcut_text(ui.ctx().format_shortcut(&FOCUS_SEARCH));
                if ui.add(find_button).clicked() {
                    self.state.sidebar_open = true;
                    self.search_focus_requested = true;
                }

                ui.separator();

                ui.radio_value(&mut self.state.view_mode, ViewMode::List, "List");
                ui.radio_value(&mut self.state.view_mode, ViewMode::Table, "Table");
            });

            ui.menu_button("Help", |ui| {
                ui.hyperlink_to("egui documentation", "https://docs.rs/egui/0.37.0");
            });
        });
    }

    fn status_bar(&self, ui: &mut egui::Ui, visible_count: usize) {
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.label(format!(
                    "Showing {visible_count} of {} · {} completed",
                    self.state.todos.len(),
                    self.state.todos.completed_count()
                ));
            },
            |ui| {
                ui.weak("Double-click to edit · right-click for more");
            },
        );
    }

    fn todo_views(
        &self,
        ui: &mut egui::Ui,
        visible_todos: &[&Todo],
        actions: &mut Vec<TodoAction>,
    ) {
        if visible_todos.is_empty() {
            // Tell the user *why* the list is empty.
            let message = if self.state.todos.is_empty() {
                "Nothing to do. Add your first item above."
            } else {
                "No todos match the current filters."
            };
            ui.weak(message);
            return;
        }

        match self.state.view_mode {
            ViewMode::List => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for &todo in visible_todos {
                            // Scope each row's widget IDs to the todo, not to its position.
                            ui.push_id(todo.id, |ui| {
                                actions.extend(TodoRow::new(todo).show(ui));
                            });
                        }
                    });
            }
            ViewMode::Table => actions.extend(TodoTable::new(visible_todos).show(ui)),
        }
    }

    /// Draws the whole app into `ui`. Separate from `eframe::App::ui` so that tests
    /// can drive it without an `eframe::Frame`.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Phase 1: draw the UI from the current state and collect what the user did.
        let mut actions: Vec<TodoAction> = Vec::new();

        self.handle_shortcuts(ui.ctx());

        // Panels are carved out of the remaining space in the order they are shown,
        // so the outer panels come first and `CentralPanel` must come last.
        egui::Panel::top("menu_bar").show(ui, |ui| self.menu_bar(ui, &mut actions));

        let visible_count = self
            .state
            .todos
            .iter()
            .filter(|todo| self.filter.matches(todo))
            .count();
        egui::Panel::bottom("status_bar").show(ui, |ui| self.status_bar(ui, visible_count));

        let focus_search = std::mem::take(&mut self.search_focus_requested);
        egui::Panel::left("sidebar")
            .resizable(true)
            .default_size(190.0)
            .size_range(160.0..=320.0)
            // Slides the panel in and out, and lets the user drag it shut.
            .show_collapsible(ui, &mut self.state.sidebar_open, |ui| {
                actions.extend(
                    FilterPanel::new(&mut self.filter, &self.state.todos)
                        .focus_search(focus_search)
                        .show(ui),
                );
            });

        egui::CentralPanel::default().show(ui, |ui| {
            actions.extend(self.add_bar.show(ui));
            ui.separator();

            // Computed after the sidebar so that typing in the search box filters
            // the list in the same frame.
            let visible_todos: Vec<&Todo> = self
                .state
                .todos
                .iter()
                .filter(|todo| self.filter.matches(todo))
                .collect();
            self.todo_views(ui, &visible_todos, &mut actions);
        });

        if let Some(dialog) = &mut self.edit_dialog {
            match dialog.show(ui.ctx()) {
                DialogOutcome::Open => {}
                DialogOutcome::Cancelled => self.edit_dialog = None,
                DialogOutcome::Saved(action) => {
                    actions.push(action);
                    self.edit_dialog = None;
                }
            }
        }

        let settings_changed =
            SettingsWindow::new(&mut self.state.settings, &mut self.settings_open)
                .storage_path(eframe::storage_dir(APP_ID))
                .show(ui.ctx());
        if settings_changed {
            self.state.settings.apply(ui.ctx());
        }

        // Phase 2: mutate. Nothing is borrowed any more, so this is always allowed.
        for action in actions {
            self.apply(action);
        }
    }
}

impl eframe::App for TodoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    /// Called on shutdown and every `auto_save_interval` (30 seconds by default).
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    use super::*;

    /// Runs the real app headlessly, starting from the sample todos.
    fn app_harness() -> Harness<'static, TodoApp> {
        let app = TodoApp::from_state(PersistedState::with_sample_todos());
        let mut harness = Harness::builder()
            .with_size(egui::vec2(760.0, 520.0))
            .build_ui_state(|ui, app: &mut TodoApp| app.show(ui), app);
        // Let the first frames settle (focus requests, panel sizing).
        harness.run();
        harness
    }

    /// Focuses the text box with the given accessible name, then types into it.
    /// `type_text` on its own sends the text to whichever widget has focus.
    fn type_into(harness: &mut Harness<'_, TodoApp>, label: &str, text: &str) {
        harness.get_by_label(label).focus();
        harness.run();
        harness.get_by_label(label).type_text(text);
        harness.run();
    }

    fn titles(harness: &Harness<'_, TodoApp>) -> Vec<String> {
        let todos = &harness.state().state.todos;
        todos.iter().map(|todo| todo.title.clone()).collect()
    }

    #[test]
    fn typing_a_title_and_pressing_enter_adds_a_todo() {
        let mut harness = app_harness();

        type_into(&mut harness, "New todo title", "Buy milk");
        harness.key_press(egui::Key::Enter);
        harness.run();

        assert_eq!(titles(&harness).last().unwrap(), "Buy milk");
        // The new row is on screen, and the box is ready for the next todo.
        harness.get_by_label("Done: Buy milk");
        assert!(harness.get_by_label("New todo title").is_focused());
    }

    #[test]
    fn ticking_a_checkbox_completes_the_todo() {
        let mut harness = app_harness();

        harness.get_by_label("Done: Water the plants").click();
        harness.run();

        assert_eq!(harness.state().state.todos.completed_count(), 1);
        harness.get_by_label("Showing 3 of 3 · 1 completed");
    }

    #[test]
    fn searching_hides_todos_that_do_not_match() {
        let mut harness = app_harness();

        type_into(&mut harness, "Search", "plants");

        harness.get_by_label("Done: Water the plants");
        assert!(
            harness
                .query_by_label("Done: Read the egui tutorial")
                .is_none()
        );
        // Filtering is a view concern: the todos themselves are untouched.
        assert_eq!(titles(&harness).len(), 3);
    }

    #[test]
    fn the_edit_dialog_saves_a_new_title() {
        let mut harness = app_harness();

        harness.get_by_label("Edit: Water the plants").click();
        harness.run();

        type_into(&mut harness, "Title", " today");
        harness.get_by_label("Save").click();
        harness.run();

        assert!(titles(&harness).contains(&"Water the plants today".to_owned()));
        // The dialog is gone again.
        assert!(harness.query_by_label("Edit todo").is_none());
    }

    #[test]
    fn cancelling_the_edit_dialog_changes_nothing() {
        let mut harness = app_harness();
        let titles_before = titles(&harness);

        harness.get_by_label("Edit: Water the plants").click();
        harness.run();
        type_into(&mut harness, "Title", " (changed)");
        harness.get_by_label("Cancel").click();
        harness.run();

        assert_eq!(titles(&harness), titles_before);
    }

    #[test]
    fn the_table_view_reports_the_same_actions_as_the_list() {
        let mut harness = app_harness();
        harness.state_mut().state.view_mode = ViewMode::Table;
        harness.run();

        harness.get_by_label("Done: Water the plants").click();
        harness.run();

        assert_eq!(harness.state().state.todos.completed_count(), 1);
    }

    #[test]
    fn the_file_menu_opens_the_settings_window() {
        let mut harness = app_harness();
        assert!(harness.query_by_label("Accent colour").is_none());

        harness.get_by_label("File").click();
        harness.run();
        harness.get_by_label("Settings…").click();
        harness.run();

        harness.get_by_label("Accent colour");

        // Choosing a theme goes straight to egui, which owns that preference.
        harness.get_by_label("Light").click();
        harness.run();
        assert_eq!(harness.ctx.theme(), egui::Theme::Light);
    }

    #[test]
    fn right_clicking_a_row_offers_delete_in_a_context_menu() {
        let mut harness = app_harness();

        // The title label does not sense clicks, so the click reaches the row.
        harness.get_by_label("Water the plants").click_secondary();
        harness.run();
        harness.get_by_label("Delete").click();
        harness.run();

        assert_eq!(
            titles(&harness),
            ["Read the egui tutorial", "Split the UI into components"]
        );
    }

    #[test]
    fn dragging_a_row_by_its_handle_reorders_the_list() {
        let mut harness = app_harness();

        // The drag handle sits just left of each row's checkbox.
        let handle_of = |harness: &Harness<'_, TodoApp>, title: &str| {
            let checkbox_label = format!("Done: {title}");
            let checkbox_rect = harness.get_by_label(&checkbox_label).rect();
            checkbox_rect.left_center() - egui::vec2(13.0, 0.0)
        };
        let drag_from = handle_of(&harness, "Read the egui tutorial");
        let drop_onto = handle_of(&harness, "Water the plants");

        harness.hover_at(drag_from);
        harness.run();
        harness.drag_at(drag_from);
        harness.run();
        harness.hover_at(drop_onto);
        harness.run();
        harness.drop_at(drop_onto);
        harness.run();

        assert_eq!(
            titles(&harness),
            [
                "Split the UI into components",
                "Water the plants",
                "Read the egui tutorial"
            ]
        );
    }
}
