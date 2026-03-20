/// FileSystemAdapter — unified file I/O across all 5 platforms.
///
/// Platform-specific path resolution:
/// - Android: `/data/data/<package>/files/` for app data
/// - iOS:     `<NSDocumentDirectory>` for documents, `<NSApplicationSupportDirectory>` for app data
/// - Windows: `%APPDATA%\JarvisAssistant\` for app data
/// - macOS:   `~/Library/Application Support/JarvisAssistant/` for app data
/// - Linux:   `~/.local/share/jarvis-assistant/` for app data

use std::fs;
use std::path::{Path, PathBuf};

use crate::pal::types::Platform;

pub struct FileSystemAdapter {
    platform: Platform,
}

impl FileSystemAdapter {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    /// Read raw bytes from a file path.
    pub fn read(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        Ok(fs::read(path)?)
    }

    /// Write raw bytes to a file path (creates or overwrites).
    pub fn write(&self, path: &str, data: &[u8]) -> anyhow::Result<()> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(fs::write(path, data)?)
    }

    /// Delete a file at the given path.
    pub fn delete(&self, path: &str) -> anyhow::Result<()> {
        Ok(fs::remove_file(path)?)
    }

    /// List entries in a directory.
    pub fn list(&self, directory: &str) -> anyhow::Result<Vec<FileInfo>> {
        let entries = fs::read_dir(directory)?
            .filter_map(|e| e.ok())
            .map(|e| {
                let meta = e.metadata().ok();
                FileInfo {
                    name: e.file_name().to_string_lossy().into_owned(),
                    path: e.path().to_string_lossy().into_owned(),
                    is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                    size_bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                }
            })
            .collect();
        Ok(entries)
    }

    /// Returns the platform-appropriate app data directory path.
    pub fn get_app_data_directory(&self) -> PathBuf {
        match self.platform {
            Platform::Android => PathBuf::from("/data/data/com.jarvis.assistant/files"),
            Platform::Ios => {
                // On iOS the real path is resolved at runtime via NSSearchPathForDirectoriesInDomains.
                // This stub returns a placeholder; the Flutter layer injects the real path.
                PathBuf::from("/var/mobile/Containers/Data/Application/jarvis/Library/Application Support")
            }
            Platform::Windows => {
                let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
                PathBuf::from(base).join("JarvisAssistant")
            }
            Platform::Macos => {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("JarvisAssistant")
            }
            Platform::Linux => {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("jarvis-assistant")
            }
        }
    }

    /// Returns the platform-appropriate documents directory path.
    pub fn get_documents_directory(&self) -> PathBuf {
        match self.platform {
            Platform::Android => {
                PathBuf::from("/storage/emulated/0/Documents/JarvisAssistant")
            }
            Platform::Ios => {
                // Placeholder; real path injected by Flutter layer at runtime.
                PathBuf::from("/var/mobile/Containers/Data/Application/jarvis/Documents")
            }
            Platform::Windows => {
                let base = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
                PathBuf::from(base).join("Documents").join("JarvisAssistant")
            }
            Platform::Macos | Platform::Linux => {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join("Documents").join("JarvisAssistant")
            }
        }
    }
}

/// Metadata for a single directory entry.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}
