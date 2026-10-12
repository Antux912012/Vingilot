//! Vingilot Engine abstraction and WebKitGTK wrapper.

pub mod mock;
pub mod traits;

pub use mock::MockEngine;
pub use traits::{BrowserEngine, EngineError, EngineResult, NavigationEvent, PageMetadata};
