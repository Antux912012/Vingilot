use crate::layout::AdaptiveLayoutManager;
use crate::sidebar::SideTabSidebar;
use crate::tabs::TabManager;
use vingilot_engine::{BrowserEngine, WebKitEngine};

/// Controller for the Vingilot main application window and browser shell.
pub struct VingilotWindow {
    app_id: String,
    title: String,
    tab_manager: TabManager,
    sidebar: SideTabSidebar,
    layout: AdaptiveLayoutManager,
    address_input: String,
    default_search_url: String,
    accent_color: Option<String>,
    is_incognito: bool,
}

impl Default for VingilotWindow {
    fn default() -> Self {
        Self::new("io.github.antux912012.Vingilot", "Vingilot")
    }
}

impl VingilotWindow {
    /// Creates a new Vingilot window controller with tabs, sidebar, and adaptive layout.
    pub fn new(app_id: &str, title: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            title: title.to_string(),
            tab_manager: TabManager::new(),
            sidebar: SideTabSidebar::new(),
            layout: AdaptiveLayoutManager::new(1024, 768),
            address_input: String::new(),
            default_search_url: "https://duckduckgo.com/?q=%s".to_string(),
            accent_color: None,
            is_incognito: false,
        }
    }

    /// Creates a new incognito window controller initialized with immutable True-Black styling.
    pub fn new_incognito(app_id: &str, title: &str) -> Self {
        let mut win = Self::new(app_id, title);
        win.is_incognito = true;
        win
    }

    pub fn is_incognito(&self) -> bool {
        self.is_incognito
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn tab_manager(&self) -> &TabManager {
        &self.tab_manager
    }

    pub fn tab_manager_mut(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    pub fn sidebar(&self) -> &SideTabSidebar {
        &self.sidebar
    }

    pub fn sidebar_mut(&mut self) -> &mut SideTabSidebar {
        &mut self.sidebar
    }

    pub fn layout(&self) -> &AdaptiveLayoutManager {
        &self.layout
    }

    pub fn layout_mut(&mut self) -> &mut AdaptiveLayoutManager {
        &mut self.layout
    }

    pub fn address_input(&self) -> &str {
        &self.address_input
    }

    pub fn set_address_input(&mut self, text: &str) {
        self.address_input = text.to_string();
    }

    pub fn set_default_search_url(&mut self, url: &str) {
        self.default_search_url = url.to_string();
    }

    pub fn accent_color(&self) -> Option<&str> {
        self.accent_color.as_deref()
    }

    /// Applies an accent color to window chrome and sidebar, returning the corresponding CSS.
    /// In incognito mode, accent color customization is guarded and immutable.
    pub fn apply_accent_color(&mut self, hex_color: &str) -> String {
        if self.is_incognito {
            return String::new();
        }
        self.accent_color = Some(hex_color.to_string());
        format!(
            ".browser-chrome {{\n    background-image: linear-gradient(135deg, alpha({}, 0.22), alpha({}, 0.08));\n    transition: background-image 200ms ease;\n}}\n.tab-row.active, .sidebar-tab.active {{\n    background-color: alpha({}, 0.28);\n}}",
            hex_color, hex_color, hex_color
        )
    }

    /// Triggers navigation in the active tab from address bar entry or search query.
    pub fn submit_address_bar(&mut self) -> Result<(), String> {
        if self.address_input.trim().is_empty() {
            return Ok(());
        }

        let target = WebKitEngine::normalize_navigation_target(
            &self.address_input,
            &self.default_search_url,
        );

        if let Some(tab) = self.tab_manager.active_tab_mut() {
            tab.navigate(&target)?;
            self.title = if tab.is_incognito {
                format!("{} (Incognito) — Vingilot", tab.title)
            } else {
                format!("{} — Vingilot", tab.title)
            };
            self.address_input = tab.url.clone();
        }
        Ok(())
    }

    /// Action: Navigate Back in active tab
    pub fn on_back_clicked(&mut self) -> Result<(), String> {
        if let Some(tab) = self.tab_manager.active_tab_mut() {
            tab.engine.go_back().map_err(|e| e.to_string())?;
            tab.sync();
            self.sync_active_tab_state();
        }
        Ok(())
    }

    /// Action: Navigate Forward in active tab
    pub fn on_forward_clicked(&mut self) -> Result<(), String> {
        if let Some(tab) = self.tab_manager.active_tab_mut() {
            tab.engine.go_forward().map_err(|e| e.to_string())?;
            tab.sync();
            self.sync_active_tab_state();
        }
        Ok(())
    }

    /// Action: Reload active tab
    pub fn on_reload_clicked(&mut self) -> Result<(), String> {
        if let Some(tab) = self.tab_manager.active_tab_mut() {
            tab.engine.reload().map_err(|e| e.to_string())?;
            tab.sync();
            self.sync_active_tab_state();
        }
        Ok(())
    }

    /// Action: Stop Loading active tab
    pub fn on_stop_clicked(&mut self) -> Result<(), String> {
        if let Some(tab) = self.tab_manager.active_tab_mut() {
            tab.engine.stop_loading().map_err(|e| e.to_string())?;
            tab.sync();
            self.sync_active_tab_state();
        }
        Ok(())
    }

    /// Opens a new standard tab.
    pub fn open_new_tab(&mut self) -> uuid::Uuid {
        let id = self.tab_manager.new_tab();
        self.sync_active_tab_state();
        id
    }

    /// Opens a new incognito tab.
    pub fn open_new_incognito_tab(&mut self) -> uuid::Uuid {
        let id = self.tab_manager.new_incognito_tab();
        self.sync_active_tab_state();
        id
    }

    /// Switches tab and synchronizes state.
    pub fn switch_tab(&mut self, index: usize) -> bool {
        let success = self.tab_manager.switch_tab(index);
        if success {
            self.sync_active_tab_state();
        }
        success
    }

    /// Handles window resizing event for adaptive layouts.
    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.layout
            .update_dimensions(width, height, &mut self.sidebar);
    }

    fn sync_active_tab_state(&mut self) {
        if let Some(tab) = self.tab_manager.active_tab() {
            self.address_input = tab.url.clone();
            self.title = if tab.is_incognito {
                format!("{} (Incognito) — Vingilot", tab.title)
            } else {
                format!("{} — Vingilot", tab.title)
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_initialization() {
        let win = VingilotWindow::default();
        assert_eq!(win.app_id(), "io.github.antux912012.Vingilot");
        assert_eq!(win.tab_manager().tabs().len(), 1);
        assert!(!win.sidebar().is_collapsed);
    }

    #[test]
    fn test_window_tab_switching_and_incognito() {
        let mut win = VingilotWindow::default();
        win.set_address_input("https://example.com");
        win.submit_address_bar().unwrap();

        // Open incognito tab
        win.open_new_incognito_tab();
        assert_eq!(win.tab_manager().tabs().len(), 2);
        assert!(win.tab_manager().active_tab().unwrap().is_incognito);
        assert!(win.title().contains("Incognito"));

        // Switch back to tab 0
        win.switch_tab(0);
        assert_eq!(win.address_input(), "https://example.com/");
    }

    #[test]
    fn test_window_resize_adaptive() {
        let mut win = VingilotWindow::default();
        win.on_window_resized(500, 800);
        assert!(win.sidebar().is_collapsed);

        win.on_window_resized(1200, 800);
        assert!(!win.sidebar().is_collapsed);
    }

    #[test]
    fn test_window_apply_accent_color() {
        let mut win = VingilotWindow::default();
        assert_eq!(win.accent_color(), None);
        let css = win.apply_accent_color("#3584e4");
        assert_eq!(win.accent_color(), Some("#3584e4"));
        assert!(css.contains(".browser-chrome"));
        assert!(css.contains("alpha(#3584e4, 0.22)"));
        assert!(css.contains(".tab-row.active, .sidebar-tab.active"));
    }

    #[test]
    fn test_incognito_window_accent_guard() {
        let mut win = VingilotWindow::new_incognito("io.github.antux912012.Vingilot", "Vingilot — Incognito");
        assert!(win.is_incognito());
        let css = win.apply_accent_color("#3584e4");
        assert!(css.is_empty());
        assert_eq!(win.accent_color(), None);
    }
}
