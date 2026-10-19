use serde::{Deserialize, Serialize};

/// Task status for the standalone download daemon (vingilot-downloadd).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Download task descriptor shared over D-Bus (`org.vingilot.DownloadManager`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadTaskSummary {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination: String,
    pub total_bytes: Option<u64>,
    pub received_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub status: TaskStatus,
    pub progress_pct: u32,
    pub is_torrent: bool,
}

impl DownloadTaskSummary {
    pub fn new(
        id: impl Into<String>,
        url: impl Into<String>,
        filename: impl Into<String>,
        destination: impl Into<String>,
    ) -> Self {
        let u = url.into();
        let is_torrent = u.starts_with("magnet:") || u.ends_with(".torrent");
        Self {
            id: id.into(),
            url: u,
            filename: filename.into(),
            destination: destination.into(),
            total_bytes: None,
            received_bytes: 0,
            speed_bytes_per_sec: 0,
            status: TaskStatus::Pending,
            progress_pct: 0,
            is_torrent,
        }
    }

    pub fn update_progress(&mut self, received: u64, total: Option<u64>, speed: u64) {
        self.received_bytes = received;
        self.total_bytes = total;
        self.speed_bytes_per_sec = speed;

        if let Some(pct) = total.and_then(|tot| (received.saturating_mul(100)).checked_div(tot)) {
            self.progress_pct = pct.min(100) as u32;
        }
    }
}

/// Request to enqueue a new download over IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddDownloadRequest {
    pub url: String,
    pub destination_dir: Option<String>,
    pub filename: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_task_summary_progress() {
        let mut task = DownloadTaskSummary::new(
            "task-1",
            "https://example.com/file.tar.gz",
            "file.tar.gz",
            "/home/user/Downloads",
        );
        assert!(!task.is_torrent);
        assert_eq!(task.status, TaskStatus::Pending);

        task.update_progress(50, Some(100), 1024);
        assert_eq!(task.progress_pct, 50);
        assert_eq!(task.speed_bytes_per_sec, 1024);
    }

    #[test]
    fn test_magnet_link_detection() {
        let task = DownloadTaskSummary::new(
            "task-2",
            "magnet:?xt=urn:btih:da39a3ee5e6b4b0d3255bfef95601890afd80709",
            "torrent_download",
            "/home/user/Downloads",
        );
        assert!(task.is_torrent);
    }
}
