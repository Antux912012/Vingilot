use rusqlite::Connection;

/// Applies production-grade performance and concurrency hardening to an open SQLite connection.
///
/// Settings applied:
/// - `foreign_keys = ON`: Enforces relational integrity across profiles, history, and bookmarks.
/// - `journal_mode = WAL`: Write-Ahead Logging allows concurrent readers during active writes.
/// - `synchronous = NORMAL`: Drastically reduces sync system call latency while guaranteeing WAL integrity.
/// - `busy_timeout = 5000`: Waits up to 5 seconds on lock contention rather than failing immediately with SQLITE_BUSY.
/// - `cache_size = -2000`: Allocates 2,000 KiB page cache per connection.
pub fn configure_sqlite_connection(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA cache_size = -2000;
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_hardening_pragmas() {
        let conn = Connection::open_in_memory().unwrap();
        configure_sqlite_connection(&conn).unwrap();

        let fk: i32 = conn.query_row("PRAGMA foreign_keys;", [], |r| r.get(0)).unwrap();
        assert_eq!(fk, 1);

        let sync_mode: i32 = conn.query_row("PRAGMA synchronous;", [], |r| r.get(0)).unwrap();
        // 1 is NORMAL in SQLite PRAGMA synchronous
        assert_eq!(sync_mode, 1);

        let timeout: i32 = conn.query_row("PRAGMA busy_timeout;", [], |r| r.get(0)).unwrap();
        assert_eq!(timeout, 5000);
    }
}
