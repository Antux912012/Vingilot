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

    /* Right side menu / options panel */
    .right-sidebar {
        background-color: @headerbar_bg_color;
        border-left: 1px solid alpha(currentColor, 0.12);
        min-width: 360px;
    }

    .right-sidebar-header {
        padding: 10px 14px;
        border-bottom: 1px solid alpha(currentColor, 0.08);
    }

    .menu-group-card {
        background-color: alpha(currentColor, 0.04);
        border-radius: 10px;
        padding: 10px 12px;
        border: 1px solid alpha(currentColor, 0.06);
    }

    /* Start Page Styling */
    .start-page-container {
        padding: 32px 24px;
    }
    .start-page-logo {
        font-size: 32px;
        font-weight: 800;
        margin-bottom: 8px;
    }
    .start-page-search {
        border-radius: 24px;
        min-height: 42px;
        padding-left: 14px;
        padding-right: 14px;
        font-size: 15px;
    }
    .start-page-section-title {
        font-size: 17px;
        font-weight: bold;
        margin-top: 14px;
        margin-bottom: 4px;
    }
    .recent-card {
        border-radius: 12px;
        background-color: alpha(currentColor, 0.05);
        border: 1px solid alpha(currentColor, 0.08);
        padding: 8px;
        transition: background-color 150ms ease, border-color 150ms ease;
    }
    .recent-card:hover {
        background-color: alpha(currentColor, 0.10);
        border-color: @accent_color;
    }
    .recent-preview-placeholder {
        border-radius: 8px;
        background-color: alpha(currentColor, 0.08);
    }
    .recent-title {
        font-weight: 600;
        font-size: 13px;
    }
    .recent-domain {
        font-size: 11px;
        opacity: 0.65;
    }
    .bookmark-tile {
        border-radius: 10px;
        padding: 10px 8px;
        background-color: alpha(currentColor, 0.04);
        border: 1px solid alpha(currentColor, 0.06);
        min-width: 100px;
        transition: all 150ms ease;
    }
    .bookmark-tile:hover {
        background-color: alpha(currentColor, 0.09);
        border-color: alpha(currentColor, 0.15);
    }
    .bookmark-tile.drop-hover {
        outline: 2px solid @accent_color;
        background-color: alpha(@accent_color, 0.20);
    }
    .bookmark-title {
        font-size: 12px;
        font-weight: 500;
    }
    .breadcrumb-btn {
        font-weight: 600;
        border-radius: 6px;
    }

    .menu-group-title {
        font-weight: 700;
        font-size: 13px;
        margin-bottom: 6px;
        color: @accent_color;
    }

    .menu-item-row {
        margin: 4px 0;
    }

    .menu-item-entry {
        border-radius: 6px;
        padding: 6px 8px;
        margin-top: 4px;
    }

    .accent-palette-box {
        padding: 4px 10px;
        border-radius: 20px;
        background-color: alpha(currentColor, 0.05);
        margin: 6px 0;
    }
    .accent-palette-label {
        font-size: 12px;
        opacity: 0.7;
        margin-right: 6px;
    }
    .accent-dot-btn {
        border-radius: 9999px;
        min-width: 18px;
        min-height: 18px;
        padding: 0;
        margin: 0 2px;
        border: 2px solid transparent;
        transition: transform 150ms ease, border-color 150ms ease;
    }
    .accent-dot-btn:hover {
        transform: scale(1.3);
        border-color: currentColor;
    }
    .accent-dot-default { background-color: #888888; }
    .accent-dot-blue { background-color: #3584e4; }
    .accent-dot-teal { background-color: #2190a4; }
    .accent-dot-green { background-color: #3a944a; }
    .accent-dot-yellow { background-color: #c88800; }
    .accent-dot-orange { background-color: #ed5b00; }
    .accent-dot-red { background-color: #e62d42; }
    .accent-dot-pink { background-color: #d56199; }
    .accent-dot-purple { background-color: #9141ac; }
    .accent-dot-slate { background-color: #6f8396; }

    .address-bar-action-btn {
        border-radius: 6px;
        padding: 4px 6px;
        opacity: 0.85;
        transition: opacity 150ms ease, background-color 150ms ease;
    }
    .address-bar-action-btn:hover {
        opacity: 1.0;
        background-color: alpha(currentColor, 0.08);
    }
    "#
}

/// Official brand color for Incognito window background.
pub const INCOGNITO_WINDOW_COLOR: &str = "#062640";

/// Official brand color for Incognito corner icon.
pub const INCOGNITO_ICON_COLOR: &str = "#ffc95c";

/// Returns CSS rules specifically for the Incognito private browsing mode.
pub fn incognito_css() -> &'static str {
    r#"
    /* Vingilot Incognito Mode */
    window, .background {
        background-color: #062640 !important;
        color: #e0e8f0;
    }

    headerbar, .header-bar {
        background-color: #062640 !important;
        border-bottom: 1px solid rgba(255, 201, 92, 0.2) !important;
        box-shadow: none;
    }

    .sidebar, sidebar {
        background-color: #062640 !important;
        border-right: 1px solid rgba(255, 255, 255, 0.12) !important;
    }

    .right-sidebar {
        background-color: #062640 !important;
        border-left: 1px solid rgba(255, 255, 255, 0.12) !important;
        min-width: 360px;
    }

    .menu-group-card {
        background-color: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 10px;
        margin: 6px 12px;
        padding: 10px 12px;
    }

    .menu-group-title {
        font-weight: 700;
        font-size: 13px;
        margin-bottom: 6px;
        color: #ffc95c !important;
    }

    paned {
        background-color: #062640 !important;
    }

    paned > :last-child {
        background-color: #062640 !important;
        border-top: 1px solid rgba(255, 255, 255, 0.12);
    }

    entry, .address-entry {
        background-color: #0a3355 !important;
        color: #ffffff !important;
        border: 1px solid #144973 !important;
    }

    .incognito-corner-icon,
    .incognito-corner-icon image,
    .incognito-corner-icon label {
        color: #ffc95c !important;
        -gtk-icon-palette: error #ffc95c, success #ffc95c, warning #ffc95c;
    }

    button.incognito-corner-icon {
        background: transparent;
        border: none;
        box-shadow: none;
        color: #ffc95c !important;
        padding: 4px 8px;
        margin-left: 6px;
        margin-right: 6px;
    }

    button.incognito-corner-icon:hover {
        background-color: rgba(255, 201, 92, 0.15);
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

    #[test]
    fn test_incognito_theme_css_colors() {
        assert_eq!(INCOGNITO_WINDOW_COLOR, "#062640");
        assert_eq!(INCOGNITO_ICON_COLOR, "#ffc95c");
        let css = incognito_css();
        assert!(css.contains("#062640"));
        assert!(css.contains("#ffc95c"));
    }
}
