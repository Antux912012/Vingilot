use crate::sidebar::SideTabSidebar;

/// Layout mode for the adaptive Libadwaita window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Desktop wide view: persistent side tab sidebar.
    WideDesktop,
    /// Mobile/narrow view: collapsed or overlay drawer sidebar.
    NarrowMobile,
}

/// Adaptive layout manager handling screen resizing, breakpoints, and kinetic scrolling.
#[derive(Debug, Clone)]
pub struct AdaptiveLayoutManager {
    pub current_width: u32,
    pub current_height: u32,
    pub breakpoint_width: u32,
    pub kinetic_scrolling: bool,
    pub layout_mode: LayoutMode,
}

impl Default for AdaptiveLayoutManager {
    fn default() -> Self {
        Self::new(1024, 768)
    }
}

impl AdaptiveLayoutManager {
    pub fn new(width: u32, height: u32) -> Self {
        let breakpoint = 720;
        let mode = if width >= breakpoint {
            LayoutMode::WideDesktop
        } else {
            LayoutMode::NarrowMobile
        };

        Self {
            current_width: width,
            current_height: height,
            breakpoint_width: breakpoint,
            kinetic_scrolling: true,
            layout_mode: mode,
        }
    }

    /// Updates window dimensions and recalculates adaptive layout mode.
    pub fn update_dimensions(&mut self, width: u32, height: u32, sidebar: &mut SideTabSidebar) {
        self.current_width = width;
        self.current_height = height;

        if width < self.breakpoint_width {
            self.layout_mode = LayoutMode::NarrowMobile;
            sidebar.collapse();
        } else {
            self.layout_mode = LayoutMode::WideDesktop;
            sidebar.expand();
        }
    }

    /// Whether kinetic scrolling should be active for scrollable areas.
    pub fn is_kinetic_scrolling_enabled(&self) -> bool {
        self.kinetic_scrolling
    }

    pub fn set_kinetic_scrolling(&mut self, enabled: bool) {
        self.kinetic_scrolling = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_layout_breakpoints() {
        let mut layout = AdaptiveLayoutManager::new(1280, 800);
        let mut sidebar = SideTabSidebar::new();
        assert_eq!(layout.layout_mode, LayoutMode::WideDesktop);
        assert!(!sidebar.is_collapsed);

        // Resize down to mobile size
        layout.update_dimensions(480, 800, &mut sidebar);
        assert_eq!(layout.layout_mode, LayoutMode::NarrowMobile);
        assert!(sidebar.is_collapsed);

        // Resize back up to desktop
        layout.update_dimensions(1024, 768, &mut sidebar);
        assert_eq!(layout.layout_mode, LayoutMode::WideDesktop);
        assert!(!sidebar.is_collapsed);
    }
}
