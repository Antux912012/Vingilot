use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported profile categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    #[default]
    Standard,
    Work,
    Study,
    Custom,
}

/// A user profile representing an isolated browsing context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub profile_type: ProfileType,
    pub color_or_icon: Option<String>,
    pub data_dir: String,
    pub default_search_engine_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Node in a hierarchical bookmark tree (either a URL bookmark or a folder).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarkNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub url: Option<String>,
    pub is_folder: bool,
    pub children: Vec<BookmarkNode>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A browser history entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub visit_count: i64,
    pub last_visited_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Search engine definition with shortcut support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchEngine {
    pub id: Uuid,
    pub name: String,
    pub search_url: String,       // e.g. "https://duckduckgo.com/?q=%s"
    pub shortcut: Option<String>, // e.g. "/d", "/g"
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

/// Application and profile user preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: Uuid,
    pub profile_id: Option<Uuid>,
    pub theme: String, // "system", "light", "dark", "true-black"
    pub default_zoom_level: u32,
    pub kinetic_scrolling_enabled: bool,
    pub javascript_enabled: bool,
    pub reader_theme: String, // "light", "sepia", "dark", "true-black"
    pub reader_font_size: u32,
    pub ai_provider_base_url: Option<String>,
    pub ai_model_name: Option<String>,
    pub ai_api_key: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            profile_id: None,
            theme: "system".to_string(),
            default_zoom_level: 100,
            kinetic_scrolling_enabled: true,
            javascript_enabled: true,
            reader_theme: "light".to_string(),
            reader_font_size: 16,
            ai_provider_base_url: Some("http://localhost:11434/v1".to_string()),
            ai_model_name: Some("llama3.2".to_string()),
            ai_api_key: None,
            updated_at: Utc::now(),
        }
    }
}

/// An item in the reading list for offline or distraction-free viewing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingListItem {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub url: String,
    pub title: String,
    pub byline: Option<String>,
    pub content_html: Option<String>,
    pub content_text: Option<String>,
    pub read_status: bool,
    pub added_at: DateTime<Utc>,
}

/// Identity of a device participating in local bundle sync.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub device_id: Uuid,
    pub device_name: String,
    pub platform: String,
}

/// Options to enable/disable specific data categories during sync export/import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncOptions {
    pub include_bookmarks: bool,
    pub include_history: bool,
    pub include_settings: bool,
    pub include_search_engines: bool,
    pub include_reading_list: bool,
    pub include_userscripts: bool,
    pub include_api_keys: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            include_bookmarks: true,
            include_history: true,
            include_settings: true,
            include_search_engines: true,
            include_reading_list: true,
            include_userscripts: true,
            include_api_keys: false, // Security constraint: API keys default to false
        }
    }
}

/// Header manifest stored within a .vingilot sync bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncManifest {
    pub schema_version: u32,
    pub application: String,
    pub app_version: String,
    pub created_at: DateTime<Utc>,
    pub device: DeviceIdentity,
    pub options: SyncOptions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_options_defaults_exclude_api_keys() {
        let opts = SyncOptions::default();
        assert!(!opts.include_api_keys);
        assert!(opts.include_bookmarks);
    }

    #[test]
    fn test_bookmark_tree_serialization() {
        let folder_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let now = Utc::now();

        let child = BookmarkNode {
            id: child_id,
            parent_id: Some(folder_id),
            title: "Rust Lang".to_string(),
            url: Some("https://www.rust-lang.org".to_string()),
            is_folder: false,
            children: Vec::new(),
            position: 0,
            created_at: now,
            updated_at: now,
        };

        let root_folder = BookmarkNode {
            id: folder_id,
            parent_id: None,
            title: "Development".to_string(),
            url: None,
            is_folder: true,
            children: vec![child],
            position: 0,
            created_at: now,
            updated_at: now,
        };

        let serialized = serde_json::to_string(&root_folder).expect("Serialization failed");
        let deserialized: BookmarkNode =
            serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(root_folder, deserialized);
        assert_eq!(deserialized.children.len(), 1);
        assert_eq!(deserialized.children[0].title, "Rust Lang");
    }

    #[test]
    fn test_profile_types() {
        let profile = Profile {
            id: Uuid::new_v4(),
            name: "Work Profile".to_string(),
            profile_type: ProfileType::Work,
            color_or_icon: Some("blue".to_string()),
            data_dir: "/tmp/vingilot/profiles/work".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&profile).expect("Serialization failed");
        let parsed: Profile = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.profile_type, ProfileType::Work);
    }
}
