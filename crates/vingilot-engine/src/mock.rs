use std::path::{Path, PathBuf};
use url::Url;

use crate::traits::{BrowserEngine, EngineError, EngineResult, NavigationEvent, PageMetadata};

/// Mock/Headless browser engine implementation used for headless tests, CLI, and fallback.
#[derive(Debug, Clone)]
pub struct MockEngine {
    current_url: String,
    current_title: String,
    history: Vec<String>,
    history_index: usize,
    zoom_level: u32,
    profile_data_dir: Option<PathBuf>,
    is_loading: bool,
    events: Vec<NavigationEvent>,
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEngine {
    pub fn new() -> Self {
        Self {
            current_url: "about:blank".to_string(),
            current_title: "New Tab".to_string(),
            history: vec!["about:blank".to_string()],
            history_index: 0,
            zoom_level: 100,
            profile_data_dir: None,
            is_loading: false,
            events: Vec::new(),
        }
    }

    pub fn recorded_events(&self) -> &[NavigationEvent] {
        &self.events
    }
}

impl BrowserEngine for MockEngine {
    fn load_url(&mut self, url: &str) -> EngineResult<()> {
        let parsed = if url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("about:")
        {
            Url::parse(url).map_err(|e| EngineError::InvalidUrl(e.to_string()))?
        } else {
            Url::parse(&format!("https://{url}"))
                .map_err(|e| EngineError::InvalidUrl(e.to_string()))?
        };

        let target_url = parsed.to_string();
        self.events.push(NavigationEvent::Started {
            url: target_url.clone(),
        });
        self.is_loading = true;

        // Truncate forward history if navigating to a new URL
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(target_url.clone());
        self.history_index = self.history.len() - 1;

        self.current_url = target_url.clone();
        self.current_title = parsed
            .host_str()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "Loaded Page".to_string());

        self.is_loading = false;
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
        if script.trim().is_empty() {
            return Ok(None);
        }
        // Mock evaluator returns dummy string for test compatibility
        Ok(Some(format!("mock_result_for: {script}")))
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
        self.profile_data_dir = Some(path.as_ref().to_path_buf());
        Ok(())
    }

    fn get_profile_data_dir(&self) -> Option<&Path> {
        self.profile_data_dir.as_deref()
    }

    fn get_metadata(&self) -> PageMetadata {
        PageMetadata {
            url: self.current_url.clone(),
            title: self.current_title.clone(),
            can_go_back: self.history_index > 0,
            can_go_forward: self.history_index + 1 < self.history.len(),
            is_loading: self.is_loading,
            estimated_progress: if self.is_loading { 50 } else { 100 },
            zoom_level: self.zoom_level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_engine_navigation() {
        let mut engine = MockEngine::new();
        assert_eq!(engine.get_page_url().unwrap(), "about:blank");

        engine.load_url("https://example.com").unwrap();
        assert_eq!(engine.get_page_url().unwrap(), "https://example.com/");
        assert_eq!(engine.get_page_title().unwrap(), "example.com");

        let meta = engine.get_metadata();
        assert!(meta.can_go_back);
        assert!(!meta.can_go_forward);

        engine.go_back().unwrap();
        assert_eq!(engine.get_page_url().unwrap(), "about:blank");

        engine.go_forward().unwrap();
        assert_eq!(engine.get_page_url().unwrap(), "https://example.com/");
    }

    #[test]
    fn test_mock_engine_zoom() {
        let mut engine = MockEngine::new();
        assert_eq!(engine.get_zoom_level(), 100);

        engine.zoom_in().unwrap();
        assert_eq!(engine.get_zoom_level(), 110);

        engine.zoom_out().unwrap();
        assert_eq!(engine.get_zoom_level(), 100);

        engine.reset_zoom().unwrap();
        assert_eq!(engine.get_zoom_level(), 100);
    }
}
