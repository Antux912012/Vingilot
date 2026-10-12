//! Vingilot Engine abstraction and WebKitGTK wrapper.

pub mod mock;
pub mod traits;
pub mod webkit;

pub use mock::MockEngine;
pub use traits::{BrowserEngine, EngineError, EngineResult, NavigationEvent, PageMetadata};
pub use webkit::{WebKitEngine, WebKitEngineConfig};
