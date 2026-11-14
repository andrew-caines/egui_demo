# Stage 6: a SQLite database with sqlx

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

## The schema: a migration

Create `migrations/0001_create_todos.sql` next to `Cargo.toml`:

@@file code/stage6/migrations/0001_create_todos.sql

A migration is a numbered SQL file. sqlx records which ones a database has had, so the
schema can grow one file at a time without ever hand-editing a user's data. `position`
exists because rows in a table have no order of their own, and the user can drag todos.

The migrations are embedded at compile time, and Cargo does not watch SQL files. Add a
`build.rs` next to `Cargo.toml` so that editing a migration triggers a rebuild:

@@file code/stage6/build.rs

## The repository: `storage.rs`

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

## The model

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

## The worker learns to save

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

## The app

`app.rs` in full:

@@file code/stage6/src/app.rs

![Stage 6 showing rows that were changed with plain SQL](../images/stage6.png)

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

## Testing persistence

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

## Going further with sqlx

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
