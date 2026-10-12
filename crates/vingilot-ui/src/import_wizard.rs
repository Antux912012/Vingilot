use std::path::PathBuf;
use vingilot_core::merge::ImportMode;

/// Wizard UI controller for importing .vingilot bundles.
#[derive(Debug, Clone)]
pub struct ImportWizard {
    is_open: bool,
    source_file: Option<PathBuf>,
    passphrase_input: Option<String>,
    mode: ImportMode,
    is_encrypted_detected: bool,
    status_message: Option<String>,
}

impl Default for ImportWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportWizard {
    pub fn new() -> Self {
        Self {
            is_open: false,
            source_file: None,
            passphrase_input: None,
            mode: ImportMode::Merge,
            is_encrypted_detected: false,
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

    pub fn set_source_file(&mut self, path: PathBuf, is_encrypted: bool) {
        self.source_file = Some(path);
        self.is_encrypted_detected = is_encrypted;
    }

    pub fn source_file(&self) -> Option<&PathBuf> {
        self.source_file.as_ref()
    }

    pub fn is_encrypted_detected(&self) -> bool {
        self.is_encrypted_detected
    }

    pub fn set_passphrase(&mut self, pass: &str) {
        self.passphrase_input = Some(pass.to_string());
    }

    pub fn passphrase(&self) -> Option<&str> {
        self.passphrase_input.as_deref()
    }

    pub fn set_mode(&mut self, mode: ImportMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> ImportMode {
        self.mode
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
    fn test_import_wizard_configuration() {
        let mut wizard = ImportWizard::new();
        assert_eq!(wizard.mode(), ImportMode::Merge);

        wizard.set_source_file(PathBuf::from("/tmp/backup.vingilot"), true);
        assert!(wizard.is_encrypted_detected());

        wizard.set_passphrase("unlock123");
        assert_eq!(wizard.passphrase(), Some("unlock123"));

        wizard.set_mode(ImportMode::Replace);
        assert_eq!(wizard.mode(), ImportMode::Replace);
    }
}
