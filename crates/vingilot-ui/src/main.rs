pub mod bookmark_panel;
pub mod history_panel;
pub mod layout;
pub mod profile_switcher;
pub mod settings_panel;
pub mod sidebar;
pub mod tabs;
pub mod window;

pub use bookmark_panel::BookmarkPanel;
pub use history_panel::HistoryPanel;
pub use layout::{AdaptiveLayoutManager, LayoutMode};
pub use profile_switcher::ProfileSwitcher;
pub use settings_panel::SettingsPanel;
pub use sidebar::{SideTabSidebar, SidebarTabItem};
pub use tabs::{Tab, TabManager};
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
