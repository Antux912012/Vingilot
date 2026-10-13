use std::path::Path;
use uuid::Uuid;
use vingilot_core::bookmark_manager::BookmarkStore;
use vingilot_core::db::Database;
use vingilot_core::error::CoreResult;
use vingilot_core::models::BookmarkBreadcrumb;
use vingilot_core::repositories::HistoryRepository;
use vingilot_core::search_engine_manager::SearchEngineManager;

/// Represents a card in the "Recently viewed" section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentCardItem {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub preview_path: Option<String>,
    pub has_preview: bool,
}

/// Represents a bookmark or folder item displayed in the bookmark grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkTileItem {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub url: Option<String>,
    pub is_folder: bool,
    pub position: i32,
}

/// View model and controller for the browser native Start Page.
#[derive(Debug, Clone)]
pub struct StartPageModel {
    pub current_folder_id: Option<Uuid>,
    pub breadcrumbs: Vec<BookmarkBreadcrumb>,
    pub recent_items: Vec<RecentCardItem>,
    pub bookmark_items: Vec<BookmarkTileItem>,
    pub search_engine_name: String,
    pub search_placeholder: String,
    pub default_search_url: String,
}

impl Default for StartPageModel {
    fn default() -> Self {
        Self {
            current_folder_id: None,
            breadcrumbs: vec![BookmarkBreadcrumb {
                id: None,
                title: "Bookmarks".to_string(),
            }],
            recent_items: Vec::new(),
            bookmark_items: Vec::new(),
            search_engine_name: "DuckDuckGo".to_string(),
            search_placeholder: "Search with DuckDuckGo".to_string(),
            default_search_url: "https://duckduckgo.com/?q=%s".to_string(),
        }
    }
}

