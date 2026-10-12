use std::path::PathBuf;
use vingilot_core::models::SyncOptions;

/// Wizard UI controller for configuring and exporting portable .vingilot sync bundles.
#[derive(Debug, Clone)]
pub struct ExportWizard {
    is_open: bool,
    options: SyncOptions,
    output_path: Option<PathBuf>,
    passphrase: Option<String>,
    status_message: Option<String>,
}

impl Default for ExportWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportWizard {
    pub fn new() -> Self {
        Self {
            is_open: false,
            options: SyncOptions::default(),
            output_path: None,
            passphrase: None,
            status_message: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn options(&self) -> &SyncOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut SyncOptions {
        &mut self.options
    }

    pub fn set_output_path(&mut self, path: PathBuf) {
        self.output_path = Some(path);
    }

    pub fn output_path(&self) -> Option<&PathBuf> {
        self.output_path.as_ref()
    }

    pub fn set_passphrase(&mut self, pass: Option<String>) {
        self.passphrase = pass;
    }

    pub fn passphrase(&self) -> Option<&str> {
        self.passphrase.as_deref()
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_wizard_configuration() {
        let mut wizard = ExportWizard::new();
        assert!(!wizard.is_open());
        wizard.open();
        assert!(wizard.is_open());

        wizard.set_output_path(PathBuf::from("/tmp/export.vingilot"));
        assert_eq!(
            wizard.output_path().unwrap().to_str(),
            Some("/tmp/export.vingilot")
        );

        wizard.set_passphrase(Some("secure_pass".to_string()));
        assert_eq!(wizard.passphrase(), Some("secure_pass"));
    }
}
