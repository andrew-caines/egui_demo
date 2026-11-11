# Building a Todo List desktop app with egui 0.37

A hands-on tutorial for experienced Rust developers who are new to [egui](https://crates.io/crates/egui).
You will build a Windows desktop todo app in six stages, ending with a set of reusable,
tested UI components, HTTP requests that never freeze the window, and todos stored in SQLite.

> **Tested against:** `egui` / `eframe` / `egui_extras` / `egui_kittest` **0.37.0**, `reqwest` 0.13,
> `sqlx` 0.9, `tokio` 1.54, Rust 1.99, Windows 11.
> Every Rust block below is copied from a file under [`code/`](code/) that compiles with zero warnings
> under `cargo clippy -- -D warnings`. egui 0.37 needs **Rust 1.95 or newer**.

**Contents**

1. [Introduction](#1-introduction)
2. [Setup & installation](#2-setup--installation)
3. [Core concepts: Hello World](#3-core-concepts-hello-world)
4. [Step-by-step project build](#4-step-by-step-project-build)
   - [Stage 1: a working list in one file](#stage-1-a-working-list-in-one-file)
   - [Stage 2: reusable components](#stage-2-reusable-components)
   - [Stage 3: a full application shell](#stage-3-a-full-application-shell)
   - [Stage 4: persistence, theming, custom painting and UI tests](#stage-4-persistence-theming-custom-painting-and-ui-tests)
   - [Stage 5: HTTP requests on a background worker](#stage-5-http-requests-on-a-background-worker)
   - [Stage 6: a SQLite database with sqlx](#stage-6-a-sqlite-database-with-sqlx)
5. [Common pitfalls & best practices](#5-common-pitfalls--best-practices)
6. [Next steps](#6-next-steps)

---

## 1. Introduction

**egui** is an *immediate mode* GUI library. There is no widget tree that you build once and
then mutate, no callbacks, and no data binding. Instead, your code runs every frame, and each
call both draws a widget and tells you what the user did with it:

```rust,ignore
if ui.button("Save").clicked() {
    save();
}
```

Developers pick egui because:

- **State stays in your own structs.** The UI is a function of your data, so it cannot drift
  out of sync with it.
- **It is plain Rust.** Layout is closures, lists are `for` loops, conditional UI is `if`.
- **It is portable and self-contained.** The same code runs natively and on the web, with no
  system toolkit to install.

egui itself only turns input into shapes to draw. **eframe** is the official framework that
opens the window, runs the event loop and renders those shapes. An app depends on `eframe`,
which re-exports `egui`.

### What we are building

![The finished app](images/stage4.png)

A todo app with a menu bar, a collapsible filter sidebar, list and table views, a modal edit
dialog, drag-to-reorder, keyboard shortcuts, a settings window, light and dark themes,
import over HTTP, and todos stored in a SQLite database.

| Stage | You add | egui features introduced |
|---|---|---|
| 1 | A working list in one file | `CentralPanel`, `TextEdit`, `ScrollArea`, layouts, `RichText`, `push_id` |
| 2 | A data model and reusable components | `impl Widget`, custom painting, `Sides`, `ComboBox`, accessibility names |
| 3 | A full application shell | `Panel`, `MenuBar`, `Modal`, `Grid`, `TableBuilder`, context menus, drag and drop, shortcuts |
| 4 | Persistence, theming, a custom widget, UI tests | `eframe::Storage`, `Style`, `Window`, `Slider`, colour picker, animation, `egui_kittest` |
| 5 | Import from a URL with `reqwest` | A background worker, `Context::request_repaint`, `Spinner`, disabled widgets, async results in an immediate mode UI |
| 6 | Todos in SQLite with `sqlx` | Loading state, saving as you go, error reporting in the status bar |

**What is left out.** "All of egui" is larger than one app can use sensibly. This tutorial
does not cover images and textures, custom fonts, multiple native windows (viewports),
plotting (`egui_plot`), custom GPU rendering, or the web target.

---

## 2. Setup & installation

You need a Rust toolchain (1.95+) with the MSVC build tools, which `rustup` installs by
default on Windows. No environment variables or extra configuration are required.

```powershell
rustc --version          # must print 1.95.0 or newer; run `rustup update` if not

cargo new todo_app
cd todo_app
cargo add eframe@0.37
```

Later stages add more dependencies. You can add them now or when you reach the stage:

```powershell
# Stage 3: tables
cargo add egui_extras@0.37

# Stage 4: saving state, and headless UI tests
cargo add eframe@0.37 --features persistence
cargo add serde --features derive
cargo add --dev egui_kittest@0.37

# Stage 5: HTTP requests, and a local mock server for the tests
cargo add reqwest@0.13 --features json
cargo add tokio@1 --features rt,sync
cargo add --dev wiremock@0.6
cargo add --dev tokio@1 --features macros

# Stage 6: SQLite
cargo add sqlx@0.9 --no-default-features --features runtime-tokio,sqlite,migrate,macros
cargo add --dev tempfile@3
```

Stage 5 makes real network requests. The first time a newly built executable connects,
Windows security software may ask you to allow it, and the request waits until you answer.

The first build compiles the renderer and takes a few minutes. Later builds take seconds.

### Running the companion code

Every stage is also a ready-to-run crate in this directory:

```powershell
cd tutorial/code
cargo run -p hello
cargo run -p stage1      # ... stage2 to stage6
cargo test --workspace
```

---

## 3. Core concepts: Hello World

Replace `src/main.rs` with:

@@file code/hello/src/main.rs

![Hello World](images/hello.png)

Run it with `cargo run`. What each part does, and why:

| Code | Why it is there |
|---|---|
| `#![cfg_attr(..., windows_subsystem = "windows")]` | Without it, a release build opens a console window behind the GUI. Debug builds keep the console so you can see `println!` and panics. |
| `use eframe::egui;` | `eframe` re-exports the exact `egui` version it was built with, so you never need a separate `egui` dependency that could drift out of step. |
| `fn main() -> eframe::Result` | Creating a window can fail (no GPU, no display). Returning the result reports that instead of panicking. |
| `NativeOptions { viewport, .. }` | A *viewport* is a native OS window. `ViewportBuilder` sets its size, title, icon and so on. |
| `run_native(name, options, creator)` | Blocks until the window closes. The `creator` closure runs once, after the renderer exists, which is why it (and not `main`) receives the context you use to load fonts or saved state. It returns a `Result` so app setup can fail cleanly. |
| `struct HelloApp` | Your state. egui stores none of it. |
| `impl eframe::App` / `fn ui` | The one required method. eframe calls it whenever the window needs repainting and gives you the root `Ui`. |
| `CentralPanel::default().show(ui, ...)` | The root `Ui` has no margin or background. `CentralPanel` fills the remaining space and adds both. |
| `ui.horizontal(\|ui\| ...)` | Layout is nested closures. Each closure gets a child `Ui` with its own layout direction. |
| `ui.text_edit_singleline(&mut self.name)` | Widgets borrow your data for the duration of the call and edit it in place. |
| `ui.button("Click me").clicked()` | Every widget returns a `Response`. You ask it what happened: `clicked()`, `hovered()`, `changed()`, `lost_focus()` and so on. |

Three ideas explain almost everything else in egui:

1. **The UI is rebuilt from your state every frame.** A frame is cheap (typically a millisecond or
   two for an app like this), and egui only runs one when there is input or an
   animation in progress. You never "update" a label; you draw it again with the new value.
2. **`Ui` is a region with a layout cursor.** Adding a widget allocates the next rectangle
   and advances the cursor. Containers such as `horizontal`, `ScrollArea` and `Grid` hand
   you a child `Ui`.
3. **egui remembers a little state for you, keyed by widget ID.** Scroll offsets, which
   text box has focus, whether a header is collapsed. IDs are derived automatically, and
   [pitfall 1](#pitfall-1-unstable-or-clashing-widget-ids) covers when you must help.

> **If you are following an older tutorial:** in earlier versions the trait method was
> `fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)` and panels were shown
> with `.show(ctx, ...)`. In 0.37 that method no longer exists. You implement `fn ui` and
> pass the `ui` you are given. See the [migration table](#coming-from-an-older-egui-version).

---

## 4. Step-by-step project build

### Stage 1: a working list in one file

**Goal:** add, complete and delete todos. One file, no abstractions yet, so the raw
mechanics are visible.

`src/main.rs`:

@@file code/stage1/src/main.rs

![Stage 1](images/stage1.png)

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

---

### Stage 2: reusable components

**Goal:** the same app plus priorities, restructured so that it can grow.

```text
src/
├── main.rs                  window setup only
├── app.rs                   owns the state, composes components, applies actions
├── model.rs                 data and rules; no egui in here
└── components/
    ├── mod.rs
    ├── priority_badge.rs    leaf widget         (impl egui::Widget)
    ├── todo_row.rs          composite component (returns an action)
    └── add_bar.rs           stateful component  (owns its draft)
```

The design rests on one rule: **components describe what the user asked for; only the app
changes state.**

```text
        ┌────────────── every frame ──────────────┐
        │                                         │
   state ──► components draw it ──► Vec<TodoAction> ──► TodoApp::apply ──► state
            (read-only borrow)        (plain data)       (the only writer)
```

That gives you three component shapes, each used below:

| Shape | Signature | Use it for |
|---|---|---|
| **Leaf widget** | `impl egui::Widget`, used as `ui.add(MyWidget::new(..))` | Something that is drawn and at most reports a `Response`: badges, icons, gauges. |
| **Composite component** | `fn show(self, ui: &mut Ui) -> Option<Action>` | A group of widgets that turns interaction into a domain action. Borrows data immutably. |
| **Stateful component** | `fn show(&mut self, ui: &mut Ui) -> Option<Action>` on a struct the app owns | UI with private working state, such as a half-typed draft. |

#### The model

`src/model.rs`:

@@file code/stage2/src/model.rs

- **No `egui` import.** The model compiles and is tested without a window. `cargo test` runs
  these tests in milliseconds.
- **`TodoAction` is plain data.** It derives `Debug` and `PartialEq`, so tests can assert on
  it, and you could later log it or push it onto an undo stack.
- **`TodoList` hides its `Vec`.** Callers can only change todos through methods that keep the
  rules (trimmed titles, unique IDs) intact.

#### A leaf widget: `PriorityBadge`

`src/components/priority_badge.rs`:

@@file code/stage2/src/components/priority_badge.rs

Every custom-painted widget follows the same three steps: **measure**, **allocate**,
**paint**. `allocate_exact_size` is the important call. It reserves the rectangle, moves the
layout cursor, and performs hit-testing according to the `Sense` you pass, returning a
`Response` like any built-in widget.

`widget_info` describes the widget to assistive technology. It also makes it findable in the
UI tests you will write in Stage 4.

#### A composite component: `TodoRow`

`src/components/todo_row.rs`:

@@file code/stage2/src/components/todo_row.rs

`TodoRow` holds `&Todo`, not `&mut Todo`. That one choice means the compiler proves that
drawing never changes data, and it means the row can be shown from any context that has a
shared reference.

The `accessible_name` calls name widgets that have no visible label of their own. Do this as
you write each widget. Stage 4 turns it into a test that fails when you forget.

#### A stateful component: `AddBar`

`src/components/add_bar.rs`:

@@file code/stage2/src/components/add_bar.rs

- **The draft lives in the component, not in the app.** `TodoApp` does not care what is
  half-typed, so it should not have fields for it.
- **`focus_requested` is a one-shot flag.** Focus can only be requested on a `Response`,
  which exists only while the widget is being drawn. So `request_focus()` records the wish
  and `show` grants it. You will use this "request now, fulfil during the next draw"
  pattern whenever something outside a widget needs to affect it.
- **`egui::Sides`** solves the classic "stretchy input with fixed buttons on the right"
  layout without measuring anything.

`src/components/mod.rs`:

@@file code/stage2/src/components/mod.rs

#### The app

`src/app.rs`:

@@file code/stage2/src/app.rs

`src/main.rs`:

@@file code/stage2/src/main.rs

![Stage 2](images/stage2.png)

`ui` is now two phases. Phase 1 holds shared borrows of the state and only *collects*
actions. Phase 2 holds no borrows and mutates freely. The borrow checker enforces the
split for you: try to call `self.todos.remove(..)` inside the row loop and it will not compile.

Run `cargo test` and the three model tests pass without a window opening.

---

### Stage 3: a full application shell

**Goal:** make it feel like a desktop application: menus, a sidebar with filters, an edit
dialog, a table view, context menus, drag-to-reorder and keyboard shortcuts.

```powershell
cargo add egui_extras@0.37
```

```text
src/components/
├── add_bar.rs           unchanged
├── priority_badge.rs    unchanged
├── todo_row.rs          + drag handle, edit button, context menu, drop target
├── todo_menu.rs         new: context menu shared by the row and the table
├── filter_panel.rs      new: sidebar
├── edit_dialog.rs       new: modal dialog
└── todo_table.rs        new: table view
```

Two of the Stage 2 components are reused without touching them. That is the payoff of
the action pattern: `AddBar` and `PriorityBadge` know nothing about the rest of the app.

#### The model grows

Todos gain notes, the action enum gains the new commands, and a `Filter` type describes
which todos are visible. `src/model.rs`, in full:

@@file code/stage3/src/model.rs

`Filter` is *view state*: changing it never changes a todo. That distinction decides how
components may treat it, as you will see in `FilterPanel`.

#### A shared context menu

`src/components/todo_menu.rs`:

@@file code/stage3/src/components/todo_menu.rs

A component does not have to be a struct. A function that takes `&mut Ui` and returns an
action is enough when there are no options to configure.

#### `TodoRow`, upgraded

`src/components/todo_row.rs`:

@@file code/stage3/src/components/todo_row.rs

Three techniques are worth studying here:

- **Making a container clickable.** `UiBuilder::new().sense(Sense::click())` gives the row's
  own `Ui` a click sense. Because the scope is registered *before* its children, the
  children sit on top of it and keep their clicks. If you instead called
  `ui.interact(rect, ..)` *after* drawing the children, the row would cover them and the
  checkbox would stop working.
- **Drag and drop is typed.** `dnd_set_drag_payload(todo.id)` stores a `TodoId`, and
  `dnd_release_payload::<TodoId>()` only returns payloads of that type. A row cannot
  accidentally accept something else that is being dragged.
- **Painting your own icon.** The grip is six circles rather than a character such as "☰",
  because egui 0.37's bundled font contains no symbols or emoji
  (see [the migration table](#coming-from-an-older-egui-version)).

#### The sidebar: `FilterPanel`

`src/components/filter_panel.rs`:

@@file code/stage3/src/components/filter_panel.rs

This component takes `&mut Filter` and edits it directly, which looks like a violation of
"components only return actions". It is not. The rule protects *domain* state. `Filter`
is view state, and editing a borrowed value in place is exactly what `ui.checkbox(&mut bool)`
does. The one thing here that touches todos, "Clear completed", still goes out as an action.

`focus_search(..)` is a builder method, mirroring how egui's own widgets take options.

#### A modal: `EditDialog`

`src/components/edit_dialog.rs`:

@@file code/stage3/src/components/edit_dialog.rs

- **Edit a copy.** The dialog owns clones of the fields. Cancel is free (drop the
  dialog), and a half-edited title never appears in the list behind it.
- **A three-way outcome enum** is clearer than `Option<Option<..>>` or a pair of booleans, and
  the `match` in the app must handle every case.
- **`Id::unique`.** `Modal` floats above everything, outside any parent `Ui`, so its ID
  must be unique across the whole app. Compare that with `Grid::new("edit_todo_fields")`,
  whose string is only a *salt* that has to be unique among its siblings.

#### A table: `TodoTable`

`src/components/todo_table.rs`:

@@file code/stage3/src/components/todo_table.rs

`TodoTable` returns the same `TodoAction`s as `TodoRow`, so the app treats the two views
identically. Note the `body.rows(..)` call: it is virtualised, so only visible rows run.

`src/components/mod.rs`:

@@file code/stage3/src/components/mod.rs

#### The app shell

`src/app.rs`:

@@file code/stage3/src/app.rs

In `src/main.rs`, only the window size changes, to make room for the sidebar:

```rust
            .with_inner_size([760.0, 520.0])
            .with_min_inner_size([520.0, 320.0]),
```

![Stage 3](images/stage3.png)

Points to notice in `app.rs`:

- **Panel order is layout.** Each `Panel` takes its space from what is left, so top and
  bottom bars are shown first, then the sidebar, and `CentralPanel` last. Add the central
  panel first and it would take everything.
- **`show_collapsible`** takes `&mut bool`, animates the slide, and clears the flag if the
  user drags the panel shut. The *View → Sidebar* checkbox edits the same flag.
- **Shortcuts are declared once** as `KeyboardShortcut` constants and used twice: to
  consume the key press and to print the platform-correct text ("Ctrl+N" or "⌘N") in the menu.
- **The methods take `&mut Vec<TodoAction>`** rather than returning one. With several
  sources of actions per frame, pushing into a shared buffer keeps the call sites short.
- **`apply` grew, `ui` did not get more complicated.** Each new feature added one `match`
  arm. `StartEditing` shows that an action does not have to change the todos: it opens a
  dialog, which later produces an `Update` action of its own.

Try it: right-click a row, double-click to edit, drag a row by its grip, press Ctrl+F,
switch *View → Table*. `cargo test` now runs seven model tests.

---

### Stage 4: persistence, theming, custom painting and UI tests

**Goal:** remember todos and settings between runs, let the user restyle the app, replace
the stock progress bar with a custom animated widget, and test the UI headlessly.

`Cargo.toml` dependencies:

```toml
[dependencies]
# "persistence" lets eframe save app state (and egui's own memory) between runs.
eframe = { version = "0.37", features = ["persistence"] }
egui_extras = "0.37"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
egui_kittest = "0.37"
```

#### Make the model serialisable

In `src/model.rs`, import serde and add the two derives to the four persisted types.
Nothing else in the file changes:

```rust
use serde::{Deserialize, Serialize};
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TodoId(u64);
```

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
```

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
```

```rust
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TodoList {
```

`Filter` is deliberately not persisted. Reopening the app to a list that is silently
filtered by last week's search is a bug from the user's point of view.

#### Settings and style

`src/settings.rs`:

@@file code/stage4/src/settings.rs

egui's look is data: a `Style` holding `Visuals` (colours, strokes) and `Spacing`. You
change it once and it stays in effect, so `apply` is called at startup and when a setting
changes, not every frame. The theme itself (light, dark or follow the system) is not in
`Settings` because egui already stores and saves that preference.

`apply` is tested against a bare `egui::Context`. No window is needed to check styling.

#### A custom animated widget: `ProgressRing`

`src/components/progress_ring.rs`:

@@file code/stage4/src/components/progress_ring.rs

`animate_value_with_time` is all it takes to animate. egui stores the previous value under
the given ID, interpolates, and keeps requesting repaints until the animation settles. The
colours come from `ui.visuals()`, so the ring follows the theme and the accent colour
without knowing that `Settings` exists.

To use it, change two places in `src/components/filter_panel.rs`. Add the import:

```rust
use super::ProgressRing;
```

and replace the `ProgressBar` line with:

```rust
        ui.vertical_centered(|ui| {
            ui.add(ProgressRing::new(completed_fraction).diameter(72.0));
        });
```

That is the whole migration, because both are `impl Widget`.

#### A floating window: `SettingsWindow`

`src/components/settings_window.rs`:

@@file code/stage4/src/components/settings_window.rs

`Window` and `Modal` are both shown on the `Context`, but they behave differently. A window
can be moved, does not block the app, and stays open while the user works. Use a modal when
the user must answer before continuing, and a window for tools and inspectors.

`src/components/mod.rs`:

@@file code/stage4/src/components/mod.rs

#### Persistence in the app

`src/main.rs`:

@@file code/stage4/src/main.rs

`src/app.rs`:

@@file code/stage4/src/app.rs

![Stage 4](images/stage4.png)

What changed, and why:

- **`PersistedState` separates what is saved from what is not.** Serialising the whole
  `TodoApp` would either save transient things (an open dialog, a half-typed draft) or
  need `#[serde(skip)]` on most fields. One nested struct makes the boundary obvious.
- **`#[serde(default)]` on every persisted struct** is what makes old save files load after
  you add a field. Without it, one new setting discards every user's data.
- **`save` is called for you** on exit and every 30 seconds. On Windows the file is
  `%APPDATA%\egui-todo-tutorial\data\app.ron`. Delete it to reset the app.
- **`APP_ID` is not the window title.** The first argument of `run_native` names the storage
  folder, so the title is set separately with `with_title`.
- **`show` is separate from `eframe::App::ui`** so that tests can drive the whole app
  without an `eframe::Frame`.

#### Testing the UI

`egui_kittest` runs your UI without a window or GPU. It finds widgets through the
accessibility tree, the same way a screen reader does, and simulates clicks and typing.
The tests at the bottom of `app.rs` above cover the main flows. Three things to know:

- **`harness.run()`** runs frames until the UI settles. Call it after each simulated input.
- **Widgets are found by accessible name.** This is why the earlier stages added `accessible_name` and
  `labelled_by`. After each `run`, the harness also *fails the test* if any interactive
  widget has no name, so accessibility regressions are caught automatically.
- **`type_text` types into whichever widget has focus.** Focus the target first, as the
  `type_into` helper does.

Components that return actions can also be tested in isolation. Append this to
`src/components/todo_row.rs`:

```rust
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
```

```powershell
cargo test
```

```text
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Seven model tests, two settings tests, nine whole-app UI tests and one component test, all
without opening a window.

---

### Stage 5: HTTP requests on a background worker

**Goal:** import todos from a URL, without the window ever freezing while the request runs.

`Cargo.toml` dependencies added in this stage:

```toml
[dependencies]
# HTTP client. "json" adds `Response::json`, which decodes a body with serde.
reqwest = { version = "0.13", features = ["json"] }
# reqwest is async, so something has to drive it: "rt" is the runtime and "sync" the
# channel the UI uses to send it work.
tokio = { version = "1", features = ["rt", "sync"] }

[dev-dependencies]
# A real HTTP server on a random local port, so tests never touch the network.
wiremock = "0.6"
tokio = { version = "1", features = ["macros"] }
```

An HTTP request takes anywhere from milliseconds to the length of its timeout. `ui` has to
return in a few milliseconds, every frame. So the request cannot happen inside `ui`, and
`ui` cannot wait for it. This stage solves that with three pieces, each ignorant of the
other two:

| File | Knows about | Does not know about |
|---|---|---|
| `import.rs` | HTTP, JSON, the model | egui, threads |
| `worker.rs` | threads, channels, the async runtime | egui, what a window is |
| `components/import_dialog.rs` | egui | HTTP, threads |

`TodoApp` is the only place that connects them:

```text
UI thread (egui)                          worker thread (tokio runtime)
----------------                          -----------------------------
worker.send(Command::ImportTodos)  ---->  commands.recv().await
        (returns at once)                 fetch_todos(..).await   (may take seconds)
worker.poll() at the top of `show` <----  events.send(Event::ImportFinished); wake()
```

#### The request: `import.rs`

Create `src/import.rs`:

@@file code/stage5/src/import.rs

What to notice:

- **`RemoteTodo` is not `NewTodo`.** The server's JSON shape and the app's model are
  different things that happen to look alike today. Decoding into a private wire type and
  converting keeps a renamed JSON field from rippling through the app.
- **One `Client`, built once, with a timeout.** A `Client` owns a connection pool. The
  default has *no* overall timeout, so a server that accepts the connection and then says
  nothing would leave the import running forever.
- **A non-2xx status is not an error to reqwest.** `send` succeeds for a 404. Check
  `status()` yourself (or call `error_for_status`).
- **`ImportError` is written for the user.** `reqwest::Error` says things like
  "error sending request for url"; the enum turns that into something a person can act on,
  and gives tests a value to compare against.

The tests run against `wiremock`, which starts a real HTTP server on a free local port.
That exercises the whole client stack (connection, status, headers, body decoding,
timeout) with no dependency on the internet.

#### The background worker: `worker.rs`

Create `src/worker.rs`:

@@file code/stage5/src/worker.rs

What to notice:

- **Messages, not shared state.** No `Arc<Mutex<..>>` appears anywhere. The UI owns the
  todos, the worker owns the HTTP client, and the only things that cross between threads
  are `Command` and `Event` values.
- **The worker does not depend on egui.** It is handed a `wake` closure and calls it when
  an event is ready. The app passes `ctx.request_repaint()`; the test passes a counter.
- **`send` and `poll` never block.** Both are safe to call in the middle of drawing.
- **The import is spawned, not awaited.** If the loop awaited the request, it could not
  receive `CancelImport` until the request had finished, which defeats the purpose.
- **`Drop` shuts the thread down.** Dropping the handle closes the command channel, the
  loop ends, the runtime is dropped and any request in flight is cancelled with it.

#### The dialog: `ImportDialog`

Create `src/components/import_dialog.rs` and re-export `ImportDialog` and `ImportOutcome`
from `components/mod.rs`:

@@file code/stage5/src/components/import_dialog.rs

This is the stateful component shape from Stage 2 again, with one difference: part of its
state (`Progress`) is set from outside. The dialog cannot know when a download finishes,
so it exposes `set_fetching`, `set_idle` and `set_failed` and lets the app call them.
`ui.add_enabled(false, ..)` greys the URL box out and stops it taking input while a
request for that URL is running.

#### The model

An imported item is not a `Todo` yet, because only the list hands out IDs. Add a type for
it, an action that carries a batch of them, and a method that appends them:

@@excerpt code/stage5/src/model.rs ||| /// A todo that does not exist yet ||| pub done: bool, ||| 1

@@excerpt code/stage5/src/model.rs ||| /// Append todos that came from outside the app. ||| Import(Vec<NewTodo>),

@@excerpt code/stage5/src/model.rs ||| /// Appends imported todos and returns how many ||| added_count += 1; ||| 4

#### Wiring it into the app

Add `mod import;` and `mod worker;` to `main.rs`. The complete `app.rs` for this stage is
in [`code/stage5/src/app.rs`](code/stage5/src/app.rs); these are the parts that changed.

`TodoApp` gains the worker handle and the transient state for one import:

@@excerpt code/stage5/src/app.rs ||| /// Does the slow work on another thread; see `worker.rs`. ||| notice: Option<String>,

`new` starts the worker, giving it a way to wake the UI. `from_state` now takes the worker
as a second argument and stores it:

@@excerpt code/stage5/src/app.rs ||| // `Context` is a cheap, thread-safe handle. ||| Self::from_state(state, worker)

`apply` gets an arm for the new action:

@@excerpt code/stage5/src/app.rs ||| TodoAction::Import(new_todos) => { ||| self.notice = Some(format!( ||| 1

Three new functions talk to the worker:

@@excerpt code/stage5/src/app.rs ||| /// Picks up whatever the worker has finished since the last frame. ||| self.worker.send(Command::CancelImport); ||| 1

`show` starts by taking in whatever the worker finished since the last frame:

@@excerpt code/stage5/src/app.rs ||| // Phase 0: take in results from the worker ||| self.handle_worker_events();

And, after the edit dialog, it shows the import dialog and acts on what the user asked for:

@@excerpt code/stage5/src/app.rs ||| // The dialog only reports what the user asked for. ||| ImportOutcome::Closed => { ||| 9

The File menu gets an "Import from URL…" item that sets
`self.import_dialog = Some(ImportDialog::new(&self.import_url))`, and the status bar shows
`notice` when there is one.

![The import dialog](images/stage5-dialog.png)

![Stage 5 after importing five todos](images/stage5.png)

What changed, and why:

- **Results enter through `apply`, like everything else.** A downloaded batch becomes a
  `TodoAction::Import`, so there is still exactly one function that changes the list.
- **`latest_import_id` handles the race that cancelling cannot.** Pressing Stop aborts the
  request, but its answer may already be sitting in the channel. Every request carries an
  ID, and an answer whose ID is not the newest is dropped. Without this, a stopped import
  could add its todos a moment after the user pressed Stop.
- **The spinner keeps the window alive while it is visible.** `ui.spinner()` requests a
  repaint each frame, so events are picked up promptly. Once the dialog closes nothing is
  animating, and that is when the `wake` callback matters: without it, a result arriving
  while the window is idle would not appear until the user moved the mouse.
- **Nothing in `ui` waits.** `worker.poll()` returns an empty list when nothing has
  arrived. There is no `recv()`, no `block_on`, no `join` on the UI thread.

#### Testing asynchronous UI

The existing tests call `harness.run()`, which steps frames until the UI stops asking for
repaints. Two things are different once a worker is involved: the result arrives in its
own time, and a visible spinner asks for repaints forever. So the tests step one frame at
a time until the thing they are waiting for is true:

@@excerpt code/stage5/src/app.rs ||| /// Steps the app one frame at a time ||| harness.step(); ||| 2

A whole-app test then reads like the user's actions, against a local mock server:

@@excerpt code/stage5/src/app.rs ||| fn importing_from_a_url_appends_the_downloaded_todos() { ||| harness.get_by_label("Imported 2 todos"); ||| 1

```powershell
cargo test -p stage5
```

```text
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The thirteen new tests cover the request (six cases, including a timeout and a server
that is not there), the worker, the dialog on its own, the model, and three whole-app
flows: a successful import, a failed one, and one stopped half-way.

---

### Stage 6: a SQLite database with sqlx

**Goal:** store the todos in a real database, written as they change, and keep the UI as
immediate as it was.

`Cargo.toml` dependencies added in this stage:

```toml
[dependencies]
# Async SQL toolkit. "runtime-tokio" makes it use the runtime the worker already has,
# "sqlite" compiles SQLite into the app (nothing to install), "migrate" runs the
# migrations folder and "macros" provides `migrate!` and `#[derive(FromRow)]`.
sqlx = { version = "0.9", default-features = false, features = ["runtime-tokio", "sqlite", "migrate", "macros"] }

[dev-dependencies]
# Throwaway folders for tests that need a real database file.
tempfile = "3"
```

Until now the whole todo list was serialised into eframe's save file on exit and every 30
seconds. That is fine for preferences, and a poor fit for data: a crash loses up to 30
seconds of work, and nothing else can read the file. From this stage on:

- **Todos** live in `todos.db`, a SQLite file next to the save file, and each change is
  written as it happens.
- **Preferences** (view mode, sidebar, settings) stay in eframe's save file.

The design rule is the one the app already follows. The in-memory `TodoList` remains what
every frame is drawn from. A change is applied to it first, so the UI responds at once,
and the matching write is then sent to the worker from Stage 5. The database is never
read while drawing; it is read once, at start-up.

#### The schema: a migration

Create `migrations/0001_create_todos.sql` next to `Cargo.toml`:

@@file code/stage6/migrations/0001_create_todos.sql

A migration is a numbered SQL file. sqlx records which ones a database has had, so the
schema can grow one file at a time without ever hand-editing a user's data. `position`
exists because rows in a table have no order of their own, and the user can drag todos.

The migrations are embedded at compile time, and Cargo does not watch SQL files. Add a
`build.rs` next to `Cargo.toml` so that editing a migration triggers a rebuild:

@@file code/stage6/build.rs

#### The repository: `storage.rs`

Create `src/storage.rs`. Every line of SQL in the app is in this file:

@@file code/stage6/src/storage.rs

What to notice:

- **`query_as` with `FromRow`.** `query_as::<_, TodoRecord>` runs the SQL and builds one
  `TodoRecord` per row, matching column names to field names at run time. A misspelt
  column is a run-time error, which is why every method has a test.
- **`TodoRecord` is not `Todo`,** for the same reason `RemoteTodo` was not `NewTodo`. The
  table stores the priority as an integer; the model has an enum. The conversion lives
  here, and the model stays free of sqlx.
- **Always `bind`.** Values never go into the SQL text. One of the tests stores a title
  that would be an injection attack if it were pasted into the statement.
- **One connection.** A pool of one suits a single-user app and SQLite's single writer,
  and it is required for in-memory databases, where each connection would otherwise get
  its own empty database.
- **`set_order` uses a transaction,** so a reorder is stored completely or not at all.

#### The model

`TodoId` now wraps an `i64`, the integer type SQLite stores, and can be rebuilt from a
stored value:

@@excerpt code/stage6/src/model.rs ||| pub struct TodoId(i64); ||| self.0 ||| 2

`TodoList` can be rebuilt from loaded rows. It carries on numbering after the highest ID
in use, so a new todo never clashes with a stored one:

@@excerpt code/stage6/src/model.rs ||| /// Rebuilds a list from todos that were loaded from storage ||| next_id: highest_id.map_or ||| 3

Three smaller changes: the `serde` derives are removed from the model, since it is no
longer serialised; the sample todos move from `app.rs` to `TodoList::with_sample_todos`;
and `TodoList::import` returns the IDs it assigned instead of a count, so the app can
save each new todo.

#### The worker learns to save

Add `mod storage;` to `main.rs`. The worker from Stage 5 gains five database commands and
two events, opens the database when it starts, and closes it when it stops:

@@file code/stage6/src/worker.rs

What to notice:

- **Writes are awaited in the loop; the import is still spawned.** The two kinds of work
  want opposite things. Writes must happen in the order the user made them, and take
  milliseconds. An import must be interruptible, and may take seconds.
- **Shutdown finishes the queue.** `recv` only returns `None` when the channel is closed
  *and* empty, and `Drop` joins the thread. A todo added a moment before the window
  closes is still written; a test proves it by dropping the worker straight after `send`.
- **A new database gets the sample todos.** "New" means the file did not exist, not that
  the table is empty, so deleting every todo does not bring the samples back.
- **A database that cannot be opened is reported, not fatal.** The app starts with an
  empty list and says in the status bar that nothing is being saved.

#### The app

`app.rs` in full:

@@file code/stage6/src/app.rs

![Stage 6 showing rows that were changed with plain SQL](images/stage6.png)

The screenshot shows the app after its database was edited from outside: one row was
inserted and one was renamed and completed with ordinary SQL while the app was closed.

What changed, and why:

- **`apply` changes memory first, then queues the write.** The frame after a click draws
  the new state without waiting for the disk. The model decides what the todo looks like
  (trimming the title, refusing a blank one) and the database is sent the result, so the
  two cannot disagree about the rules.
- **`todos_loaded` gates the UI.** For the few milliseconds before the first
  `TodosLoaded` event, the central panel shows a spinner instead of the add bar, and
  Import is disabled. Anything added before the load would be thrown away when the loaded
  list replaced it.
- **`pending_writes` and `storage_error` make saving visible.** "Saving…" appears only
  when a write is slow, and a failed write is shown in the status bar instead of being
  lost in a log.
- **`PersistedState` shrank to preferences.** eframe's `save` still runs on exit and every
  30 seconds, but it no longer carries anything the user would miss.

To look at the data yourself, open `%APPDATA%\egui-todo-tutorial\data\todos.db` with any
SQLite tool while the app is closed, for example `sqlite3 todos.db "SELECT * FROM todos"`.

#### Testing persistence

The repository tests use `DbLocation::InMemory`, so each test gets a private, empty
database and they can run in parallel. The whole-app tests do the same through
`app_harness`, which now waits for the todos to load before returning.

The test that matters most starts the app twice on one database file:

@@excerpt code/stage6/src/app.rs ||| fn changes_made_in_the_ui_are_still_there_after_a_restart() { ||| harness.get_by_label("Showing 3 of 3 · 1 completed"); ||| 1

```powershell
cargo test -p stage6
```

```text
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

#### Going further with sqlx

**Compile-time checked queries.** This stage checks its SQL at run time, with tests. sqlx
can instead check it while compiling, against a real database:

```rust,ignore
let records = sqlx::query_as!(
    TodoRecord,
    r#"SELECT id, title, notes, priority, done AS "done: bool" FROM todos ORDER BY position, id"#
)
.fetch_all(&self.pool)
.await?;
```

The macro connects to the database named by the `DATABASE_URL` environment variable at
build time, so a misspelt column or a wrong type fails the build. The cost is setup: every
build needs either that database or a query cache generated with `cargo sqlx prepare` and
committed to the repository. That trade is usually worth making on a team project and
rarely on a small one.

**Another database.** To move to Postgres, enable sqlx's `postgres` feature, replace
`SqlitePool` with `PgPool`, write `$1, $2` where SQLite uses `?`, and adjust the column
types in the migration. Nothing outside `storage.rs` and `migrations/` changes.

---

## 5. Common pitfalls & best practices

### Pitfall 1: unstable or clashing widget IDs

egui stores per-widget memory (focus, scroll position, open/closed, animations) under an
ID derived from the widget's *position in the code and its parent*. Two problems follow.

**Dynamic lists.** If rows are identified by position, deleting row 2 makes row 3 inherit
row 2's state: the wrong text box has focus, the wrong header is expanded. The fix is to
key each row by something stable, as every stage here does:

```rust
                        ui.push_id(todo.id, |ui| {
                            actions.extend(TodoRow::new(todo).show(ui));
                        });
```

Use a real identity (a database key, a UUID, a counter like `TodoId`), not the loop index.
An index is just the position again.

**Containers with the same salt.** `ComboBox`, `Grid`, `ScrollArea`, `CollapsingHeader` and
`Panel` take an ID salt. Two siblings with the same salt share state, and egui paints a
red warning over them in debug builds. Give each a distinct string
(`"draft_priority"`, `"priority_filter"`).

In 0.37, the two kinds of ID have separate spellings. A string passed to a container is a
*salt*, unique only among siblings. `egui::Id::unique(..)` is for things that live outside
the `Ui` tree, such as a `Modal`. `Id::new` is deprecated in favour of these.

### Pitfall 2: mutating state in the middle of drawing

The first instinct is to change data right where the click happens:

```rust,ignore
for todo in &self.todos {
    if ui.button("Delete").clicked() {
        self.todos.retain(|other| other.id != todo.id); // error[E0502]: already borrowed
    }
}
```

The borrow checker stops this one, but the workarounds people reach for (cloning the list
every frame, indexing with `self.todos[i]` and removing mid-loop, `RefCell`) either waste
work or skip and mis-draw items for a frame.

Draw first, mutate after. Record what the user asked for as data and apply it once the
borrows have ended:

```rust
        // Phase 2: mutate. Nothing is borrowed any more, so this is always allowed.
        for action in actions {
            self.apply(action);
        }
```

The change appears on the next frame, which egui schedules immediately after any input,
so the user never sees the delay. The same rule gives you one place to add logging,
validation or undo.

### Pitfall 3: assuming `ui` runs continuously, or blocking inside it

egui is *reactive*: `ui` runs when there is input, when an animation is in progress, or
when something asks for a repaint. Otherwise the app sleeps and uses no CPU. Two mistakes
come from misunderstanding this.

**Background results never show up.** If a thread or async task finishes while the user is
idle, nothing triggers a frame and the result stays invisible until they move the mouse.
Clone the `egui::Context` (it is cheap and thread-safe), give it to the worker, and call
`ctx.request_repaint()` when the result is ready. For a clock or countdown, use
`ctx.request_repaint_after(duration)`. Do not call `request_repaint()` unconditionally
every frame; that turns the app into a busy loop.

**The window freezes.** `ui` runs on the UI thread and must return in a few milliseconds.
A file read, HTTP request or `thread::sleep` inside it freezes the window for that long.
Start the work elsewhere, store the receiving end of a channel in your app struct, and
poll it with `try_recv()` at the top of `ui`. Stage 5 builds exactly this, and Stage 6
reuses it for database writes. For per-frame work that is merely heavy, such
as filtering a very large list, cache the result and recompute only when the inputs change.

### Coming from an older egui version

Most egui examples online predate 0.37, and several of their idioms no longer compile.

| Older code | egui 0.37 |
|---|---|
| `fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)` | `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`. Use `ui.ctx()` when you need the context. |
| `Box::new(\|cc\| Box::new(MyApp::default()))` | The creator returns a `Result`: `Box::new(\|cc\| Ok(Box::new(MyApp::default())))`. |
| `CentralPanel::default().show(ctx, ..)` | `.show(ui, ..)`. Panels nest inside a `Ui`. |
| `TopBottomPanel::top("id")`, `SidePanel::left("id")` | `Panel::top("id")`, `Panel::left("id")`. |
| `egui::menu::bar(ui, ..)` | `egui::MenuBar::new().ui(ui, ..)`. |
| `egui::Id::new("x")` | `egui::Id::unique("x")` for global IDs; a plain value where a salt is expected. |
| `WidgetType::Button` in `WidgetInfo` | `egui::Role::Button`. |
| Emoji in labels, e.g. `ui.button("🗑")` | The bundled font has no emoji. They render only if a system font supplies them, and panic in headless tests. Use text, paint the icon, or enable egui's `monochrome_emoji_fonts` feature. |

### Best practices used in this tutorial

- **Keep the model free of egui.** It is then testable, reusable, and unaffected by UI churn.
- **Choose the component shape by what it owns:** nothing (leaf `Widget`), a borrowed
  value (composite returning an action), or private working state (stateful struct).
- **Borrow immutably by default.** Hand a component `&mut` only for view state it is meant
  to edit.
- **Return actions; apply them in one place.**
- **Name every interactive widget** with a visible label, `labelled_by` or
  `accessible_name`, and let `egui_kittest` enforce it.
- **Read colours and spacing from `ui.visuals()` and `ui.spacing()`** in custom widgets so
  they follow the theme.
- **Persist a dedicated struct with `#[serde(default)]`,** not the whole app.
- **Do slow work on a worker thread and talk to it with messages.** `send` and `poll`
  never block, and no state is shared.
- **Give every request an ID** and ignore answers that are not for the newest one.
- **Change memory first, then save.** Draw from your own structs, never from the database.
- **Keep wire formats and table rows in their own types,** converted at the edge, so the
  model depends on neither the API nor the schema.
- **Keep all SQL in one module, always `bind` values,** and change the schema only through
  numbered migrations.

---

## 6. Next steps

Two extensions that reuse what you built. Each touches the model, an action, one
component and a test.

### 1. Undo for delete

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

### 2. Due dates

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

### Further reading

- [egui API documentation](https://docs.rs/egui/0.37.0)
- [eframe API documentation](https://docs.rs/eframe/0.37.0)
- [egui_kittest documentation](https://docs.rs/egui_kittest/0.37.0)
- [reqwest API documentation](https://docs.rs/reqwest/0.13)
- [sqlx API documentation](https://docs.rs/sqlx/0.9) and its [repository](https://github.com/launchbadge/sqlx), which documents `sqlx-cli` and migrations
- [tokio tutorial](https://tokio.rs/tokio/tutorial), for the runtime and channels the worker uses
- [The egui web demo](https://www.egui.rs), which shows every built-in widget with a link to its source
