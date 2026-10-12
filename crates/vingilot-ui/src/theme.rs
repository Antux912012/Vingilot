/// Application-wide visual theme definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[default]
    System,
    Light,
    Dark,
    TrueBlack,
}

impl AppTheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::TrueBlack => "true-black",
        }
    }

    /// Returns custom CSS for Libadwaita window styling, particularly pure black for AMOLED screens.
    pub fn custom_css(&self) -> Option<&'static str> {
        match self {
            Self::TrueBlack => Some(
                r#"
                /* Vingilot True Black AMOLED Mode */
                window, .background {
                    background-color: #000000;
                    color: #ffffff;
                }
                headerbar, .header-bar {
                    background-color: #000000;
                    border-bottom: none;
                }
                entry, .address-entry {
                    background-color: #0a0a0a;
                    color: #ffffff;
                    border: 1px solid #222222;
                }
                .sidebar, sidebar {
                    background-color: #000000;
                    border-right: 1px solid #1a1a1a;
                }
                paned > :last-child {
                    border-top: 1px solid #1a1a1a;
                    background-color: #000000;
                }
                "#,
            ),
            _ => None,
        }
    }
}

/// Returns the base CSS styling to unify the header bar and the sidebar seamlessly.
pub fn base_css() -> &'static str {
    r#"
    /* Unified Chrome Shell - Headerbar & Sidebar seamless integration */
    window, .background {
        background-color: @headerbar_bg_color;
    }

    headerbar {
        background-color: @headerbar_bg_color;
        border-bottom: none;
        box-shadow: none;
        padding-left: 0px;
    }

    .sidebar {
        background-color: @headerbar_bg_color;
        border-right: 1px solid alpha(currentColor, 0.12);
    }

    /* Paned styling: background matches chrome, webview gets clean top border */
    paned {
        background-color: @headerbar_bg_color;
    }

    paned > :last-child {
        border-top: 1px solid alpha(currentColor, 0.12);
        background-color: @window_bg_color;
    }

    paned separator {
        background: transparent;
        border-right: 1px solid alpha(currentColor, 0.12);
        min-width: 1px;
    }

    /* Tab and sidebar action buttons matching top bar appearance */
    .sidebar button {
        border-radius: 8px;
        margin: 2px 4px;
        padding: 0;
        min-height: 36px;
        background: transparent;
        border: none;
        transition: background-color 150ms ease, color 150ms ease;
    }

    .sidebar button:hover {
        background-color: alpha(currentColor, 0.08);
    }

    .sidebar button:active {
        background-color: alpha(currentColor, 0.14);
    }

    .sidebar button.suggested-action {
        background-color: alpha(@accent_color, 0.22);
        color: @accent_color;
        font-weight: 600;
    }

    .sidebar button.suggested-action:hover {
        background-color: alpha(@accent_color, 0.32);
    }

    /* Horizontal separator inside sidebar dividing pinned and normal tabs */
    .sidebar separator {
        background-color: alpha(currentColor, 0.12);
        min-height: 1px;
        margin: 6px 8px;
    }
    "#
}

/// Theme manager coordinating window theme switching.
#[derive(Debug, Clone, Default)]
pub struct ThemeManager {
    current_theme: AppTheme,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_theme(&self) -> AppTheme {
        self.current_theme
    }

    pub fn set_theme(&mut self, theme: AppTheme) {
        self.current_theme = theme;
    }

    pub fn is_true_black(&self) -> bool {
        self.current_theme == AppTheme::TrueBlack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_black_theme_css() {
        let mut mgr = ThemeManager::new();
        assert_eq!(mgr.current_theme(), AppTheme::System);
        assert!(!mgr.is_true_black());
        assert!(mgr.current_theme().custom_css().is_none());

        mgr.set_theme(AppTheme::TrueBlack);
        assert!(mgr.is_true_black());
        let css = mgr.current_theme().custom_css().unwrap();
        assert!(css.contains("#000000"));
        assert!(css.contains("True Black AMOLED"));
    }
}
