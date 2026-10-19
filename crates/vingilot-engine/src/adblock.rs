use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Target resource types supported by WebKit content blocker rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceType {
    Document,
    Image,
    StyleSheet,
    Script,
    Font,
    Raw,
    Media,
    Popup,
}

/// Load types supported by WebKit content blocker triggers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoadType {
    FirstParty,
    ThirdParty,
}

/// Action types supported by WebKit content blockers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionType {
    Block,
    BlockCookies,
    CssDisplayNone,
    IgnorePreviousRules,
}

/// Trigger definition in WebKit content blocker format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct WebKitRuleTrigger {
    pub url_filter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_filter_is_case_sensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_type: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unless_domain: Option<Vec<String>>,
}

/// Action definition in WebKit content blocker format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WebKitRuleAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

/// A single WebKit content blocker rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebKitContentRule {
    pub trigger: WebKitRuleTrigger,
    pub action: WebKitRuleAction,
}

/// Information about a configured filter list subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterListConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub last_updated: Option<String>,
    pub rule_count: usize,
}

/// Conversion statistics during rule compilation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ConversionStats {
    pub total_input_rules: usize,
    pub converted_rules: usize,
    pub unsupported_rules: usize,
    pub comments_and_empty: usize,
}

/// Controller and compiler for native WebKit content filters.
#[derive(Debug, Clone)]
pub struct AdBlockManager {
    enabled: bool,
    allowlist: HashSet<String>,
    filter_lists: Vec<FilterListConfig>,
    cache_dir: PathBuf,
    blocked_requests_count: u64,
    last_compilation_error: Option<String>,
    last_update_timestamp: Option<String>,
}

