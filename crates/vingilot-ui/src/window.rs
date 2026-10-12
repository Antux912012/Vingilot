use vingilot_engine::{BrowserEngine, WebKitEngine};

/// Controller for the Vingilot main application window and browser shell.
pub struct VingilotWindow {
    app_id: String,
    title: String,
    engine: WebKitEngine,
    address_input: String,
    default_search_url: String,
}

impl Default for VingilotWindow {
    fn default() -> Self {
        Self::new("io.github.vingilot.Vingilot", "Vingilot")
    }
}

impl VingilotWindow {
    /// Creates a new Vingilot window controller.
    pub fn new(app_id: &str, title: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            title: title.to_string(),
            engine: WebKitEngine::default(),
            address_input: String::new(),
            default_search_url: "https://duckduckgo.com/?q=%s".to_string(),
        }
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn engine(&self) -> &WebKitEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut WebKitEngine {
        &mut self.engine
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

    /// Triggers navigation from address bar entry or search query.
    pub fn submit_address_bar(&mut self) -> Result<(), String> {
        if self.address_input.trim().is_empty() {
            return Ok(());
        }

        let target = WebKitEngine::normalize_navigation_target(
            &self.address_input,
            &self.default_search_url,
        );

        self.engine
            .load_url(&target)
            .map_err(|e| format!("Navigation error: {e}"))?;

        if let Ok(title) = self.engine.get_page_title() {
            self.title = format!("{title} — Vingilot");
        }
        self.address_input = self.engine.get_page_url().unwrap_or(target);
        Ok(())
    }

    /// Action: Navigate Back
    pub fn on_back_clicked(&mut self) -> Result<(), String> {
        self.engine.go_back().map_err(|e| e.to_string())?;
        self.sync_state();
        Ok(())
    }

    /// Action: Navigate Forward
    pub fn on_forward_clicked(&mut self) -> Result<(), String> {
        self.engine.go_forward().map_err(|e| e.to_string())?;
        self.sync_state();
        Ok(())
    }

    /// Action: Reload
    pub fn on_reload_clicked(&mut self) -> Result<(), String> {
        self.engine.reload().map_err(|e| e.to_string())?;
        self.sync_state();
        Ok(())
    }

    /// Action: Stop Loading
    pub fn on_stop_clicked(&mut self) -> Result<(), String> {
        self.engine.stop_loading().map_err(|e| e.to_string())?;
        self.sync_state();
        Ok(())
    }

    fn sync_state(&mut self) {
        if let Ok(url) = self.engine.get_page_url() {
            self.address_input = url;
        }
        if let Ok(title) = self.engine.get_page_title() {
            self.title = format!("{title} — Vingilot");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_initialization() {
        let win = VingilotWindow::default();
        assert_eq!(win.app_id(), "io.github.vingilot.Vingilot");
        assert_eq!(win.title(), "Vingilot");
    }

    #[test]
    fn test_window_navigation_and_essentials() {
        let mut win = VingilotWindow::default();

        win.set_address_input("https://example.com");
        win.submit_address_bar().unwrap();
        assert_eq!(win.address_input(), "https://example.com/");
        assert_eq!(win.title(), "example.com — Vingilot");

        let meta = win.engine().get_metadata();
        assert!(meta.can_go_back);
        assert!(!meta.can_go_forward);

        win.on_back_clicked().unwrap();
        assert_eq!(win.address_input(), "about:blank");

        win.on_forward_clicked().unwrap();
        assert_eq!(win.address_input(), "https://example.com/");
    }

    #[test]
    fn test_window_search_query_submission() {
        let mut win = VingilotWindow::default();
        win.set_address_input("rust programming language");
        win.submit_address_bar().unwrap();
        assert_eq!(
            win.address_input(),
            "https://duckduckgo.com/?q=rust+programming+language"
        );
    }
}
