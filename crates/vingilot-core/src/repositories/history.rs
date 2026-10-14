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

    pub fn update_preview(
        &self,
        profile_id: Uuid,
        url: &str,
        preview_path: &str,
    ) -> CoreResult<()> {
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

    pub fn delete_entry(&self, profile_id: Uuid, entry_id: Uuid) -> CoreResult<()> {
        self.db.conn().execute(
            "DELETE FROM history WHERE profile_id = ?1 AND id = ?2",
            params![profile_id.to_string(), entry_id.to_string()],
        )?;
        Ok(())
    }

    pub fn search(
        &self,
        profile_id: Uuid,
        query: &str,
        limit: usize,
    ) -> CoreResult<Vec<HistoryEntry>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, profile_id, url, title, visit_count, last_visited_at, created_at, preview_path, preview_updated, hidden
            FROM history
            WHERE profile_id = ?1
              AND (url LIKE ?2 OR title LIKE ?2)
            ORDER BY last_visited_at DESC
            LIMIT ?3
            "#,
        )?;

        let rows = stmt.query_map(
            params![profile_id.to_string(), pattern, limit as i64],
            |row| {
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
            },
        )?;

        let mut entries = Vec::new();
        for item in rows {
            entries.push(item?);
        }
        Ok(entries)
    }

    pub fn clear_range(
        &self,
        profile_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> CoreResult<usize> {
        let count = self.db.conn().execute(
            r#"
            DELETE FROM history
            WHERE profile_id = ?1
              AND last_visited_at >= ?2
              AND last_visited_at <= ?3
            "#,
            params![profile_id.to_string(), from.to_rfc3339(), to.to_rfc3339(),],
        )?;
        Ok(count)
    }

    pub fn clear_for_profile(&self, profile_id: Uuid) -> CoreResult<usize> {
        let count = self.db.conn().execute(
            "DELETE FROM history WHERE profile_id = ?1",
            params![profile_id.to_string()],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Profile, ProfileType};
    use crate::repositories::ProfileRepository;
    use chrono::Duration;

    #[test]
    fn test_history_search_and_clear_range() {
        let db = Database::in_memory().unwrap();
        let prof_repo = ProfileRepository::new(&db);
        let profile = Profile {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            profile_type: ProfileType::Standard,
            color_or_icon: None,
            data_dir: "default".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        prof_repo.insert(&profile).unwrap();

        let hist_repo = HistoryRepository::new(&db);
        let entry1 = hist_repo
            .record_visit(
                profile.id,
                "https://rust-lang.org",
                Some("Rust Programming Language"),
            )
            .unwrap();
        let entry2 = hist_repo
            .record_visit(
                profile.id,
                "https://crates.io",
                Some("Rust Package Registry"),
            )
            .unwrap();
        let _ = hist_repo
            .record_visit(profile.id, "https://github.com", Some("GitHub"))
            .unwrap();

        // Search test
        let results = hist_repo.search(profile.id, "Rust", 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.url == "https://rust-lang.org"));
        assert!(results.iter().any(|r| r.url == "https://crates.io"));

        // Delete single entry
        hist_repo.delete_entry(profile.id, entry1.id).unwrap();
        let results_after = hist_repo.search(profile.id, "Rust", 10).unwrap();
        assert_eq!(results_after.len(), 1);
        assert_eq!(results_after[0].id, entry2.id);

        // Clear range
        let now = Utc::now();
        let from = now - Duration::hours(1);
        let to = now + Duration::hours(1);
        let deleted = hist_repo.clear_range(profile.id, from, to).unwrap();
        assert_eq!(deleted, 2); // entry2 and github

        let remaining = hist_repo.list_recent(profile.id, 10).unwrap();
        assert_eq!(remaining.len(), 0);
    }
}
