use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;
use vingilot_common::{AddDownloadRequest, DownloadTaskSummary, TaskStatus, TrayMenuItem, TrayState, TrayStatus};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/vingilot-downloadd.sock";

#[derive(Debug)]
pub struct EngineState {
    pub tasks: HashMap<String, DownloadTaskSummary>,
    pub download_dir: PathBuf,
}

impl EngineState {
    pub fn new(download_dir: PathBuf) -> Self {
        Self {
            tasks: HashMap::new(),
            download_dir,
        }
    }

    pub fn add_task(&mut self, req: AddDownloadRequest) -> DownloadTaskSummary {
        let id = Uuid::new_v4().to_string();
        let dest_dir = req
            .destination_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| self.download_dir.clone());

        let filename = req.filename.unwrap_or_else(|| {
            if req.url.starts_with("magnet:") {
                "torrent_stream".to_string()
            } else if let Ok(parsed) = url::Url::parse(&req.url) {
                parsed
                    .path_segments()
                    .and_then(|mut s| s.next_back())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("download.bin")
                    .to_string()
            } else {
                "download.bin".to_string()
            }
        });

        let dest_path = dest_dir.join(&filename).to_string_lossy().to_string();
        let task = DownloadTaskSummary::new(&id, req.url, filename, dest_path);
        self.tasks.insert(id.clone(), task.clone());
        task
    }

    pub fn list_tasks(&self) -> Vec<DownloadTaskSummary> {
        self.tasks.values().cloned().collect()
    }

    pub fn pause_task(&mut self, id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(id) {
            if task.status == TaskStatus::Downloading || task.status == TaskStatus::Pending {
                task.status = TaskStatus::Paused;
                task.speed_bytes_per_sec = 0;
                return true;
            }
        }
        false
    }

    pub fn resume_task(&mut self, id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(id) {
            if task.status == TaskStatus::Paused {
                task.status = TaskStatus::Downloading;
                return true;
            }
        }
        false
    }

    pub fn cancel_task(&mut self, id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(id) {
            task.status = TaskStatus::Cancelled;
            task.speed_bytes_per_sec = 0;
            return true;
        }
        false
    }

    pub fn generate_tray_state(&self) -> TrayState {
        let mut tray = TrayState::new("Vingilot Download Manager", "io.github.antux912012.Vingilot.downloadd");
        let active_tasks: Vec<_> = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Downloading)
            .collect();

        if active_tasks.is_empty() {
            tray.status = TrayStatus::Idle;
            tray.tooltip = "Vingilot Download Manager (Idle)".to_string();
            tray.add_item(TrayMenuItem::action("status", "No Active Downloads"));
        } else {
            tray.status = TrayStatus::Running;
            let total_speed: u64 = active_tasks.iter().map(|t| t.speed_bytes_per_sec).sum();
            let speed_mb = (total_speed as f64) / (1024.0 * 1024.0);
            tray.tooltip = format!("Downloading {} items ({:.1} MB/s)", active_tasks.len(), speed_mb);

            for t in active_tasks.iter().take(3) {
                let label = format!("[{}%] {} ({:.1} MB/s)", t.progress_pct, t.filename, (t.speed_bytes_per_sec as f64) / 1048576.0);
                tray.add_item(TrayMenuItem::action(format!("task_{}", t.id), label));
            }
        }

        tray.add_item(TrayMenuItem::separator());
        tray.add_item(TrayMenuItem::action("pause_all", "Pause All Downloads"));
        tray.add_item(TrayMenuItem::action("resume_all", "Resume All Downloads"));
        tray.add_item(TrayMenuItem::separator());
        tray.add_item(TrayMenuItem::action("quit", "Quit Download Manager"));

        tray
    }
}

pub struct DownloadDaemon {
    pub state: Arc<Mutex<EngineState>>,
    socket_path: PathBuf,
}

