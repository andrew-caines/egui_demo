-- The schema, version 1. sqlx records which migration files have run in a table of
-- its own (_sqlx_migrations), so each file is applied exactly once per database.
CREATE TABLE todos (
    id       INTEGER PRIMARY KEY,
    title    TEXT    NOT NULL,
    notes    TEXT    NOT NULL DEFAULT '',
    -- 0 = low, 1 = medium, 2 = high.
    priority INTEGER NOT NULL DEFAULT 1 CHECK (priority BETWEEN 0 AND 2),
    -- SQLite has no boolean type; 0 and 1 are the convention.
    done     INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1)),
    -- Where the todo sits in the list. Lower comes first.
    position INTEGER NOT NULL
);
