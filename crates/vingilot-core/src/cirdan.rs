use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stored credential managed by Círdan.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Custom Debug implementation redacting sensitive password ciphertext to prevent accidental logging.
impl fmt::Debug for CirdanCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CirdanCredential")
            .field("id", &self.id)
            .field("profile_id", &self.profile_id)
            .field("title", &self.title)
            .field("url_or_domain", &self.url_or_domain)
            .field("username", &self.username)
            .field("encrypted_password", &"[REDACTED]")
            .field("nonce_len", &self.nonce.len())
            .field("notes", &self.notes)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl CirdanCredential {
    /// Explicitly zeroize secret buffers in memory.
    pub fn zeroize(&mut self) {
        for byte in self.encrypted_password.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        for byte in self.nonce.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

/// Volatile zeroization on drop to prevent lingering secrets in freed memory.
impl Drop for CirdanCredential {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Standalone export bundle for Círdan password manager.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CirdanExportBundle {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub credentials: Vec<CirdanCredential>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cirdan_credential_debug_redacted() {
        let cred = CirdanCredential {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            title: "Test Site".to_string(),
            url_or_domain: "https://example.com".to_string(),
            username: "admin".to_string(),
            encrypted_password: b"super_secret_password".to_vec(),
            nonce: vec![1, 2, 3, 4],
            notes: Some("Secret note".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let debug_str = format!("{cred:?}");
        assert!(!debug_str.contains("super_secret_password"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(debug_str.contains("nonce_len: 4"));
    }

    #[test]
    fn test_cirdan_credential_drop_zeroization() {
        let mut cred = CirdanCredential {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            title: "Test Site".to_string(),
            url_or_domain: "https://example.com".to_string(),
            username: "admin".to_string(),
            encrypted_password: vec![0xAA; 16],
            nonce: vec![0xBB; 12],
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let pw_ptr = cred.encrypted_password.as_mut_ptr();
        let nonce_ptr = cred.nonce.as_mut_ptr();

        unsafe {
            assert_eq!(*pw_ptr, 0xAA);
            assert_eq!(*nonce_ptr, 0xBB);
            cred.zeroize();
            assert_eq!(*pw_ptr, 0);
            assert_eq!(*nonce_ptr, 0);
        }
    }
}

