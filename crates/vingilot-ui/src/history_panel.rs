use vingilot_core::models::HistoryEntry;

/// UI panel for browsing and searching browsing history.
#[derive(Debug, Clone, Default)]
pub struct HistoryPanel {
    is_open: bool,
    search_filter: String,
}

impl HistoryPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn set_search_filter(&mut self, filter: &str) {
        self.search_filter = filter.trim().to_lowercase();
    }

    pub fn search_filter(&self) -> &str {
        &self.search_filter
    }

    /// Filters history entries according to current search query.
    pub fn filter_entries<'a>(&self, entries: &'a [HistoryEntry]) -> Vec<&'a HistoryEntry> {
        if self.search_filter.is_empty() {
            return entries.iter().collect();
        }

        entries
            .iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&self.search_filter)
                    || e.title
                        .as_deref()
                        .map(|t| t.to_lowercase().contains(&self.search_filter))
                        .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_history_panel_search_filtering() {
        let now = Utc::now();
        let e1 = HistoryEntry {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            url: "https://rust-lang.org".to_string(),
            title: Some("Rust Programming".to_string()),
            visit_count: 5,
            last_visited_at: now,
            created_at: now,
        };
        let e2 = HistoryEntry {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            url: "https://wikipedia.org".to_string(),
            title: Some("Wikipedia The Free Encyclopedia".to_string()),
            visit_count: 2,
            last_visited_at: now,
            created_at: now,
        };

        let mut panel = HistoryPanel::new();
        let all = vec![e1, e2];
        assert_eq!(panel.filter_entries(&all).len(), 2);

        panel.set_search_filter("rust");
        let filtered = panel.filter_entries(&all);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].url, "https://rust-lang.org");
    }
}
