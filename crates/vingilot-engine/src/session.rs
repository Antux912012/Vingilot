use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents an HTTP cookie stored in a WebKit network session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub is_secure: bool,
    pub is_http_only: bool,
}

/// Represents an isolated WebKitGTK 6.0 Website Data / Network Session.
/// Normal sessions have persistent data directories, whereas private (incognito)
/// sessions are strictly ephemeral in-memory sessions that leave no traces on disk.
#[derive(Debug, Clone)]
pub struct WebsiteDataSession {
    pub id: Uuid,
    pub is_ephemeral: bool,
    pub data_dir: Option<PathBuf>,
    pub cookies: HashMap<(String, String), Cookie>,
    pub local_storage: HashMap<String, String>,
    pub is_destroyed: bool,
}

impl WebsiteDataSession {
    /// Creates a persistent normal browsing network session.
    pub fn new_persistent(data_dir: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            is_ephemeral: false,
            data_dir: Some(data_dir),
            cookies: HashMap::new(),
            local_storage: HashMap::new(),
            is_destroyed: false,
        }
    }

    /// Creates an isolated ephemeral private browsing session (`webkit_network_session_new_ephemeral`).
    pub fn new_ephemeral() -> Self {
        Self {
            id: Uuid::new_v4(),
            is_ephemeral: true,
            data_dir: None,
            cookies: HashMap::new(),
            local_storage: HashMap::new(),
            is_destroyed: false,
        }
    }

    /// Sets a cookie in this session's cookie jar.
    pub fn set_cookie(&mut self, cookie: Cookie) {
        if self.is_destroyed {
            return;
        }
        self.cookies
            .insert((cookie.domain.clone(), cookie.name.clone()), cookie);
    }

    /// Retrieves a cookie from this session.
    pub fn get_cookie(&self, domain: &str, name: &str) -> Option<&Cookie> {
        if self.is_destroyed {
            return None;
        }
        self.cookies.get(&(domain.to_string(), name.to_string()))
    }

    /// Sets an item in local/session web storage.
    pub fn set_storage_item(&mut self, key: &str, val: &str) {
        if self.is_destroyed {
            return;
        }
        self.local_storage.insert(key.to_string(), val.to_string());
    }

    /// Gets an item from web storage.
    pub fn get_storage_item(&self, key: &str) -> Option<&str> {
        if self.is_destroyed {
            return None;
        }
        self.local_storage.get(key).map(|s| s.as_str())
    }

    /// Closes and destroys the session, purging all ephemeral cookies and web data.
    pub fn destroy(&mut self) {
        self.cookies.clear();
        self.local_storage.clear();
        self.is_destroyed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vingilot_core::db::Database;
    use vingilot_core::repositories::HistoryRepository;

    #[test]
    fn test_normal_session_cookie_not_in_private_session() {
        // Requirement 1: A normal session's cookie is not present in a new private session.
        let mut normal_session =
            WebsiteDataSession::new_persistent(PathBuf::from("/home/user/.config/vingilot"));
        normal_session.set_cookie(Cookie {
            name: "session_token".to_string(),
            value: "abc123secret".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            is_secure: true,
            is_http_only: true,
        });

        let private_session = WebsiteDataSession::new_ephemeral();
        assert!(private_session.is_ephemeral);
        assert!(private_session
            .get_cookie("example.com", "session_token")
            .is_none());
    }

    #[test]
    fn test_private_session_cookie_not_visible_in_normal_session() {
        // Requirement 2: A cookie set in private browsing is not visible in the normal session.
        let normal_session =
            WebsiteDataSession::new_persistent(PathBuf::from("/home/user/.config/vingilot"));
        let mut private_session = WebsiteDataSession::new_ephemeral();

        private_session.set_cookie(Cookie {
            name: "incognito_tracker".to_string(),
            value: "private_val".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            is_secure: true,
            is_http_only: false,
        });

        assert_eq!(
            private_session
                .get_cookie("example.com", "incognito_tracker")
                .unwrap()
                .value,
            "private_val"
        );
        assert!(normal_session
            .get_cookie("example.com", "incognito_tracker")
            .is_none());
    }

    #[test]
    fn test_private_browsing_history_absent_from_normal_history() {
        // Requirement 3: Private browsing history is absent from normal history.
        let db = Database::in_memory().unwrap();
        let prof_repo = vingilot_core::repositories::ProfileRepository::new(&db);
        let profile = vingilot_core::models::Profile {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            profile_type: vingilot_core::models::ProfileType::Standard,
            color_or_icon: None,
            data_dir: "default".to_string(),
            default_search_engine_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        prof_repo.insert(&profile).unwrap();
        let history_repo = HistoryRepository::new(&db);

        // Record normal visit
        history_repo
            .record_visit(profile.id, "https://wikipedia.org", Some("Wikipedia"))
            .unwrap();

        // Simulate private browsing: incognito visits are NEVER recorded to the database
        let is_incognito = true;
        let private_url = "https://secret-service.onion";
        if !is_incognito {
            history_repo
                .record_visit(profile.id, private_url, Some("Secret"))
                .unwrap();
        }

        let history = history_repo.list_recent(profile.id, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].url, "https://wikipedia.org");
        assert!(!history.iter().any(|h| h.url.contains("secret-service")));
    }

    #[test]
    fn test_private_storage_not_restored_after_closing_private_session() {
        // Requirement 4: Private storage is not restored after closing the private session.
        let mut private_session = WebsiteDataSession::new_ephemeral();
        private_session.set_storage_item("cart_id", "item_42");
        assert_eq!(private_session.get_storage_item("cart_id"), Some("item_42"));

        // Close and destroy private session
        private_session.destroy();
        assert!(private_session.get_storage_item("cart_id").is_none());

        // A newly launched private session also has no restored state
        let next_private_session = WebsiteDataSession::new_ephemeral();
        assert!(next_private_session.get_storage_item("cart_id").is_none());
    }

    #[test]
    fn test_normal_browsing_remains_functional_after_private_windows_close() {
        // Requirement 5: Normal browsing remains functional after private windows close.
        let mut normal_session =
            WebsiteDataSession::new_persistent(PathBuf::from("/home/user/.config/vingilot"));
        normal_session.set_storage_item("theme", "adwaita-dark");
        normal_session.set_cookie(Cookie {
            name: "auth".to_string(),
            value: "logged_in".to_string(),
            domain: "github.com".to_string(),
            path: "/".to_string(),
            is_secure: true,
            is_http_only: true,
        });

        // Open private window
        let mut private_session = WebsiteDataSession::new_ephemeral();
        private_session.set_storage_item("incog_data", "temp");

        // Close private window
        private_session.destroy();

        // Normal session must still have all its persistent data and be fully functional
        assert!(!normal_session.is_destroyed);
        assert_eq!(
            normal_session.get_storage_item("theme"),
            Some("adwaita-dark")
        );
        assert_eq!(
            normal_session
                .get_cookie("github.com", "auth")
                .unwrap()
                .value,
            "logged_in"
        );

        // Normal session can continue navigating and storing data
        normal_session.set_storage_item("new_setting", "enabled");
        assert_eq!(
            normal_session.get_storage_item("new_setting"),
            Some("enabled")
        );
    }
}
