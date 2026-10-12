//! Vingilot Engine abstraction and WebKitGTK wrapper.

pub mod download;
pub mod mock;
pub mod permissions;
pub mod traits;
pub mod webkit;

pub use download::{DownloadItem, DownloadManager, DownloadStatus};
pub use mock::MockEngine;
pub use permissions::{PermissionDecision, PermissionManager, PermissionPrompt, PermissionType};
pub use traits::{BrowserEngine, EngineError, EngineResult, NavigationEvent, PageMetadata};
pub use webkit::{WebKitEngine, WebKitEngineConfig};
