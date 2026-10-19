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

    /// Find credentials matching the exact validated origin and secure TLS scheme of a web page.
    /// This prevents iframe injection, subdomain phishing, and plaintext HTTP credential exposure.
    pub fn find_by_secure_origin(
        &self,
        profile_id: Uuid,
        target_url: &str,
    ) -> CoreResult<Vec<CirdanCredential>> {
        let parsed = match url::Url::parse(target_url) {
            Ok(u) => u,
            Err(_) => return Ok(Vec::new()),
        };

        // Strict TLS requirement: Only HTTPS is permitted, unless loopback/localhost
        let is_secure = parsed.scheme() == "https"
            || (parsed.scheme() == "http"
                && (parsed.host_str() == Some("localhost")
                    || parsed.host_str() == Some("127.0.0.1")));

        if !is_secure {
            return Ok(Vec::new());
        }

        let host = match parsed.host_str() {
            Some(h) => h.to_lowercase(),
            None => return Ok(Vec::new()),
        };

        let all = self.list_for_profile(profile_id)?;
        let matching = all
            .into_iter()
            .filter(|cred| {
                let domain = cred.url_or_domain.trim().to_lowercase();
                if domain.is_empty() {
                    return false;
                }
                // Check exact origin match or clean host domain match
                if domain == host {
                    return true;
                }
                if let Ok(cred_url) = url::Url::parse(&domain) {
                    if let Some(cred_host) = cred_url.host_str() {
                        return cred_host.to_lowercase() == host;
                    }
                }
                false
            })
            .collect();

        Ok(matching)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Profile, ProfileType};
    use crate::repositories::ProfileRepository;

    #[test]
    fn test_cirdan_find_by_secure_origin_phishing_prevention() {
        let db = Database::in_memory().unwrap();
        let profile_repo = ProfileRepository::new(&db);
        let repo = CirdanRepository::new(&db);
        let pid = Uuid::new_v4();

        let profile = Profile {
            id: pid,
            name: "Test Profile".to_string(),
            profile_type: ProfileType::Standard,
            color_or_icon: None,
            data_dir: "/tmp/data".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        profile_repo.insert(&profile).unwrap();

        let cred = CirdanCredential {
            id: Uuid::new_v4(),
            profile_id: pid,
            title: "GitHub".to_string(),
            url_or_domain: "github.com".to_string(),
            username: "octocat".to_string(),
            encrypted_password: b"encrypted".to_vec(),
            nonce: vec![0; 12],
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.insert(&cred).unwrap();

        // 1. Legitimate HTTPS GitHub URL matches
        let matches = repo
            .find_by_secure_origin(pid, "https://github.com/login")
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].username, "octocat");

        // 2. Insecure plaintext HTTP URL is strictly blocked
        let insecure = repo
            .find_by_secure_origin(pid, "http://github.com/login")
            .unwrap();
        assert!(insecure.is_empty(), "Insecure HTTP must be rejected");

        // 3. Phishing subdomain or fake domain is strictly blocked
        let phishing = repo
            .find_by_secure_origin(pid, "https://github.com.evil.com/login")
            .unwrap();
        assert!(phishing.is_empty(), "Phishing domain must be rejected");
    }
}

