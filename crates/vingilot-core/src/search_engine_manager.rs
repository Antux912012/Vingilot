use chrono::Utc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::SearchEngine;
use crate::repositories::SearchEngineRepository;

/// Manages built-in and user-configured search engines and prefix shortcut routing.
pub struct SearchEngineManager<'a> {
    repo: SearchEngineRepository<'a>,
}

impl<'a> SearchEngineManager<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: SearchEngineRepository::new(db),
        }
    }

    /// Initializes default search engines if none are registered.
    pub fn initialize_defaults(&self) -> CoreResult<Vec<SearchEngine>> {
        let existing = self.repo.list_all()?;
        if !existing.is_empty() {
            // Ensure /e (Ecosia) is seeded even on existing databases if missing
            if !existing
                .iter()
                .any(|e| e.shortcut.as_deref() == Some("/e") || e.name.eq_ignore_ascii_case("Ecosia"))
            {
                let ecosia = SearchEngine {
                    id: Uuid::new_v4(),
                    name: "Ecosia".to_string(),
                    search_url: "https://www.ecosia.org/search?q=%s".to_string(),
                    shortcut: Some("/e".to_string()),
                    is_default: false,
                    created_at: Utc::now(),
                };
                self.repo.insert(&ecosia)?;
            }
            return self.repo.list_all();
        }

        let defaults = vec![
            (
                "DuckDuckGo",
                "https://duckduckgo.com/?q=%s",
                Some("/d"),
                true,
            ),
            (
                "Google",
                "https://www.google.com/search?q=%s",
                Some("/g"),
                false,
            ),
            (
                "Bing",
                "https://www.bing.com/search?q=%s",
                Some("/b"),
                false,
            ),
            (
                "Brave Search",
                "https://search.brave.com/search?q=%s",
                Some("/br"),
                false,
            ),
            (
                "Ecosia",
                "https://www.ecosia.org/search?q=%s",
                Some("/e"),
                false,
            ),
            (
                "YouTube",
                "https://www.youtube.com/results?search_query=%s",
                Some("/y"),
                false,
            ),
            (
                "Wikipedia",
                "https://en.wikipedia.org/wiki/Special:Search?search=%s",
                Some("/w"),
                false,
            ),
        ];

        let now = Utc::now();
        let mut created = Vec::new();

        for (name, url, shortcut, is_default) in defaults {
            let engine = SearchEngine {
                id: Uuid::new_v4(),
                name: name.to_string(),
                search_url: url.to_string(),
                shortcut: shortcut.map(|s| s.to_string()),
                is_default,
                created_at: now,
            };
            self.repo.insert(&engine)?;
            created.push(engine);
        }

        Ok(created)
    }

    pub fn list_all(&self) -> CoreResult<Vec<SearchEngine>> {
        self.repo.list_all()
    }

    pub fn get_default(&self) -> CoreResult<Option<SearchEngine>> {
        self.repo.get_default()
    }

    pub fn set_default(&self, id: Uuid) -> CoreResult<()> {
        self.repo.set_default(id)
    }

    /// Resolves an address bar input string, extracting any `/prefix` search shortcut.
    ///
    /// Supported prefixes:
    /// - `/d query` -> DuckDuckGo
    /// - `/g query` -> Google
    /// - `/b query` -> Bing
    /// - `/br query` -> Brave Search
    /// - `/y query` -> YouTube
    /// - `/w query` -> Wikipedia
    pub fn resolve_query(&self, input: &str) -> CoreResult<String> {
        let trimmed = input.trim();
        let engines = self.repo.list_all()?;

        for engine in &engines {
            if let Some(shortcut) = &engine.shortcut {
                let prefix_space = format!("{shortcut} ");
                if trimmed.starts_with(&prefix_space) {
                    let query = trimmed[prefix_space.len()..].trim();
                    let encoded = urlencoding_simple(query);
                    return Ok(engine.search_url.replace("%s", &encoded));
                }
            }
        }

        // Fall back to default search engine
        let default_url = self
            .get_default()?
            .map(|e| e.search_url)
            .unwrap_or_else(|| "https://duckduckgo.com/?q=%s".to_string());

        let encoded = urlencoding_simple(trimmed);
        Ok(default_url.replace("%s", &encoded))
    }
}

fn urlencoding_simple(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_manager_defaults_and_prefixes() {
        let db = Database::in_memory().unwrap();
        let mgr = SearchEngineManager::new(&db);
        let list = mgr.initialize_defaults().unwrap();
        assert_eq!(list.len(), 7);

        // Test default query
        let q1 = mgr.resolve_query("rust language").unwrap();
        assert_eq!(q1, "https://duckduckgo.com/?q=rust+language");

        // Test Google prefix
        let q2 = mgr.resolve_query("/g linus torvalds").unwrap();
        assert_eq!(q2, "https://www.google.com/search?q=linus+torvalds");

        // Test Wikipedia prefix
        let q3 = mgr.resolve_query("/w Earendil").unwrap();
        assert_eq!(
            q3,
            "https://en.wikipedia.org/wiki/Special:Search?search=Earendil"
        );

        // Test YouTube prefix
        let q4 = mgr.resolve_query("/y rust conference").unwrap();
        assert_eq!(
            q4,
            "https://www.youtube.com/results?search_query=rust+conference"
        );

        // Test Ecosia prefix
        let q5 = mgr.resolve_query("/e plant trees").unwrap();
        assert_eq!(q5, "https://www.ecosia.org/search?q=plant+trees");
    }
}
