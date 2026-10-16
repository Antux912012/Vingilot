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

    popover,
    popover.background {
        background: transparent;
        background-color: transparent;
        border: none;
        box-shadow: none;
    }
    popover > contents {
        background-color: @popover_bg_color;
        color: @popover_fg_color;
        border-radius: 12px;
        padding: 12px;
        border: 1px solid alpha(white, 0.08);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45);
    }
    popover > arrow {
        background-color: @popover_bg_color;
        border: 1px solid alpha(white, 0.08);
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
    .sidebar button:not(.zen-tab-close-badge) {
        border-radius: 8px;
        margin: 0;
        padding: 0;
        min-height: 36px;
        background: transparent;
        border: none;
        transition: background-color 150ms ease, color 150ms ease;
    }

    .sidebar button:not(.zen-tab-close-badge):hover {
        background-color: alpha(currentColor, 0.08);
    }

    .sidebar button:not(.zen-tab-close-badge):active {
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

    .tab-overlay-container {
        margin: 2px 4px;
    }

    .zen-tab-close-badge {
        opacity: 0;
        min-width: 16px;
        min-height: 16px;
        padding: 1px;
        margin-top: -3px;
        margin-left: -3px;
        border-radius: 9999px;
        background-color: #3a3d42;
        color: #ffffff;
        border: 1px solid rgba(255, 255, 255, 0.18);
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
        transition: opacity 140ms ease, transform 140ms ease, background-color 140ms ease;
    }

    .zen-tab-close-badge image {
        -gtk-icon-size: 10px;
    }

    /* Show the top-left close badge when hovering anywhere over the tab overlay */
    .tab-overlay-container:hover .zen-tab-close-badge {
        opacity: 1;
    }

    .zen-tab-close-badge:hover {
        background-color: #e01b24;
        color: #ffffff;
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

    /* Sliders styling: Hue continuous spectrum and Nord palette */
    .hue-slider trough {
        background-image: linear-gradient(to right,
            #ff0000 0%,
            #ffff00 17%,
            #00ff00 33%,
            #00ffff 50%,
            #0000ff 67%,
            #ff00ff 83%,
            #ff0000 100%
        );
        min-height: 12px;
        border-radius: 6px;
        border: 1px solid alpha(currentColor, 0.18);
    }
    .hue-slider highlight {
        background: transparent;
    }
    .hue-slider slider {
        min-width: 18px;
        min-height: 18px;
        margin: -3px 0;
        border-radius: 9999px;
    }

    .nord-slider trough,
    .scale-has-marks-below contents trough {
        background-image: linear-gradient(to right,
            #2E3440 0%, #4C566A 25%,
            #D8DEE9 26%, #ECEFF4 43%,
            #8FBCBB 44%, #5E81AC 68%,
            #BF616A 69%, #B48EAD 100%
        );
        min-height: 12px;
        border-radius: 6px;
        border: 1px solid alpha(currentColor, 0.18);
    }
    .nord-slider highlight {
        background: transparent;
    }
    .nord-slider slider {
        min-width: 18px;
        min-height: 18px;
        margin: -3px 0;
        border-radius: 9999px;
    }
    .nord-slider marks,
    .scale-has-marks-below marks {
        font-size: 10px;
        color: alpha(currentColor, 0.7);
        margin-top: 4px;
    }

    /* Circular Theme Swatches */
    button.theme-swatch {
        min-width: 34px;
        min-height: 34px;
        border-radius: 9999px;
        padding: 0;
        margin: 0 4px;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
        transition: transform 120ms ease, box-shadow 120ms ease;
    }
    button.theme-swatch:hover {
        transform: scale(1.08);
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
    }
    button.theme-swatch:checked {
        outline: 2px solid @accent_color;
        outline-offset: 2px;
    }
    button.theme-swatch image {
        opacity: 0;
        transition: opacity 120ms ease;
    }
    button.theme-swatch:checked image {
        opacity: 1;
    }

    /* System Swatch: Split circle (linear-gradient 45deg: white 50%, #2E3440 50%) */
    button.theme-swatch-system {
        background: linear-gradient(45deg, #ffffff 50%, #2E3440 50%) !important;
        border: 1px solid alpha(currentColor, 0.25) !important;
    }
    button.theme-swatch-system:checked image {
        color: #3584e4 !important;
    }

    /* Light Swatch: White circle */
    button.theme-swatch-light {
        background: #ffffff !important;
        border: 1px solid rgba(0, 0, 0, 0.25) !important;
    }
    button.theme-swatch-light:checked image {
        color: #2E3440 !important;
    }

    /* Custom / Nord Swatch: Cream / Nord Frost circle */
    button.theme-swatch-nord {
        background: linear-gradient(135deg, #88C0D0 50%, #81A1C1 50%) !important;
        border: 1px solid alpha(currentColor, 0.25) !important;
    }
    button.theme-swatch-nord:checked image {
        color: #2E3440 !important;
    }

    /* Dark Swatch: Dark circle (#2E3440) */
    button.theme-swatch-dark {
        background: #2E3440 !important;
        border: 1px solid rgba(255, 255, 255, 0.25) !important;
    }
    button.theme-swatch-dark:checked image {
        color: #ffffff !important;
    }

    /* Chrome/Apostrophe Style Menu Action Rows */
    button.menu-action-row {
        border-radius: 8px;
        padding: 6px 10px;
        min-height: 38px;
        background: transparent;
        border: none;
        box-shadow: none;
        transition: background-color 140ms ease;
    }
    button.menu-action-row:hover {
        background-color: alpha(currentColor, 0.08);
    }
    button.menu-action-row:active {
        background-color: alpha(currentColor, 0.14);
    }
    .menu-action-row .dim-label {
        opacity: 0.55;
        font-size: 12px;
        font-weight: 500;
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

    /* Immutable True-Black Incognito Mode */
    .incognito-window {
        background-color: #000000;
        color: #ffffff;
        --accent-bg-color: #1e1e1e;
        --accent-fg-color: #ffffff;
        --window-bg-color: #000000;
        --view-bg-color: #050505;
        --headerbar-bg-color: #0a0a0a;
    }

    .incognito-window headerbar {
        background-color: #0a0a0a;
        border-bottom: 1px solid #1a1a1a;
    }
    "#
}

/// Official brand color for Incognito window background (True-Black).
pub const INCOGNITO_WINDOW_COLOR: &str = "#000000";

/// Official brand color for Incognito corner icon.
pub const INCOGNITO_ICON_COLOR: &str = "#ffc95c";

/// Returns CSS rules specifically for the Incognito private browsing mode.
pub fn incognito_css() -> &'static str {
    r#"
    /* Vingilot Immutable True-Black Incognito Mode */
    .incognito-window,
    window.incognito,
    window.incognito-window {
        background-color: #000000 !important;
        color: #ffffff !important;
        --accent-bg-color: #1e1e1e !important;
        --accent-fg-color: #ffffff !important;
        --window-bg-color: #000000 !important;
        --view-bg-color: #050505 !important;
        --headerbar-bg-color: #0a0a0a !important;
    }

    .incognito-window headerbar,
    window.incognito headerbar,
    window.incognito-window headerbar,
    .incognito-window .header-bar {
        background-color: #0a0a0a !important;
        border-bottom: 1px solid #1a1a1a !important;
        box-shadow: none;
    }

    .incognito-window .sidebar,
    .incognito-window sidebar,
    window.incognito .sidebar,
    window.incognito sidebar {
        background-color: #000000 !important;
        border-right: 1px solid #1a1a1a !important;
    }

    .incognito-window paned,
    window.incognito paned {
        background-color: #000000 !important;
    }

    .incognito-window paned > :last-child,
    window.incognito paned > :last-child {
        background-color: #050505 !important;
        border-top: 1px solid #1a1a1a !important;
    }

    .incognito-window entry,
    .incognito-window .address-entry,
    window.incognito entry,
    window.incognito .address-entry {
        background-color: #0a0a0a !important;
        color: #ffffff !important;
        border: 1px solid #222222 !important;
    }

    .incognito-window .browser-chrome,
    window.incognito .browser-chrome {
        background-image: none !important;
        background-color: #0a0a0a !important;
    }

    .incognito-window .tab-row.active,
    .incognito-window .sidebar-tab.active,
    .incognito-window .sidebar-tab.suggested-action,
    .incognito-window button.suggested-action,
    window.incognito .tab-row.active,
    window.incognito .sidebar-tab.active,
    window.incognito .sidebar-tab.suggested-action,
    window.incognito button.suggested-action {
        background-color: #1e1e1e !important;
        color: #ffffff !important;
    }

    .incognito-window .start-page-container,
    window.incognito .start-page-container {
        background: #000000 !important;
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
        assert_eq!(INCOGNITO_WINDOW_COLOR, "#000000");
        assert_eq!(INCOGNITO_ICON_COLOR, "#ffc95c");
        let css = incognito_css();
        assert!(css.contains("#000000"));
        assert!(css.contains("#0a0a0a"));
        assert!(css.contains("#1e1e1e"));
        assert!(css.contains("#ffc95c"));
        assert!(css.contains(".incognito-window"));
    }

    #[test]
    fn test_base_css_contains_incognito_window_rules() {
        let css = base_css();
        assert!(css.contains(".incognito-window"));
        assert!(css.contains("--window-bg-color: #000000;"));
        assert!(css.contains("--accent-bg-color: #1e1e1e;"));
        assert!(css.contains("--headerbar-bg-color: #0a0a0a;"));
    }

    #[test]
    fn test_zen_close_badge_css() {
        let css = base_css();
        assert!(css.contains(".tab-overlay-container"));
        assert!(css.contains(".zen-tab-close-badge"));
        assert!(css.contains(".tab-overlay-container:hover .zen-tab-close-badge"));
        assert!(css.contains("opacity: 1"));
    }

    #[test]
    fn test_hue_and_nord_slider_css() {
        let css = base_css();
        assert!(css.contains(".hue-slider trough"));
        assert!(css.contains(".nord-slider trough"));
        assert!(css.contains(".scale-has-marks-below contents trough"));
        assert!(css.contains("#2E3440")); // Nord 0
        assert!(css.contains("#BF616A")); // Aurora Red
    }
}
