use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{CoreError, CoreResult};
use crate::models::{Profile, ProfileType};

pub struct ProfileRepository<'a> {
    db: &'a Database,
}

impl<'a> ProfileRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, profile: &Profile) -> CoreResult<()> {
        let profile_type_str = match profile.profile_type {
            ProfileType::Standard => "standard",
            ProfileType::Work => "work",
            ProfileType::Study => "study",
            ProfileType::Custom => "custom",
        };

        self.db.conn().execute(
            r#"
            INSERT INTO profiles (
                id, name, profile_type, color_or_icon, data_dir,
                default_search_engine_id, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                profile.id.to_string(),
                profile.name,
                profile_type_str,
                profile.color_or_icon,
                profile.data_dir,
                profile.default_search_engine_id.map(|u| u.to_string()),
                profile.created_at.to_rfc3339(),
                profile.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: Uuid) -> CoreResult<Option<Profile>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, name, profile_type, color_or_icon, data_dir,
                   default_search_engine_id, created_at, updated_at
            FROM profiles WHERE id = ?1
            "#,
        )?;

        let row = stmt
            .query_row(params![id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let profile_type_str: String = row.get(2)?;
                let color_or_icon: Option<String> = row.get(3)?;
                let data_dir: String = row.get(4)?;
                let default_search_engine_id_str: Option<String> = row.get(5)?;
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;

                Ok((
                    id_str,
                    name,
                    profile_type_str,
                    color_or_icon,
                    data_dir,
                    default_search_engine_id_str,
                    created_at_str,
                    updated_at_str,
                ))
            })
            .optional()?;

        match row {
            Some((id_str, name, pt_str, color, data_dir, dse_id_str, created_str, updated_str)) => {
                let id =
                    Uuid::parse_str(&id_str).map_err(|e| CoreError::Validation(e.to_string()))?;
                let profile_type = match pt_str.as_str() {
                    "work" => ProfileType::Work,
                    "study" => ProfileType::Study,
                    "custom" => ProfileType::Custom,
                    _ => ProfileType::Standard,
                };
                let default_search_engine_id = match dse_id_str {
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

                Ok(Some(Profile {
                    id,
                    name,
                    profile_type,
                    color_or_icon: color,
                    data_dir,
                    default_search_engine_id,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list_all(&self) -> CoreResult<Vec<Profile>> {
        let mut stmt = self.db.conn().prepare(
            r#"
            SELECT id, name, profile_type, color_or_icon, data_dir,
                   default_search_engine_id, created_at, updated_at
            FROM profiles ORDER BY created_at ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let profile_type_str: String = row.get(2)?;
            let color_or_icon: Option<String> = row.get(3)?;
            let data_dir: String = row.get(4)?;
            let default_search_engine_id_str: Option<String> = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;

            Ok((
                id_str,
                name,
                profile_type_str,
                color_or_icon,
                data_dir,
                default_search_engine_id_str,
                created_at_str,
                updated_at_str,
            ))
        })?;

        let mut profiles = Vec::new();
        for item in rows {
            let (id_str, name, pt_str, color, data_dir, dse_id_str, created_str, updated_str) =
                item?;
            let id = Uuid::parse_str(&id_str).map_err(|e| CoreError::Validation(e.to_string()))?;
            let profile_type = match pt_str.as_str() {
                "work" => ProfileType::Work,
                "study" => ProfileType::Study,
                "custom" => ProfileType::Custom,
                _ => ProfileType::Standard,
            };
            let default_search_engine_id = match dse_id_str {
                Some(s) => {
                    Some(Uuid::parse_str(&s).map_err(|e| CoreError::Validation(e.to_string()))?)
                }
                None => None,
            };
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| CoreError::Validation(e.to_string()))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| CoreError::Validation(e.to_string()))?
                .with_timezone(&Utc);

            profiles.push(Profile {
                id,
                name,
                profile_type,
                color_or_icon: color,
                data_dir,
                default_search_engine_id,
                created_at,
                updated_at,
            });
        }

        Ok(profiles)
    }

    pub fn delete(&self, id: Uuid) -> CoreResult<bool> {
        let count = self.db.conn().execute(
            "DELETE FROM profiles WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(count > 0)
    }
}
