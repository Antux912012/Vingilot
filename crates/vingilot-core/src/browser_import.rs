use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{CoreError, CoreResult};
use crate::models::{BookmarkNode, HistoryEntry};

/// Represents an external browser detected on the local system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserType {
    Firefox,
    Chrome,
    Chromium,
    Brave,
    Edge,
    Vivaldi,
}

/// Discovered profile directory for an external browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedBrowserProfile {
    pub browser_type: BrowserType,
    pub profile_name: String,
    pub path: PathBuf,
}

/// Service detecting local browser installations on Linux.
pub struct BrowserDetector;

impl BrowserDetector {
    /// Discovers browser profile paths in standard Linux XDG directories (~/.mozilla, ~/.config).
    pub fn detect_installed_profiles() -> Vec<DetectedBrowserProfile> {
        let mut profiles = Vec::new();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/antonio".to_string());
        let home_path = Path::new(&home);

        // Firefox
        let ff_dir = home_path.join(".mozilla/firefox");
        if ff_dir.is_dir() {
            profiles.push(DetectedBrowserProfile {
                browser_type: BrowserType::Firefox,
                profile_name: "Default Firefox Profile".to_string(),
                path: ff_dir,
            });
        }

        // Chromium-based browsers
        let chromium_candidates = [
            (
                BrowserType::Chrome,
                ".config/google-chrome/Default",
                "Google Chrome",
            ),
            (
                BrowserType::Chromium,
                ".config/chromium/Default",
                "Chromium",
            ),
            (
                BrowserType::Brave,
                ".config/BraveSoftware/Brave-Browser/Default",
                "Brave",
            ),
            (
                BrowserType::Edge,
                ".config/microsoft-edge/Default",
                "Microsoft Edge",
            ),
            (BrowserType::Vivaldi, ".config/vivaldi/Default", "Vivaldi"),
        ];

        for (btype, rel_path, name) in chromium_candidates {
            let path = home_path.join(rel_path);
            if path.is_dir() {
                profiles.push(DetectedBrowserProfile {
                    browser_type: btype,
                    profile_name: name.to_string(),
                    path,
                });
            }
        }

        profiles
    }
}

/// Importer for Mozilla Firefox bookmarks and history (from places.sqlite or JSON/HTML backups).
pub struct FirefoxImporter;

impl FirefoxImporter {
    /// Imports bookmarks from a Firefox JSON backup file.
    pub fn import_bookmarks_json(json_content: &str) -> CoreResult<Vec<BookmarkNode>> {
        let value: serde_json::Value =
            serde_json::from_str(json_content).map_err(|e| CoreError::Import(e.to_string()))?;

        let mut bookmarks = Vec::new();
        Self::parse_firefox_json_node(&value, None, &mut bookmarks);
        Ok(bookmarks)
    }

    fn parse_firefox_json_node(
        node: &serde_json::Value,
        parent_id: Option<Uuid>,
        out: &mut Vec<BookmarkNode>,
    ) {
        let now = chrono::Utc::now();
        let title = node
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let uri = node
            .get("uri")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let is_folder =
            node.get("typeCode").and_then(|v| v.as_i64()).unwrap_or(0) == 2 || uri.is_none();
        let id = Uuid::new_v4();

        let mut children = Vec::new();
        if let Some(children_arr) = node.get("children").and_then(|v| v.as_array()) {
            for child_val in children_arr {
                Self::parse_firefox_json_node(child_val, Some(id), &mut children);
            }
        }

        out.push(BookmarkNode {
            id,
            parent_id,
            title,
            url: uri,
            is_folder,
            children,
            position: 0,
            created_at: now,
            updated_at: now,
        });
    }

