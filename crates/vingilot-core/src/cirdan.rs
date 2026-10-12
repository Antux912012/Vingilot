use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stored credential managed by Círdan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CirdanCredential {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub title: String,
    pub url_or_domain: String,
    pub username: String,
    /// Encrypted password ciphertext (or plaintext if unlocked in-memory).
    pub encrypted_password: Vec<u8>,
    pub nonce: Vec<u8>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Standalone export bundle for Círdan password manager.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CirdanExportBundle {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub credentials: Vec<CirdanCredential>,
}
