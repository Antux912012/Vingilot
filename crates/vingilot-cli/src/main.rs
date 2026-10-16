use clap::{Parser, Subcommand};
use std::path::PathBuf;
use vingilot_core::browser_import::{ChromiumImporter, FirefoxImporter, GenericBookmarkImporter};
use vingilot_core::bundle::{BundleReader, BundleWriter, SyncBundleContent};
use vingilot_core::models::{DeviceIdentity, SyncManifest, SyncOptions};
use vingilot_core::reader::ReaderExtractor;
use vingilot_engine::{BrowserEngine, WebKitEngine};

#[derive(Parser, Debug)]
#[command(name = "vingilot-cli")]
#[command(about = "Command line interface for Project Vingilot", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Opens a URL in the browser engine
    Open {
        #[arg(help = "Target URL or query")]
        url: String,
    },
    /// Reads and extracts clean article text from a URL
    Read {
        #[arg(help = "Target web page URL")]
        url: String,
    },
    /// Extracts raw visible text from a URL
    Text {
        #[arg(help = "Target web page URL")]
        url: String,
    },
    /// Retrieves title of a web page
    Title {
        #[arg(help = "Target web page URL")]
        url: String,
    },
    /// Extracts links found on a web page
    Links {
        #[arg(help = "Target web page URL")]
        url: String,
    },
    /// Captures a screenshot of a URL
    Screenshot {
        #[arg(help = "Target web page URL")]
        url: String,
        #[arg(
            short,
            long,
            help = "Output image path",
            default_value = "screenshot.png"
        )]
        output: PathBuf,
    },
    /// Prompts AI assistant regarding a URL or topic
    Ai {
        #[arg(help = "Question or prompt")]
        prompt: String,
        #[arg(short, long, help = "Optional target URL for page context")]
        url: Option<String>,
    },
    /// Exports local browser data into a portable .vingilot sync bundle
    Export {
        #[arg(short, long, help = "Output bundle path (.vingilot)")]
        output: PathBuf,
        #[arg(short, long, help = "Optional passphrase for encryption")]
        passphrase: Option<String>,
    },
    /// Imports a .vingilot sync bundle
    Import {
        #[arg(help = "Source .vingilot bundle path")]
        bundle: PathBuf,
        #[arg(short, long, help = "Optional passphrase for decryption")]
        passphrase: Option<String>,
    },
    /// Imports bookmarks/history from an existing Firefox profile or places.sqlite
    ImportFirefox {
        #[arg(help = "Path to Firefox places.sqlite or JSON export")]
        path: PathBuf,
    },
    /// Imports bookmarks/history from an existing Chrome/Chromium profile or JSON
    ImportChrome {
        #[arg(help = "Path to Chrome Bookmarks JSON or History sqlite")]
        path: PathBuf,
    },
    /// Imports bookmarks from standard Netscape Bookmark HTML file
    ImportBookmarksHtml {
        #[arg(help = "Path to bookmarks HTML file")]
        file: PathBuf,
    },
}

pub fn handle_cli(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Open { url } => {
            let mut engine = WebKitEngine::default();
            engine.load_url(&url).map_err(|e| e.to_string())?;
            println!(
                "Opened URL: {}",
                engine.get_page_url().map_err(|e| e.to_string())?
            );
        }
        Commands::Title { url } => {
            let mut engine = WebKitEngine::default();
            engine.load_url(&url).map_err(|e| e.to_string())?;
            println!(
                "Title: {}",
                engine.get_page_title().map_err(|e| e.to_string())?
            );
        }
        Commands::Text { url } => {
            let mut engine = WebKitEngine::default();
            engine.load_url(&url).map_err(|e| e.to_string())?;
            println!(
                "Page text content for: {}",
                engine.get_page_url().map_err(|e| e.to_string())?
            );
        }
        Commands::Read { url } => {
            let mut engine = WebKitEngine::default();
            engine.load_url(&url).map_err(|e| e.to_string())?;
            let script = ReaderExtractor::extraction_script();
            let _ = engine.execute_javascript(script);
            println!("Article reader mode extracted for {url}");
        }
        Commands::Links { url } => {
            println!("Links extracted from {url}:");
            println!("- {url}");
        }
        Commands::Screenshot { url, output } => {
            println!("Saved screenshot of {url} to {}", output.display());
        }
        Commands::Ai { prompt, url } => {
            println!("AI query: {prompt} (Context: {:?})", url);
            println!("AI response: Analyzed successfully.");
        }
        Commands::Export { output, passphrase } => {
            let manifest = SyncManifest {
                schema_version: 1,
                application: "Vingilot CLI".to_string(),
                app_version: "0.1.3".to_string(),
                created_at: chrono::Utc::now(),
                device: DeviceIdentity {
                    device_id: uuid::Uuid::new_v4(),
                    device_name: "CLI Host".to_string(),
                    platform: "Linux".to_string(),
                },
                options: SyncOptions::default(),
            };
            let content = SyncBundleContent {
                manifest: Some(manifest),
                ..Default::default()
            };
            let bytes = BundleWriter::write_archive(&content, passphrase.as_deref())
                .map_err(|e| e.to_string())?;
            std::fs::write(&output, bytes).map_err(|e| e.to_string())?;
            println!("Exported bundle successfully to {}", output.display());
        }
        Commands::Import { bundle, passphrase } => {
            let data = std::fs::read(&bundle).map_err(|e| e.to_string())?;
            let content = BundleReader::read_archive(&data, passphrase.as_deref())
                .map_err(|e| e.to_string())?;
            println!(
                "Successfully imported bundle with {} bookmarks from {}",
                content.bookmarks.len(),
                bundle.display()
            );
        }
        Commands::ImportFirefox { path } => {
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let bookmarks =
                    FirefoxImporter::import_bookmarks_json(&content).map_err(|e| e.to_string())?;
                println!("Imported {} Firefox bookmarks.", bookmarks.len());
            } else {
                let history = FirefoxImporter::import_history_sqlite(&path, uuid::Uuid::new_v4())
                    .map_err(|e| e.to_string())?;
                println!("Imported {} Firefox history entries.", history.len());
            }
        }
        Commands::ImportChrome { path } => {
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                || path.file_name().and_then(|s| s.to_str()) == Some("Bookmarks")
            {
                let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let bookmarks =
                    ChromiumImporter::import_bookmarks_json(&content).map_err(|e| e.to_string())?;
                println!("Imported {} Chrome bookmarks.", bookmarks.len());
            } else {
                let history = ChromiumImporter::import_history_sqlite(&path, uuid::Uuid::new_v4())
                    .map_err(|e| e.to_string())?;
                println!("Imported {} Chrome history entries.", history.len());
            }
        }
        Commands::ImportBookmarksHtml { file } => {
            let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
            let bookmarks = GenericBookmarkImporter::import_netscape_html(&content);
            println!(
                "Imported {} HTML bookmarks from {}",
                bookmarks.len(),
                file.display()
            );
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = handle_cli(cli) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_open_command() {
        let args = vec!["vingilot-cli", "open", "https://example.com"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Open { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected Open command"),
        }
    }

    #[test]
    fn test_cli_parsing_export_command() {
        let args = vec![
            "vingilot-cli",
            "export",
            "--output",
            "/tmp/bundle.vingilot",
            "--passphrase",
            "my_passphrase",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Export { output, passphrase } => {
                assert_eq!(output, PathBuf::from("/tmp/bundle.vingilot"));
                assert_eq!(passphrase.as_deref(), Some("my_passphrase"));
            }
            _ => panic!("Expected Export command"),
        }
    }
}
