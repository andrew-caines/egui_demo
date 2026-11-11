//! Saving and loading todos with SQLite, through sqlx.
//!
//! Everything SQL lives in this file. The rest of the app sees a [`Repository`] with
//! a handful of methods that speak in model types, so swapping SQLite for Postgres
//! later would change this file and the migrations folder, and nothing else.

use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

use crate::model::{Priority, Todo, TodoId};

/// Every storage operation can fail the same way, so one alias keeps signatures short.
pub type StorageResult<T> = Result<T, sqlx::Error>;

/// Where the database lives.
#[derive(Clone, Debug)]
pub enum DbLocation {
    File(PathBuf),
    /// A private database that disappears when it is closed. Used by tests.
    InMemory,
}

/// One row of the `todos` table, exactly as SQLite stores it.
///
/// `FromRow` matches columns to fields *by name* when the query runs. Keeping this
/// separate from [`Todo`] lets the table use plain SQL types (integers for the ID
/// and priority) without those choices leaking into the model.
#[derive(sqlx::FromRow)]
struct TodoRecord {
    id: i64,
    title: String,
    notes: String,
    priority: i64,
    done: bool,
}

impl From<TodoRecord> for Todo {
    fn from(record: TodoRecord) -> Self {
        Self {
            id: TodoId::from_raw(record.id),
            title: record.title,
            notes: record.notes,
            priority: priority_from_db(record.priority),
            done: record.done,
        }
    }
}

fn priority_to_db(priority: Priority) -> i64 {
    match priority {
        Priority::Low => 0,
        Priority::Medium => 1,
        Priority::High => 2,
    }
}

