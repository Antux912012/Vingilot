use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Predefined symbolic icons available for profile selection.
pub const AVAILABLE_PROFILE_ICONS: &[&str] = &[
    "avatar-default-symbolic",
    "user-home-symbolic",
    "security-high-symbolic",
    "system-users-symbolic",
    "applications-games-symbolic",
    "applications-development-symbolic",
    "applications-science-symbolic",
    "weather-clear-night-symbolic",
    "starred-symbolic",
    "folder-music-symbolic",
];

/// A Zen Browser-style profile definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserProfile {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub builtin: bool,
    pub ephemeral: bool,
    pub high_security: bool,
}

impl BrowserProfile {
    /// Returns the default standard profile.
    pub fn standard() -> Self {
        BrowserProfile {
            id: "standard".to_string(),
            name: "Standard Profile".to_string(),
            icon: "avatar-default-symbolic".to_string(),
            builtin: true,
            ephemeral: false,
            high_security: false,
        }
    }

    /// Returns the 3 mandatory built-in profiles.
    pub fn default_profiles() -> Vec<Self> {
        vec![
            BrowserProfile {
                id: "standard".to_string(),
                name: "Standard Profile".to_string(),
                icon: "avatar-default-symbolic".to_string(),
                builtin: true,
                ephemeral: false,
                high_security: false,
            },
            BrowserProfile {
                id: "guest".to_string(),
                name: "Guest Profile".to_string(),
                icon: "system-users-symbolic".to_string(),
                builtin: true,
                ephemeral: true,
                high_security: false,
            },
            BrowserProfile {
                id: "high-security".to_string(),
                name: "High Security".to_string(),
                icon: "security-high-symbolic".to_string(),
                builtin: true,
                ephemeral: true,
                high_security: true,
            },
        ]
    }
}

/// Resolves the default profiles configuration file path.
pub fn default_profiles_path() -> PathBuf {
    vingilot_core::previews::default_data_dir().join("profiles.json")
}

/// In-memory manager and file persistence for profiles.json.
#[derive(Debug, Clone)]
pub struct ProfileStorage {
    profiles: Vec<BrowserProfile>,
    active_profile_id: String,
}

impl Default for ProfileStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileStorage {
    pub fn new() -> Self {
        Self {
            profiles: BrowserProfile::default_profiles(),
            active_profile_id: "standard".to_string(),
        }
    }

    /// Loads profiles from default path, ensuring built-ins always exist.
    pub fn load_or_default() -> Self {
        Self::load_from_path(&default_profiles_path())
    }

    /// Loads profiles from `profiles.json`, ensuring built-ins always exist.
    pub fn load_from_path(path: &Path) -> Self {
        let mut profiles = if path.exists() {
            if let Ok(data) = fs::read_to_string(path) {
                serde_json::from_str::<Vec<BrowserProfile>>(&data)
                    .unwrap_or_else(|_| BrowserProfile::default_profiles())
            } else {
                BrowserProfile::default_profiles()
            }
        } else {
            BrowserProfile::default_profiles()
        };

        // Guarantee that all 3 built-in profiles are present
        for def in BrowserProfile::default_profiles() {
            if !profiles.iter().any(|p| p.id == def.id) {
                profiles.push(def);
            }
        }

        Self {
            profiles,
            active_profile_id: "standard".to_string(),
        }
    }

    /// Persists profiles to default location.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to_path(&default_profiles_path())
    }

    /// Persists profiles to disk at `path`.
    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data = serde_json::to_string_pretty(&self.profiles)?;
        fs::write(path, data)
    }

    pub fn profiles(&self) -> &[BrowserProfile] {
        &self.profiles
    }

    pub fn active_profile_id(&self) -> &str {
        &self.active_profile_id
    }

    pub fn set_active_profile_id(&mut self, id: &str) -> bool {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_profile_id = id.to_string();
            true
        } else {
            false
        }
    }

    pub fn active_profile(&self) -> Option<&BrowserProfile> {
        self.profiles.iter().find(|p| p.id == self.active_profile_id)
    }

    pub fn get_profile(&self, id: &str) -> Option<&BrowserProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn add_profile(&mut self, name: &str, icon: &str) -> BrowserProfile {
        let cleaned = name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();
        let base_id = if cleaned.is_empty() {
            "profile".to_string()
        } else {
            cleaned
        };

        let mut id = base_id.clone();
        let mut counter = 1;
        while self.profiles.iter().any(|p| p.id == id) {
            id = format!("{base_id}-{counter}");
            counter += 1;
        }

        let new_prof = BrowserProfile {
            id: id.clone(),
            name: name.to_string(),
            icon: icon.to_string(),
            builtin: false,
            ephemeral: false,
            high_security: false,
        };
        self.profiles.push(new_prof.clone());
        self.active_profile_id = id;
        new_prof
    }

    pub fn rename_profile(&mut self, id: &str, new_name: &str) -> bool {
        if let Some(p) = self.profiles.iter_mut().find(|p| p.id == id) {
            p.name = new_name.to_string();
            true
        } else {
            false
        }
    }

    pub fn set_profile_icon(&mut self, id: &str, new_icon: &str) -> bool {
        if let Some(p) = self.profiles.iter_mut().find(|p| p.id == id) {
            p.icon = new_icon.to_string();
            true
        } else {
            false
        }
    }

    pub fn delete_profile(&mut self, id: &str) -> bool {
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            if self.profiles[pos].builtin {
                return false; // Cannot delete built-in profile
            }
            self.profiles.remove(pos);
            if self.active_profile_id == id {
                self.active_profile_id = "standard".to_string();
            }
            true
        } else {
            false
        }
    }
}

