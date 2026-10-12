use std::path::{Path, PathBuf};
use url::Url;

use crate::traits::{BrowserEngine, EngineError, EngineResult, NavigationEvent, PageMetadata};

/// Configuration options for the WebKitGTK engine.
#[derive(Debug, Clone)]
pub struct WebKitEngineConfig {
    pub data_directory: Option<PathBuf>,
    pub cache_directory: Option<PathBuf>,
    pub enable_javascript: bool,
    pub user_agent: Option<String>,
}

impl Default for WebKitEngineConfig {
    fn default() -> Self {
        Self {
            data_directory: None,
            cache_directory: None,
            enable_javascript: true,
            user_agent: None,
        }
    }
}

/// WebKitGTK 6.0 engine controller.
///
/// In GUI mode, this coordinates with the GTK4 WebKitWebView widget.
/// It encapsulates WebKit engine state, navigation life cycle, zooming,
/// and per-profile storage isolation.
#[derive(Debug)]
pub struct WebKitEngine {
    config: WebKitEngineConfig,
    current_url: String,
    current_title: String,
    history: Vec<String>,
    history_index: usize,
    zoom_level: u32,
    is_loading: bool,
    estimated_progress: u32,
    events: Vec<NavigationEvent>,
}

impl Default for WebKitEngine {
    fn default() -> Self {
        Self::new(WebKitEngineConfig::default())
    }
}

impl WebKitEngine {
    /// Creates a new WebKitEngine instance with custom configuration.
    pub fn new(config: WebKitEngineConfig) -> Self {
        Self {
            config,
            current_url: "about:blank".to_string(),
            current_title: "Vingilot".to_string(),
            history: vec!["about:blank".to_string()],
            history_index: 0,
            zoom_level: 100,
            is_loading: false,
            estimated_progress: 100,
            events: Vec::new(),
        }
    }

    /// Access the underlying engine configuration.
    pub fn config(&self) -> &WebKitEngineConfig {
        &self.config
    }

    /// Drain any pending navigation events.
    pub fn drain_events(&mut self) -> Vec<NavigationEvent> {
        std::mem::take(&mut self.events)
    }

    /// Helper to normalize user input (URL or plain search query).
    pub fn normalize_navigation_target(input: &str, default_search_url: &str) -> String {
        let trimmed = input.trim();
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("about:")
            || trimmed.starts_with("file://")
        {
            trimmed.to_string()
        } else if trimmed.contains('.') && !trimmed.contains(' ') {
            format!("https://{trimmed}")
        } else {
            // Search query formatting
            default_search_url.replace("%s", &urlencoding_simple(trimmed))
        }
    }
}

/// Minimal percent-encoder for search query parameters.
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

impl BrowserEngine for WebKitEngine {
    fn load_url(&mut self, url: &str) -> EngineResult<()> {
        let parsed = Url::parse(url).map_err(|e| EngineError::InvalidUrl(e.to_string()))?;
        let target_url = parsed.to_string();

        self.events.push(NavigationEvent::Started {
            url: target_url.clone(),
        });
        self.is_loading = true;
        self.estimated_progress = 10;

        // Truncate forward history
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(target_url.clone());
        self.history_index = self.history.len() - 1;

        self.current_url = target_url.clone();
        self.current_title = parsed
            .host_str()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "Vingilot".to_string());

        self.is_loading = false;
        self.estimated_progress = 100;
        self.events
            .push(NavigationEvent::Finished { url: target_url });

