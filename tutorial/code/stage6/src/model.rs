//! The data model. Nothing in this file depends on egui, so it can be unit tested
//! without opening a window and reused behind any other front end.

/// Identifies a todo for its whole lifetime. A newtype stops a `TodoId` from being
/// confused with an index or any other number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TodoId(i64);

impl TodoId {
    /// Rebuilds an ID that was stored outside the app. Signed, because that is the
    /// integer type SQLite stores.
    pub fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

impl Priority {
    /// Every variant, in display order. Lets the UI build pickers with a loop.
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    pub id: TodoId,
    pub title: String,
    pub notes: String,
    pub priority: Priority,
    pub done: bool,
}

/// A todo that does not exist yet: what an import hands over. It has no ID because
/// only the [`TodoList`] gives those out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewTodo {
    pub title: String,
    pub done: bool,
}

/// Everything the UI is allowed to ask for. Components return these instead of
/// mutating the list, which keeps every state change in one place: `TodoApp::apply`.
#[derive(Clone, Debug, PartialEq)]
pub enum TodoAction {
    Add {
        title: String,
        priority: Priority,
    },
    Toggle(TodoId),
    Delete(TodoId),
    /// Open the edit dialog for this todo. Changes nothing by itself.
    StartEditing(TodoId),
    /// Commit the values typed into the edit dialog.
    Update {
        id: TodoId,
        title: String,
        notes: String,
        priority: Priority,
    },
    SetPriority(TodoId, Priority),
    /// Move `moved` into the list position currently held by `target`.
    Reorder {
        moved: TodoId,
        target: TodoId,
    },
    ClearCompleted,
    /// Append todos that came from outside the app.
    Import(Vec<NewTodo>),
}

/// Owns the todos and hands out IDs. The fields are private so the "no two todos
/// share an ID" rule cannot be broken from outside.
#[derive(Debug, Default)]
pub struct TodoList {
    todos: Vec<Todo>,
    next_id: i64,
}

impl TodoList {
    /// Rebuilds a list from todos that were loaded from storage, in the given order.
    pub fn from_todos(todos: Vec<Todo>) -> Self {
        // Carry on numbering after the highest ID in use.
        let highest_id = todos.iter().map(|todo| todo.id.raw()).max();
        Self {
            next_id: highest_id.map_or(0, |id| id + 1),
            todos,
        }
    }

    /// What a new user sees the first time the app starts.
    pub fn with_sample_todos() -> Self {
        let mut todos = Self::default();
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
        todos
    }

    /// Adds a todo and returns its ID, or `None` if the title is blank.
    pub fn add(&mut self, title: &str, priority: Priority) -> Option<TodoId> {
        let title = title.trim();
        if title.is_empty() {
            return None;
        }

        let id = TodoId(self.next_id);
        self.next_id += 1;
        self.todos.push(Todo {
            id,
            title: title.to_owned(),
            notes: String::new(),
            priority,
            done: false,
        });
        Some(id)
    }

    /// Appends imported todos and returns the IDs they were given. Items with a
    /// blank title are skipped, by the same rule as [`TodoList::add`].
    pub fn import(&mut self, new_todos: Vec<NewTodo>) -> Vec<TodoId> {
        let mut added_ids = Vec::new();
        for new_todo in new_todos {
            if let Some(id) = self.add(&new_todo.title, Priority::default()) {
                if new_todo.done {
                    self.toggle(id);
                }
                added_ids.push(id);
            }
        }
        added_ids
    }

    pub fn get(&self, id: TodoId) -> Option<&Todo> {
        self.todos.iter().find(|todo| todo.id == id)
    }

    fn get_mut(&mut self, id: TodoId) -> Option<&mut Todo> {
        self.todos.iter_mut().find(|todo| todo.id == id)
    }

    pub fn toggle(&mut self, id: TodoId) {
        if let Some(todo) = self.get_mut(id) {
            todo.done = !todo.done;
        }
    }