impl DownloadDaemon {
    pub fn new(socket_path: impl AsRef<Path>, download_dir: impl AsRef<Path>) -> Self {
        Self {
            state: Arc::new(Mutex::new(EngineState::new(download_dir.as_ref().to_path_buf()))),
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    pub fn run(&self) -> std::io::Result<()> {
        if self.socket_path.exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        println!(
            "[vingilot-downloadd] Daemon listening on Unix socket: {}",
            self.socket_path.display()
        );

        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let state_clone = Arc::clone(&self.state);
                    thread::spawn(move || {
                        let _ = handle_client(s, state_clone);
                    });
                }
                Err(e) => {
                    eprintln!("[vingilot-downloadd] Socket accept error: {e}");
                    break;
                }
            }
        }

        Ok(())
    }
}

fn handle_client(mut stream: UnixStream, state: Arc<Mutex<EngineState>>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let response = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
        let cmd = val["command"].as_str().unwrap_or("");
        match cmd {
            "add" => {
                let url = val["url"].as_str().unwrap_or("").to_string();
                let dest = val["dest"].as_str().map(|s| s.to_string());
                let filename = val["filename"].as_str().map(|s| s.to_string());
                let mut lock = state.lock().unwrap();
                let task = lock.add_task(AddDownloadRequest {
                    url,
                    destination_dir: dest,
                    filename,
                });
                serde_json::to_string(&task).unwrap_or_else(|_| "{}".to_string())
            }
            "list" => {
                let lock = state.lock().unwrap();
                let list = lock.list_tasks();
                serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())
            }
            "pause" => {
                let id = val["id"].as_str().unwrap_or("");
                let mut lock = state.lock().unwrap();
                let ok = lock.pause_task(id);
                format!(r#"{{"success": {}}}"#, ok)
            }
            "resume" => {
                let id = val["id"].as_str().unwrap_or("");
                let mut lock = state.lock().unwrap();
                let ok = lock.resume_task(id);
                format!(r#"{{"success": {}}}"#, ok)
            }
            "cancel" => {
                let id = val["id"].as_str().unwrap_or("");
                let mut lock = state.lock().unwrap();
                let ok = lock.cancel_task(id);
                format!(r#"{{"success": {}}}"#, ok)
            }
            "tray_state" => {
                let lock = state.lock().unwrap();
                let tray = lock.generate_tray_state();
                serde_json::to_string(&tray).unwrap_or_else(|_| "{}".to_string())
            }
            _ => r#"{"error": "Unknown command"}"#.to_string(),
        }
    } else {
        r#"{"error": "Invalid JSON"}"#.to_string()
    };

    stream.write_all(response.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

pub fn send_ipc_command(socket_path: impl AsRef<Path>, cmd_json: &serde_json::Value) -> Result<String, String> {
    let mut stream = UnixStream::connect(socket_path.as_ref())
        .map_err(|e| format!("Could not connect to daemon socket: {e}"))?;

    let req_str = cmd_json.to_string();
    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Write failed: {e}"))?;
    stream.write_all(b"\n").map_err(|e| format!("Flush failed: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut res = String::new();
    reader.read_line(&mut res).map_err(|e| format!("Read failed: {e}"))?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_state_task_lifecycle() {
        let temp = tempfile::tempdir().unwrap();
        let mut state = EngineState::new(temp.path().to_path_buf());

        let task = state.add_task(AddDownloadRequest {
            url: "https://example.com/archive.zip".to_string(),
            destination_dir: None,
            filename: None,
        });

        assert_eq!(task.filename, "archive.zip");
        assert_eq!(task.status, TaskStatus::Pending);

        // Update to downloading
        if let Some(t) = state.tasks.get_mut(&task.id) {
            t.status = TaskStatus::Downloading;
            t.update_progress(500, Some(1000), 524288);
        }

        let tray = state.generate_tray_state();
        assert_eq!(tray.status, TrayStatus::Running);
        assert!(tray.items.iter().any(|item| item.label.contains("archive.zip")));

        // Pause task
        assert!(state.pause_task(&task.id));
        assert_eq!(state.tasks.get(&task.id).unwrap().status, TaskStatus::Paused);

        // Resume task
        assert!(state.resume_task(&task.id));
        assert_eq!(state.tasks.get(&task.id).unwrap().status, TaskStatus::Downloading);

        // Cancel task
        assert!(state.cancel_task(&task.id));
        assert_eq!(state.tasks.get(&task.id).unwrap().status, TaskStatus::Cancelled);
    }
}
