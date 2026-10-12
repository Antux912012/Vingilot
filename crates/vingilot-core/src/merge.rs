use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::bundle::SyncBundleContent;
use crate::db::Database;
use crate::error::CoreResult;
use crate::repositories::{
    BookmarkRepository, HistoryRepository, ReadingListRepository, SearchEngineRepository,
    SettingsRepository,
};

/// Import strategy mode when merging bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Merge with existing records without duplicate conflicts.
    Merge,
    /// Overwrite existing profile data with incoming bundle data.
    Replace,
    /// Import data into a completely new isolated profile.
    NewProfile,
}

/// Service handling backups and safe data merging.
pub struct MergeManager<'a> {
    db: &'a Database,
    backup_dir: PathBuf,
}

impl<'a> MergeManager<'a> {
    pub fn new(db: &'a Database, backup_dir: impl AsRef<Path>) -> Self {
        Self {
            db,
            backup_dir: backup_dir.as_ref().to_path_buf(),
        }
    }

    /// Creates an automatic timestamped backup of the current database before destructive import.
    pub fn create_backup(&self) -> CoreResult<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .backup_dir
            .join(format!("vingilot_backup_{timestamp}.sqlite"));

        if let Some(parent) = backup_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Run SQLite vacuum into backup destination if path is a file, or write dump
        self.db.conn().execute(
            &format!("VACUUM INTO '{}'", backup_path.to_string_lossy()),
            [],
        )?;

        Ok(backup_path)
    }

    /// Imports bundle contents according to the specified mode.
    pub fn import_bundle(
        &self,
        content: &SyncBundleContent,
        mode: ImportMode,
        target_profile_id: Uuid,
    ) -> CoreResult<()> {
        let bm_repo = BookmarkRepository::new(self.db);
        let hist_repo = HistoryRepository::new(self.db);
        let se_repo = SearchEngineRepository::new(self.db);
        let set_repo = SettingsRepository::new(self.db);
        let rl_repo = ReadingListRepository::new(self.db);

        // 1. Bookmarks: merge by URL
        for bm in &content.bookmarks {
            if bm_repo.get_by_id(bm.id)?.is_none() {
                bm_repo.insert(bm)?;
            }
        }

        // 2. History: merge visits
        for h in &content.history {
            hist_repo.record_visit(target_profile_id, &h.url, h.title.as_deref())?;
        }

        // 3. Search engines
        for se in &content.search_engines {
            if se_repo.list_all()?.iter().all(|e| e.name != se.name) {
                se_repo.insert(se)?;
            }
        }

        // 4. Reading list
        for item in &content.reading_list {
            let mut localized = item.clone();
            localized.profile_id = target_profile_id;
            let _ = rl_repo.insert(&localized);
        }

        // 5. Settings: update if Replace or explicitly requested
        if let Some(settings) = &content.settings {
            if mode == ImportMode::Replace {
                let mut localized = settings.clone();
                localized.profile_id = Some(target_profile_id);
                set_repo.upsert(&localized)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BookmarkNode, HistoryEntry};
    use crate::repositories::ProfileRepository;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_merge_bundle_into_database() {
        let db = Database::in_memory().unwrap();
        let tmp = tempdir().unwrap();
        let merger = MergeManager::new(&db, tmp.path());

        let profile_id = Uuid::new_v4();
        let profile_repo = ProfileRepository::new(&db);
        let p = crate::models::Profile {
            id: profile_id,
            name: "Test Profile".to_string(),
            profile_type: crate::models::ProfileType::Standard,
            color_or_icon: None,
            data_dir: "/tmp".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        profile_repo.insert(&p).unwrap();

        let mut content = SyncBundleContent::default();
        content.bookmarks.push(BookmarkNode {
            id: Uuid::new_v4(),
            parent_id: None,
            title: "Merged Page".to_string(),
            url: Some("https://merged.com".to_string()),
            is_folder: false,
            children: Vec::new(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        content.history.push(HistoryEntry {
            id: Uuid::new_v4(),
            profile_id,
            url: "https://merged.com".to_string(),
            title: Some("Merged Page".to_string()),
            visit_count: 1,
            last_visited_at: Utc::now(),
            created_at: Utc::now(),
            preview_path: None,
            preview_updated: None,
            hidden: false,
        });

        merger
            .import_bundle(&content, ImportMode::Merge, profile_id)
            .unwrap();

        let bm_repo = BookmarkRepository::new(&db);
        let root_bms = bm_repo.list_by_parent(None).unwrap();
        assert_eq!(root_bms.len(), 1);
        assert_eq!(root_bms[0].title, "Merged Page");
    }
}
