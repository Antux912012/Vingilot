use std::path::PathBuf;
use vingilot_core::browser_import::{BrowserDetector, DetectedBrowserProfile};

/// Supported import source options for the selection wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSourceType {
    VingilotBundle(PathBuf),
    DetectedBrowser(DetectedBrowserProfile),
    HtmlBookmarkFile(PathBuf),
    JsonBookmarkFile(PathBuf),
}

/// UI controller for browser import source selection dialog.
#[derive(Debug, Clone, Default)]
pub struct BrowserImportDialog {
    is_open: bool,
    detected_profiles: Vec<DetectedBrowserProfile>,
    selected_source: Option<ImportSourceType>,
    import_bookmarks: bool,
    import_history: bool,
    status_message: Option<String>,
}

impl BrowserImportDialog {
    pub fn new() -> Self {
        let detected = BrowserDetector::detect_installed_profiles();
        Self {
            is_open: false,
            detected_profiles: detected,
            selected_source: None,
            import_bookmarks: true,
            import_history: true,
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

    pub fn detected_profiles(&self) -> &[DetectedBrowserProfile] {
        &self.detected_profiles
    }

    pub fn select_source(&mut self, source: ImportSourceType) {
        self.selected_source = Some(source);
    }

    pub fn selected_source(&self) -> Option<&ImportSourceType> {
        self.selected_source.as_ref()
    }

    pub fn set_import_options(&mut self, bookmarks: bool, history: bool) {
        self.import_bookmarks = bookmarks;
        self.import_history = history;
    }

    pub fn import_bookmarks(&self) -> bool {
        self.import_bookmarks
    }

    pub fn import_history(&self) -> bool {
        self.import_history
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
    fn test_browser_import_dialog_selection() {
        let mut dialog = BrowserImportDialog::new();
        assert!(!dialog.is_open());
        dialog.open();
        assert!(dialog.is_open());

        dialog.select_source(ImportSourceType::HtmlBookmarkFile(PathBuf::from(
            "/tmp/bookmarks.html",
        )));
        assert!(matches!(
            dialog.selected_source(),
            Some(ImportSourceType::HtmlBookmarkFile(_))
        ));

        dialog.set_import_options(true, false);
        assert!(dialog.import_bookmarks());
        assert!(!dialog.import_history());
    }
}
