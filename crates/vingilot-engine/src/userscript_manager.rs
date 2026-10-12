use std::path::{Path, PathBuf};

use crate::traits::{BrowserEngine, EngineResult};
use crate::userscripts::{RunAt, Userscript, UserscriptParser};
use crate::webkit::WebKitEngine;

/// Manages loading and injecting userscripts into WebKitGTK browsing contexts.
#[derive(Debug, Clone)]
pub struct UserscriptManager {
    scripts_dir: PathBuf,
    scripts: Vec<Userscript>,
}

impl UserscriptManager {
    pub fn new(scripts_dir: impl AsRef<Path>) -> Self {
        Self {
            scripts_dir: scripts_dir.as_ref().to_path_buf(),
            scripts: Vec::new(),
        }
    }

    pub fn scripts(&self) -> &[Userscript] {
        &self.scripts
    }

    pub fn scripts_mut(&mut self) -> &mut [Userscript] {
        &mut self.scripts
    }

    pub fn add_script(&mut self, script: Userscript) {
        self.scripts.push(script);
    }

    /// Loads all `.user.js` scripts located inside the userscript directory.
    pub fn load_from_disk(&mut self) -> EngineResult<usize> {
        if !self.scripts_dir.exists() {
            let _ = std::fs::create_dir_all(&self.scripts_dir);
            return Ok(0);
        }

        let mut loaded_count = 0;
        if let Ok(entries) = std::fs::read_dir(&self.scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.ends_with(".user.js"))
                        .unwrap_or(false)
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(script) = UserscriptParser::parse(&content) {
                            self.scripts.push(script);
                            loaded_count += 1;
                        }
                    }
                }
            }
        }
        Ok(loaded_count)
    }

    /// Injects matching scripts into the engine for the active page URL.
    pub fn inject_matching_scripts(
        &self,
        engine: &mut WebKitEngine,
        current_url: &str,
        run_at: RunAt,
    ) -> EngineResult<usize> {
        let mut injected = 0;
        for script in &self.scripts {
            if script.run_at == run_at && script.matches_url(current_url) {
                let _ = engine.execute_javascript(&script.code)?;
                injected += 1;
            }
        }
        Ok(injected)
    }

    pub fn toggle_script(&mut self, name: &str) -> bool {
        if let Some(script) = self.scripts.iter_mut().find(|s| s.name == name) {
            script.is_enabled = !script.is_enabled;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userscript_manager_injection() {
        let mut mgr = UserscriptManager::new("/tmp/vingilot/userscripts");

        let script = Userscript {
            name: "Title Modifier".to_string(),
            description: None,
            version: None,
            match_patterns: vec!["https://example.com/*".to_string()],
            exclude_patterns: Vec::new(),
            run_at: RunAt::DocumentEnd,
            code: "document.title = 'Modified';".to_string(),
            is_enabled: true,
        };

        mgr.add_script(script);

        let mut engine = WebKitEngine::default();
        engine.load_url("https://example.com").unwrap();

        let count = mgr
            .inject_matching_scripts(&mut engine, "https://example.com/page", RunAt::DocumentEnd)
            .unwrap();
        assert_eq!(count, 1);

        // Does not inject for non-matching URL
        let non_matching_count = mgr
            .inject_matching_scripts(&mut engine, "https://other.org/page", RunAt::DocumentEnd)
            .unwrap();
        assert_eq!(non_matching_count, 0);
    }
}
