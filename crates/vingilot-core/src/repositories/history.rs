use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{CoreError, CoreResult};
use crate::models::HistoryEntry;

pub struct HistoryRepository<'a> {
    db: &'a Database,
}

impl<'a> HistoryRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn record_visit(
        &self,
        profile_id: Uuid,
        url: &str,
        title: Option<&str>,
    ) -> CoreResult<HistoryEntry> {
        let existing = self.get_by_url(profile_id, url)?;
        let now = Utc::now();

        if let Some(mut entry) = existing {
            entry.visit_count += 1;
            entry.last_visited_at = now;
            if title.is_some() {
                entry.title = title.map(|s| s.to_string());
            }

            self.db.conn().execute(
                r#"
                UPDATE history
                SET visit_count = ?1, last_visited_at = ?2, title = ?3
                WHERE id = ?4
                "#,
                params![
                    entry.visit_count,
                    entry.last_visited_at.to_rfc3339(),
                    entry.title,
                    entry.id.to_string(),
                ],
            )?;

            Ok(entry)
        } else {
            let entry = HistoryEntry {
                id: Uuid::new_v4(),
                profile_id,
                url: url.to_string(),
                title: title.map(|s| s.to_string()),
                visit_count: 1,
                last_visited_at: now,
                created_at: now,
                preview_path: None,
                preview_updated: None,
                hidden: false,
            };

            self.db.conn().execute(
                r#"
                INSERT INTO history (
                    id, profile_id, url, title, visit_count, last_visited_at, created_at, preview_path, preview_updated, hidden
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
                params![
                    entry.id.to_string(),
                    entry.profile_id.to_string(),
                    entry.url,
                    entry.title,
                    entry.visit_count,
                    entry.last_visited_at.to_rfc3339(),
                    entry.created_at.to_rfc3339(),
                    entry.preview_path,
                    entry.preview_updated,
                    if entry.hidden { 1 } else { 0 },
                ],
            )?;

            Ok(entry)
        }
    }

    pub fn get_by_url(&self, profile_id: Uuid, url: &str) -> CoreResult<Option<HistoryEntry>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, profile_id, url, title, visit_count, last_visited_at, created_at, preview_path, preview_updated, hidden
            FROM history
            WHERE profile_id = ?1 AND url = ?2
            "#,
        )?;

        let row = stmt
            .query_row(params![profile_id.to_string(), url], |row| {
                let id_str: String = row.get(0)?;
                let profile_id_str: String = row.get(1)?;
                let url: String = row.get(2)?;
                let title: Option<String> = row.get(3)?;
                let visit_count: i64 = row.get(4)?;
                let last_visited_at_str: String = row.get(5)?;
                let created_at_str: String = row.get(6)?;
                let preview_path: Option<String> = row.get(7)?;
                let preview_updated: Option<i64> = row.get(8)?;
                let hidden_int: i32 = row.get(9).unwrap_or(0);

                Ok((
                    id_str,
                    profile_id_str,
                    url,
                    title,
                    visit_count,
                    last_visited_at_str,
                    created_at_str,
                    preview_path,
                    preview_updated,
                    hidden_int != 0,
                ))
            })
            .optional()?;

        match row {
            Some((
                id_str,
                profile_id_str,
                url,
                title,
                visit_count,
                last_visited_str,
                created_str,
                preview_path,
                preview_updated,
                hidden,
            )) => {
                let id =
                    Uuid::parse_str(&id_str).map_err(|e| CoreError::Validation(e.to_string()))?;
                let profile_id = Uuid::parse_str(&profile_id_str)
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
                let last_visited_at = DateTime::parse_from_rfc3339(&last_visited_str)
                    .map_err(|e| CoreError::Validation(e.to_string()))?
                    .with_timezone(&Utc);
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map_err(|e| CoreError::Validation(e.to_string()))?
                    .with_timezone(&Utc);

                Ok(Some(HistoryEntry {
                    id,
                    profile_id,
                    url,
                    title,
                    visit_count,
                    last_visited_at,
                    created_at,
                    preview_path,
                    preview_updated,
                    hidden,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list_recent(&self, profile_id: Uuid, limit: usize) -> CoreResult<Vec<HistoryEntry>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, profile_id, url, title, visit_count, last_visited_at, created_at, preview_path, preview_updated, hidden
            FROM history
            WHERE profile_id = ?1
              AND hidden = 0
              AND url NOT LIKE 'about:%'
              AND url NOT LIKE 'browser:%'
            ORDER BY last_visited_at DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![profile_id.to_string(), limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let pid_str: String = row.get(1)?;
            let url: String = row.get(2)?;
            let title: Option<String> = row.get(3)?;
            let visit_count: i64 = row.get(4)?;
            let last_visited_at_str: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let preview_path: Option<String> = row.get(7)?;
            let preview_updated: Option<i64> = row.get(8)?;
            let hidden_int: i32 = row.get(9).unwrap_or(0);

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let profile_id = Uuid::parse_str(&pid_str).unwrap_or_default();
            let last_visited_at = DateTime::parse_from_rfc3339(&last_visited_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(HistoryEntry {
                id,
                profile_id,
                url,
                title,
                visit_count,
                last_visited_at,
                created_at,
                preview_path,
                preview_updated,
                hidden: hidden_int != 0,
            })
        })?;

        let mut entries = Vec::new();
        for item in rows {
            entries.push(item?);
        }
        Ok(entries)
    }

    pub fn update_preview(&self, profile_id: Uuid, url: &str, preview_path: &str) -> CoreResult<()> {
        let now_ts = Utc::now().timestamp();
        self.db.conn().execute(
            r#"
            UPDATE history
            SET preview_path = ?1, preview_updated = ?2
            WHERE profile_id = ?3 AND url = ?4
            "#,
            params![preview_path, now_ts, profile_id.to_string(), url],
        )?;
        Ok(())
    }

    pub fn hide_from_recent(&self, profile_id: Uuid, url: &str) -> CoreResult<()> {
        self.db.conn().execute(
            "UPDATE history SET hidden = 1 WHERE profile_id = ?1 AND url = ?2",
            params![profile_id.to_string(), url],
        )?;
        Ok(())
    }

    pub fn clear_for_profile(&self, profile_id: Uuid) -> CoreResult<usize> {
        let count = self.db.conn().execute(
            "DELETE FROM history WHERE profile_id = ?1",
            params![profile_id.to_string()],
        )?;
        Ok(count)
    }
}
