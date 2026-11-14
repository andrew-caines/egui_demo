# Next steps

Two extensions that reuse what you built. Each touches the model, an action, one
component and a test.

## 1. Undo for delete

Deleting is currently permanent. Add a way back.

- In `model.rs`, make `TodoList::remove` return the removed `Todo` together with its
  index, and add `insert_at(index, todo)`.
- Add `TodoAction::UndoDelete`. In `TodoApp`, keep `last_deleted: Option<(usize, Todo)>`,
  set it in the `Delete` arm of `apply` and consume it in the new arm. The undo also has
  to reach the database: send `InsertTodo` followed by `SetTodoOrder`.
- Show an "Undo" button in the status bar while `last_deleted` is `Some`, and bind Ctrl+Z
  with a third `KeyboardShortcut` constant.
- Test it twice: a model test for `remove` followed by `insert_at`, and a kittest test that
  clicks a Delete button, then Undo, and compares the titles before and after.

*Stretch:* generalise to a full undo stack by storing the inverse of every action.

## 2. Due dates

Give todos an optional due date and highlight the overdue ones.

- Enable the date picker: `cargo add egui_extras@0.37 --features datepicker`. It uses the
  `jiff` crate's date type, so read the `egui_extras::DatePickerButton` documentation for
  the exact type to store.
- Add `due: Option<..>` to `Todo` and a nullable `due` column in a new migration,
  `migrations/0002_add_due_date.sql`, so existing databases gain the column without
  losing data. Extend `TodoRecord` and the SQL in `storage.rs`, and add the picker to
  `EditDialog`.
- Write a `DueBadge` leaf widget modelled on `PriorityBadge` that shows the date and turns
  red when it is in the past. Use it in both `TodoRow` and `TodoTable`.
- Add an "Overdue" option to `StatusFilter`. Pass today's date into `Filter::matches` as a
  parameter rather than reading the clock inside it, so the unit test stays deterministic.

## Further reading

- [egui API documentation](https://docs.rs/egui/0.37.0)
- [eframe API documentation](https://docs.rs/eframe/0.37.0)
- [egui_kittest documentation](https://docs.rs/egui_kittest/0.37.0)
- [reqwest API documentation](https://docs.rs/reqwest/0.13)
- [sqlx API documentation](https://docs.rs/sqlx/0.9) and its [repository](https://github.com/launchbadge/sqlx), which documents `sqlx-cli` and migrations
- [tokio tutorial](https://tokio.rs/tokio/tutorial), for the runtime and channels the worker uses
- [The egui web demo](https://www.egui.rs), which shows every built-in widget with a link to its source
