use uuid::Uuid;
use vingilot_engine::{BrowserEngine, WebKitEngine, WebKitEngineConfig};

/// Represents an individual browser tab.
#[derive(Debug)]
pub struct Tab {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub engine: WebKitEngine,
    pub is_incognito: bool,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub favicon_ptr: usize,
    pub is_start_page: bool,
    pub profile_id: String,
}

impl Tab {
    /// Creates a new tab. If `is_incognito` is true, runs in incognito mode.
    pub fn new(is_incognito: bool) -> Self {
        Self::new_with_profile(is_incognito, "standard")
    }

    /// Creates a new tab associated with a specific profile.
    pub fn new_with_profile(is_incognito: bool, profile_id: &str) -> Self {
        let config = WebKitEngineConfig {
            data_directory: None, // Ephemeral if incognito
            cache_directory: None,
            enable_javascript: true,
            user_agent: None,
        };

        let title = if is_incognito {
            "Incognito Mode".to_string()
        } else {
            "Start Page".to_string()
        };

        Self {
            id: Uuid::new_v4(),
            title,
            url: "about:blank".to_string(),
            engine: WebKitEngine::new(config),
            is_incognito,
            is_pinned: false,
            is_muted: false,
            favicon_ptr: 0,
            is_start_page: !is_incognito,
            profile_id: profile_id.to_string(),
        }
    }

    /// Navigates the tab to a URL.
    pub fn navigate(&mut self, url: &str) -> Result<(), String> {
        let trimmed = url.trim();
        if trimmed == "about:blank" || trimmed == "browser://start" || trimmed.is_empty() {
            self.is_start_page = true;
            self.url = trimmed.to_string();
            self.title = if self.is_incognito {
                "Incognito Mode".to_string()
            } else {
                "Start Page".to_string()
            };
            return Ok(());
        }
        self.is_start_page = false;
        self.engine.load_url(trimmed).map_err(|e| e.to_string())?;
        self.sync();
        Ok(())
    }

    /// Synchronizes URL and title from the active engine.
    pub fn sync(&mut self) {
        if let Ok(url) = self.engine.get_page_url() {
            self.url = url;
        }
        if let Ok(title) = self.engine.get_page_title() {
            if self.is_incognito {
                self.title = format!("{title} (Incognito)");
            } else {
                self.title = title;
            }
        }
    }
}

fn default_profile_id() -> String {
    "standard".to_string()
}

/// Metadata of a recently closed tab for reopening.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClosedTab {
    pub url: String,
    pub title: String,
    pub is_pinned: bool,
    #[serde(default = "default_profile_id")]
    pub profile_id: String,
}

/// Persisted tab session record for application restart.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedSessionTab {
    pub url: String,
    pub title: String,
    pub is_pinned: bool,
    #[serde(default = "default_profile_id")]
    pub profile_id: String,
}

