use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Status of an active or finished download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Downloading {
        received_bytes: u64,
        total_bytes: Option<u64>,
    },
    Completed {
        destination: PathBuf,
    },
    Failed {
        error_message: String,
    },
    Cancelled,
}

/// An individual file download operation.
#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub id: Uuid,
    pub source_url: String,
    pub destination_path: PathBuf,
    pub status: DownloadStatus,
}

impl DownloadItem {
    pub fn progress_percentage(&self) -> Option<u32> {
        match &self.status {
            DownloadStatus::Downloading {
                received_bytes,
                total_bytes,
            } => total_bytes.and_then(|total| {
                (received_bytes * 100)
                    .checked_div(total)
                    .map(|pct| pct.min(100) as u32)
            }),
            DownloadStatus::Completed { .. } => Some(100),
            _ => None,
        }
    }
}

/// Download manager tracking active and completed downloads.
#[derive(Debug, Clone)]
pub struct DownloadManager {
    download_dir: PathBuf,
    items: Vec<DownloadItem>,
}

impl DownloadManager {
    pub fn new(download_dir: impl AsRef<Path>) -> Self {
        Self {
            download_dir: download_dir.as_ref().to_path_buf(),
            items: Vec::new(),
        }
    }

    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }

    pub fn items(&self) -> &[DownloadItem] {
        &self.items
    }

    /// Starts tracking a new download.
    pub fn start_download(&mut self, url: &str, filename: &str) -> Uuid {
        let id = Uuid::new_v4();
        let destination = self.download_dir.join(filename);
        let item = DownloadItem {
            id,
            source_url: url.to_string(),
            destination_path: destination,
            status: DownloadStatus::Pending,
        };
        self.items.push(item);
        id
    }

    /// Updates download byte progress.
    pub fn update_progress(&mut self, id: Uuid, received: u64, total: Option<u64>) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = DownloadStatus::Downloading {
                received_bytes: received,
                total_bytes: total,
            };
        }
    }

    /// Marks a download as completed.
    pub fn mark_completed(&mut self, id: Uuid) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = DownloadStatus::Completed {
                destination: item.destination_path.clone(),
            };
        }
    }

    /// Cancels an in-flight download.
    pub fn cancel(&mut self, id: Uuid) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            match item.status {
                DownloadStatus::Downloading { .. } | DownloadStatus::Pending => {
                    item.status = DownloadStatus::Cancelled;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_lifecycle() {
        let mut dm = DownloadManager::new("/tmp/downloads");
        let id = dm.start_download("https://example.com/archive.zip", "archive.zip");
        assert_eq!(dm.items().len(), 1);

        dm.update_progress(id, 50, Some(100));
        assert_eq!(dm.items()[0].progress_percentage(), Some(50));

        dm.mark_completed(id);
        assert_eq!(dm.items()[0].progress_percentage(), Some(100));
        assert!(matches!(
            dm.items()[0].status,
            DownloadStatus::Completed { .. }
        ));
    }

    #[test]
    fn test_download_cancellation() {
        let mut dm = DownloadManager::new("/tmp/downloads");
        let id = dm.start_download("https://example.com/large.iso", "large.iso");
        assert!(dm.cancel(id));
        assert_eq!(dm.items()[0].status, DownloadStatus::Cancelled);
    }
}