impl AdBlockManager {
    /// Creates a new AdBlockManager with default filter lists and configuration.
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let mut mgr = Self {
            enabled: true,
            allowlist: HashSet::new(),
            filter_lists: Vec::new(),
            cache_dir: cache_dir.as_ref().to_path_buf(),
            blocked_requests_count: 0,
            last_compilation_error: None,
            last_update_timestamp: None,
        };
        mgr.init_default_lists();
        mgr
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn toggle_enabled(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    pub fn allowlist(&self) -> &HashSet<String> {
        &self.allowlist
    }

    pub fn is_allowlisted(&self, host: &str) -> bool {
        let clean = host.trim().to_lowercase();
        if clean.is_empty() {
            return false;
        }
        if self.allowlist.contains(&clean) {
            return true;
        }
        // Subdomain matching: if allowlist contains "example.com", "sub.example.com" is allowlisted (zero-allocation)
        for allowed in &self.allowlist {
            if clean.len() > allowed.len()
                && clean.ends_with(allowed.as_str())
                && clean.as_bytes()[clean.len() - allowed.len() - 1] == b'.'
            {
                return true;
            }
        }
        false
    }

    pub fn add_to_allowlist(&mut self, host: &str) {
        let clean = host.trim().to_lowercase();
        if !clean.is_empty() {
            self.allowlist.insert(clean);
        }
    }

    pub fn remove_from_allowlist(&mut self, host: &str) -> bool {
        let clean = host.trim().to_lowercase();
        self.allowlist.remove(&clean)
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn filter_lists(&self) -> &[FilterListConfig] {
        &self.filter_lists
    }

    pub fn filter_lists_mut(&mut self) -> &mut [FilterListConfig] {
        &mut self.filter_lists
    }

    pub fn blocked_requests_count(&self) -> u64 {
        self.blocked_requests_count
    }

    pub fn rule_count(&self) -> usize {
        let bundled = self.get_bundled_rules().len();
        let list_rules: usize = self.filter_lists.iter().map(|l| l.rule_count).sum();
        if list_rules > 0 {
            list_rules
        } else {
            bundled
        }
    }

    pub fn increment_blocked_requests(&mut self) {
        self.blocked_requests_count = self.blocked_requests_count.saturating_add(1);
    }

    pub fn reset_blocked_counter(&mut self) {
        self.blocked_requests_count = 0;
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_compilation_error.as_deref()
    }

    pub fn last_update(&self) -> Option<&str> {
        self.last_update_timestamp.as_deref()
    }

    /// Default maintained filter lists with licensing reviewed.
    fn init_default_lists(&mut self) {
        self.filter_lists = vec![
            FilterListConfig {
                id: "easylist".to_string(),
                name: "EasyList (Primary Ad Servers & Banners)".to_string(),
                url: "https://easylist.to/easylist/easylist.txt".to_string(),
                enabled: true,
                last_updated: None,
                rule_count: 0,
            },
            FilterListConfig {
                id: "easyprivacy".to_string(),
                name: "EasyPrivacy (Tracking Scripts & Pixels)".to_string(),
                url: "https://easylist.to/easylist/easyprivacy.txt".to_string(),
                enabled: true,
                last_updated: None,
                rule_count: 0,
            },
            FilterListConfig {
                id: "fanboy-annoyance".to_string(),
                name: "Fanboy's Annoyance (Cookie Notices & Popups)".to_string(),
                url: "https://easylist.to/easylist/fanboy-annoyance.txt".to_string(),
                enabled: false,
                last_updated: None,
                rule_count: 0,
            },
        ];
    }

    /// Returns high-confidence built-in network and cosmetic rules for immediate protection
    /// without requiring an active network connection on fresh installs.
    pub fn get_bundled_rules(&self) -> Vec<WebKitContentRule> {
        let mut rules = Vec::new();

        // 1. Common advertising network domains
        let ad_domains = [
            "doubleclick.net",
            "googlesyndication.com",
            "googleadservices.com",
            "adservice.google.com",
            "adnxs.com",
            "adform.net",
            "openx.net",
            "criteo.com",
            "rubiconproject.com",
            "pubmatic.com",
            "taboola.com",
            "outbrain.com",
            "zedo.com",
            "adroll.com",
            "advertising.com",
            "amazon-adsystem.com",
            "scorecardresearch.com",
            "casale-media.com",
            "popcash.net",
            "popads.net",
            "propellerads.com",
            "adcolony.com",
            "admob.com",
            "smartadserver.com",
            "exponential.com",
        ];

        for domain in ad_domains {
            rules.push(WebKitContentRule {
                trigger: WebKitRuleTrigger {
                    url_filter: format!(
                        r"^https?://([a-zA-Z0-9.-]+\.)?{}/.*",
                        regex_escape(domain)
                    ),
                    load_type: None,
                    resource_type: None,
                    url_filter_is_case_sensitive: Some(false),
                    if_domain: None,
                    unless_domain: None,
                },
                action: WebKitRuleAction {
                    action_type: "block".to_string(),
                    selector: None,
                },
            });
        }

        // 2. Common tracking scripts & telemetry
        let tracker_domains = [
            "google-analytics.com",
            "analytics.google.com",
            "hotjar.com",
            "quantserve.com",
            "segment.io",
            "mixpanel.com",
            "optimizely.com",
            "crazyegg.com",
            "newrelic.com",
            "statcounter.com",
            "yandex.ru/metrika",
            "mc.yandex.ru",
            "chartbeat.com",
            "clicky.com",
            "clarity.ms",
            "mouseflow.com",
        ];

        for tracker in tracker_domains {
            rules.push(WebKitContentRule {
                trigger: WebKitRuleTrigger {
                    url_filter: format!(
                        r"^https?://([a-zA-Z0-9.-]+\.)?{}/.*",
                        regex_escape(tracker)
                    ),
                    load_type: None,
                    resource_type: None,
                    url_filter_is_case_sensitive: Some(false),
                    if_domain: None,
                    unless_domain: None,
                },
                action: WebKitRuleAction {
                    action_type: "block".to_string(),
                    selector: None,
                },
            });
        }

        // 3. Common cosmetic hiding rules for generic advertising containers
        let generic_selectors = [
            ".ad-banner",
            ".advertisement",
            ".adsbygoogle",
            ".sponsored-post",
            "#ad-header",
            "#banner-ad",
            ".trc_related_container",
            ".OUTBRAIN",
            ".taboola-container",
        ];

        for selector in generic_selectors {
            rules.push(WebKitContentRule {
                trigger: WebKitRuleTrigger {
                    url_filter: ".*".to_string(),
                    load_type: None,
                    resource_type: None,
                    url_filter_is_case_sensitive: None,
                    if_domain: None,
                    unless_domain: None,
                },
                action: WebKitRuleAction {
                    action_type: "css-display-none".to_string(),
                    selector: Some(selector.to_string()),
                },
            });
        }

        rules
    }

    /// Converts standard Adblock Plus / EasyList rules into WebKit Content Blocker format.
    pub fn parse_abp_rules(&self, content: &str) -> (Vec<WebKitContentRule>, ConversionStats) {
        let mut rules = Vec::new();
        let mut stats = ConversionStats::default();

        for line in content.lines() {
            stats.total_input_rules += 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('[') {
                stats.comments_and_empty += 1;
                continue;
            }

            if let Some(rule) = Self::convert_single_abp_rule(trimmed) {
                rules.push(rule);
                stats.converted_rules += 1;
            } else {
                stats.unsupported_rules += 1;
            }
        }

        (rules, stats)
    }

    /// Parses an individual ABP rule line.
    fn convert_single_abp_rule(line: &str) -> Option<WebKitContentRule> {
        // 1. Cosmetic Element Hiding: ##selector or domain##selector
        if let Some(pos) = line.find("##") {
            let domain_part = &line[..pos];
            let selector = &line[pos + 2..];

            if selector.trim().is_empty() {
                return None;
            }

            let if_domain = if domain_part.is_empty() {
                None
            } else {
                Some(
                    domain_part
                        .split(',')
                        .map(|d| format!("*{}", d.trim()))
                        .collect(),
                )
            };

            return Some(WebKitContentRule {
                trigger: WebKitRuleTrigger {
                    url_filter: ".*".to_string(),
                    if_domain,
                    ..Default::default()
                },
                action: WebKitRuleAction {
                    action_type: "css-display-none".to_string(),
                    selector: Some(selector.trim().to_string()),
                },
            });
        }

        // 2. Exception Rule: @@
        let is_exception = line.starts_with("@@");
        let rule_body = if is_exception { &line[2..] } else { line };

        // Split options: rule$options
        let (pattern_part, options_part) = match rule_body.find('$') {
            Some(idx) => (&rule_body[..idx], Some(&rule_body[idx + 1..])),
            None => (rule_body, None),
        };

        let mut load_type = None;
        let mut resource_type = None;
        let mut if_domain = None;
        let mut unless_domain = None;

        if let Some(opts) = options_part {
            for opt in opts.split(',') {
                let opt_trimmed = opt.trim();
                match opt_trimmed {
                    "third-party" | "~first-party" => {
                        load_type = Some(vec!["third-party".to_string()]);
                    }
                    "first-party" | "~third-party" => {
                        load_type = Some(vec!["first-party".to_string()]);
                    }
                    "script" => {
                        resource_type = Some(vec!["script".to_string()]);
                    }
                    "image" => {
                        resource_type = Some(vec!["image".to_string()]);
                    }
                    "stylesheet" => {
                        resource_type = Some(vec!["style-sheet".to_string()]);
                    }
                    "subdocument" | "document" => {
                        resource_type = Some(vec!["document".to_string()]);
                    }
                    opt if opt.starts_with("domain=") => {
                        let domains_str = &opt[7..];
                        let mut included = Vec::new();
                        let mut excluded = Vec::new();
                        for d in domains_str.split('|') {
                            if let Some(exc) = d.strip_prefix('~') {
                                excluded.push(format!("*{}", exc.trim()));
                            } else {
                                included.push(format!("*{}", d.trim()));
                            }
                        }
                        if !included.is_empty() {
                            if_domain = Some(included);
                        }
                        if !excluded.is_empty() {
                            unless_domain = Some(excluded);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Convert ABP pattern to WebKit regex
        let regex_filter = Self::abp_pattern_to_regex(pattern_part)?;

        let action_type = if is_exception {
            "ignore-previous-rules".to_string()
        } else {
            "block".to_string()
        };

        Some(WebKitContentRule {
            trigger: WebKitRuleTrigger {
                url_filter: regex_filter,
                load_type,
                resource_type,
                if_domain,
                unless_domain,
                url_filter_is_case_sensitive: Some(false),
            },
            action: WebKitRuleAction {
                action_type,
                selector: None,
            },
        })
    }

    /// Converts an ABP URL pattern into a WebKit-compatible regular expression.
    fn abp_pattern_to_regex(pattern: &str) -> Option<String> {
        let p = pattern.trim();
        if p.is_empty() {
            return None;
        }

        let mut regex = String::with_capacity(p.len() * 2);

        let body = if let Some(domain_prefix) = p.strip_prefix("||") {
            regex.push_str(r"^https?://([a-zA-Z0-9.-]+\.)?");
            domain_prefix
        } else if let Some(exact_start) = p.strip_prefix('|') {
            regex.push('^');
            exact_start
        } else {
            p
        };

        let (body, exact_end) = if let Some(stripped) = body.strip_suffix('|') {
            (stripped, true)
        } else {
            (body, false)
        };

        for ch in body.chars() {
            match ch {
                '*' => regex.push_str(".*"),
                '^' => regex.push_str(r"([/?#:]|$)"),
                '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '$' => {
                    regex.push('\\');
                    regex.push(ch);
                }
                c => regex.push(c),
            }
        }

        if exact_end {
            regex.push('$');
        }

        Some(regex)
    }

    /// Compiles all active rules, built-ins, and allowlist rules into the final WebKit JSON format.
    pub fn compile_to_json(&self) -> Result<String, String> {
        let mut all_rules = self.get_bundled_rules();

        // If allowlist is populated, add ignore-previous-rules exceptions for allowlisted hosts
        for host in &self.allowlist {
            all_rules.push(WebKitContentRule {
                trigger: WebKitRuleTrigger {
                    url_filter: ".*".to_string(),
                    if_domain: Some(vec![format!("*{host}")]),
                    ..Default::default()
                },
                action: WebKitRuleAction {
                    action_type: "ignore-previous-rules".to_string(),
                    selector: None,
                },
            });
        }

        serde_json::to_string_pretty(&all_rules).map_err(|e| e.to_string())
    }
}

fn regex_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '$' | '^' | '*' => {
                out.push('\\');
                out.push(ch);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adblock_manager_initialization_and_toggle() {
        let mut mgr = AdBlockManager::new("/tmp/test_adblock");
        assert!(mgr.is_enabled());
        assert_eq!(mgr.filter_lists().len(), 3);

        mgr.toggle_enabled();
        assert!(!mgr.is_enabled());
        mgr.toggle_enabled();
        assert!(mgr.is_enabled());
    }

    #[test]
    fn test_allowlist_subdomain_matching() {
        let mut mgr = AdBlockManager::new("/tmp/test_adblock");
        assert!(!mgr.is_allowlisted("example.com"));

        mgr.add_to_allowlist("example.com");
        assert!(mgr.is_allowlisted("example.com"));
        assert!(mgr.is_allowlisted("sub.example.com"));
        assert!(mgr.is_allowlisted("deep.sub.example.com"));
        assert!(!mgr.is_allowlisted("notexample.com"));

        assert!(mgr.remove_from_allowlist("example.com"));
        assert!(!mgr.is_allowlisted("example.com"));
    }

    #[test]
    fn test_abp_rule_conversion() {
        let mgr = AdBlockManager::new("/tmp/test_adblock");

        let abp_sample = r#"
! EasyList sample rules
[Adblock Plus 2.0]
||doubleclick.net^$third-party
||googleadservices.com^$script
##.sponsored-banner
example.org##.ad-box
@@||trusted-site.com^
"#;

        let (rules, stats) = mgr.parse_abp_rules(abp_sample);
        assert_eq!(stats.converted_rules, 5);
        assert_eq!(stats.comments_and_empty, 3);
        assert_eq!(rules.len(), 5);

        // Check cosmetic rule
        let cosmetic = rules
            .iter()
            .find(|r| {
                r.action.action_type == "css-display-none"
                    && r.action.selector.as_deref() == Some(".sponsored-banner")
            })
            .unwrap();
        assert_eq!(cosmetic.trigger.url_filter, ".*");

        // Check domain cosmetic rule
        let domain_cosmetic = rules
            .iter()
            .find(|r| r.action.selector.as_deref() == Some(".ad-box"))
            .unwrap();
        assert_eq!(
            domain_cosmetic.trigger.if_domain.as_ref().unwrap(),
            &vec!["*example.org".to_string()]
        );

        // Check exception rule
        let exception = rules
            .iter()
            .find(|r| r.action.action_type == "ignore-previous-rules")
            .unwrap();
        assert!(exception.trigger.url_filter.contains("trusted-site\\.com"));
    }

    #[test]
    fn test_compile_bundled_rules_valid_json() {
        let mut mgr = AdBlockManager::new("/tmp/test_adblock");
        assert_eq!(mgr.cache_dir(), Path::new("/tmp/test_adblock"));
        mgr.add_to_allowlist("news.ycombinator.com");

        let json = mgr.compile_to_json().unwrap();
        assert!(json.contains(r"doubleclick\\.net"));
        assert!(json.contains(r"google-analytics\\.com"));
        assert!(json.contains("news.ycombinator.com"));

        // Validate JSON parsing
        let parsed: Vec<WebKitContentRule> = serde_json::from_str(&json).unwrap();
        assert!(parsed.len() > 30);
    }
}
