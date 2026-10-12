use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::ReadingListItem;

pub struct ReadingListRepository<'a> {
    db: &'a Database,
}

impl<'a> ReadingListRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, item: &ReadingListItem) -> CoreResult<()> {
        self.db.conn().execute(
            r#"
            INSERT INTO reading_list (
                id, profile_id, url, title, byline, content_html, content_text,
                read_status, added_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                item.id.to_string(),
                item.profile_id.to_string(),
                item.url,
                item.title,
                item.byline,
                item.content_html,
                item.content_text,
                if item.read_status { 1 } else { 0 },
                item.added_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_for_profile(&self, profile_id: Uuid) -> CoreResult<Vec<ReadingListItem>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, profile_id, url, title, byline, content_html, content_text,
                   read_status, added_at
            FROM reading_list
            WHERE profile_id = ?1
            ORDER BY added_at DESC
            "#,
        )?;

        let rows = stmt.query_map(params![profile_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let profile_id_str: String = row.get(1)?;
            let url: String = row.get(2)?;
            let title: String = row.get(3)?;
            let byline: Option<String> = row.get(4)?;
            let content_html: Option<String> = row.get(5)?;
            let content_text: Option<String> = row.get(6)?;
            let read_status: i32 = row.get(7)?;
            let added_at_str: String = row.get(8)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let pid = Uuid::parse_str(&profile_id_str).unwrap_or_default();
            let added_at = DateTime::parse_from_rfc3339(&added_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(ReadingListItem {
                id,
                profile_id: pid,
                url,
                title,
                byline,
                content_html,
                content_text,
                read_status: read_status != 0,
                added_at,
            })
        })?;

        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn mark_read(&self, id: Uuid, read: bool) -> CoreResult<bool> {
        let count = self.db.conn().execute(
            "UPDATE reading_list SET read_status = ?1 WHERE id = ?2",
            params![if read { 1 } else { 0 }, id.to_string()],
        )?;
        Ok(count > 0)
    }

    pub fn delete(&self, id: Uuid) -> CoreResult<bool> {
        let count = self.db.conn().execute(
            "DELETE FROM reading_list WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(count > 0)
    }
}
