use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use crate::cirdan::CirdanCredential;
use crate::db::Database;
use crate::error::CoreResult;

pub struct CirdanRepository<'a> {
    db: &'a Database,
}

impl<'a> CirdanRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, cred: &CirdanCredential) -> CoreResult<()> {
        self.db.conn().execute(
            r#"
            INSERT INTO cirdan_credentials (
                id, profile_id, title, url_or_domain, username,
                encrypted_password, nonce, notes, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                cred.id.to_string(),
                cred.profile_id.to_string(),
                cred.title,
                cred.url_or_domain,
                cred.username,
                cred.encrypted_password,
                cred.nonce,
                cred.notes,
                cred.created_at.to_rfc3339(),
                cred.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_for_profile(&self, profile_id: Uuid) -> CoreResult<Vec<CirdanCredential>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, profile_id, title, url_or_domain, username,
                   encrypted_password, nonce, notes, created_at, updated_at
            FROM cirdan_credentials
            WHERE profile_id = ?1
            ORDER BY title ASC
            "#,
        )?;

        let rows = stmt.query_map(params![profile_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let pid_str: String = row.get(1)?;
            let title: String = row.get(2)?;
            let url_or_domain: String = row.get(3)?;
            let username: String = row.get(4)?;
            let encrypted_password: Vec<u8> = row.get(5)?;
            let nonce: Vec<u8> = row.get(6)?;
            let notes: Option<String> = row.get(7)?;
            let created_at_str: String = row.get(8)?;
            let updated_at_str: String = row.get(9)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let pid = Uuid::parse_str(&pid_str).unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(CirdanCredential {
                id,
                profile_id: pid,
                title,
                url_or_domain,
                username,
                encrypted_password,
                nonce,
                notes,
                created_at,
                updated_at,
            })
        })?;

        let mut credentials = Vec::new();
        for item in rows {
            credentials.push(item?);
        }
        Ok(credentials)
    }

    pub fn delete(&self, id: Uuid) -> CoreResult<bool> {
        let count = self.db.conn().execute(
            "DELETE FROM cirdan_credentials WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(count > 0)
    }
}
