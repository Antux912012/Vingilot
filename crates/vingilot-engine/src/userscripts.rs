use serde::{Deserialize, Serialize};

/// Point in DOM lifecycle when a userscript should be injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RunAt {
    #[default]
    DocumentEnd,
    DocumentStart,
}

/// Metadata and code definition for a userscript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Userscript {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub match_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub run_at: RunAt,
    pub code: String,
    pub is_enabled: bool,
}

impl Userscript {
    /// Determines whether the userscript matches a given URL.
    pub fn matches_url(&self, url: &str) -> bool {
        if !self.is_enabled {
            return false;
        }

        // Check exclusions first
        for exclude in &self.exclude_patterns {
            if pattern_matches(exclude, url) {
                return false;
            }
        }

        // Check inclusions
        if self.match_patterns.is_empty() {
            return true;
        }

        self.match_patterns
            .iter()
            .any(|pattern| pattern_matches(pattern, url))
    }
}

/// Simple pattern matching supporting wildcards (e.g. `*://*.example.com/*`, `https://*.wikipedia.org/*`).
fn pattern_matches(pattern: &str, target: &str) -> bool {
    if pattern == "*" || pattern == "<all_urls>" {
        return true;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return target == pattern;
    }

    let mut current_target = target;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // First part must be a prefix
            if !current_target.starts_with(part) {
                return false;
            }
            current_target = &current_target[part.len()..];
        } else if i == parts.len() - 1 && !pattern.ends_with('*') {
            // Last part must be a suffix
            return current_target.ends_with(part);
        } else {
            // Intermediate part must exist in remaining string
            if let Some(pos) = current_target.find(part) {
                current_target = &current_target[pos + part.len()..];
            } else {
                return false;
            }
        }
    }
    true
}

/// Parser for Greasemonkey/Tampermonkey `// ==UserScript==` header metadata blocks.
pub struct UserscriptParser;

impl UserscriptParser {
    pub fn parse(script_content: &str) -> Result<Userscript, String> {
        let mut name = "Untitled Script".to_string();
        let mut description = None;
        let mut version = None;
        let mut match_patterns = Vec::new();
        let mut exclude_patterns = Vec::new();
        let mut run_at = RunAt::DocumentEnd;

        let mut in_header = false;

        for line in script_content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("==UserScript==") {
                in_header = true;
                continue;
            }
            if trimmed.contains("==/UserScript==") {
                break;
            }

            if in_header && trimmed.starts_with("//") {
                let directive_line = trimmed.trim_start_matches('/').trim();
                if let Some(directive) = directive_line.strip_prefix('@') {
                    let mut parts = directive.split_whitespace();
                    let key = parts.next().unwrap_or("");
                    let val = parts.collect::<Vec<_>>().join(" ");

                    match key {
                        "name" => name = val,
                        "description" => description = Some(val),
                        "version" => version = Some(val),
                        "match" | "include" => match_patterns.push(val),
                        "exclude" => exclude_patterns.push(val),
                        "run-at" => {
                            if val.contains("document-start") {
                                run_at = RunAt::DocumentStart;
                            } else {
                                run_at = RunAt::DocumentEnd;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(Userscript {
            name,
            description,
            version,
            match_patterns,
            exclude_patterns,
            run_at,
            code: script_content.to_string(),
            is_enabled: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userscript_metadata_parser() {
        let script = r#"
        // ==UserScript==
        // @name         Dark Theme Enhancer
        // @description  Forces clean dark background on websites
        // @version      1.0.0
        // @match        https://*.wikipedia.org/*
        // @exclude      https://*.wikipedia.org/wiki/Special:*
        // @run-at       document-start
        // ==/UserScript==

        console.log("Dark Theme Enhancer Loaded");
        "#;

        let parsed = UserscriptParser::parse(script).unwrap();
        assert_eq!(parsed.name, "Dark Theme Enhancer");
        assert_eq!(
            parsed.description.as_deref(),
            Some("Forces clean dark background on websites")
        );
        assert_eq!(parsed.version.as_deref(), Some("1.0.0"));
        assert_eq!(parsed.run_at, RunAt::DocumentStart);
        assert_eq!(parsed.match_patterns.len(), 1);
        assert_eq!(parsed.exclude_patterns.len(), 1);

        assert!(parsed.matches_url("https://en.wikipedia.org/wiki/Elendil"));
    }
}