fn priority_from_db(value: i64) -> Priority {
    match value {
        0 => Priority::Low,
        2 => Priority::High,
        // The table's CHECK constraint rules out anything but 0, 1 and 2.
        _ => Priority::Medium,
    }
}

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    /// Opens the database, creating the file if needed, and brings its schema up
    /// to date.
    pub async fn open(location: &DbLocation) -> StorageResult<Self> {
        let connect_options = match location {
            DbLocation::File(path) => SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
            DbLocation::InMemory => SqliteConnectOptions::new().in_memory(true),
        };

        let pool = SqlitePoolOptions::new()
            // One connection is the right size here: the worker runs one statement at
            // a time, and SQLite allows only one writer anyway. It is also what makes
            // `InMemory` work, because every new in-memory connection would otherwise
            // get its own empty database.
            .max_connections(1)
            // Keep that connection for the life of the app. Recycling it would wipe
            // an in-memory database.
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(connect_options)
            .await?;

        // `migrate!` reads the `migrations` folder at compile time and embeds the SQL
        // in the executable. Running it applies whichever files this database has not
        // seen yet, so it is safe to call on every start.
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Returns every todo in list order.
    pub async fn load_all(&self) -> StorageResult<Vec<Todo>> {
        let records: Vec<TodoRecord> = sqlx::query_as(
            "SELECT id, title, notes, priority, done FROM todos ORDER BY position, id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records.into_iter().map(Todo::from).collect())
    }

    /// Adds a todo at the end of the list.
    pub async fn insert(&self, todo: &Todo) -> StorageResult<()> {
        // Values always go through `bind`, never into the SQL text. That is what
        // makes a title like `'); DROP TABLE todos; --` harmless.
        sqlx::query(
            "INSERT INTO todos (id, title, notes, priority, done, position)
             VALUES (?, ?, ?, ?, ?, (SELECT COALESCE(MAX(position), -1) + 1 FROM todos))",
        )
        .bind(todo.id.raw())
        .bind(&todo.title)
        .bind(&todo.notes)
        .bind(priority_to_db(todo.priority))
        .bind(todo.done)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Writes the editable fields of an existing todo.
    pub async fn update(&self, todo: &Todo) -> StorageResult<()> {
        sqlx::query("UPDATE todos SET title = ?, notes = ?, priority = ?, done = ? WHERE id = ?")
            .bind(&todo.title)
            .bind(&todo.notes)
            .bind(priority_to_db(todo.priority))
            .bind(todo.done)
            .bind(todo.id.raw())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: TodoId) -> StorageResult<()> {
        sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id.raw())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_completed(&self) -> StorageResult<()> {
        sqlx::query("DELETE FROM todos WHERE done = 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Stores a new list order: `ordered_ids[0]` comes first, and so on.
    pub async fn set_order(&self, ordered_ids: &[TodoId]) -> StorageResult<()> {
        // A transaction makes the renumbering all-or-nothing. Without it, a crash
        // half-way through would leave some rows with old positions and some with new.
        let mut transaction = self.pool.begin().await?;
        for (position, id) in ordered_ids.iter().enumerate() {
            sqlx::query("UPDATE todos SET position = ? WHERE id = ?")
                .bind(position as i64)
                .bind(id.raw())
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await
    }

    /// Closes the connection, which flushes everything to the file.
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn todo(id: i64, title: &str) -> Todo {
        Todo {
            id: TodoId::from_raw(id),
            title: title.to_owned(),
            notes: String::new(),
            priority: Priority::Medium,
            done: false,
        }
    }

    /// An empty in-memory database holding the given titles, with IDs 1, 2, 3, ...
    async fn repository_with(titles: &[&str]) -> Repository {
        let repository = Repository::open(&DbLocation::InMemory).await.unwrap();
        for (index, title) in titles.iter().enumerate() {
            repository
                .insert(&todo(index as i64 + 1, title))
                .await
                .unwrap();
        }
        repository
    }

    async fn titles(repository: &Repository) -> Vec<String> {
        let todos = repository.load_all().await.unwrap();
        todos.into_iter().map(|todo| todo.title).collect()
    }

    #[tokio::test]
    async fn a_new_database_is_empty() {
        let repository = repository_with(&[]).await;

        assert!(repository.load_all().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn every_field_survives_a_round_trip() {
        let repository = repository_with(&[]).await;
        let original = Todo {
            id: TodoId::from_raw(42),
            title: "Robert'); DROP TABLE todos; --".to_owned(),
            notes: "line one\nline two".to_owned(),
            priority: Priority::High,
            done: true,
        };

        repository.insert(&original).await.unwrap();

        assert_eq!(repository.load_all().await.unwrap(), [original]);
    }

    #[tokio::test]
    async fn todos_load_in_the_order_they_were_inserted() {
        let repository = repository_with(&["First", "Second", "Third"]).await;

        assert_eq!(titles(&repository).await, ["First", "Second", "Third"]);
    }

    #[tokio::test]
    async fn update_changes_only_the_matching_row() {
        let repository = repository_with(&["Keep", "Change"]).await;
        let mut changed = todo(2, "Changed");
        changed.notes = "with notes".to_owned();
        changed.priority = Priority::Low;
        changed.done = true;

        repository.update(&changed).await.unwrap();

        let todos = repository.load_all().await.unwrap();
        assert_eq!(todos, [todo(1, "Keep"), changed]);
    }

    #[tokio::test]
    async fn delete_and_delete_completed_remove_the_right_rows() {
        let repository = repository_with(&["Open", "Done", "Unwanted"]).await;
        let mut done = todo(2, "Done");
        done.done = true;
        repository.update(&done).await.unwrap();

        repository.delete(TodoId::from_raw(3)).await.unwrap();
        assert_eq!(titles(&repository).await, ["Open", "Done"]);

        repository.delete_completed().await.unwrap();
        assert_eq!(titles(&repository).await, ["Open"]);
    }

    #[tokio::test]
    async fn set_order_is_what_load_all_returns() {
        let repository = repository_with(&["A", "B", "C"]).await;
        let id = TodoId::from_raw;

        repository.set_order(&[id(3), id(1), id(2)]).await.unwrap();
        assert_eq!(titles(&repository).await, ["C", "A", "B"]);

        // A todo added after a reorder still goes to the end.
        repository.insert(&todo(4, "D")).await.unwrap();
        assert_eq!(titles(&repository).await, ["C", "A", "B", "D"]);
    }

    #[tokio::test]
    async fn reopening_a_file_keeps_its_data_and_reruns_no_migration() {
        let folder = tempfile::tempdir().unwrap();
        let location = DbLocation::File(folder.path().join("todos.db"));

        let repository = Repository::open(&location).await.unwrap();
        repository.insert(&todo(1, "Still here")).await.unwrap();
        repository.close().await;

        // If the migration ran a second time, `CREATE TABLE` would fail here.
        let reopened = Repository::open(&location).await.unwrap();
        assert_eq!(titles(&reopened).await, ["Still here"]);
        reopened.close().await;
    }
}
