use crate::tabs::TabManager;

/// Side tab sidebar controller supporting expand/collapse and active tab list rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideTabSidebar {
    pub is_collapsed: bool,
    pub width_pixels: u32,
}

impl Default for SideTabSidebar {
    fn default() -> Self {
        Self {
            is_collapsed: false,
            width_pixels: 240,
        }
    }
}

impl SideTabSidebar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_collapse(&mut self) {
        self.is_collapsed = !self.is_collapsed;
    }

    pub fn collapse(&mut self) {
        self.is_collapsed = true;
    }

    pub fn expand(&mut self) {
        self.is_collapsed = false;
    }

    /// Renders view models for tab list items in the sidebar.
    pub fn render_items<'a>(&self, manager: &'a TabManager) -> Vec<SidebarTabItem<'a>> {
        manager
            .tabs()
            .iter()
            .enumerate()
            .map(|(i, tab)| SidebarTabItem {
                index: i,
                id: tab.id,
                title: &tab.title,
                url: &tab.url,
                is_active: i == manager.active_index(),
                is_incognito: tab.is_incognito,
            })
            .collect()
    }
}

/// View model representing an item in the side tab sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarTabItem<'a> {
    pub index: usize,
    pub id: uuid::Uuid,
    pub title: &'a str,
    pub url: &'a str,
    pub is_active: bool,
    pub is_incognito: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_collapse_toggle() {
        let mut sidebar = SideTabSidebar::new();
        assert!(!sidebar.is_collapsed);
        sidebar.toggle_collapse();
        assert!(sidebar.is_collapsed);
        sidebar.toggle_collapse();
        assert!(!sidebar.is_collapsed);
    }

    #[test]
    fn test_sidebar_render_items() {
        let mut mgr = TabManager::new();
        mgr.new_incognito_tab();
        let sidebar = SideTabSidebar::new();
        let items = sidebar.render_items(&mgr);
        assert_eq!(items.len(), 2);
        assert!(!items[0].is_active);
        assert!(items[1].is_active);
        assert!(items[1].is_incognito);
    }
}
