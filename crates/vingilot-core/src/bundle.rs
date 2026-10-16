use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::crypto::{decrypt_bundle_data, encrypt_bundle_data, is_bundle_encrypted};
use crate::error::{CoreError, CoreResult};
use crate::models::{
    BookmarkNode, HistoryEntry, Profile, ReadingListItem, SearchEngine, SyncManifest, UserSettings,
};

/// Structure representing all portable data contents inside a sync bundle.
#[derive(Debug, Clone, Default)]
pub struct SyncBundleContent {
    pub manifest: Option<SyncManifest>,
    pub profiles: Vec<Profile>,
    pub bookmarks: Vec<BookmarkNode>,
    pub history: Vec<HistoryEntry>,
    pub settings: Option<UserSettings>,
    pub search_engines: Vec<SearchEngine>,
    pub reading_list: Vec<ReadingListItem>,
}

/// Writer for generating directory and .vingilot zip bundle packages.
pub struct BundleWriter;

impl BundleWriter {
    /// Serializes bundle contents into a .vingilot ZIP archive in memory, with optional passphrase encryption.
    pub fn write_archive(
        content: &SyncBundleContent,
        passphrase: Option<&str>,
    ) -> CoreResult<Vec<u8>> {
        let manifest = content.manifest.as_ref().ok_or_else(|| {
            CoreError::Validation("SyncManifest is required to write bundle".to_string())
        })?;

        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            // 1. manifest.json
            zip.start_file("manifest.json", options)
                .map_err(|e| CoreError::Export(e.to_string()))?;
            let manifest_bytes = serde_json::to_vec_pretty(manifest)?;
            zip.write_all(&manifest_bytes)?;

            // 2. profiles.json
            zip.start_file("profiles.json", options)
                .map_err(|e| CoreError::Export(e.to_string()))?;
            let profiles_bytes = serde_json::to_vec_pretty(&content.profiles)?;
            zip.write_all(&profiles_bytes)?;

            // 3. bookmarks.json
            if manifest.options.include_bookmarks {
                zip.start_file("bookmarks.json", options)
                    .map_err(|e| CoreError::Export(e.to_string()))?;
                let bm_bytes = serde_json::to_vec_pretty(&content.bookmarks)?;
                zip.write_all(&bm_bytes)?;
            }

            // 4. history.jsonl
            if manifest.options.include_history {
                zip.start_file("history.jsonl", options)
                    .map_err(|e| CoreError::Export(e.to_string()))?;
                for entry in &content.history {
                    let line = serde_json::to_string(entry)?;
                    zip.write_all(line.as_bytes())?;
                    zip.write_all(b"\n")?;
                }
            }

            // 5. settings.json
            if manifest.options.include_settings {
                if let Some(settings) = &content.settings {
                    let mut exported_settings = settings.clone();
                    // Security constraint: exclude API keys unless specifically requested
                    if !manifest.options.include_api_keys {
                        exported_settings.ai_api_key = None;
                    }
                    zip.start_file("settings.json", options)
                        .map_err(|e| CoreError::Export(e.to_string()))?;
                    let set_bytes = serde_json::to_vec_pretty(&exported_settings)?;
                    zip.write_all(&set_bytes)?;
                }
            }

            // 6. search_engines.json
            if manifest.options.include_search_engines {
                zip.start_file("search_engines.json", options)
                    .map_err(|e| CoreError::Export(e.to_string()))?;
                let se_bytes = serde_json::to_vec_pretty(&content.search_engines)?;
                zip.write_all(&se_bytes)?;
            }

            // 7. reading_list.json
            if manifest.options.include_reading_list {
                zip.start_file("reading_list.json", options)
                    .map_err(|e| CoreError::Export(e.to_string()))?;
                let rl_bytes = serde_json::to_vec_pretty(&content.reading_list)?;
                zip.write_all(&rl_bytes)?;
            }

            zip.finish().map_err(|e| CoreError::Export(e.to_string()))?;
        }

        let zip_bytes = buffer.into_inner();
        if let Some(pass) = passphrase {
            encrypt_bundle_data(&zip_bytes, pass)
        } else {
            Ok(zip_bytes)
        }
    }
}

/// Reader and validator for .vingilot ZIP bundles.
pub struct BundleReader;

impl BundleReader {
    pub fn is_encrypted(data: &[u8]) -> bool {
        is_bundle_encrypted(data)
    }

