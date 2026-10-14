pub mod ai_panel;
pub mod bookmark_panel;
pub mod browser_import_dialog;
pub mod export_wizard;
pub mod gui;
pub mod history_panel;
pub mod import_wizard;
pub mod layout;
pub mod profile_switcher;
pub mod reader_view;
pub mod settings_panel;
pub mod sidebar;
pub mod start_page;
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
pub use start_page::{BookmarkTileItem, RecentCardItem, StartPageModel};
pub use tabs::{Tab, TabManager};
pub use theme::{AppTheme, ThemeManager};
pub use window::VingilotWindow;

fn main() {
    if std::env::var("GSK_RENDERER").is_err() {
        std::env::set_var("GSK_RENDERER", "gl");
    }
    println!("Launching Project Vingilot (Libadwaita / WebKitGTK)");
    let args: Vec<String> = std::env::args().collect();
    let is_incognito = args.iter().any(|arg| arg == "--incognito" || arg == "-i");

    let initial_url = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .cloned();

    let mut win = VingilotWindow::default();
    println!("Application ID: {}", win.app_id());
    println!("Title: {}", win.title());
    println!("Active tabs: {}", win.tab_manager().tabs().len());

    if is_incognito {
        println!("Launching in incognito mode");
        win.open_new_incognito_tab();
    }

    // Launch native Libadwaita GUI if display server is present
    let launched = gui::run_gui(is_incognito, initial_url);
    if !launched {
        println!("Exiting headless runner (no interactive display or GTK libraries).");
    }
}
