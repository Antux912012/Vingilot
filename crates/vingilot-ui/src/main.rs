pub mod ai_panel;
pub mod bookmark_panel;
pub mod browser_import_dialog;
pub mod export_wizard;
pub mod history_panel;
pub mod import_wizard;
pub mod layout;
pub mod profile_switcher;
pub mod reader_view;
pub mod settings_panel;
pub mod sidebar;
pub mod tabs;
pub mod theme;
pub mod window;

pub use ai_panel::{AiAssistantPanel, AiChatEntry};
pub use bookmark_panel::BookmarkPanel;
pub use browser_import_dialog::{BrowserImportDialog, ImportSourceType};
pub use export_wizard::ExportWizard;
pub use history_panel::HistoryPanel;
pub use import_wizard::ImportWizard;
pub use layout::{AdaptiveLayoutManager, LayoutMode};
pub use profile_switcher::ProfileSwitcher;
pub use reader_view::{ReaderTheme, ReaderView};
pub use settings_panel::SettingsPanel;
pub use sidebar::{SideTabSidebar, SidebarTabItem};
pub use tabs::{Tab, TabManager};
pub use theme::{AppTheme, ThemeManager};
pub use window::VingilotWindow;

fn main() {
    println!("Launching Project Vingilot (Libadwaita / WebKitGTK)");
    let mut win = VingilotWindow::default();
    println!("Application ID: {}", win.app_id());
    println!("Title: {}", win.title());
    println!("Active tabs: {}", win.tab_manager().tabs().len());

    // Check if launched with --incognito flag ("Navigating Shadowy Seas")
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--incognito" || arg == "-i") {
        println!("Launching in incognito mode: Navigating Shadowy Seas");
        win.open_new_incognito_tab();
    }
}
