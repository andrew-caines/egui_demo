# Common pitfalls & best practices

[Previous: Stage 6: a SQLite database with sqlx](09-stage-6-sqlite.md) · [Contents](../README.md) · [Next: Next steps](11-next-steps.md)

## Pitfall 1: unstable or clashing widget IDs

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

## Pitfall 2: mutating state in the middle of drawing

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

## Pitfall 3: assuming `ui` runs continuously, or blocking inside it

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

## Coming from an older egui version

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

## Best practices used in this tutorial

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

[Previous: Stage 6: a SQLite database with sqlx](09-stage-6-sqlite.md) · [Contents](../README.md) · [Next: Next steps](11-next-steps.md)
