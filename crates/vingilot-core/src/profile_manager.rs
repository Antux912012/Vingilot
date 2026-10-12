use chrono::Utc;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::{Profile, ProfileType};
use crate::repositories::ProfileRepository;

/// Manages profiles, persistence, default seed profiles, and profile data directories.
pub struct ProfileManager<'a> {
    repo: ProfileRepository<'a>,
    base_data_dir: PathBuf,
}

impl<'a> ProfileManager<'a> {
    pub fn new(db: &'a Database, base_data_dir: impl AsRef<Path>) -> Self {
        Self {
            repo: ProfileRepository::new(db),
            base_data_dir: base_data_dir.as_ref().to_path_buf(),
        }
    }

    /// Seeds default profiles (Standard, Work, Study, Custom) if no profiles exist.
    pub fn initialize_default_profiles(&self) -> CoreResult<Vec<Profile>> {
        let existing = self.repo.list_all()?;
        if !existing.is_empty() {
            return Ok(existing);
        }

        let defaults = vec![
            (
                "Standard",
                ProfileType::Standard,
                Some("blue".to_string()),
                "standard",
            ),
            ("Work", ProfileType::Work, Some("green".to_string()), "work"),
            (
                "Study",
                ProfileType::Study,
                Some("purple".to_string()),
                "study",
            ),
            (
                "Custom",
                ProfileType::Custom,
                Some("orange".to_string()),
                "custom",
            ),
        ];

        let mut created = Vec::new();
        let now = Utc::now();

        for (name, ptype, color, slug) in defaults {
            let data_dir = self.base_data_dir.join("profiles").join(slug);
            let profile = Profile {
                id: Uuid::new_v4(),
                name: name.to_string(),
                profile_type: ptype,
                color_or_icon: color,
                data_dir: data_dir.to_string_lossy().to_string(),
                default_search_engine_id: None,
                created_at: now,
                updated_at: now,
            };
            self.repo.insert(&profile)?;
            created.push(profile);
        }

        Ok(created)
    }

    pub fn list_profiles(&self) -> CoreResult<Vec<Profile>> {
        self.repo.list_all()
    }

    pub fn get_profile(&self, id: Uuid) -> CoreResult<Option<Profile>> {
        self.repo.get_by_id(id)
    }

    pub fn create_profile(
        &self,
        name: &str,
        profile_type: ProfileType,
        color: Option<String>,
    ) -> CoreResult<Profile> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let slug = name.to_lowercase().replace(' ', "-");
        let data_dir = self.base_data_dir.join("profiles").join(&slug);

        let profile = Profile {
            id,
            name: name.to_string(),
            profile_type,
            color_or_icon: color,
            data_dir: data_dir.to_string_lossy().to_string(),
            default_search_engine_id: None,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&profile)?;
        Ok(profile)
    }

    pub fn delete_profile(&self, id: Uuid) -> CoreResult<bool> {
        self.repo.delete(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_default_profiles() {
        let db = Database::in_memory().unwrap();
        let manager = ProfileManager::new(&db, "/tmp/vingilot-test");

        let profiles = manager.initialize_default_profiles().unwrap();
        assert_eq!(profiles.len(), 4);
        assert_eq!(profiles[0].name, "Standard");
        assert_eq!(profiles[1].name, "Work");
        assert_eq!(profiles[2].name, "Study");
        assert_eq!(profiles[3].name, "Custom");

        // Idempotent call returns existing
        let again = manager.initialize_default_profiles().unwrap();
        assert_eq!(again.len(), 4);
    }
}
