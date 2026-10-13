use crate::tabs::TabManager;

/// Side tab sidebar controller supporting expand/collapse and active tab list rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideTabSidebar {
    pub is_collapsed: bool,
    pub width_pixels: u32,
}

pub const MIN_COLLAPSED_WIDTH: u32 = 62;
pub const DEFAULT_EXPANDED_WIDTH: u32 = 220;

impl Default for SideTabSidebar {
    fn default() -> Self {
        Self {
            is_collapsed: false,
            width_pixels: DEFAULT_EXPANDED_WIDTH,
        }
    }
}

impl SideTabSidebar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_collapse(&mut self) {
        if self.is_collapsed {
            self.expand();
        } else {
            self.collapse();
        }
    }

    pub fn collapse(&mut self) {
        self.is_collapsed = true;
        self.width_pixels = MIN_COLLAPSED_WIDTH;
    }

    pub fn expand(&mut self) {
        self.is_collapsed = false;
        self.width_pixels = DEFAULT_EXPANDED_WIDTH;
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
                is_pinned: tab.is_pinned,
            })
            .collect()
    }

    /// Renders pinned tab items (above the horizontal separator).
    pub fn render_pinned_items<'a>(&self, manager: &'a TabManager) -> Vec<SidebarTabItem<'a>> {
        self.render_items(manager)
            .into_iter()
            .filter(|item| item.is_pinned)
            .collect()
    }

    /// Renders normal tab items (below the horizontal separator).
    pub fn render_normal_items<'a>(&self, manager: &'a TabManager) -> Vec<SidebarTabItem<'a>> {
        self.render_items(manager)
            .into_iter()
            .filter(|item| !item.is_pinned)
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
    pub is_pinned: bool,
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
        assert!(!items[0].is_pinned);

        mgr.set_pinned(0, true);
        let pinned = sidebar.render_pinned_items(&mgr);
        let normal = sidebar.render_normal_items(&mgr);
        assert_eq!(pinned.len(), 1);
        assert_eq!(normal.len(), 1);
        assert!(pinned[0].is_pinned);
        assert!(!normal[0].is_pinned);
    }
}