    /// Safely imports history entries from a Firefox `places.sqlite` copy.
    pub fn import_history_sqlite(
        places_sqlite_path: &Path,
        target_profile_id: Uuid,
    ) -> CoreResult<Vec<HistoryEntry>> {
        if !places_sqlite_path.exists() {
            return Err(CoreError::Import(format!(
                "places.sqlite does not exist at {}",
                places_sqlite_path.display()
            )));
        }

        // Copy database to temporary file to avoid locking running Firefox
        let temp_dir = tempfile::tempdir().map_err(CoreError::Io)?;
        let temp_copy = temp_dir.path().join("temp_places.sqlite");
        std::fs::copy(places_sqlite_path, &temp_copy)?;

        let conn = rusqlite::Connection::open(&temp_copy)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT url, title, visit_count, last_visit_date
            FROM moz_places
            WHERE url IS NOT NULL
            ORDER BY last_visit_date DESC
            LIMIT 5000
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let url: String = row.get(0)?;
            let title: Option<String> = row.get(1)?;
            let visit_count: i64 = row.get(2).unwrap_or(1);
            let last_visit_raw: Option<i64> = row.get(3)?;

            // Firefox places stores timestamps in microseconds since Unix epoch
            let last_visited_at = match last_visit_raw {
                Some(micros) => chrono::DateTime::from_timestamp(micros / 1_000_000, 0)
                    .unwrap_or_else(chrono::Utc::now),
                None => chrono::Utc::now(),
            };

            Ok(HistoryEntry {
                id: Uuid::new_v4(),
                profile_id: target_profile_id,
                url,
                title,
                visit_count,
                last_visited_at,
                created_at: last_visited_at,
                preview_path: None,
                preview_updated: None,
                hidden: false,
            })
        })?;

        let mut entries = Vec::new();
        for item in rows {
            entries.push(item?);
        }
        Ok(entries)
    }
}

/// Importer for Google Chrome, Chromium, Brave, Edge, and Vivaldi.
pub struct ChromiumImporter;

impl ChromiumImporter {
    /// Imports bookmarks from Chromium `Bookmarks` JSON file.
    pub fn import_bookmarks_json(json_content: &str) -> CoreResult<Vec<BookmarkNode>> {
        let value: serde_json::Value =
            serde_json::from_str(json_content).map_err(|e| CoreError::Import(e.to_string()))?;

        let mut bookmarks = Vec::new();
        if let Some(roots) = value.get("roots").and_then(|v| v.as_object()) {
            for (_key, root_folder) in roots {
                Self::parse_chromium_node(root_folder, None, &mut bookmarks);
            }
        }
        Ok(bookmarks)
    }

    fn parse_chromium_node(
        node: &serde_json::Value,
        parent_id: Option<Uuid>,
        out: &mut Vec<BookmarkNode>,
    ) {
        let now = chrono::Utc::now();
        let name = node
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let url = node
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let node_type = node
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("folder");
        let is_folder = node_type == "folder" || url.is_none();
        let id = Uuid::new_v4();

        let mut children = Vec::new();
        if let Some(children_arr) = node.get("children").and_then(|v| v.as_array()) {
            for child_val in children_arr {
                Self::parse_chromium_node(child_val, Some(id), &mut children);
            }
        }

        out.push(BookmarkNode {
            id,
            parent_id,
            title: name,
            url,
            is_folder,
            children,
            position: 0,
            created_at: now,
            updated_at: now,
        });
    }