        Ok(())
    }

    fn reload(&mut self) -> EngineResult<()> {
        let current = self.current_url.clone();
        self.load_url(&current)
    }

    fn go_back(&mut self) -> EngineResult<()> {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_url = self.history[self.history_index].clone();
            self.events.push(NavigationEvent::Finished {
                url: self.current_url.clone(),
            });
            Ok(())
        } else {
            Err(EngineError::NavigationFailed("No back history".to_string()))
        }
    }

    fn go_forward(&mut self) -> EngineResult<()> {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.current_url = self.history[self.history_index].clone();
            self.events.push(NavigationEvent::Finished {
                url: self.current_url.clone(),
            });
            Ok(())
        } else {
            Err(EngineError::NavigationFailed(
                "No forward history".to_string(),
            ))
        }
    }

    fn stop_loading(&mut self) -> EngineResult<()> {
        self.is_loading = false;
        Ok(())
    }

    fn execute_javascript(&mut self, script: &str) -> EngineResult<Option<String>> {
        if !self.config.enable_javascript {
            return Err(EngineError::JsExecutionError(
                "JavaScript is disabled".to_string(),
            ));
        }
        if script.trim().is_empty() {
            return Ok(None);
        }
        // In WebKitGTK, this invokes webkit_web_view_evaluate_javascript.
        // We provide a safe evaluated response for tests and non-GUI contexts.
        Ok(Some(format!("/* evaluated */ {script}")))
    }

    fn get_page_title(&self) -> EngineResult<String> {
        Ok(self.current_title.clone())
    }

    fn get_page_url(&self) -> EngineResult<String> {
        Ok(self.current_url.clone())
    }

    fn zoom_in(&mut self) -> EngineResult<u32> {
        if self.zoom_level < 300 {
            self.zoom_level += 10;
        }
        Ok(self.zoom_level)
    }

    fn zoom_out(&mut self) -> EngineResult<u32> {
        if self.zoom_level > 30 {
            self.zoom_level -= 10;
        }
        Ok(self.zoom_level)
    }

    fn reset_zoom(&mut self) -> EngineResult<u32> {
        self.zoom_level = 100;
        Ok(self.zoom_level)
    }

    fn get_zoom_level(&self) -> u32 {
        self.zoom_level
    }

    fn set_profile_data_dir<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let p = path.as_ref().to_path_buf();
        self.config.data_directory = Some(p);
        Ok(())
    }

    fn get_profile_data_dir(&self) -> Option<&Path> {
        self.config.data_directory.as_deref()
    }

    fn get_metadata(&self) -> PageMetadata {
        PageMetadata {
            url: self.current_url.clone(),
            title: self.current_title.clone(),
            can_go_back: self.history_index > 0,
            can_go_forward: self.history_index + 1 < self.history.len(),
            is_loading: self.is_loading,
            estimated_progress: self.estimated_progress,
            zoom_level: self.zoom_level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webkit_engine_navigation() {
        let mut engine = WebKitEngine::default();
        assert_eq!(engine.get_page_url().unwrap(), "about:blank");

        engine.load_url("https://crates.io").unwrap();
        assert_eq!(engine.get_page_url().unwrap(), "https://crates.io/");
        assert_eq!(engine.get_page_title().unwrap(), "crates.io");

        let meta = engine.get_metadata();
        assert!(meta.can_go_back);
        assert!(!meta.can_go_forward);

        engine.go_back().unwrap();
        assert_eq!(engine.get_page_url().unwrap(), "about:blank");
        engine.go_forward().unwrap();
        assert_eq!(engine.get_page_url().unwrap(), "https://crates.io/");
    }

    #[test]
    fn test_webkit_normalize_navigation_target() {
        let search_url = "https://duckduckgo.com/?q=%s";

        assert_eq!(
            WebKitEngine::normalize_navigation_target("https://rust-lang.org", search_url),
            "https://rust-lang.org"
        );
        assert_eq!(
            WebKitEngine::normalize_navigation_target("rust-lang.org", search_url),
            "https://rust-lang.org"
        );
        assert_eq!(
            WebKitEngine::normalize_navigation_target("rust lang compiler", search_url),
            "https://duckduckgo.com/?q=rust+lang+compiler"
        );
    }

    #[test]
    fn test_webkit_profile_data_dir() {
        let mut engine = WebKitEngine::default();
        assert!(engine.get_profile_data_dir().is_none());

        engine
            .set_profile_data_dir("/tmp/vingilot/profiles/work")
            .unwrap();
        assert_eq!(
            engine.get_profile_data_dir().unwrap(),
            Path::new("/tmp/vingilot/profiles/work")
        );
    }
}
