use thiserror::Error;

/// Core error type representing all errors in vingilot-core operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Convenience type alias for Results returning `CoreError`.
pub type CoreResult<T> = Result<T, CoreError>;
