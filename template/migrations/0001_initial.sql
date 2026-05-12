-- Orchestra initial migration
-- This file runs automatically on `cargo run` (via sqlx::migrate!)
-- Add your own tables below, or create new files: 0002_add_posts.sql, etc.

CREATE TABLE IF NOT EXISTS examples (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
