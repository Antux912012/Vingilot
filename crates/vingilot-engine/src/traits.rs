use std::path::Path;
use thiserror::Error;

/// Engine-specific errors.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("JavaScript execution error: {0}")]
    JsExecutionError(String),

    #[error("Engine initialization error: {0}")]
    InitError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Feature not supported: {0}")]
    NotSupported(String),
}

/// Convenience result alias for engine operations.
pub type EngineResult<T> = Result<T, EngineError>;

/// Navigation events dispatched by the browser engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationEvent {
    Started { url: String },
    Committed { url: String },
    Finished { url: String },
    Failed { url: String, error_message: String },
    TitleChanged { title: String },
    EstimatedLoadProgress { progress: u32 }, // 0 to 100
}

/// Snapshot of the current page metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageMetadata {
    pub url: String,
    pub title: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub estimated_progress: u32,
    pub zoom_level: u32, // percentage, e.g. 100
}

/// Trait defining browser engine capabilities and controls.
pub trait BrowserEngine: Send + Sync {
    /// Load a URL in the browser engine.
    fn load_url(&mut self, url: &str) -> EngineResult<()>;

    /// Reload the current page.
    fn reload(&mut self) -> EngineResult<()>;

    /// Navigate back in the browsing history.
    fn go_back(&mut self) -> EngineResult<()>;

    /// Navigate forward in the browsing history.
    fn go_forward(&mut self) -> EngineResult<()>;

    /// Stop loading the current page.
    fn stop_loading(&mut self) -> EngineResult<()>;

    /// Execute JavaScript within the context of the current page.
    fn execute_javascript(&mut self, script: &str) -> EngineResult<Option<String>>;

    /// Retrieve the current page title.
    fn get_page_title(&self) -> EngineResult<String>;

    /// Retrieve the current page URL.
    fn get_page_url(&self) -> EngineResult<String>;

    /// Zoom in (increase zoom level).
    fn zoom_in(&mut self) -> EngineResult<u32>;

    /// Zoom out (decrease zoom level).
    fn zoom_out(&mut self) -> EngineResult<u32>;

    /// Reset zoom to default 100%.
    fn reset_zoom(&mut self) -> EngineResult<u32>;

    /// Get current zoom level (in percent).
    fn get_zoom_level(&self) -> u32;

    /// Set profile data directory for cookies, cache, and storage.
    fn set_profile_data_dir<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()>;

    /// Retrieve the configured profile data directory.
    fn get_profile_data_dir(&self) -> Option<&Path>;

    /// Get current page metadata.
    fn get_metadata(&self) -> PageMetadata;
}