/// Resolves standard profile storage paths.
pub fn get_profile_data_dirs(profile_id: &str) -> (PathBuf, PathBuf) {
    let base = vingilot_core::previews::default_data_dir()
        .join("profiles")
        .join(profile_id);
    let data_dir = base.join("data");
    let cache_dir = base.join("cache");
    let _ = fs::create_dir_all(&data_dir);
    let _ = fs::create_dir_all(&cache_dir);
    (data_dir, cache_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profiles_preservation() {
        let storage = ProfileStorage::new();
        assert_eq!(storage.profiles().len(), 3);
        assert_eq!(storage.profiles()[0].id, "standard");
        assert_eq!(storage.profiles()[1].id, "guest");
        assert_eq!(storage.profiles()[2].id, "high-security");

        assert!(!storage.profiles()[0].ephemeral);
        assert!(storage.profiles()[1].ephemeral);
        assert!(storage.profiles()[2].ephemeral);
        assert!(storage.profiles()[2].high_security);
    }

    #[test]
    fn test_add_rename_and_delete_custom_profile() {
        let mut storage = ProfileStorage::new();
        let custom = storage.add_profile("Work Projects", "applications-development-symbolic");
        assert_eq!(custom.id, "work-projects");
        assert_eq!(custom.name, "Work Projects");
        assert_eq!(storage.active_profile_id(), "work-projects");
        assert_eq!(storage.profiles().len(), 4);

        // Rename
        assert!(storage.rename_profile("work-projects", "Work & Client Projects"));
        assert_eq!(storage.get_profile("work-projects").unwrap().name, "Work & Client Projects");

        // Change icon
        assert!(storage.set_profile_icon("work-projects", "folder-music-symbolic"));
        assert_eq!(storage.get_profile("work-projects").unwrap().icon, "folder-music-symbolic");

        // Cannot delete built-in profile
        assert!(!storage.delete_profile("standard"));
        assert!(!storage.delete_profile("guest"));
        assert!(!storage.delete_profile("high-security"));

        // Can delete custom profile
        assert!(storage.delete_profile("work-projects"));
        assert_eq!(storage.profiles().len(), 3);
        assert_eq!(storage.active_profile_id(), "standard");
    }

    #[test]
    fn test_persistence_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("vingilot-test-{}", uuid::Uuid::new_v4()));
        let file_path = temp_dir.join("profiles.json");

        let mut storage = ProfileStorage::load_from_path(&file_path);
        storage.add_profile("Study", "applications-science-symbolic");
        storage.save_to_path(&file_path).unwrap();

        let reloaded = ProfileStorage::load_from_path(&file_path);
        assert_eq!(reloaded.profiles().len(), 4);
        assert!(reloaded.get_profile("study").is_some());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_xdg_data_dir_resolution() {
        let original_xdg = std::env::var("XDG_DATA_HOME").ok();
        let test_dir = std::env::temp_dir().join(format!("vingilot-xdg-test-{}", uuid::Uuid::new_v4()));
        std::env::set_var("XDG_DATA_HOME", &test_dir);

        let p_path = default_profiles_path();
        assert_eq!(p_path, test_dir.join("vingilot").join("profiles.json"));

        let (data_dir, cache_dir) = get_profile_data_dirs("test-profile");
        assert_eq!(data_dir, test_dir.join("vingilot").join("profiles").join("test-profile").join("data"));
        assert_eq!(cache_dir, test_dir.join("vingilot").join("profiles").join("test-profile").join("cache"));
        assert!(data_dir.exists());
        assert!(cache_dir.exists());

        if let Some(orig) = original_xdg {
            std::env::set_var("XDG_DATA_HOME", orig);
        } else {
            std::env::remove_var("XDG_DATA_HOME");
        }
        let _ = fs::remove_dir_all(&test_dir);
    }
}
