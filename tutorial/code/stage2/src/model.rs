//! The data model. Nothing in this file depends on egui, so it can be unit tested
//! without opening a window and reused behind any other front end.

/// Identifies a todo for its whole lifetime. A newtype stops a `TodoId` from being
/// confused with an index or any other number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TodoId(u64);

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
    pub priority: Priority,
    pub done: bool,
}

/// Everything the UI is allowed to ask for. Components return these instead of
/// mutating the list, which keeps every state change in one place: `TodoApp::apply`.
#[derive(Clone, Debug, PartialEq)]
pub enum TodoAction {
    Add { title: String, priority: Priority },
    Toggle(TodoId),
    Delete(TodoId),
}

/// Owns the todos and hands out IDs. The fields are private so the "IDs are never
/// reused" rule cannot be broken from outside.
#[derive(Debug, Default)]
pub struct TodoList {
    todos: Vec<Todo>,
    next_id: u64,
}

impl TodoList {
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
            priority,
            done: false,
        });
        Some(id)
    }

    pub fn toggle(&mut self, id: TodoId) {
        if let Some(todo) = self.todos.iter_mut().find(|todo| todo.id == id) {
            todo.done = !todo.done;
        }
    }

    pub fn remove(&mut self, id: TodoId) {
        self.todos.retain(|todo| todo.id != id);
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut list = TodoList::default();
        let first = list.add("First", Priority::Medium).unwrap();
        list.add("Second", Priority::Medium).unwrap();

        list.toggle(first);
        assert_eq!(list.completed_count(), 1);
        assert!(list.iter().next().unwrap().done);

        list.toggle(first);
        assert_eq!(list.completed_count(), 0);
    }

    #[test]
    fn ids_are_not_reused_after_a_removal() {
        let mut list = TodoList::default();
        let first = list.add("First", Priority::Medium).unwrap();
        list.remove(first);
        let second = list.add("Second", Priority::Medium).unwrap();

        assert_ne!(first, second);
        assert_eq!(list.len(), 1);
    }
}
