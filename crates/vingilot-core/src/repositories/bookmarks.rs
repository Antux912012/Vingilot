use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{CoreError, CoreResult};
use crate::models::BookmarkNode;

pub struct BookmarkRepository<'a> {
    db: &'a Database,
}

impl<'a> BookmarkRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, node: &BookmarkNode) -> CoreResult<()> {
        self.db.conn().execute(
            r#"
            INSERT INTO bookmarks (
                id, parent_id, title, url, is_folder, position, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                node.id.to_string(),
                node.parent_id.map(|u| u.to_string()),
                node.title,
                node.url,
                if node.is_folder { 1 } else { 0 },
                node.position,
                node.created_at.to_rfc3339(),
                node.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: Uuid) -> CoreResult<Option<BookmarkNode>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at
            FROM bookmarks WHERE id = ?1
            "#,
        )?;

        let row = stmt
            .query_row(params![id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let parent_id_str: Option<String> = row.get(1)?;
                let title: String = row.get(2)?;
                let url: Option<String> = row.get(3)?;
                let is_folder: i32 = row.get(4)?;
                let position: i32 = row.get(5)?;
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;

                Ok((
                    id_str,
                    parent_id_str,
                    title,
                    url,
                    is_folder,
                    position,
                    created_at_str,
                    updated_at_str,
                ))
            })
            .optional()?;

        match row {
            Some((
                id_str,
                parent_id_str,
                title,
                url,
                is_folder,
                position,
                created_str,
                updated_str,
            )) => {
                let id =
                    Uuid::parse_str(&id_str).map_err(|e| CoreError::Validation(e.to_string()))?;
                let parent_id = match parent_id_str {
                    Some(s) => Some(
                        Uuid::parse_str(&s).map_err(|e| CoreError::Validation(e.to_string()))?,
                    ),
                    None => None,
                };
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map_err(|e| CoreError::Validation(e.to_string()))?
                    .with_timezone(&Utc);
                let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                    .map_err(|e| CoreError::Validation(e.to_string()))?
                    .with_timezone(&Utc);

                Ok(Some(BookmarkNode {
                    id,
                    parent_id,
                    title,
                    url,
                    is_folder: is_folder != 0,
                    children: Vec::new(),
                    position,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list_by_parent(&self, parent_id: Option<Uuid>) -> CoreResult<Vec<BookmarkNode>> {
        let (sql, param_val) = match parent_id {
            Some(pid) => (
                "SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at FROM bookmarks WHERE parent_id = ?1 ORDER BY CASE WHEN is_folder != 0 THEN 0 ELSE 1 END, position ASC, title ASC, created_at ASC",
                Some(pid.to_string()),
            ),
            None => (
                "SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at FROM bookmarks WHERE parent_id IS NULL ORDER BY CASE WHEN is_folder != 0 THEN 0 ELSE 1 END, position ASC, title ASC, created_at ASC",
                None,
            ),
        };

        let mut stmt = self.db.conn().prepare(sql)?;
        let rows = match param_val {
            Some(pid_str) => stmt.query_map(params![pid_str], Self::map_row)?,
            None => stmt.query_map([], Self::map_row)?,
        };

        let mut nodes = Vec::new();
        for item in rows {
            nodes.push(item?);
        }
        Ok(nodes)
    }

    pub fn count_children(&self, parent_id: Option<Uuid>) -> CoreResult<usize> {
        let (sql, param_val) = match parent_id {
            Some(pid) => (
                "SELECT COUNT(*) FROM bookmarks WHERE parent_id = ?1",
                Some(pid.to_string()),
            ),
            None => (
                "SELECT COUNT(*) FROM bookmarks WHERE parent_id IS NULL",
                None,
            ),
        };

        let count: i64 = match param_val {
            Some(pid_str) => self
                .db
                .conn()
                .query_row(sql, params![pid_str], |row| row.get(0))?,
            None => self.db.conn().query_row(sql, [], |row| row.get(0))?,
        };

        Ok(count as usize)
    }

    pub fn rename(&self, id: Uuid, new_title: &str) -> CoreResult<bool> {
        let now = Utc::now();
        let count = self.db.conn().execute(
            "UPDATE bookmarks SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_title, now.to_rfc3339(), id.to_string()],
        )?;
        Ok(count > 0)
    }

    pub fn move_node(&self, id: Uuid, new_parent_id: Option<Uuid>, position: i32) -> CoreResult<bool> {
        let now = Utc::now();
        let count = self.db.conn().execute(
            "UPDATE bookmarks SET parent_id = ?1, position = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                new_parent_id.map(|u| u.to_string()),
                position,
                now.to_rfc3339(),
                id.to_string()
            ],
        )?;
        Ok(count > 0)
    }

    pub fn update_position(&self, id: Uuid, position: i32) -> CoreResult<bool> {
        let now = Utc::now();
        let count = self.db.conn().execute(
            "UPDATE bookmarks SET position = ?1, updated_at = ?2 WHERE id = ?3",
            params![position, now.to_rfc3339(), id.to_string()],
        )?;
        Ok(count > 0)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<BookmarkNode> {
        let id_str: String = row.get(0)?;
        let parent_id_str: Option<String> = row.get(1)?;
        let title: String = row.get(2)?;
        let url: Option<String> = row.get(3)?;
        let is_folder: i32 = row.get(4)?;
        let position: i32 = row.get(5)?;
        let created_at_str: String = row.get(6)?;
        let updated_at_str: String = row.get(7)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let parent_id = parent_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(BookmarkNode {
            id,
            parent_id,
            title,
            url,
            is_folder: is_folder != 0,
            children: Vec::new(),
            position,
            created_at,
            updated_at,
        })
    }

    pub fn delete(&self, id: Uuid) -> CoreResult<bool> {
        let count = self.db.conn().execute(
            "DELETE FROM bookmarks WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(count > 0)
    }
}
