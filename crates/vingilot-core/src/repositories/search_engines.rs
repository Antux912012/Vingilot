use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{CoreError, CoreResult};
use crate::models::SearchEngine;

pub struct SearchEngineRepository<'a> {
    db: &'a Database,
}

impl<'a> SearchEngineRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, engine: &SearchEngine) -> CoreResult<()> {
        self.db.conn().execute(
            r#"
            INSERT INTO search_engines (
                id, name, search_url, shortcut, is_default, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                engine.id.to_string(),
                engine.name,
                engine.search_url,
                engine.shortcut,
                if engine.is_default { 1 } else { 0 },
                engine.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_all(&self) -> CoreResult<Vec<SearchEngine>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, name, search_url, shortcut, is_default, created_at
            FROM search_engines
            ORDER BY is_default DESC, name ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let search_url: String = row.get(2)?;
            let shortcut: Option<String> = row.get(3)?;
            let is_default: i32 = row.get(4)?;
            let created_at_str: String = row.get(5)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(SearchEngine {
                id,
                name,
                search_url,
                shortcut,
                is_default: is_default != 0,
                created_at,
            })
        })?;

        let mut engines = Vec::new();
        for item in rows {
            engines.push(item?);
        }
        Ok(engines)
    }

    pub fn get_default(&self) -> CoreResult<Option<SearchEngine>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, name, search_url, shortcut, is_default, created_at
            FROM search_engines
            WHERE is_default = 1
            LIMIT 1
            "#,
        )?;

        let row = stmt
            .query_row([], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let search_url: String = row.get(2)?;
                let shortcut: Option<String> = row.get(3)?;
                let is_default: i32 = row.get(4)?;
                let created_at_str: String = row.get(5)?;

                let id = Uuid::parse_str(&id_str).unwrap_or_default();
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(SearchEngine {
                    id,
                    name,
                    search_url,
                    shortcut,
                    is_default: is_default != 0,
                    created_at,
                })
            })
            .optional()?;

        Ok(row)
    }

    pub fn set_default(&self, id: Uuid) -> CoreResult<()> {
        let conn = self.db.conn();
        conn.execute("UPDATE search_engines SET is_default = 0", [])?;
        let affected = conn.execute(
            "UPDATE search_engines SET is_default = 1 WHERE id = ?1",
            params![id.to_string()],
        )?;
        if affected == 0 {
            return Err(CoreError::NotFound(format!("Search engine {id} not found")));
        }
        Ok(())
    }
}
