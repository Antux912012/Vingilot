use serde::{Deserialize, Serialize};

/// System tray item execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayStatus {
    Idle,
    Running,
    Busy,
    Error,
}

/// Generalized tray menu item descriptor used across Vingilot daemons and YT Extractor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
    pub is_separator: bool,
}

impl TrayMenuItem {
    pub fn action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
            checked: false,
            is_separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            enabled: false,
            checked: false,
            is_separator: true,
        }
    }
}

/// Unified system tray representation for D-Bus StatusNotifierItem and AppIndicator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrayState {
    pub title: String,
    pub tooltip: String,
    pub icon_name: String,
    pub status: TrayStatus,
    pub items: Vec<TrayMenuItem>,
}

impl TrayState {
    pub fn new(title: impl Into<String>, icon_name: impl Into<String>) -> Self {
        let t = title.into();
        Self {
            tooltip: t.clone(),
            title: t,
            icon_name: icon_name.into(),
            status: TrayStatus::Idle,
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: TrayMenuItem) {
        self.items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_construction() {
        let mut tray = TrayState::new("Vingilot Service", "io.github.antux912012.Vingilot");
        tray.add_item(TrayMenuItem::action("open", "Open Dashboard"));
        tray.add_item(TrayMenuItem::separator());
        tray.add_item(TrayMenuItem::action("quit", "Quit"));

        assert_eq!(tray.items.len(), 3);
        assert!(!tray.items[0].is_separator);
        assert!(tray.items[1].is_separator);
        assert_eq!(tray.items[2].id, "quit");
    }
}