impl StartPageModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines whether the input text should be treated as a direct URL or a search query.
    ///
    /// URL detection rules:
    /// - Begins with `http://` or `https://`
    /// - Begins with `localhost`
    /// - Contains a dot and no spaces
    /// - Appears to be a hostname
    pub fn input_is_url(input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("about:")
            || trimmed.starts_with("file://")
        {
            return true;
        }

        if trimmed.starts_with("localhost") {
            return true;
        }

        if trimmed.contains('.') && !trimmed.contains(' ') {
            return true;
        }

        false
    }

    /// Normalizes a URL input string. If scheme is missing, prepends `https://`.
    pub fn normalize_url(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("about:")
            || trimmed.starts_with("file://")
        {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        }
    }

    /// Extracts domain or host from a URL for clean display on recent cards.
    pub fn extract_domain(url: &str) -> String {
        let trimmed = url.trim();
        if let Some(stripped) = trimmed.strip_prefix("https://") {
            stripped.split('/').next().unwrap_or(stripped).to_string()
        } else if let Some(stripped) = trimmed.strip_prefix("http://") {
            stripped.split('/').next().unwrap_or(stripped).to_string()
        } else if let Some(_stripped) = trimmed.strip_prefix("file://") {
            "local file".to_string()
        } else {
            trimmed.split('/').next().unwrap_or(trimmed).to_string()
        }
    }

    /// Constructs the target URL from a search or address bar input.
    pub fn resolve_search_input(&self, input: &str) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if Self::input_is_url(trimmed) {
            Some(Self::normalize_url(trimmed))
        } else {
            let encoded = urlencoding_simple(trimmed);
            Some(self.default_search_url.replace("%s", &encoded))
        }
    }

    /// Reloads search engine preferences, recent cards, and current bookmark folder from database.
    pub fn reload(&mut self, db: &Database, profile_id: Uuid) -> CoreResult<()> {
        // 1. Search engine
        let se_mgr = SearchEngineManager::new(db);
        if let Ok(Some(default_engine)) = se_mgr.get_default() {
            self.search_engine_name = default_engine.name.clone();
            self.search_placeholder = format!("Search with {}", default_engine.name);
            self.default_search_url = default_engine.search_url;
        } else {
            self.search_engine_name = "DuckDuckGo".to_string();
            self.search_placeholder = "Search with DuckDuckGo".to_string();
            self.default_search_url = "https://duckduckgo.com/?q=%s".to_string();
        }

        // 2. Recent items (limit 5)
        let hist_repo = HistoryRepository::new(db);
        let recent_entries = hist_repo.list_recent(profile_id, 5)?;
        self.recent_items = recent_entries
            .into_iter()
            .map(|e| {
                let domain = Self::extract_domain(&e.url);
                let title = e.title.unwrap_or_else(|| domain.clone());
                let has_preview = e
                    .preview_path
                    .as_ref()
                    .map(|p| Path::new(p).exists())
                    .unwrap_or(false);
                RecentCardItem {
                    url: e.url,
                    title,
                    domain,
                    preview_path: e.preview_path,
                    has_preview,
                }
            })
            .collect();

        // 3. Bookmark items and breadcrumbs
        let store = BookmarkStore::new(db);
        self.breadcrumbs = store.get_breadcrumb_path(self.current_folder_id)?;

        let nodes = match self.current_folder_id {
            Some(fid) => store.get_children(fid)?,
            None => store.get_root_items()?,
        };

        self.bookmark_items = nodes
            .into_iter()
            .map(|node| BookmarkTileItem {
                id: node.id,
                parent_id: node.parent_id,
                title: node.title,
                url: node.url,
                is_folder: node.is_folder,
                position: node.position,
            })
            .collect();

        Ok(())
    }

    /// Enters a bookmark folder.
    pub fn navigate_folder(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        folder_id: Option<Uuid>,
    ) -> CoreResult<()> {
        self.current_folder_id = folder_id;
        self.reload(db, profile_id)
    }

    /// Creates a new folder inside the currently opened folder or at root.
    pub fn create_folder(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        title: &str,
    ) -> CoreResult<Uuid> {
        let store = BookmarkStore::new(db);
        let folder = store.create_folder(title, self.current_folder_id)?;
        self.reload(db, profile_id)?;
        Ok(folder.id)
    }

    /// Creates a new bookmark inside the currently opened folder or at root.
    pub fn create_bookmark(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        title: &str,
        url: &str,
    ) -> CoreResult<Uuid> {
        let store = BookmarkStore::new(db);
        let bm = store.create_bookmark(title, url, self.current_folder_id)?;
        self.reload(db, profile_id)?;
        Ok(bm.id)
    }

    /// Renames a bookmark or folder.
    pub fn rename_item(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        item_id: Uuid,
        new_title: &str,
    ) -> CoreResult<bool> {
        let store = BookmarkStore::new(db);
        let ok = store.rename_item(item_id, new_title)?;
        if ok {
            self.reload(db, profile_id)?;
        }
        Ok(ok)
    }

    /// Deletes a bookmark or folder.
    pub fn delete_item(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        item_id: Uuid,
    ) -> CoreResult<bool> {
        let store = BookmarkStore::new(db);
        let ok = store.delete_item(item_id)?;
        if ok {
            self.reload(db, profile_id)?;
        }
        Ok(ok)
    }

    /// Moves a bookmark or folder into a destination folder or to root.
    pub fn move_item(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        item_id: Uuid,
        target_folder_id: Option<Uuid>,
    ) -> CoreResult<bool> {
        let store = BookmarkStore::new(db);
        let ok = store.move_item(item_id, target_folder_id, None)?;
        if ok {
            self.reload(db, profile_id)?;
        }
        Ok(ok)
    }

    /// Moves an item out to the root bookmark level.
    pub fn move_item_to_root(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        item_id: Uuid,
    ) -> CoreResult<bool> {
        self.move_item(db, profile_id, item_id, None)
    }

    /// Removes a website from the "Recently viewed" section (marks hidden = 1 in history).
    pub fn remove_from_recent(
        &mut self,
        db: &Database,
        profile_id: Uuid,
        url: &str,
    ) -> CoreResult<()> {
        let hist_repo = HistoryRepository::new(db);
        hist_repo.hide_from_recent(profile_id, url)?;
        self.reload(db, profile_id)
    }
}

