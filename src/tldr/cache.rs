//! TLDR cache management for downloading and storing TLDR pages.
//!
//! This module handles:
//! - Downloading TLDR page archives from GitHub releases
//! - Extracting and caching pages locally
//! - Cache validation and updates
//! - Platform and language-aware page lookup

use super::parser::TldrParser;
use super::types::{Platform, TldrPage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::fs;
#[cfg(feature = "remote-backends")]
use tokio::io::AsyncWriteExt;
#[cfg(feature = "remote-backends")]
use tracing::debug;
use tracing::{info, warn};

/// Default URL for TLDR pages archive.
pub const DEFAULT_ARCHIVE_URL: &str =
    "https://github.com/tldr-pages/tldr/releases/latest/download/tldr.zip";

/// Default cache expiry duration (7 days).
pub const DEFAULT_CACHE_EXPIRY: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Errors that can occur during cache operations.
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to create cache directory: {0}")]
    DirectoryCreation(#[source] std::io::Error),

    #[error("Failed to download archive: {0}")]
    Download(String),

    #[error("Failed to extract archive: {0}")]
    Extraction(String),

    #[error("Failed to read cache: {0}")]
    Read(#[source] std::io::Error),

    #[error("Failed to write cache: {0}")]
    Write(#[source] std::io::Error),

    #[error("Page not found: {0}")]
    PageNotFound(String),

    #[error("Invalid cache format: {0}")]
    InvalidFormat(String),

    #[error("Network error: {0}")]
    Network(String),
}

/// Cache statistics and metadata.
#[derive(Debug, Clone)]
pub struct CacheInfo {
    /// Path to the cache directory
    pub path: PathBuf,
    /// Total size of cached pages in bytes
    pub size_bytes: u64,
    /// Number of cached pages
    pub page_count: usize,
    /// Last update timestamp
    pub last_updated: Option<SystemTime>,
    /// Available platforms
    pub platforms: Vec<Platform>,
    /// Available languages
    pub languages: Vec<String>,
}

/// TLDR cache manager for storing and retrieving pages.
pub struct TldrCache {
    /// Base cache directory
    cache_dir: PathBuf,
    /// Archive download URL
    archive_url: String,
    /// Cache expiry duration
    expiry: Duration,
    /// In-memory cache of parsed pages (command -> platform -> page)
    page_cache: HashMap<String, HashMap<Platform, TldrPage>>,
}

impl TldrCache {
    /// Create a new cache manager with default settings.
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = Self::default_cache_dir()?;
        Ok(Self {
            cache_dir,
            archive_url: DEFAULT_ARCHIVE_URL.to_string(),
            expiry: DEFAULT_CACHE_EXPIRY,
            page_cache: HashMap::new(),
        })
    }

    /// Create a cache manager with a custom directory.
    pub fn with_dir(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            archive_url: DEFAULT_ARCHIVE_URL.to_string(),
            expiry: DEFAULT_CACHE_EXPIRY,
            page_cache: HashMap::new(),
        }
    }

    /// Set a custom archive URL.
    pub fn with_archive_url(mut self, url: impl Into<String>) -> Self {
        self.archive_url = url.into();
        self
    }

    /// Set a custom cache expiry duration.
    pub fn with_expiry(mut self, expiry: Duration) -> Self {
        self.expiry = expiry;
        self
    }

    /// Get the default cache directory.
    fn default_cache_dir() -> Result<PathBuf, CacheError> {
        let cache_dir = dirs::cache_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            .ok_or_else(|| {
                CacheError::DirectoryCreation(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cannot determine cache directory",
                ))
            })?;

        Ok(cache_dir.join("caro").join("tldr"))
    }

    /// Check if the cache exists and is valid.
    pub async fn is_valid(&self) -> bool {
        let marker = self.cache_dir.join(".tldr_cache_marker");
        if !marker.exists() {
            return false;
        }

        // Check if cache has expired
        match fs::metadata(&marker).await {
            Ok(meta) => {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                        return elapsed < self.expiry;
                    }
                }
                true
            }
            Err(_) => false,
        }
    }

    /// Get cache information.
    pub async fn info(&self) -> Result<CacheInfo, CacheError> {
        let mut size_bytes = 0u64;
        let mut page_count = 0usize;
        let mut platforms = Vec::new();
        let mut languages = Vec::new();

        if self.cache_dir.exists() {
            // Scan for pages directories
            let mut entries = fs::read_dir(&self.cache_dir)
                .await
                .map_err(CacheError::Read)?;

            while let Some(entry) = entries.next_entry().await.map_err(CacheError::Read)? {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if name_str.starts_with("pages") {
                    let lang = if name_str == "pages" {
                        "en".to_string()
                    } else {
                        name_str.strip_prefix("pages.").unwrap_or("en").to_string()
                    };
                    if !languages.contains(&lang) {
                        languages.push(lang);
                    }

                    // Scan platform directories
                    let lang_dir = entry.path();
                    if let Ok(mut platform_entries) = fs::read_dir(&lang_dir).await {
                        while let Some(platform_entry) =
                            platform_entries.next_entry().await.ok().flatten()
                        {
                            let platform_name = platform_entry.file_name();
                            if let Some(platform) =
                                Platform::from_dir_name(&platform_name.to_string_lossy())
                            {
                                if !platforms.contains(&platform) {
                                    platforms.push(platform);
                                }

                                // Count pages
                                let platform_dir = platform_entry.path();
                                if let Ok(mut page_entries) = fs::read_dir(&platform_dir).await {
                                    while let Some(page_entry) =
                                        page_entries.next_entry().await.ok().flatten()
                                    {
                                        if page_entry
                                            .file_name()
                                            .to_string_lossy()
                                            .ends_with(".md")
                                        {
                                            page_count += 1;
                                            if let Ok(meta) = page_entry.metadata().await {
                                                size_bytes += meta.len();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let last_updated = self
            .cache_dir
            .join(".tldr_cache_marker")
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok());

        Ok(CacheInfo {
            path: self.cache_dir.clone(),
            size_bytes,
            page_count,
            last_updated,
            platforms,
            languages,
        })
    }

    /// Update the cache by downloading the latest archive.
    #[cfg(feature = "remote-backends")]
    pub async fn update(&mut self) -> Result<(), CacheError> {
        info!("Updating TLDR cache from {}", self.archive_url);

        // Create cache directory
        fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(CacheError::DirectoryCreation)?;

        // Download archive
        let archive_data = self.download_archive().await?;

        // Extract archive
        self.extract_archive(&archive_data).await?;

        // Update marker file
        let marker = self.cache_dir.join(".tldr_cache_marker");
        let mut file = fs::File::create(&marker)
            .await
            .map_err(CacheError::Write)?;
        file.write_all(b"tldr cache marker")
            .await
            .map_err(CacheError::Write)?;

        // Clear in-memory cache
        self.page_cache.clear();

        info!("TLDR cache updated successfully");
        Ok(())
    }

    /// Update without network (stub for non-remote builds).
    #[cfg(not(feature = "remote-backends"))]
    pub async fn update(&mut self) -> Result<(), CacheError> {
        Err(CacheError::Network(
            "Remote backends feature not enabled".to_string(),
        ))
    }

    /// Download the archive from the configured URL.
    #[cfg(feature = "remote-backends")]
    async fn download_archive(&self) -> Result<Vec<u8>, CacheError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| CacheError::Network(e.to_string()))?;

        let response = client
            .get(&self.archive_url)
            .send()
            .await
            .map_err(|e| CacheError::Download(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CacheError::Download(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| CacheError::Download(e.to_string()))?;

        debug!("Downloaded {} bytes", bytes.len());
        Ok(bytes.to_vec())
    }

    /// Extract the downloaded archive to the cache directory.
    #[cfg(feature = "remote-backends")]
    async fn extract_archive(&self, data: &[u8]) -> Result<(), CacheError> {
        let cache_dir = self.cache_dir.clone();

        // Use blocking task for zip extraction
        let data = data.to_vec();
        tokio::task::spawn_blocking(move || {
            let cursor = std::io::Cursor::new(data);
            let mut archive =
                zip::ZipArchive::new(cursor).map_err(|e| CacheError::Extraction(e.to_string()))?;

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| CacheError::Extraction(e.to_string()))?;

                let name = file.name().to_string();

                // Security: skip entries with path traversal
                if name.contains("..") {
                    warn!("Skipping potentially malicious path: {}", name);
                    continue;
                }

                // Only extract pages directories
                if !name.starts_with("pages") {
                    continue;
                }

                let outpath = cache_dir.join(&name);

                if file.is_dir() {
                    std::fs::create_dir_all(&outpath)
                        .map_err(|e| CacheError::DirectoryCreation(e))?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| CacheError::DirectoryCreation(e))?;
                    }

                    let mut outfile = std::fs::File::create(&outpath)
                        .map_err(|e| CacheError::Write(e))?;

                    std::io::copy(&mut file, &mut outfile)
                        .map_err(|e| CacheError::Write(e))?;
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| CacheError::Extraction(format!("Task panicked: {}", e)))?
    }

    /// Look up a page by command name with platform fallback.
    pub async fn find_page(&mut self, command: &str) -> Result<TldrPage, CacheError> {
        self.find_page_with_options(command, Platform::current(), "en")
            .await
    }

    /// Look up a page with specific platform and language.
    pub async fn find_page_with_options(
        &mut self,
        command: &str,
        platform: Platform,
        language: &str,
    ) -> Result<TldrPage, CacheError> {
        let command = command.to_lowercase();

        // Check in-memory cache first
        if let Some(platforms) = self.page_cache.get(&command) {
            if let Some(page) = platforms.get(&platform) {
                return Ok(page.clone());
            }
            // Try fallback platforms
            for fallback in platform.search_order().iter().skip(1) {
                if let Some(page) = platforms.get(fallback) {
                    return Ok(page.clone());
                }
            }
        }

        // Search on disk with platform fallback
        let search_order = platform.search_order();
        let languages = if language == "en" {
            vec!["en".to_string()]
        } else {
            vec![language.to_string(), "en".to_string()]
        };

        for lang in &languages {
            let pages_dir = if lang == "en" {
                self.cache_dir.join("pages")
            } else {
                self.cache_dir.join(format!("pages.{}", lang))
            };

            for search_platform in &search_order {
                let page_path = pages_dir
                    .join(search_platform.as_dir_name())
                    .join(format!("{}.md", command));

                if page_path.exists() {
                    let content = fs::read_to_string(&page_path)
                        .await
                        .map_err(CacheError::Read)?;

                    match TldrParser::parse(&content, *search_platform, lang) {
                        Ok(page) => {
                            // Cache the parsed page
                            self.page_cache
                                .entry(command.clone())
                                .or_default()
                                .insert(*search_platform, page.clone());

                            return Ok(page);
                        }
                        Err(e) => {
                            warn!("Failed to parse page {}: {}", page_path.display(), e);
                            continue;
                        }
                    }
                }
            }
        }

        Err(CacheError::PageNotFound(command))
    }

    /// List all available commands for a platform.
    pub async fn list_commands(&self, platform: Platform) -> Result<Vec<String>, CacheError> {
        let mut commands = Vec::new();

        for search_platform in platform.search_order() {
            let platform_dir = self
                .cache_dir
                .join("pages")
                .join(search_platform.as_dir_name());

            if !platform_dir.exists() {
                continue;
            }

            let mut entries = fs::read_dir(&platform_dir)
                .await
                .map_err(CacheError::Read)?;

            while let Some(entry) = entries.next_entry().await.map_err(CacheError::Read)? {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".md") {
                    let cmd = name_str.trim_end_matches(".md").to_string();
                    if !commands.contains(&cmd) {
                        commands.push(cmd);
                    }
                }
            }
        }

        commands.sort();
        Ok(commands)
    }

    /// Clear the cache.
    pub async fn clear(&mut self) -> Result<(), CacheError> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .await
                .map_err(CacheError::Write)?;
        }
        self.page_cache.clear();
        info!("TLDR cache cleared");
        Ok(())
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl Default for TldrCache {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cache_dir: PathBuf::from(".tldr_cache"),
            archive_url: DEFAULT_ARCHIVE_URL.to_string(),
            expiry: DEFAULT_CACHE_EXPIRY,
            page_cache: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache() -> (TldrCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = TldrCache::with_dir(temp_dir.path().to_path_buf());
        (cache, temp_dir)
    }

    #[tokio::test]
    async fn test_cache_not_valid_when_empty() {
        let (cache, _temp) = create_test_cache();
        assert!(!cache.is_valid().await);
    }

    #[tokio::test]
    async fn test_cache_info_empty() {
        let (cache, _temp) = create_test_cache();
        let info = cache.info().await.unwrap();
        assert_eq!(info.page_count, 0);
        assert_eq!(info.size_bytes, 0);
    }

    #[tokio::test]
    async fn test_find_page_not_found() {
        let (mut cache, _temp) = create_test_cache();
        let result = cache.find_page("nonexistent_command").await;
        assert!(matches!(result, Err(CacheError::PageNotFound(_))));
    }

    #[tokio::test]
    async fn test_list_commands_empty_cache() {
        let (cache, _temp) = create_test_cache();
        let commands = cache.list_commands(Platform::Common).await.unwrap();
        assert!(commands.is_empty());
    }

    #[tokio::test]
    async fn test_find_page_with_cached_content() {
        let (mut cache, temp) = create_test_cache();

        // Create a test page
        let pages_dir = temp.path().join("pages").join("common");
        std::fs::create_dir_all(&pages_dir).unwrap();

        let page_content = r#"# test

> A test command.

- Run test:

`test {{arg}}`
"#;
        std::fs::write(pages_dir.join("test.md"), page_content).unwrap();

        let page = cache.find_page("test").await.unwrap();
        assert_eq!(page.name, "test");
        assert_eq!(page.description, "A test command.");
        assert_eq!(page.examples.len(), 1);
    }

    #[tokio::test]
    async fn test_platform_fallback() {
        let (mut cache, temp) = create_test_cache();

        // Create a page only in common
        let pages_dir = temp.path().join("pages").join("common");
        std::fs::create_dir_all(&pages_dir).unwrap();

        let page_content = "# shared\n\n> A shared command.\n\n- Run:\n\n`shared`\n";
        std::fs::write(pages_dir.join("shared.md"), page_content).unwrap();

        // Should find it even when looking for linux platform
        let page = cache
            .find_page_with_options("shared", Platform::Linux, "en")
            .await
            .unwrap();
        assert_eq!(page.name, "shared");
    }
}
