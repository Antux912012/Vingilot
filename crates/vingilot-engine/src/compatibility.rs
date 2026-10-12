use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compatibility test outcome for a website.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityStatus {
    Pass,
    PassWithQuirks,
    Fail,
    Untested,
}

/// Site-specific quirks configuration applied to solve domain-level issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SiteQuirks {
    pub custom_user_agent: Option<String>,
    pub enable_webgl: bool,
    pub enable_media_stream: bool,
    pub inject_css: Option<String>,
}

/// A registered website in the compatibility test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityTestTarget {
    pub domain: String,
    pub test_url: String,
    pub category: String, // e.g., "Search", "Social", "Streaming", "Documentation"
    pub status: CompatibilityStatus,
    pub quirks: Option<SiteQuirks>,
    pub notes: Option<String>,
}

/// Compatibility test runner and registry.
#[derive(Debug, Clone, Default)]
pub struct WebCompatibilitySuite {
    targets: Vec<CompatibilityTestTarget>,
    site_quirks_map: HashMap<String, SiteQuirks>,
}

impl WebCompatibilitySuite {
    pub fn new() -> Self {
        let mut suite = Self::default();
        suite.register_default_sites();
        suite
    }

    /// Registers the default test battery of representative modern web properties.
    pub fn register_default_sites(&mut self) {
        let default_targets = vec![
            CompatibilityTestTarget {
                domain: "duckduckgo.com".to_string(),
                test_url: "https://duckduckgo.com/".to_string(),
                category: "Search".to_string(),
                status: CompatibilityStatus::Pass,
                quirks: None,
                notes: Some("Core search and instant answers functional".to_string()),
            },
            CompatibilityTestTarget {
                domain: "wikipedia.org".to_string(),
                test_url: "https://en.wikipedia.org/wiki/Main_Page".to_string(),
                category: "Reference".to_string(),
                status: CompatibilityStatus::Pass,
                quirks: None,
                notes: Some("Renders cleanly, reader mode compatible".to_string()),
            },
            CompatibilityTestTarget {
                domain: "github.com".to_string(),
                test_url: "https://github.com/".to_string(),
                category: "Development".to_string(),
                status: CompatibilityStatus::Pass,
                quirks: None,
                notes: Some("Modern Web Components and dark mode support".to_string()),
            },
            CompatibilityTestTarget {
                domain: "rust-lang.org".to_string(),
                test_url: "https://www.rust-lang.org/".to_string(),
                category: "Documentation".to_string(),
                status: CompatibilityStatus::Pass,
                quirks: None,
                notes: Some("Static documentation and book search".to_string()),
            },
            CompatibilityTestTarget {
                domain: "youtube.com".to_string(),
                test_url: "https://www.youtube.com/".to_string(),
                category: "Streaming".to_string(),
                status: CompatibilityStatus::PassWithQuirks,
                quirks: Some(SiteQuirks {
                    custom_user_agent: None,
                    enable_webgl: true,
                    enable_media_stream: true,
                    inject_css: None,
                }),
                notes: Some("Media stream playback with Wayland HW acceleration".to_string()),
            },
        ];

        for target in default_targets {
            if let Some(quirks) = &target.quirks {
                self.site_quirks_map
                    .insert(target.domain.clone(), quirks.clone());
            }
            self.targets.push(target);
        }
    }

    pub fn targets(&self) -> &[CompatibilityTestTarget] {
        &self.targets
    }

    pub fn get_quirks_for_domain(&self, domain: &str) -> Option<&SiteQuirks> {
        self.site_quirks_map.get(domain)
    }

    pub fn record_result(
        &mut self,
        domain: &str,
        status: CompatibilityStatus,
        notes: Option<&str>,
    ) -> bool {
        if let Some(target) = self.targets.iter_mut().find(|t| t.domain == domain) {
            target.status = status;
            if let Some(n) = notes {
                target.notes = Some(n.to_string());
            }
            true
        } else {
            false
        }
    }

    pub fn summary(&self) -> (usize, usize, usize, usize) {
        let (mut pass, mut quirks, mut fail, mut untested) = (0, 0, 0, 0);
        for t in &self.targets {
            match t.status {
                CompatibilityStatus::Pass => pass += 1,
                CompatibilityStatus::PassWithQuirks => quirks += 1,
                CompatibilityStatus::Fail => fail += 1,
                CompatibilityStatus::Untested => untested += 1,
            }
        }
        (pass, quirks, fail, untested)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_compatibility_suite() {
        let mut suite = WebCompatibilitySuite::new();
        assert_eq!(suite.targets().len(), 5);

        let (pass, quirks, fail, untested) = suite.summary();
        assert!(pass >= 4);
        assert_eq!(quirks, 1);
        assert_eq!(fail, 0);
        assert_eq!(untested, 0);

        let yt_quirks = suite.get_quirks_for_domain("youtube.com").unwrap();
        assert!(yt_quirks.enable_webgl);

        suite.record_result("wikipedia.org", CompatibilityStatus::Pass, Some("Verified"));
        assert_eq!(
            suite
                .targets()
                .iter()
                .find(|t| t.domain == "wikipedia.org")
                .unwrap()
                .notes
                .as_deref(),
            Some("Verified")
        );
    }
}