    pub fn set_priority(&mut self, id: TodoId, priority: Priority) {
        if let Some(todo) = self.get_mut(id) {
            todo.priority = priority;
        }
    }

    /// Replaces the editable fields. A blank title is ignored so that a todo can
    /// never end up without one.
    pub fn update(&mut self, id: TodoId, title: &str, notes: &str, priority: Priority) {
        let title = title.trim();
        if title.is_empty() {
            return;
        }
        if let Some(todo) = self.get_mut(id) {
            todo.title = title.to_owned();
            todo.notes = notes.trim().to_owned();
            todo.priority = priority;
        }
    }

    pub fn remove(&mut self, id: TodoId) {
        self.todos.retain(|todo| todo.id != id);
    }

    pub fn clear_completed(&mut self) {
        self.todos.retain(|todo| !todo.done);
    }

    /// Moves `moved` into the position currently held by `target`, shifting the
    /// todos in between. Dragging down therefore lands *after* the target and
    /// dragging up lands *before* it, which is what a user expects from a drop.
    pub fn reorder(&mut self, moved: TodoId, target: TodoId) {
        let position_of = |id| self.todos.iter().position(|todo| todo.id == id);
        let (Some(from), Some(to)) = (position_of(moved), position_of(target)) else {
            return;
        };
        let todo = self.todos.remove(from);
        self.todos.insert(to, todo);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Todo> {
        self.todos.iter()
    }

    pub fn len(&self) -> usize {
        self.todos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.todos.is_empty()
    }

    pub fn completed_count(&self) -> usize {
        self.todos.iter().filter(|todo| todo.done).count()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusFilter {
    #[default]
    All,
    Active,
    Completed,
}

impl StatusFilter {
    pub const ALL: [Self; 3] = [Self::All, Self::Active, Self::Completed];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }
}

/// Which todos are shown. This is *view* state: it changes what the user sees but
/// never the todos themselves, so the UI is allowed to edit it directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub status: StatusFilter,
    /// Case-insensitive text to look for in the title and notes.
    pub search: String,
    /// `None` shows every priority.
    pub priority: Option<Priority>,
}

impl Filter {
    pub fn matches(&self, todo: &Todo) -> bool {
        let status_matches = match self.status {
            StatusFilter::All => true,
            StatusFilter::Active => !todo.done,
            StatusFilter::Completed => todo.done,
        };

        let priority_matches = self
            .priority
            .is_none_or(|priority| priority == todo.priority);

        let needle = self.search.trim().to_lowercase();
        let search_matches = needle.is_empty()
            || todo.title.to_lowercase().contains(&needle)
            || todo.notes.to_lowercase().contains(&needle);

        status_matches && priority_matches && search_matches
    }

