//! Vingilot Core library: data models, repositories, sync, and import/export logic.

pub mod bookmark_manager;
pub mod cirdan;
pub mod db;
pub mod error;
pub mod models;
pub mod profile_manager;
pub mod repositories;

pub use bookmark_manager::BookmarkManager;
pub use cirdan::*;
pub use db::Database;
pub use error::{CoreError, CoreResult};
pub use models::*;
pub use profile_manager::ProfileManager;
pub use repositories::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_repositories_in_memory() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Profile repo test
        let profile_repo = ProfileRepository::new(&db);
        let profile_id = Uuid::new_v4();
        let profile = Profile {
            id: profile_id,
            name: "Default Profile".to_string(),
            profile_type: ProfileType::Standard,
            color_or_icon: None,
            data_dir: "/tmp/data".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        profile_repo
            .insert(&profile)
            .expect("Failed to insert profile");
        let fetched_profile = profile_repo
            .get_by_id(profile_id)
            .expect("Query failed")
            .expect("Profile not found");
        assert_eq!(fetched_profile.name, "Default Profile");

        // Search engine repo test
        let se_repo = SearchEngineRepository::new(&db);
        let se_id = Uuid::new_v4();
        let engine = SearchEngine {
            id: se_id,
            name: "DuckDuckGo".to_string(),
            search_url: "https://duckduckgo.com/?q=%s".to_string(),
            shortcut: Some("/d".to_string()),
            is_default: true,
            created_at: Utc::now(),
        };
        se_repo
            .insert(&engine)
            .expect("Failed to insert search engine");
        let default_se = se_repo
            .get_default()
            .expect("Query failed")
            .expect("No default search engine");
        assert_eq!(default_se.name, "DuckDuckGo");

        // Bookmark repo test
        let bm_repo = BookmarkRepository::new(&db);
        let bm_id = Uuid::new_v4();
        let bm = BookmarkNode {
            id: bm_id,
            parent_id: None,
            title: "Example Domain".to_string(),
            url: Some("https://example.com".to_string()),
            is_folder: false,
            children: Vec::new(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        bm_repo.insert(&bm).expect("Failed to insert bookmark");
        let fetched_bm = bm_repo
            .get_by_id(bm_id)
            .expect("Query failed")
            .expect("Bookmark not found");
        assert_eq!(fetched_bm.title, "Example Domain");

        // History repo test
        let hist_repo = HistoryRepository::new(&db);
        hist_repo
            .record_visit(profile_id, "https://example.com", Some("Example Domain"))
            .expect("Failed to record visit");
        hist_repo
            .record_visit(profile_id, "https://example.com", Some("Example Domain"))
            .expect("Failed to record second visit");
        let history_entry = hist_repo
            .get_by_url(profile_id, "https://example.com")
            .expect("Query failed")
            .expect("History entry not found");
        assert_eq!(history_entry.visit_count, 2);

        // Settings repo test
        let settings_repo = SettingsRepository::new(&db);
        let settings = UserSettings {
            profile_id: Some(profile_id),
            theme: "true-black".to_string(),
            ..UserSettings::default()
        };
        settings_repo
            .upsert(&settings)
            .expect("Failed to upsert settings");
        let fetched_settings = settings_repo
            .get_for_profile(Some(profile_id))
            .expect("Query failed")
            .expect("Settings not found");
        assert_eq!(fetched_settings.theme, "true-black");

        // Reading list repo test
        let rl_repo = ReadingListRepository::new(&db);
        let rl_id = Uuid::new_v4();
        let rl_item = ReadingListItem {
            id: rl_id,
            profile_id,
            url: "https://example.com/article".to_string(),
            title: "An Interesting Article".to_string(),
            byline: Some("Author Name".to_string()),
            content_html: Some("<p>Hello</p>".to_string()),
            content_text: Some("Hello".to_string()),
            read_status: false,
            added_at: Utc::now(),
        };
        rl_repo
            .insert(&rl_item)
            .expect("Failed to insert reading list item");
        let list = rl_repo
            .list_for_profile(profile_id)
            .expect("Failed to list reading list");
        assert_eq!(list.len(), 1);
        assert!(!list[0].read_status);
        rl_repo.mark_read(rl_id, true).expect("Failed to mark read");
        let updated_list = rl_repo.list_for_profile(profile_id).expect("Query failed");
        assert!(updated_list[0].read_status);
    }
}