fn urlencoding_simple(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        assert!(StartPageModel::input_is_url("https://example.com"));
        assert!(StartPageModel::input_is_url("http://example.com/path"));
        assert!(StartPageModel::input_is_url("localhost:8080"));
        assert!(StartPageModel::input_is_url("rust-lang.org"));
        assert!(StartPageModel::input_is_url("news.ycombinator.com"));

        assert!(!StartPageModel::input_is_url("gtk4 drag and drop"));
        assert!(!StartPageModel::input_is_url("rust documentation"));
        assert!(!StartPageModel::input_is_url(""));
    }

    #[test]
    fn test_search_resolution() {
        let model = StartPageModel::default();

        let res_url = model.resolve_search_input("rust-lang.org").unwrap();
        assert_eq!(res_url, "https://rust-lang.org");

        let res_query = model.resolve_search_input("gtk4 drag and drop").unwrap();
        assert_eq!(res_query, "https://duckduckgo.com/?q=gtk4+drag+and+drop");
    }

    #[test]
    fn test_start_page_model_lifecycle() {
        let db = Database::in_memory().unwrap();
        let pm = vingilot_core::profile_manager::ProfileManager::new(&db, "/tmp");
        let profiles = pm.initialize_default_profiles().unwrap();
        let profile_id = profiles[0].id;

        // Populate search engines
        let se_mgr = SearchEngineManager::new(&db);
        se_mgr.initialize_defaults().unwrap();

        // Populate history
        let hist_repo = HistoryRepository::new(&db);
        hist_repo
            .record_visit(profile_id, "https://rust-lang.org", Some("Rust Language"))
            .unwrap();
        hist_repo
            .record_visit(
                profile_id,
                "https://news.ycombinator.com",
                Some("Hacker News"),
            )
            .unwrap();

        let mut model = StartPageModel::new();
        model.reload(&db, profile_id).unwrap();

        assert_eq!(model.search_engine_name, "DuckDuckGo");
        assert_eq!(model.search_placeholder, "Search with DuckDuckGo");
        assert_eq!(model.recent_items.len(), 2);
        assert_eq!(model.recent_items[0].domain, "news.ycombinator.com");

        // Create folder and bookmark
        let folder_id = model.create_folder(&db, profile_id, "Development").unwrap();
        let _bm_id = model
            .create_bookmark(&db, profile_id, "GitHub", "https://github.com")
            .unwrap();

        assert_eq!(model.bookmark_items.len(), 2);
        assert!(model.bookmark_items[0].is_folder); // Folders first!

        // Enter folder
        model
            .navigate_folder(&db, profile_id, Some(folder_id))
            .unwrap();
        assert_eq!(model.current_folder_id, Some(folder_id));
        assert_eq!(model.breadcrumbs.len(), 2);
        assert_eq!(model.breadcrumbs[1].title, "Development");
        assert_eq!(model.bookmark_items.len(), 0);

        // Add bookmark inside folder
        let sub_bm = model
            .create_bookmark(&db, profile_id, "GitLab", "https://gitlab.com")
            .unwrap();
        assert_eq!(model.bookmark_items.len(), 1);

        // Move bookmark to root
        assert!(model.move_item_to_root(&db, profile_id, sub_bm).unwrap());
        assert_eq!(model.bookmark_items.len(), 0);

        // Navigate back to root
        model.navigate_folder(&db, profile_id, None).unwrap();
        assert_eq!(model.bookmark_items.len(), 3); // Dev folder, GitHub, GitLab

        // Hide recent item
        model
            .remove_from_recent(&db, profile_id, "https://rust-lang.org")
            .unwrap();
        assert_eq!(model.recent_items.len(), 1);
        assert_eq!(model.recent_items[0].url, "https://news.ycombinator.com");
    }

    #[test]
    fn test_start_page_nested_folder_and_moving_item() {
        let db = Database::in_memory().unwrap();
        let pm = vingilot_core::profile_manager::ProfileManager::new(&db, "/tmp");
        let profiles = pm.initialize_default_profiles().unwrap();
        let profile_id = profiles[0].id;

        let mut model = StartPageModel::new();
        model.reload(&db, profile_id).unwrap();

        // Create folder A and bookmark 1 at root
        let folder_a = model.create_folder(&db, profile_id, "Folder A").unwrap();
        let bm_1 = model
            .create_bookmark(&db, profile_id, "Site 1", "https://site1.com")
            .unwrap();

        assert_eq!(model.bookmark_items.len(), 2);

        // Move bookmark 1 into Folder A
        let moved = model
            .move_item(&db, profile_id, bm_1, Some(folder_a))
            .unwrap();
        assert!(moved);
        assert_eq!(model.bookmark_items.len(), 1); // Only Folder A at root

        // Enter Folder A
        model
            .navigate_folder(&db, profile_id, Some(folder_a))
            .unwrap();
        assert_eq!(model.bookmark_items.len(), 1); // Site 1 inside Folder A

        // Create Folder B inside Folder A
        let folder_b = model.create_folder(&db, profile_id, "Folder B").unwrap();
        assert_eq!(model.bookmark_items.len(), 2); // Folder B + Site 1

        // Move Site 1 into Folder B
        let moved2 = model
            .move_item(&db, profile_id, bm_1, Some(folder_b))
            .unwrap();
        assert!(moved2);
        assert_eq!(model.bookmark_items.len(), 1); // Only Folder B inside Folder A

        // Navigate to Folder B
        model
            .navigate_folder(&db, profile_id, Some(folder_b))
            .unwrap();
        assert_eq!(model.bookmark_items.len(), 1);
        assert_eq!(model.bookmark_items[0].title, "Site 1");
        assert_eq!(model.breadcrumbs.len(), 3); // Bookmarks > Folder A > Folder B

        // Move Folder B into Folder B (should fail cycle check)
        let cycle_move = model
            .move_item(&db, profile_id, folder_b, Some(folder_b))
            .unwrap();
        assert!(!cycle_move);

        // Move Folder A into Folder B (should fail cycle check)
        let cycle_move2 = model
            .move_item(&db, profile_id, folder_a, Some(folder_b))
            .unwrap();
        assert!(!cycle_move2);
    }
}
