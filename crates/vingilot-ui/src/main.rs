pub mod window;

use vingilot_engine::BrowserEngine;
pub use window::VingilotWindow;

fn main() {
    println!("Launching Project Vingilot (Libadwaita / WebKitGTK)");
    let mut win = VingilotWindow::default();
    println!("Application ID: {}", win.app_id());
    println!("Title: {}", win.title());

    // Navigate to default welcome screen / home
    if let Err(e) = win.engine_mut().load_url("about:blank") {
        eprintln!("Failed to load initial page: {e}");
    }
}
