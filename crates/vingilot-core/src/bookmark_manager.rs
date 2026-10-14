use chrono::Utc;
use uuid::Uuid;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::{BookmarkBreadcrumb, BookmarkNode};
use crate::repositories::BookmarkRepository;

/// Service for managing bookmark tree structures, folders, moving, and reordering.
pub struct BookmarkManager<'a> {
    repo: BookmarkRepository<'a>,
}

/// Alias for BookmarkManager as per service layer specification.
pub type BookmarkStore<'a> = BookmarkManager<'a>;

impl<'a> BookmarkManager<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: BookmarkRepository::new(db),
        }
    }

    /// Returns the items located in the root bookmark level.
    pub fn get_root_items(&self) -> CoreResult<Vec<BookmarkNode>> {
        self.repo.list_by_parent(None)
    }

    /// Returns the direct child items inside a folder.
    pub fn get_children(&self, folder_id: Uuid) -> CoreResult<Vec<BookmarkNode>> {
        self.repo.list_by_parent(Some(folder_id))
    }

    /// Creates a bookmark item.
    pub fn add_bookmark(
        &self,
        parent_id: Option<Uuid>,
        title: &str,
        url: &str,
    ) -> CoreResult<BookmarkNode> {
        let now = Utc::now();
        let position = self.repo.count_children(parent_id)? as i32;

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

    /// Creates a bookmark item (alias for service API).
    pub fn create_bookmark(
        &self,
        title: &str,
        url: &str,
        parent_id: Option<Uuid>,
    ) -> CoreResult<BookmarkNode> {
        self.add_bookmark(parent_id, title, url)
    }

    /// Creates a folder in the bookmark tree.
    pub fn add_folder(&self, parent_id: Option<Uuid>, title: &str) -> CoreResult<BookmarkNode> {
        let now = Utc::now();
        let position = self.repo.count_children(parent_id)? as i32;

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

    /// Creates a folder in the bookmark tree (alias for service API).
    pub fn create_folder(&self, title: &str, parent_id: Option<Uuid>) -> CoreResult<BookmarkNode> {
        self.add_folder(parent_id, title)
    }

    /// Renames a folder or bookmark item.
    pub fn rename_item(&self, item_id: Uuid, new_title: &str) -> CoreResult<bool> {
        self.repo.rename(item_id, new_title)
    }

    /// Updates an existing bookmark's title and URL.
    pub fn update_bookmark(
        &self,
        item_id: Uuid,
        new_title: &str,
        new_url: &str,
    ) -> CoreResult<bool> {
        self.repo.update_bookmark(item_id, new_title, new_url)
    }

    /// Searches bookmarks and folders matching the query string.
    pub fn search(&self, query: &str) -> CoreResult<Vec<BookmarkNode>> {
        self.repo.search(query)
    }

    /// Counts all descendants inside a folder.
    pub fn count_descendants(&self, folder_id: Uuid) -> CoreResult<usize> {
        self.repo.count_descendants(folder_id)
    }

    /// Collects all URLs contained inside a folder and its subfolders.
    pub fn list_all_urls_in_folder(&self, folder_id: Uuid) -> CoreResult<Vec<String>> {
        self.repo.list_all_urls_in_folder(folder_id)
    }

    /// Deletes a folder and all its contents recursively.
    pub fn delete_folder_recursive(&self, folder_id: Uuid) -> CoreResult<usize> {
        self.repo.delete_folder_recursive(folder_id)
    }

    /// Deletes a folder or bookmark item.
    pub fn delete_item(&self, item_id: Uuid) -> CoreResult<bool> {
        self.repo.delete(item_id)
    }

    /// Moves a bookmark or folder into a new parent folder or to root.
    /// Cycle-prevention prevents moving a folder into itself or into its descendants.
    pub fn move_item(
        &self,
        item_id: Uuid,
        new_parent_id: Option<Uuid>,
        position: Option<i32>,
    ) -> CoreResult<bool> {
        let item = match self.repo.get_by_id(item_id)? {
            Some(i) => i,
            None => return Ok(false),
        };

        // Prevent invalid cycle moves for folders
        if let Some(target_parent) = new_parent_id {
            if item.is_folder {
                if target_parent == item_id {
                    return Ok(false);
                }
                if self.is_descendant(item_id, target_parent)? {
                    return Ok(false);
                }
            }
        }

        let pos = match position {
            Some(p) => p,
            None => self.repo.count_children(new_parent_id)? as i32,
        };

        self.repo.move_node(item_id, new_parent_id, pos)
    }

    /// Moves an item into a destination folder.
    pub fn move_item_to_folder(&self, item_id: Uuid, folder_id: Uuid) -> CoreResult<bool> {
        self.move_item(item_id, Some(folder_id), None)
    }

    /// Moves an item out to the root bookmark area.
    pub fn move_item_to_root(&self, item_id: Uuid) -> CoreResult<bool> {
        self.move_item(item_id, None, None)
    }

    /// Checks if `possible_descendant_id` is inside `ancestor_id`.
    pub fn is_descendant(
        &self,
        ancestor_id: Uuid,
        possible_descendant_id: Uuid,
    ) -> CoreResult<bool> {
        let mut curr = Some(possible_descendant_id);
        while let Some(id) = curr {
            if id == ancestor_id {
                return Ok(true);
            }
            if let Some(node) = self.repo.get_by_id(id)? {
                curr = node.parent_id;
            } else {
                break;
            }
        }
        Ok(false)
    }

    /// Builds the breadcrumb trail from root to the specified folder.
    pub fn get_breadcrumb_path(
        &self,
        folder_id: Option<Uuid>,
    ) -> CoreResult<Vec<BookmarkBreadcrumb>> {
        let mut crumbs = Vec::new();
        let mut curr = folder_id;
        while let Some(id) = curr {
            if let Some(node) = self.repo.get_by_id(id)? {
                crumbs.push(BookmarkBreadcrumb {
                    id: Some(node.id),
                    title: node.title,
                });
                curr = node.parent_id;
            } else {
                break;
            }
        }
        crumbs.reverse();
        crumbs.insert(
            0,
            BookmarkBreadcrumb {
                id: None,
                title: "Bookmarks".to_string(),
            },
        );
        Ok(crumbs)
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

    #[test]
    fn test_bookmark_move_rename_and_breadcrumbs() {
        let db = Database::in_memory().unwrap();
        let store = BookmarkStore::new(&db);

        let work = store.create_folder("Work", None).unwrap();
        let proj = store.create_folder("Projects", Some(work.id)).unwrap();
        let bm = store
            .create_bookmark("GitHub", "https://github.com", None)
            .unwrap();

        // Check root items
        let root_items = store.get_root_items().unwrap();
        assert_eq!(root_items.len(), 2); // Work folder and GitHub bookmark
        assert!(root_items[0].is_folder); // Folders first!

        // Move bookmark into Projects folder
        assert!(store.move_item_to_folder(bm.id, proj.id).unwrap());
        let proj_children = store.get_children(proj.id).unwrap();
        assert_eq!(proj_children.len(), 1);
        assert_eq!(proj_children[0].id, bm.id);

        // Breadcrumb check
        let crumbs = store.get_breadcrumb_path(Some(proj.id)).unwrap();
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[0].title, "Bookmarks");
        assert_eq!(crumbs[1].title, "Work");
        assert_eq!(crumbs[2].title, "Projects");

        // Move back to root
        assert!(store.move_item_to_root(bm.id).unwrap());
        assert_eq!(store.get_children(proj.id).unwrap().len(), 0);
        assert_eq!(store.get_root_items().unwrap().len(), 2);

        // Rename
        assert!(store.rename_item(bm.id, "GitHub Code").unwrap());
        let updated = store.get_bookmark(bm.id).unwrap().unwrap();
        assert_eq!(updated.title, "GitHub Code");

        // Cycle prevention: cannot move Work into Projects (its child)
        assert!(!store.move_item_to_folder(work.id, proj.id).unwrap());
        // Cannot move Work into Work
        assert!(!store.move_item_to_folder(work.id, work.id).unwrap());

        // Delete item
        assert!(store.delete_item(bm.id).unwrap());
        assert!(store.get_bookmark(bm.id).unwrap().is_none());
    }

    #[test]
    fn test_bookmark_update_search_and_recursive_delete() {
        let db = Database::in_memory().unwrap();
        let store = BookmarkStore::new(&db);

        let folder = store.create_folder("Reading List", None).unwrap();
        let subfolder = store.create_folder("Articles", Some(folder.id)).unwrap();
        let bm1 = store
            .create_bookmark("Mozilla", "https://mozilla.org", Some(folder.id))
            .unwrap();
        let bm2 = store
            .create_bookmark(
                "Rust Book",
                "https://doc.rust-lang.org/book",
                Some(subfolder.id),
            )
            .unwrap();

        // Count descendants
        assert_eq!(store.count_descendants(folder.id).unwrap(), 3); // subfolder, bm1, bm2
        assert_eq!(store.count_descendants(subfolder.id).unwrap(), 1); // bm2

        // List URLs in folder
        let urls = store.list_all_urls_in_folder(folder.id).unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://mozilla.org".to_string()));
        assert!(urls.contains(&"https://doc.rust-lang.org/book".to_string()));

        // Update bookmark
        assert!(store
            .update_bookmark(bm1.id, "Mozilla Foundation", "https://www.mozilla.org")
            .unwrap());
        let updated = store.get_bookmark(bm1.id).unwrap().unwrap();
        assert_eq!(updated.title, "Mozilla Foundation");
        assert_eq!(updated.url.as_deref(), Some("https://www.mozilla.org"));

        // Search bookmarks
        let search_res = store.search("Mozilla").unwrap();
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].id, bm1.id);

        let search_book = store.search("book").unwrap();
        assert_eq!(search_book.len(), 1);
        assert_eq!(search_book[0].id, bm2.id);

        // Recursive delete folder
        let deleted_count = store.delete_folder_recursive(folder.id).unwrap();
        assert_eq!(deleted_count, 4); // folder + subfolder + bm1 + bm2
        assert_eq!(store.get_root_items().unwrap().len(), 0);
        assert!(store.get_bookmark(bm1.id).unwrap().is_none());
        assert!(store.get_bookmark(bm2.id).unwrap().is_none());
    }
}
