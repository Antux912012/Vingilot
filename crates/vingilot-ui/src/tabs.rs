use uuid::Uuid;
use vingilot_engine::{BrowserEngine, WebKitEngine, WebKitEngineConfig};

/// Represents an individual browser tab.
#[derive(Debug)]
pub struct Tab {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub engine: WebKitEngine,
    pub is_incognito: bool, // "Navigating Shadowy Seas"
    pub is_pinned: bool,
    pub favicon_ptr: usize,
    pub is_start_page: bool,
}

impl Tab {
    /// Creates a new tab. If `is_incognito` is true, runs in "Navigating Shadowy Seas" mode.
    pub fn new(is_incognito: bool) -> Self {
        let config = WebKitEngineConfig {
            data_directory: None, // Ephemeral if incognito
            cache_directory: None,
            enable_javascript: true,
            user_agent: None,
        };

        let title = if is_incognito {
            "Navigating Shadowy Seas".to_string()
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
            favicon_ptr: 0,
            is_start_page: !is_incognito,
        }
    }

    /// Navigates the tab to a URL.
    pub fn navigate(&mut self, url: &str) -> Result<(), String> {
        let trimmed = url.trim();
        if trimmed == "about:blank" || trimmed == "browser://start" || trimmed.is_empty() {
            self.is_start_page = true;
            self.url = trimmed.to_string();
            self.title = if self.is_incognito {
                "Navigating Shadowy Seas".to_string()
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
                self.title = format!("{title} (Shadowy Seas)");
            } else {
                self.title = title;
            }
        }
    }
}

/// Manages multiple tabs and tracking active selection.
#[derive(Debug)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
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

    /// Opens a new standard tab.
    pub fn new_tab(&mut self) -> Uuid {
        let tab = Tab::new(false);
        let id = tab.id;
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// Opens a new "Navigating Shadowy Seas" incognito tab.
    pub fn new_incognito_tab(&mut self) -> Uuid {
        let tab = Tab::new(true);
        let id = tab.id;
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// Closes the tab at `index`. If last tab is closed, opens a fresh tab.
    pub fn close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.tabs.push(Tab::new(false));
            self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        }
        true
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
    fn test_incognito_tab_shadowy_seas() {
        let mut mgr = TabManager::new();
        let incognito_id = mgr.new_incognito_tab();
        let tab = mgr.active_tab().unwrap();
        assert_eq!(tab.id, incognito_id);
        assert!(tab.is_incognito);
        assert_eq!(tab.title, "Navigating Shadowy Seas");
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
}