    /// Reads and parses an archive, decrypting if passphrase is provided.
    pub fn read_archive(data: &[u8], passphrase: Option<&str>) -> CoreResult<SyncBundleContent> {
        let raw_zip_data = if Self::is_encrypted(data) {
            let pass = passphrase.ok_or_else(|| {
                CoreError::Import("Archive is encrypted; passphrase required".to_string())
            })?;
            decrypt_bundle_data(data, pass)?
        } else {
            data.to_vec()
        };

        let cursor = Cursor::new(raw_zip_data);
        let mut zip = ZipArchive::new(cursor)
            .map_err(|e| CoreError::Import(format!("Corrupted archive: {e}")))?;

        let mut content = SyncBundleContent::default();

        // 1. Read manifest.json (required)
        {
            let mut manifest_file = zip
                .by_name("manifest.json")
                .map_err(|_| CoreError::Import("Missing manifest.json in bundle".to_string()))?;
            let mut str_val = String::new();
            manifest_file.read_to_string(&mut str_val)?;
            let manifest: SyncManifest = serde_json::from_str(&str_val)?;
            content.manifest = Some(manifest);
        }

        // 2. Read profiles.json
        if let Ok(mut file) = zip.by_name("profiles.json") {
            let mut str_val = String::new();
            file.read_to_string(&mut str_val)?;
            content.profiles = serde_json::from_str(&str_val).unwrap_or_default();
        }

        // 3. Read bookmarks.json
        if let Ok(mut file) = zip.by_name("bookmarks.json") {
            let mut str_val = String::new();
            file.read_to_string(&mut str_val)?;
            content.bookmarks = serde_json::from_str(&str_val).unwrap_or_default();
        }

        // 4. Read history.jsonl
        if let Ok(mut file) = zip.by_name("history.jsonl") {
            let mut str_val = String::new();
            file.read_to_string(&mut str_val)?;
            for line in str_val.lines() {
                if let Ok(entry) = serde_json::from_str(line) {
                    content.history.push(entry);
                }
            }
        }

        // 5. Read settings.json
        if let Ok(mut file) = zip.by_name("settings.json") {
            let mut str_val = String::new();
            file.read_to_string(&mut str_val)?;
            content.settings = serde_json::from_str(&str_val).ok();
        }

        // 6. Read search_engines.json
        if let Ok(mut file) = zip.by_name("search_engines.json") {
            let mut str_val = String::new();
            file.read_to_string(&mut str_val)?;
            content.search_engines = serde_json::from_str(&str_val).unwrap_or_default();
        }

        // 7. Read reading_list.json
        if let Ok(mut file) = zip.by_name("reading_list.json") {
            let mut str_val = String::new();
            file.read_to_string(&mut str_val)?;
            content.reading_list = serde_json::from_str(&str_val).unwrap_or_default();
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DeviceIdentity, SyncOptions};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_bundle_write_and_read_roundtrip() {
        let manifest = SyncManifest {
            schema_version: 1,
            application: "Vingilot".to_string(),
            app_version: "0.1.3".to_string(),
            created_at: Utc::now(),
            device: DeviceIdentity {
                device_id: Uuid::new_v4(),
                device_name: "ThinkPad".to_string(),
                platform: "Linux".to_string(),
            },
            options: SyncOptions::default(),
        };

        let mut content = SyncBundleContent {
            manifest: Some(manifest),
            ..Default::default()
        };

        let bm = BookmarkNode {
            id: Uuid::new_v4(),
            parent_id: None,
            title: "Tolkien Gateway".to_string(),
            url: Some("https://tolkiengateway.net".to_string()),
            is_folder: false,
            children: Vec::new(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        content.bookmarks.push(bm);

        // Write unencrypted
        let archive = BundleWriter::write_archive(&content, None).unwrap();
        assert!(!BundleReader::is_encrypted(&archive));

        let read_back = BundleReader::read_archive(&archive, None).unwrap();
        assert_eq!(read_back.bookmarks.len(), 1);
        assert_eq!(read_back.bookmarks[0].title, "Tolkien Gateway");

        // Write encrypted
        let enc_archive = BundleWriter::write_archive(&content, Some("secret123")).unwrap();
        assert!(BundleReader::is_encrypted(&enc_archive));

        let read_enc = BundleReader::read_archive(&enc_archive, Some("secret123")).unwrap();
        assert_eq!(read_enc.bookmarks.len(), 1);
        assert_eq!(read_enc.bookmarks[0].title, "Tolkien Gateway");
    }
}
