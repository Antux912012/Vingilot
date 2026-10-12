//! Vingilot Core library: data models, repositories, sync, and import/export logic.

pub mod error;
pub mod models;

pub use error::{CoreError, CoreResult};
pub use models::*;
