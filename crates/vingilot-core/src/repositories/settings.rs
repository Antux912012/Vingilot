use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::UserSettings;

pub struct SettingsRepository<'a> {
    db: &'a Database,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn upsert(&self, settings: &UserSettings) -> CoreResult<()> {
        self.db.conn().execute(
            r#"
            INSERT INTO settings (
                id, profile_id, theme, default_zoom_level, kinetic_scrolling_enabled,
                javascript_enabled, reader_theme, reader_font_size, ai_provider_base_url,
                ai_model_name, ai_api_key, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                theme = excluded.theme,
                default_zoom_level = excluded.default_zoom_level,
                kinetic_scrolling_enabled = excluded.kinetic_scrolling_enabled,
                javascript_enabled = excluded.javascript_enabled,
                reader_theme = excluded.reader_theme,
                reader_font_size = excluded.reader_font_size,
                ai_provider_base_url = excluded.ai_provider_base_url,
                ai_model_name = excluded.ai_model_name,
                ai_api_key = excluded.ai_api_key,
                updated_at = excluded.updated_at
            "#,
            params![
                settings.id.to_string(),
                settings.profile_id.map(|u| u.to_string()),
                settings.theme,
                settings.default_zoom_level,
                if settings.kinetic_scrolling_enabled {
                    1
                } else {
                    0
                },
                if settings.javascript_enabled { 1 } else { 0 },
                settings.reader_theme,
                settings.reader_font_size,
                settings.ai_provider_base_url,
                settings.ai_model_name,
                settings.ai_api_key,
                settings.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_for_profile(&self, profile_id: Option<Uuid>) -> CoreResult<Option<UserSettings>> {
        let (sql, param_val) = match profile_id {
            Some(pid) => (
                "SELECT id, profile_id, theme, default_zoom_level, kinetic_scrolling_enabled, javascript_enabled, reader_theme, reader_font_size, ai_provider_base_url, ai_model_name, ai_api_key, updated_at FROM settings WHERE profile_id = ?1",
                Some(pid.to_string()),
            ),
            None => (
                "SELECT id, profile_id, theme, default_zoom_level, kinetic_scrolling_enabled, javascript_enabled, reader_theme, reader_font_size, ai_provider_base_url, ai_model_name, ai_api_key, updated_at FROM settings WHERE profile_id IS NULL",
                None,
            ),
        };

        let mut stmt = self.db.conn().prepare(sql)?;
        let row = match param_val {
            Some(pid_str) => stmt.query_row(params![pid_str], Self::map_row).optional()?,
            None => stmt.query_row([], Self::map_row).optional()?,
        };

        Ok(row)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<UserSettings> {
        let id_str: String = row.get(0)?;
        let profile_id_str: Option<String> = row.get(1)?;
        let theme: String = row.get(2)?;
        let default_zoom_level: u32 = row.get(3)?;
        let kinetic_scrolling: i32 = row.get(4)?;
        let javascript: i32 = row.get(5)?;
        let reader_theme: String = row.get(6)?;
        let reader_font_size: u32 = row.get(7)?;
        let ai_base_url: Option<String> = row.get(8)?;
        let ai_model: Option<String> = row.get(9)?;
        let ai_key: Option<String> = row.get(10)?;
        let updated_str: String = row.get(11)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let profile_id = profile_id_str.and_then(|s| Uuid::parse_str(&s).ok());
        let updated_at = DateTime::parse_from_rfc3339(&updated_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(UserSettings {
            id,
            profile_id,
            theme,
            default_zoom_level,
            kinetic_scrolling_enabled: kinetic_scrolling != 0,
            javascript_enabled: javascript != 0,
            reader_theme,
            reader_font_size,
            ai_provider_base_url: ai_base_url,
            ai_model_name: ai_model,
            ai_api_key: ai_key,
            updated_at,
        })
    }
}
