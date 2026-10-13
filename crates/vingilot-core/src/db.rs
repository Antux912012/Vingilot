use rusqlite::Connection;
use std::path::Path;

use crate::error::CoreResult;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Connects to a SQLite database file at `path` and runs schema migrations.
    pub fn open<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Creates an in-memory SQLite database, primarily for unit testing.
    pub fn in_memory() -> CoreResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Access the underlying SQLite connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Mutable access to the underlying SQLite connection.
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Initializes tables and indexes.
    pub fn init_schema(&self) -> CoreResult<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                profile_type TEXT NOT NULL,
                color_or_icon TEXT,
                data_dir TEXT NOT NULL,
                default_search_engine_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                parent_id TEXT,
                title TEXT NOT NULL,
                url TEXT,
                is_folder INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES bookmarks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                url TEXT NOT NULL,
                title TEXT,
                visit_count INTEGER NOT NULL DEFAULT 1,
                last_visited_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                preview_path TEXT,
                preview_updated INTEGER,
                hidden INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS search_engines (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                search_url TEXT NOT NULL,
                shortcut TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                id TEXT PRIMARY KEY,
                profile_id TEXT UNIQUE,
                theme TEXT NOT NULL,
                default_zoom_level INTEGER NOT NULL,
                kinetic_scrolling_enabled INTEGER NOT NULL,
                javascript_enabled INTEGER NOT NULL,
                reader_theme TEXT NOT NULL,
                reader_font_size INTEGER NOT NULL,
                ai_provider_base_url TEXT,
                ai_model_name TEXT,
                ai_api_key TEXT,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS reading_list (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                byline TEXT,
                content_html TEXT,
                content_text TEXT,
                read_status INTEGER NOT NULL DEFAULT 0,
                added_at TEXT NOT NULL,
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS cirdan_credentials (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url_or_domain TEXT NOT NULL,
                username TEXT NOT NULL,
                encrypted_password BLOB NOT NULL,
                nonce BLOB NOT NULL,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_history_profile_url ON history(profile_id, url);
            CREATE INDEX IF NOT EXISTS idx_bookmarks_parent ON bookmarks(parent_id);
            CREATE INDEX IF NOT EXISTS idx_reading_list_profile ON reading_list(profile_id);
            CREATE INDEX IF NOT EXISTS idx_cirdan_profile_domain ON cirdan_credentials(profile_id, url_or_domain);
            "#,
        )?;

        // Run migrations for existing databases
        let _ = self
            .conn
            .execute("ALTER TABLE history ADD COLUMN preview_path TEXT;", []);
        let _ = self.conn.execute(
            "ALTER TABLE history ADD COLUMN preview_updated INTEGER;",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE history ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;",
            [],
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_init() {
        let db = Database::in_memory().expect("Failed to initialize database");
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .expect("Query failed");
        assert!(count >= 6);
    }
}
