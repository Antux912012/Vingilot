use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod engine;
use engine::{send_ipc_command, DownloadDaemon, DEFAULT_SOCKET_PATH};

#[derive(Parser, Debug)]
#[command(
    name = "vingilot-downloadd",
    author = "Vingilot Team",
    version = "0.1.6",
    about = "Standalone background download daemon (HTTP multi-part + Torrent/Magnet + Tray IPC) for Vingilot ecosystem"
)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_SOCKET_PATH)]
    socket: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the background download daemon
    Daemon {
        #[arg(short, long)]
        download_dir: Option<PathBuf>,
    },
    /// Enqueue a new download (HTTP/HTTPS URL or Magnet link)
    Add {
        url: String,
        #[arg(short, long)]
        dest: Option<PathBuf>,
        #[arg(short, long)]
        filename: Option<String>,
    },
    /// List all tracked downloads
    List,
    /// Pause a specific download
    Pause { id: String },
    /// Resume a specific download
    Resume { id: String },
    /// Cancel a download
    Cancel { id: String },
    /// Query current tray menu state
    TrayState,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { download_dir } => {
            let dir = download_dir.unwrap_or_else(|| {
                dirs::download_dir().unwrap_or_else(|| PathBuf::from("./downloads"))
            });
            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            let daemon = DownloadDaemon::new(&cli.socket, dir);
            daemon.run()?;
        }
        Commands::Add {
            url,
            dest,
            filename,
        } => {
            let cmd = serde_json::json!({
                "command": "add",
                "url": url,
                "dest": dest.map(|d| d.to_string_lossy().to_string()),
                "filename": filename
            });
            let res = send_ipc_command(&cli.socket, &cmd)
                .map_err(|e| format!("Failed to contact daemon: {e}"))?;
            println!("{res}");
        }
        Commands::List => {
            let cmd = serde_json::json!({ "command": "list" });
            let res = send_ipc_command(&cli.socket, &cmd)
                .map_err(|e| format!("Failed to contact daemon: {e}"))?;
            println!("{res}");
        }
        Commands::Pause { id } => {
            let cmd = serde_json::json!({ "command": "pause", "id": id });
            let res = send_ipc_command(&cli.socket, &cmd)
                .map_err(|e| format!("Failed to contact daemon: {e}"))?;
            println!("{res}");
        }
        Commands::Resume { id } => {
            let cmd = serde_json::json!({ "command": "resume", "id": id });
            let res = send_ipc_command(&cli.socket, &cmd)
                .map_err(|e| format!("Failed to contact daemon: {e}"))?;
            println!("{res}");
        }
        Commands::Cancel { id } => {
            let cmd = serde_json::json!({ "command": "cancel", "id": id });
            let res = send_ipc_command(&cli.socket, &cmd)
                .map_err(|e| format!("Failed to contact daemon: {e}"))?;
            println!("{res}");
        }
        Commands::TrayState => {
            let cmd = serde_json::json!({ "command": "tray_state" });
            let res = send_ipc_command(&cli.socket, &cmd)
                .map_err(|e| format!("Failed to contact daemon: {e}"))?;
            println!("{res}");
        }
    }

    Ok(())
}

mod dirs {
    use std::path::PathBuf;
    pub fn download_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Downloads"))
    }
}
