use chrono::Utc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::BookmarkNode;
use crate::repositories::BookmarkRepository;

/// Service for managing bookmark tree structures, folders, moving, and reordering.
pub struct BookmarkManager<'a> {
    repo: BookmarkRepository<'a>,
}

impl<'a> BookmarkManager<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: BookmarkRepository::new(db),
        }
    }

    /// Creates a bookmark item.
    pub fn add_bookmark(
        &self,
        parent_id: Option<Uuid>,
        title: &str,
        url: &str,
    ) -> CoreResult<BookmarkNode> {
        let now = Utc::now();
        let siblings = self.repo.list_by_parent(parent_id)?;
        let position = siblings.len() as i32;

        let node = BookmarkNode {
            id: Uuid::new_v4(),
            parent_id,
            title: title.to_string(),
            url: Some(url.to_string()),
            is_folder: false,
            children: Vec::new(),
            position,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&node)?;
        Ok(node)
    }

    /// Creates a folder in the bookmark tree.
    pub fn add_folder(&self, parent_id: Option<Uuid>, title: &str) -> CoreResult<BookmarkNode> {
        let now = Utc::now();
        let siblings = self.repo.list_by_parent(parent_id)?;
        let position = siblings.len() as i32;

        let node = BookmarkNode {
            id: Uuid::new_v4(),
            parent_id,
            title: title.to_string(),
            url: None,
            is_folder: true,
            children: Vec::new(),
            position,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&node)?;
        Ok(node)
    }

    /// Returns the complete bookmark tree starting from the root folders/bookmarks.
    pub fn get_tree(&self) -> CoreResult<Vec<BookmarkNode>> {
        self.build_subtree(None)
    }

    fn build_subtree(&self, parent_id: Option<Uuid>) -> CoreResult<Vec<BookmarkNode>> {
        let mut nodes = self.repo.list_by_parent(parent_id)?;
        for node in &mut nodes {
            if node.is_folder {
                node.children = self.build_subtree(Some(node.id))?;
            }
        }
        Ok(nodes)
    }

    pub fn delete_bookmark(&self, id: Uuid) -> CoreResult<bool> {
        self.repo.delete(id)
    }

    pub fn get_bookmark(&self, id: Uuid) -> CoreResult<Option<BookmarkNode>> {
        self.repo.get_by_id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_tree_construction() {
        let db = Database::in_memory().unwrap();
        let mgr = BookmarkManager::new(&db);

        let folder = mgr.add_folder(None, "Tech").unwrap();
        let _bookmark1 = mgr
            .add_bookmark(Some(folder.id), "Rust", "https://rust-lang.org")
            .unwrap();
        let _bookmark2 = mgr
            .add_bookmark(None, "DuckDuckGo", "https://duckduckgo.com")
            .unwrap();

        let tree = mgr.get_tree().unwrap();
        assert_eq!(tree.len(), 2); // 1 folder and 1 root bookmark
        let tech_folder = tree.iter().find(|n| n.title == "Tech").unwrap();
        assert_eq!(tech_folder.children.len(), 1);
        assert_eq!(tech_folder.children[0].title, "Rust");
    }
}
