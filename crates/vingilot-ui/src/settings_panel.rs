use vingilot_core::models::{Profile, SearchEngine, UserSettings};

/// UI controller for user preferences and browser settings.
#[derive(Debug, Clone)]
pub struct SettingsPanel {
    is_open: bool,
    current_settings: UserSettings,
    available_search_engines: Vec<SearchEngine>,
    available_profiles: Vec<Profile>,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new(UserSettings::default())
    }
}

impl SettingsPanel {
    pub fn new(settings: UserSettings) -> Self {
        Self {
            is_open: false,
            current_settings: settings,
            available_search_engines: Vec::new(),
            available_profiles: Vec::new(),
        }
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

    pub fn settings(&self) -> &UserSettings {
        &self.current_settings
    }

    pub fn settings_mut(&mut self) -> &mut UserSettings {
        &mut self.current_settings
    }

    pub fn set_available_search_engines(&mut self, engines: Vec<SearchEngine>) {
        self.available_search_engines = engines;
    }

    pub fn available_search_engines(&self) -> &[SearchEngine] {
        &self.available_search_engines
    }

    pub fn set_available_profiles(&mut self, profiles: Vec<Profile>) {
        self.available_profiles = profiles;
    }

    pub fn available_profiles(&self) -> &[Profile] {
        &self.available_profiles
    }

    /// Updates current UI theme ("system", "light", "dark", "true-black").
    pub fn set_theme(&mut self, theme: &str) {
        self.current_settings.theme = theme.to_string();
    }

    /// Updates reader view theme ("light", "sepia", "dark", "true-black").
    pub fn set_reader_theme(&mut self, theme: &str) {
        self.current_settings.reader_theme = theme.to_string();
    }

    /// Sets AI model parameters.
    pub fn set_ai_config(
        &mut self,
        base_url: Option<String>,
        model: Option<String>,
        api_key: Option<String>,
    ) {
        self.current_settings.ai_provider_base_url = base_url;
        self.current_settings.ai_model_name = model;
        self.current_settings.ai_api_key = api_key;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_panel_controls() {
        let mut panel = SettingsPanel::default();
        assert!(!panel.is_open());
        panel.toggle();
        assert!(panel.is_open());

        panel.set_theme("true-black");
        assert_eq!(panel.settings().theme, "true-black");

        panel.set_reader_theme("sepia");
        assert_eq!(panel.settings().reader_theme, "sepia");

        panel.set_ai_config(
            Some("http://localhost:11434/v1".to_string()),
            Some("llama3.2".to_string()),
            None,
        );
        assert_eq!(panel.settings().ai_model_name.as_deref(), Some("llama3.2"));
    }
}