    /// Safely imports history entries from a Chromium `History` SQLite file copy.
    pub fn import_history_sqlite(
        history_sqlite_path: &Path,
        target_profile_id: Uuid,
    ) -> CoreResult<Vec<HistoryEntry>> {
        if !history_sqlite_path.exists() {
            return Err(CoreError::Import(format!(
                "History database does not exist at {}",
                history_sqlite_path.display()
            )));
        }

        let temp_dir = tempfile::tempdir().map_err(CoreError::Io)?;
        let temp_copy = temp_dir.path().join("temp_chromium_history.sqlite");
        std::fs::copy(history_sqlite_path, &temp_copy)?;

        let conn = rusqlite::Connection::open(&temp_copy)?;
        let mut stmt = conn.prepare(
            r#"
            SELECT url, title, visit_count, last_visit_time
            FROM urls
            WHERE url IS NOT NULL
            ORDER BY last_visit_time DESC
            LIMIT 5000
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let url: String = row.get(0)?;
            let title: Option<String> = row.get(1)?;
            let visit_count: i64 = row.get(2).unwrap_or(1);
            let chrome_timestamp: Option<i64> = row.get(3)?;

            // Chrome stores microseconds since January 1, 1601 UTC (difference: 11644473600 seconds)
            let last_visited_at = match chrome_timestamp {
                Some(micros) if micros > 11_644_473_600_000_000 => {
                    let unix_seconds = (micros - 11_644_473_600_000_000) / 1_000_000;
                    chrono::DateTime::from_timestamp(unix_seconds, 0)
                        .unwrap_or_else(chrono::Utc::now)
                }
                _ => chrono::Utc::now(),
            };

            Ok(HistoryEntry {
                id: Uuid::new_v4(),
                profile_id: target_profile_id,
                url,
                title,
                visit_count,
                last_visited_at,
                created_at: last_visited_at,
                preview_path: None,
                preview_updated: None,
                hidden: false,
            })
        })?;

        let mut entries = Vec::new();
        for item in rows {
            entries.push(item?);
        }
        Ok(entries)
    }
}

/// Generic Netscape Bookmark HTML and flat JSON parser.
pub struct GenericBookmarkImporter;

impl GenericBookmarkImporter {
    /// Parses standard Netscape Bookmark HTML format (`<DT><A HREF="...">Title</A>`).
    pub fn import_netscape_html(html_content: &str) -> Vec<BookmarkNode> {
        let mut bookmarks = Vec::new();
        let now = chrono::Utc::now();

        for line in html_content.lines() {
            let lower = line.to_lowercase();
            if let Some(href_pos) = lower.find("href=\"") {
                let after_href = &line[href_pos + 6..];
                if let Some(quote_end) = after_href.find('"') {
                    let url = &after_href[..quote_end];
                    if let Some(tag_end) = after_href.find('>') {
                        let after_tag = &after_href[tag_end + 1..];
                        let title = if let Some(close_tag) = after_tag.to_lowercase().find("</a") {
                            &after_tag[..close_tag]
                        } else {
                            url
                        };

                        bookmarks.push(BookmarkNode {
                            id: Uuid::new_v4(),
                            parent_id: None,
                            title: title.trim().to_string(),
                            url: Some(url.trim().to_string()),
                            is_folder: false,
                            children: Vec::new(),
                            position: bookmarks.len() as i32,
                            created_at: now,
                            updated_at: now,
                        });
                    }
                }
            }
        }

        bookmarks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_netscape_html() {
        let sample = r#"
        <!DOCTYPE NETSCAPE-Bookmark-file-1>
        <DL><p>
            <DT><A HREF="https://www.gnu.org/">GNU Project</A>
            <DT><A HREF="https://www.kernel.org/">The Linux Kernel</A>
        </DL><p>
        "#;

        let bookmarks = GenericBookmarkImporter::import_netscape_html(sample);
        assert_eq!(bookmarks.len(), 2);
        assert_eq!(bookmarks[0].title, "GNU Project");
        assert_eq!(bookmarks[0].url.as_deref(), Some("https://www.gnu.org/"));
        assert_eq!(bookmarks[1].title, "The Linux Kernel");
    }

    #[test]
    fn test_import_chromium_json() {
        let sample_json = r#"{
            "roots": {
                "bookmark_bar": {
                    "name": "Bookmarks bar",
                    "type": "folder",
                    "children": [
                        {
                            "name": "Rust Language",
                            "type": "url",
                            "url": "https://rust-lang.org"
                        }
                    ]
                }
            }
        }"#;

        let nodes = ChromiumImporter::import_bookmarks_json(sample_json).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].title, "Bookmarks bar");
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].title, "Rust Language");
    }
}