/// Manages multiple tabs and tracking active selection.
#[derive(Debug)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
    closed_tabs: Vec<ClosedTab>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        let initial_tab = Tab::new(false);
        Self {
            tabs: vec![initial_tab],
            active_index: 0,
            closed_tabs: Vec::new(),
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_index)
    }

    pub fn closed_tabs(&self) -> &[ClosedTab] {
        &self.closed_tabs
    }

    pub fn can_reopen_tab(&self) -> bool {
        !self.closed_tabs.is_empty()
    }

    /// Opens a new standard tab.
    pub fn new_tab(&mut self) -> Uuid {
        self.new_tab_for_profile("standard")
    }

    /// Opens a new tab associated with a specific profile.
    pub fn new_tab_for_profile(&mut self, profile_id: &str) -> Uuid {
        let tab = Tab::new_with_profile(false, profile_id);
        let id = tab.id;
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// Opens a new incognito tab.
    pub fn new_incognito_tab(&mut self) -> Uuid {
        let tab = Tab::new(true);
        let id = tab.id;
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// Duplicates the tab at `index`, inserting it next to the source tab.
    pub fn duplicate_tab(&mut self, index: usize) -> Option<Uuid> {
        let source = self.tabs.get(index)?;
        let mut new_tab = Tab::new_with_profile(source.is_incognito, &source.profile_id);
        new_tab.url = source.url.clone();
        new_tab.title = source.title.clone();
        new_tab.is_start_page = source.is_start_page;
        new_tab.is_pinned = source.is_pinned;
        let id = new_tab.id;

        let insert_idx = index + 1;
        self.tabs.insert(insert_idx, new_tab);
        self.active_index = insert_idx;
        Some(id)
    }

    /// Closes the tab at `index`. If last tab is closed, opens a fresh tab.
    /// Non-incognito non-ephemeral closed tabs are pushed to `closed_tabs` stack.
    pub fn close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        let removed = self.tabs.remove(index);

        // Store into closed_tabs history if not incognito or ephemeral profile
        if !removed.is_incognito
            && removed.profile_id != "guest"
            && removed.profile_id != "high-security"
            && !removed.url.is_empty()
            && removed.url != "about:blank"
        {
            self.closed_tabs.push(ClosedTab {
                url: removed.url,
                title: removed.title,
                is_pinned: removed.is_pinned,
                profile_id: removed.profile_id,
            });
            if self.closed_tabs.len() > 30 {
                self.closed_tabs.remove(0);
            }
        }

        if self.tabs.is_empty() {
            self.tabs.push(Tab::new(false));
            self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        }
        true
    }

    /// Closes all tabs belonging to the same profile as `keep_index`, preserving tabs from other profiles.
    pub fn close_other_tabs(&mut self, keep_index: usize) -> bool {
        if keep_index >= self.tabs.len() {
            return false;
        }
        let keep_id = self.tabs[keep_index].id;
        let target_profile_id = self.tabs[keep_index].profile_id.clone();

        let mut remaining_tabs = Vec::new();
        for tab in self.tabs.drain(..) {
            if tab.id == keep_id || tab.profile_id != target_profile_id {
                remaining_tabs.push(tab);
            } else {
                if !tab.is_incognito
                    && tab.profile_id != "guest"
                    && tab.profile_id != "high-security"
                    && !tab.url.is_empty()
                    && tab.url != "about:blank"
                {
                    self.closed_tabs.push(ClosedTab {
                        url: tab.url,
                        title: tab.title,
                        is_pinned: tab.is_pinned,
                        profile_id: tab.profile_id,
                    });
                }
            }
        }

        self.tabs = remaining_tabs;
        if let Some(new_idx) = self.tabs.iter().position(|t| t.id == keep_id) {
            self.active_index = new_idx;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len().saturating_sub(1);
        }
        true
    }

    /// Closes all tabs to the right of `from_index` belonging to the same profile, preserving tabs from other profiles.
    pub fn close_tabs_to_right(&mut self, from_index: usize) -> bool {
        if from_index >= self.tabs.len() {
            return false;
        }
        let from_id = self.tabs[from_index].id;
        let target_profile_id = self.tabs[from_index].profile_id.clone();

        let mut remaining_tabs = Vec::new();
        let mut past_from = false;

        for tab in self.tabs.drain(..) {
            if !past_from {
                if tab.id == from_id {
                    past_from = true;
                }
                remaining_tabs.push(tab);
            } else {
                if tab.profile_id == target_profile_id {
                    if !tab.is_incognito
                        && tab.profile_id != "guest"
                        && tab.profile_id != "high-security"
                        && !tab.url.is_empty()
                        && tab.url != "about:blank"
                    {
                        self.closed_tabs.push(ClosedTab {
                            url: tab.url,
                            title: tab.title,
                            is_pinned: tab.is_pinned,
                            profile_id: tab.profile_id,
                        });
                    }
                } else {
                    remaining_tabs.push(tab);
                }
            }
        }

        self.tabs = remaining_tabs;
        if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len().saturating_sub(1);
        }
        true
    }

    /// Reopens the most recently closed tab matching `profile_id` (or falls back to the last closed tab).
    pub fn reopen_closed_tab_for_profile(&mut self, profile_id: &str) -> Option<Uuid> {
        let idx = self
            .closed_tabs
            .iter()
            .rposition(|t| t.profile_id == profile_id)
            .or_else(|| {
                if !self.closed_tabs.is_empty() {
                    Some(self.closed_tabs.len() - 1)
                } else {
                    None
                }
            })?;
        let closed = self.closed_tabs.remove(idx);
        let mut tab = Tab::new_with_profile(false, &closed.profile_id);
        tab.url = closed.url.clone();
        tab.title = closed.title.clone();
        tab.is_pinned = closed.is_pinned;
        tab.is_start_page = closed.url == "about:blank" || closed.url == "browser://start";
        let id = tab.id;
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        Some(id)
    }

    /// Reopens the most recently closed tab (scoped to active tab's profile if available).
    pub fn reopen_closed_tab(&mut self) -> Option<Uuid> {
        let active_prof = self
            .active_tab()
            .map(|t| t.profile_id.clone())
            .unwrap_or_else(|| "standard".to_string());
        self.reopen_closed_tab_for_profile(&active_prof)
    }

    /// Sets muted state on a tab.
    pub fn set_muted(&mut self, index: usize, muted: bool) -> bool {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.is_muted = muted;
            true
        } else {
            false
        }
    }

    /// Toggles audio mute state on a tab.
    pub fn toggle_mute(&mut self, index: usize) -> bool {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.is_muted = !tab.is_muted;
            true
        } else {
            false
        }
    }

    /// Switches the active tab to `index`.
    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    /// Closes the tab by its unique UUID.
    pub fn close_tab_by_id(&mut self, id: Uuid) -> bool {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.close_tab(pos)
        } else {
            false
        }
    }

    /// Sets pinned status for the tab at `index`.
    pub fn set_pinned(&mut self, index: usize, pinned: bool) -> bool {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.is_pinned = pinned;
            true
        } else {
            false
        }
    }

    /// Toggles pinned status for the tab at `index`.
    pub fn toggle_pin(&mut self, index: usize) -> bool {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.is_pinned = !tab.is_pinned;
            true
        } else {
            false
        }
    }

    /// Reorders tab from `from_index` to `to_index`.
    pub fn reorder_tab(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.tabs.len() || to_index >= self.tabs.len() {
            return false;
        }
        if from_index == to_index {
            return true;
        }
        let active_id = self.tabs.get(self.active_index).map(|t| t.id);
        let tab = self.tabs.remove(from_index);
        self.tabs.insert(to_index, tab);

        // Update active index
        if let Some(act_id) = active_id {
            if let Some(new_pos) = self.tabs.iter().position(|t| t.id == act_id) {
                self.active_index = new_pos;
            }
        }
        true
    }

    /// Saves session to JSON. Only normal non-incognito non-ephemeral tabs are saved.
    pub fn save_session(&self, path: &std::path::Path) -> std::io::Result<()> {
        let normal_tabs: Vec<SavedSessionTab> = self
            .tabs
            .iter()
            .filter(|t| {
                !t.is_incognito
                    && t.profile_id != "guest"
                    && t.profile_id != "high-security"
                    && !t.url.is_empty()
                    && t.url != "about:blank"
            })
            .map(|t| SavedSessionTab {
                url: t.url.clone(),
                title: t.title.clone(),
                is_pinned: t.is_pinned,
                profile_id: t.profile_id.clone(),
            })
            .collect();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&normal_tabs)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Restores session from JSON.
    pub fn restore_session(&mut self, path: &std::path::Path) -> std::io::Result<usize> {
        if !path.exists() {
            return Ok(0);
        }
        let content = std::fs::read_to_string(path)?;
        let saved_tabs: Vec<SavedSessionTab> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if saved_tabs.is_empty() {
            return Ok(0);
        }

        // Replace blank initial tab if only 1 blank tab exists
        let replace_initial = self.tabs.len() == 1
            && (self.tabs[0].url.is_empty() || self.tabs[0].url == "about:blank");
        if replace_initial {
            self.tabs.clear();
        }

        let count = saved_tabs.len();
        for saved in saved_tabs {
            let mut tab = Tab::new_with_profile(false, &saved.profile_id);
            tab.url = saved.url.clone();
            tab.title = saved.title;
            tab.is_pinned = saved.is_pinned;
            tab.is_start_page = saved.url == "about:blank" || saved.url == "browser://start";
            self.tabs.push(tab);
        }
        self.active_index = 0;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_manager_creation_and_switch() {
        let mut mgr = TabManager::new();
        assert_eq!(mgr.tabs().len(), 1);
        assert_eq!(mgr.active_index(), 0);

        let id2 = mgr.new_tab();
        assert_eq!(mgr.tabs().len(), 2);
        assert_eq!(mgr.active_index(), 1);
        assert_eq!(mgr.active_tab().unwrap().id, id2);

        assert!(mgr.switch_tab(0));
        assert_eq!(mgr.active_index(), 0);
    }

    #[test]
    fn test_incognito_tab() {
        let mut mgr = TabManager::new();
        let incognito_id = mgr.new_incognito_tab();
        let tab = mgr.active_tab().unwrap();
        assert_eq!(tab.id, incognito_id);
        assert!(tab.is_incognito);
        assert_eq!(tab.title, "Incognito Mode");
    }

    #[test]
    fn test_tab_close_preserves_at_least_one() {
        let mut mgr = TabManager::new();
        assert_eq!(mgr.tabs().len(), 1);
        mgr.close_tab(0);
        assert_eq!(mgr.tabs().len(), 1);
    }

    #[test]
    fn test_tab_pinning_and_reorder() {
        let mut mgr = TabManager::new();
        let _id2 = mgr.new_tab();
        assert_eq!(mgr.tabs().len(), 2);

        assert!(!mgr.tabs()[0].is_pinned);
        assert!(mgr.set_pinned(0, true));
        assert!(mgr.tabs()[0].is_pinned);

        assert!(mgr.reorder_tab(1, 0));
        assert!(!mgr.tabs()[0].is_pinned);
        assert!(mgr.tabs()[1].is_pinned);
    }

    #[test]
    fn test_tab_duplicate_mute_and_extended_closures() {
        let mut mgr = TabManager::new();
        mgr.active_tab_mut().unwrap().url = "https://tab0.com".to_string();
        mgr.active_tab_mut().unwrap().title = "Tab 0".to_string();

        let _id1 = mgr.new_tab();
        mgr.active_tab_mut().unwrap().url = "https://tab1.com".to_string();
        mgr.active_tab_mut().unwrap().title = "Tab 1".to_string();

        let _id2 = mgr.new_tab();
        mgr.active_tab_mut().unwrap().url = "https://tab2.com".to_string();
        mgr.active_tab_mut().unwrap().title = "Tab 2".to_string();

        assert_eq!(mgr.tabs().len(), 3);

        // Duplicate tab 1
        let dup_id = mgr.duplicate_tab(1).unwrap();
        assert_eq!(mgr.tabs().len(), 4);
        assert_eq!(mgr.tabs()[2].id, dup_id);
        assert_eq!(mgr.tabs()[2].url, "https://tab1.com");

        // Audio mute toggle
        assert!(!mgr.tabs()[2].is_muted);
        assert!(mgr.toggle_mute(2));
        assert!(mgr.tabs()[2].is_muted);

        // Close tabs to right of tab 1
        assert!(mgr.close_tabs_to_right(1));
        assert_eq!(mgr.tabs().len(), 2);
        assert_eq!(mgr.tabs()[0].url, "https://tab0.com");
        assert_eq!(mgr.tabs()[1].url, "https://tab1.com");

        // Reopen closed tab
        assert!(mgr.can_reopen_tab());
        let reopened_id = mgr.reopen_closed_tab().unwrap();
        assert_eq!(mgr.tabs().len(), 3);
        assert_eq!(mgr.active_tab().unwrap().id, reopened_id);

        // Close other tabs keeping tab 0
        assert!(mgr.close_other_tabs(0));
        assert_eq!(mgr.tabs().len(), 1);
        assert_eq!(mgr.tabs()[0].url, "https://tab0.com");
    }

    #[test]
    fn test_tab_session_persistence_excludes_incognito() {
        let temp_dir =
            std::env::temp_dir().join(format!("vingilot_test_session_{}", uuid::Uuid::new_v4()));
        let session_file = temp_dir.join("session.json");

        let mut mgr = TabManager::new();
        mgr.active_tab_mut().unwrap().url = "https://normal-tab.org".to_string();
        mgr.active_tab_mut().unwrap().title = "Normal Tab".to_string();

        // Incognito tab
        mgr.new_incognito_tab();
        mgr.active_tab_mut().unwrap().url = "https://secret-bank.org".to_string();
        mgr.active_tab_mut().unwrap().title = "Secret Bank".to_string();

        assert_eq!(mgr.tabs().len(), 2);

        // Save session
        mgr.save_session(&session_file).unwrap();

        // Verify JSON contents directly to guarantee privacy
        let saved_content = std::fs::read_to_string(&session_file).unwrap();
        assert!(saved_content.contains("https://normal-tab.org"));
        assert!(!saved_content.contains("https://secret-bank.org"));

        // Restore into fresh manager
        let mut fresh_mgr = TabManager::new();
        let restored = fresh_mgr.restore_session(&session_file).unwrap();
        assert_eq!(restored, 1);
        assert_eq!(fresh_mgr.tabs().len(), 1);
        assert_eq!(fresh_mgr.tabs()[0].url, "https://normal-tab.org");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tab_profiles_isolation_and_session_persistence() {
        let temp_dir =
            std::env::temp_dir().join(format!("vingilot_test_prof_{}", uuid::Uuid::new_v4()));
        let session_file = temp_dir.join("session.json");

        let mut mgr = TabManager::new();
        // Standard tab
        mgr.active_tab_mut().unwrap().url = "https://standard.org".to_string();
        assert_eq!(mgr.tabs()[0].profile_id, "standard");

        // Guest tab
        let _guest_id = mgr.new_tab_for_profile("guest");
        mgr.active_tab_mut().unwrap().url = "https://ephemeral-guest.org".to_string();

        // High security tab
        let _hs_id = mgr.new_tab_for_profile("high-security");
        mgr.active_tab_mut().unwrap().url = "https://classified.onion".to_string();

        // Custom profile tab
        let _work_id = mgr.new_tab_for_profile("work");
        mgr.active_tab_mut().unwrap().url = "https://work-portal.internal".to_string();

        assert_eq!(mgr.tabs().len(), 4);

        // Save session
        mgr.save_session(&session_file).unwrap();

        // Verify JSON contents: standard and work should be saved, guest and high-security must NOT
        let content = std::fs::read_to_string(&session_file).unwrap();
        assert!(content.contains("https://standard.org"));
        assert!(content.contains("https://work-portal.internal"));
        assert!(!content.contains("https://ephemeral-guest.org"));
        assert!(!content.contains("https://classified.onion"));

        // Close guest tab: should NOT be placed in closed_tabs history
        mgr.close_tab(1);
        assert!(!mgr.closed_tabs().iter().any(|c| c.url.contains("guest")));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_profile_scoped_tab_closures_and_reopen() {
        let mut mgr = TabManager::new();
        // Tab 0: Standard (keep)
        mgr.active_tab_mut().unwrap().url = "https://standard-0.com".to_string();
        // Tab 1: Work
        mgr.new_tab_for_profile("work");
        mgr.active_tab_mut().unwrap().url = "https://work-1.com".to_string();
        // Tab 2: Standard
        mgr.new_tab_for_profile("standard");
        mgr.active_tab_mut().unwrap().url = "https://standard-2.com".to_string();
        // Tab 3: Standard
        mgr.new_tab_for_profile("standard");
        mgr.active_tab_mut().unwrap().url = "https://standard-3.com".to_string();
        // Tab 4: Work
        mgr.new_tab_for_profile("work");
        mgr.active_tab_mut().unwrap().url = "https://work-4.com".to_string();

        assert_eq!(mgr.tabs().len(), 5);

        // Close tabs to right from Tab 2 (Standard):
        // Tab 3 (Standard) should close, but Tab 4 (Work) must remain!
        assert!(mgr.close_tabs_to_right(2));
        assert_eq!(mgr.tabs().len(), 4);
        assert_eq!(mgr.tabs()[0].url, "https://standard-0.com");
        assert_eq!(mgr.tabs()[1].url, "https://work-1.com");
        assert_eq!(mgr.tabs()[2].url, "https://standard-2.com");
        assert_eq!(mgr.tabs()[3].url, "https://work-4.com");

        // Close other tabs keeping Tab 0 (Standard):
        // Tab 2 (Standard) should close, but Tab 1 (Work) and Tab 3 (Work) must remain!
        assert!(mgr.close_other_tabs(0));
        assert_eq!(mgr.tabs().len(), 3);
        assert_eq!(mgr.tabs()[0].url, "https://standard-0.com");
        assert_eq!(mgr.tabs()[1].url, "https://work-1.com");
        assert_eq!(mgr.tabs()[2].url, "https://work-4.com");

        // Reopen closed tab for standard profile:
        let reopened = mgr.reopen_closed_tab_for_profile("standard");
        assert!(reopened.is_some());
        assert_eq!(mgr.active_tab().unwrap().profile_id, "standard");
        assert_eq!(mgr.active_tab().unwrap().url, "https://standard-2.com");
    }
}
