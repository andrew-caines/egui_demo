# Stage 1: a working list in one file

**Goal:** add, complete and delete todos. One file, no abstractions yet, so the raw
mechanics are visible.

`src/main.rs`:

@@file code/stage1/src/main.rs

![Stage 1](../images/stage1.png)

Why it is written this way:

- **`egui::TextEdit::singleline(..)` instead of `ui.text_edit_singleline(..)`.** The `ui.*`
  helpers are shortcuts for the common case. Every widget is also a builder struct that you
  configure and pass to `ui.add(..)`, which is how you reach options like `hint_text`.
- **`lost_focus()` plus `key_pressed(Enter)`.** There is no "on submit" event because there
  are no events. A single-line text box gives up focus when Enter is pressed, so that
  combination is the idiomatic test for "the user submitted this field".
- **`ui.input(|input| ...)`.** Input state is read through a closure because it sits behind a
  lock. Keep the closure short and never call other `ui` methods inside it.
- **`ui.push_id(todo.id, ...)`.** Each row contains identical widgets. Keying the row by a
  stable ID keeps egui's per-widget memory attached to the todo when rows are added or removed.
- **`id_to_delete`.** The list is borrowed by the `for` loop, so the delete button cannot
  remove from it. Recording the request and acting after the loop is the seed of the pattern
  that Stage 2 turns into the app's architecture.

This is a complete app, but it will not scale. The data rules (`trim`, "no blank titles")
are tangled into UI code, nothing can be unit tested, and every new feature makes `ui` longer.
