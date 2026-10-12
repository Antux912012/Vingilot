use uuid::Uuid;
use vingilot_core::models::BookmarkNode;

/// UI panel for managing bookmarks and hierarchical folders.
#[derive(Debug, Clone, Default)]
pub struct BookmarkPanel {
    selected_node_id: Option<Uuid>,
    is_open: bool,
}

impl BookmarkPanel {
    pub fn new() -> Self {
        Self::default()
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

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn select_node(&mut self, id: Uuid) {
        self.selected_node_id = Some(id);
    }

    pub fn selected_node_id(&self) -> Option<Uuid> {
        self.selected_node_id
    }

    /// Renders a flattened view of the bookmark tree for UI display with indentation levels.
    pub fn flatten_tree<'a>(&self, nodes: &'a [BookmarkNode]) -> Vec<BookmarkItemView<'a>> {
        let mut views = Vec::new();
        for node in nodes {
            self.flatten_recursive(node, 0, &mut views);
        }
        views
    }

    fn flatten_recursive<'a>(
        &self,
        node: &'a BookmarkNode,
        depth: usize,
        out: &mut Vec<BookmarkItemView<'a>>,
    ) {
        out.push(BookmarkItemView {
            id: node.id,
            title: &node.title,
            url: node.url.as_deref(),
            is_folder: node.is_folder,
            depth,
            is_selected: self.selected_node_id == Some(node.id),
        });

        for child in &node.children {
            self.flatten_recursive(child, depth + 1, out);
        }
    }
}

/// View model item for displaying bookmarks in GTK list/tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkItemView<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub url: Option<&'a str>,
    pub is_folder: bool,
    pub depth: usize,
    pub is_selected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_bookmark_panel_flattening() {
        let child = BookmarkNode {
            id: Uuid::new_v4(),
            parent_id: None,
            title: "Child".to_string(),
            url: Some("https://child.org".to_string()),
            is_folder: false,
            children: Vec::new(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let parent = BookmarkNode {
            id: Uuid::new_v4(),
            parent_id: None,
            title: "Parent Folder".to_string(),
            url: None,
            is_folder: true,
            children: vec![child],
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut panel = BookmarkPanel::new();
        panel.select_node(parent.id);
        let tree = [parent];
        let items = panel.flatten_tree(&tree);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Parent Folder");
        assert_eq!(items[0].depth, 0);
        assert!(items[0].is_selected);

        assert_eq!(items[1].title, "Child");
        assert_eq!(items[1].depth, 1);
        assert!(!items[1].is_selected);
    }
}
