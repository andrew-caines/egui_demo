# Building a Todo List desktop app with egui 0.37

A hands-on tutorial for experienced Rust developers who are new to [egui](https://crates.io/crates/egui).
You will build a Windows desktop todo app in six stages, ending with a set of reusable,
tested UI components, HTTP requests that never freeze the window, and todos stored in SQLite.

> **Tested against:** `egui` / `eframe` / `egui_extras` / `egui_kittest` **0.37.0**, `reqwest` 0.13,
> `sqlx` 0.9, `tokio` 1.54, Rust 1.99, Windows 11.
> Every Rust block in this tutorial is copied from a file under [`code/`](code/) that compiles with zero warnings
> under `cargo clippy -- -D warnings`. egui 0.37 needs **Rust 1.95 or newer**.

## Contents

Each chapter is its own page, with links to the previous and next chapter at the top and bottom.

### Getting started

1. [Introduction](chapters/01-introduction.md)
   - [What we are building](chapters/01-introduction.md#what-we-are-building)
2. [Setup & installation](chapters/02-setup.md)
   - [Running the companion code](chapters/02-setup.md#running-the-companion-code)
3. [Core concepts: Hello World](chapters/03-hello-world.md)

### Step-by-step project build

4. [Stage 1: a working list in one file](chapters/04-stage-1-one-file.md)
5. [Stage 2: reusable components](chapters/05-stage-2-components.md)
   - [The model](chapters/05-stage-2-components.md#the-model)
   - [A leaf widget: `PriorityBadge`](chapters/05-stage-2-components.md#a-leaf-widget-prioritybadge)
   - [A composite component: `TodoRow`](chapters/05-stage-2-components.md#a-composite-component-todorow)
   - [A stateful component: `AddBar`](chapters/05-stage-2-components.md#a-stateful-component-addbar)
   - [The app](chapters/05-stage-2-components.md#the-app)
6. [Stage 3: a full application shell](chapters/06-stage-3-app-shell.md)
   - [The model grows](chapters/06-stage-3-app-shell.md#the-model-grows)
   - [A shared context menu](chapters/06-stage-3-app-shell.md#a-shared-context-menu)
   - [`TodoRow`, upgraded](chapters/06-stage-3-app-shell.md#todorow-upgraded)
   - [The sidebar: `FilterPanel`](chapters/06-stage-3-app-shell.md#the-sidebar-filterpanel)
   - [A modal: `EditDialog`](chapters/06-stage-3-app-shell.md#a-modal-editdialog)
   - [A table: `TodoTable`](chapters/06-stage-3-app-shell.md#a-table-todotable)
   - [The app shell](chapters/06-stage-3-app-shell.md#the-app-shell)
7. [Stage 4: persistence, theming, custom painting and UI tests](chapters/07-stage-4-persistence.md)
   - [Make the model serialisable](chapters/07-stage-4-persistence.md#make-the-model-serialisable)
   - [Settings and style](chapters/07-stage-4-persistence.md#settings-and-style)
   - [A custom animated widget: `ProgressRing`](chapters/07-stage-4-persistence.md#a-custom-animated-widget-progressring)
   - [A floating window: `SettingsWindow`](chapters/07-stage-4-persistence.md#a-floating-window-settingswindow)
   - [Persistence in the app](chapters/07-stage-4-persistence.md#persistence-in-the-app)
   - [Testing the UI](chapters/07-stage-4-persistence.md#testing-the-ui)
8. [Stage 5: HTTP requests on a background worker](chapters/08-stage-5-http.md)
   - [The request: `import.rs`](chapters/08-stage-5-http.md#the-request-importrs)
   - [The background worker: `worker.rs`](chapters/08-stage-5-http.md#the-background-worker-workerrs)
   - [The dialog: `ImportDialog`](chapters/08-stage-5-http.md#the-dialog-importdialog)
   - [The model](chapters/08-stage-5-http.md#the-model)
   - [Wiring it into the app](chapters/08-stage-5-http.md#wiring-it-into-the-app)
   - [Testing asynchronous UI](chapters/08-stage-5-http.md#testing-asynchronous-ui)
9. [Stage 6: a SQLite database with sqlx](chapters/09-stage-6-sqlite.md)
   - [The schema: a migration](chapters/09-stage-6-sqlite.md#the-schema-a-migration)
   - [The repository: `storage.rs`](chapters/09-stage-6-sqlite.md#the-repository-storagers)
   - [The model](chapters/09-stage-6-sqlite.md#the-model)
   - [The worker learns to save](chapters/09-stage-6-sqlite.md#the-worker-learns-to-save)
   - [The app](chapters/09-stage-6-sqlite.md#the-app)
   - [Testing persistence](chapters/09-stage-6-sqlite.md#testing-persistence)
   - [Going further with sqlx](chapters/09-stage-6-sqlite.md#going-further-with-sqlx)

### After the build

10. [Common pitfalls & best practices](chapters/10-pitfalls.md)
    - [Pitfall 1: unstable or clashing widget IDs](chapters/10-pitfalls.md#pitfall-1-unstable-or-clashing-widget-ids)
    - [Pitfall 2: mutating state in the middle of drawing](chapters/10-pitfalls.md#pitfall-2-mutating-state-in-the-middle-of-drawing)
    - [Pitfall 3: assuming `ui` runs continuously, or blocking inside it](chapters/10-pitfalls.md#pitfall-3-assuming-ui-runs-continuously-or-blocking-inside-it)
    - [Coming from an older egui version](chapters/10-pitfalls.md#coming-from-an-older-egui-version)
    - [Best practices used in this tutorial](chapters/10-pitfalls.md#best-practices-used-in-this-tutorial)
11. [Next steps](chapters/11-next-steps.md)
    - [1. Undo for delete](chapters/11-next-steps.md#1-undo-for-delete)
    - [2. Due dates](chapters/11-next-steps.md#2-due-dates)
    - [Further reading](chapters/11-next-steps.md#further-reading)

## Concepts

In-depth pages for the types named in the tutorial, starting from the [concepts index](concepts/README.md):

- [`CentralPanel`](concepts/central-panel.md)
- [`Ui`](concepts/ui.md)
- [`NativeOptions`](concepts/native-options.md)
- [`ViewportBuilder`](concepts/viewport-builder.md)
- [`run_native`](concepts/run-native.md)
- [`eframe::App`](concepts/app-trait.md)
- [`Layout`](concepts/layout.md)
- [`Response`](concepts/response.md)