    pub fn is_active(&self) -> bool {
        *self != Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a list from titles and returns it with the IDs in insertion order.
    fn list_of(titles: &[&str]) -> (TodoList, Vec<TodoId>) {
        let mut list = TodoList::default();
        let ids = titles
            .iter()
            .map(|title| list.add(title, Priority::Medium).unwrap())
            .collect();
        (list, ids)
    }

    fn titles(list: &TodoList) -> Vec<&str> {
        list.iter().map(|todo| todo.title.as_str()).collect()
    }

    #[test]
    fn add_trims_the_title_and_rejects_blank_input() {
        let mut list = TodoList::default();

        assert!(list.add("   ", Priority::Low).is_none());
        assert!(list.is_empty());

        list.add("  Buy milk  ", Priority::High);
        let todo = list.iter().next().expect("one todo was added");
        assert_eq!(todo.title, "Buy milk");
        assert_eq!(todo.priority, Priority::High);
        assert!(!todo.done);
    }

    #[test]
    fn toggle_flips_only_the_matching_todo() {
        let (mut list, ids) = list_of(&["First", "Second"]);

        list.toggle(ids[0]);
        assert_eq!(list.completed_count(), 1);
        assert!(list.get(ids[0]).unwrap().done);

        list.toggle(ids[0]);
        assert_eq!(list.completed_count(), 0);
    }

    #[test]
    fn ids_are_not_reused_after_a_removal() {
        let (mut list, ids) = list_of(&["First"]);
        list.remove(ids[0]);
        let second = list.add("Second", Priority::Medium).unwrap();

        assert_ne!(ids[0], second);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn a_list_rebuilt_from_stored_todos_keeps_their_order_and_numbers_on_from_them() {
        let (original, ids) = list_of(&["First", "Second"]);
        let mut stored_todos: Vec<Todo> = original.iter().cloned().collect();
        stored_todos.reverse();

        let mut rebuilt = TodoList::from_todos(stored_todos);
        assert_eq!(titles(&rebuilt), ["Second", "First"]);

        let new_id = rebuilt.add("Third", Priority::Medium).unwrap();
        assert!(!ids.contains(&new_id));
    }

    #[test]
    fn update_changes_fields_but_refuses_a_blank_title() {
        let (mut list, ids) = list_of(&["Old title"]);

        list.update(ids[0], " New title ", " some notes ", Priority::High);
        let todo = list.get(ids[0]).unwrap();
        assert_eq!(todo.title, "New title");
        assert_eq!(todo.notes, "some notes");
        assert_eq!(todo.priority, Priority::High);

        list.update(ids[0], "   ", "", Priority::Low);
        assert_eq!(list.get(ids[0]).unwrap().title, "New title");
    }

    #[test]
    fn clear_completed_keeps_active_todos() {
        let (mut list, ids) = list_of(&["Keep", "Drop"]);
        list.toggle(ids[1]);

        list.clear_completed();
        assert_eq!(titles(&list), ["Keep"]);
    }

    #[test]
    fn reorder_moves_a_todo_into_the_targets_position() {
        let (mut list, ids) = list_of(&["A", "B", "C", "D"]);

        // Dragging down: A lands after C.
        list.reorder(ids[0], ids[2]);
        assert_eq!(titles(&list), ["B", "C", "A", "D"]);

        // Dragging up: D lands before B.
        list.reorder(ids[3], ids[1]);
        assert_eq!(titles(&list), ["D", "B", "C", "A"]);

        // Dropping a todo on itself changes nothing.
        list.reorder(ids[2], ids[2]);
        assert_eq!(titles(&list), ["D", "B", "C", "A"]);
    }

    #[test]
    fn import_appends_todos_and_skips_blank_titles() {
        let (mut list, _) = list_of(&["Existing"]);
        let new_todo = |title: &str, done| NewTodo {
            title: title.to_owned(),
            done,
        };

        let added_ids = list.import(vec![
            new_todo("Imported and open", false),
            new_todo("   ", false),
            new_todo("Imported and done", true),
        ]);

        assert_eq!(added_ids.len(), 2);
        assert_eq!(
            titles(&list),
            ["Existing", "Imported and open", "Imported and done"]
        );
        assert_eq!(list.completed_count(), 1);
    }

    #[test]
    fn filter_combines_status_priority_and_search() {
        let (mut list, ids) = list_of(&["Buy milk", "Call the bank"]);
        list.toggle(ids[0]);
        list.update(
            ids[1],
            "Call the bank",
            "Ask about the MILK subsidy",
            Priority::High,
        );
        let matching = |filter: &Filter| -> Vec<TodoId> {
            list.iter()
                .filter(|todo| filter.matches(todo))
                .map(|todo| todo.id)
                .collect()
        };

        let mut filter = Filter::default();
        assert!(!filter.is_active());
        assert_eq!(matching(&filter), ids);

        // The search is case-insensitive and also looks inside the notes.
        filter.search = "milk".to_owned();
        assert!(filter.is_active());
        assert_eq!(matching(&filter), ids);

        filter.status = StatusFilter::Active;
        assert_eq!(matching(&filter), [ids[1]]);

        filter.priority = Some(Priority::Low);
        assert!(matching(&filter).is_empty());
    }
}
