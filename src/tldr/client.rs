//! TLDR client for fetching and providing command documentation.
//!
//! This client uses a CLI-first approach:
//! 1. If a `tldr` CLI is installed (tealdeer, tlrc, etc.), use it
//! 2. Fall back to our minimal cache implementation when no CLI is available
//!
//! This follows KISS/DRY principles by leveraging existing maintained tools.

use super::cache::{CacheError, CacheInfo, TldrCache};
use super::parser::TldrParser;
use super::types::{Platform, TldrPage};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for the TLDR client.
#[derive(Debug, Clone)]
pub struct TldrConfig {
    /// Path to the cache directory (None = default)
    pub cache_dir: Option<PathBuf>,
    /// Archive URL for downloading pages
    pub archive_url: Option<String>,
    /// Cache expiry duration
    pub cache_expiry: Duration,
    /// Preferred language for pages
    pub language: String,
    /// Whether to auto-update cache when expired
    pub auto_update: bool,
    /// Maximum number of pages to cache in memory
    pub max_memory_cache: usize,
    /// Whether to prefer CLI over cache (default: true)
    pub prefer_cli: bool,
}

impl Default for TldrConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            archive_url: None,
            cache_expiry: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            language: "en".to_string(),
            auto_update: true,
            max_memory_cache: 100,
            prefer_cli: true,
        }
    }
}

/// Source of TLDR data.
#[derive(Debug, Clone, PartialEq)]
pub enum TldrSource {
    /// Using system tldr CLI (tealdeer, tlrc, etc.)
    Cli(String), // CLI name
    /// Using our internal cache
    Cache,
    /// No TLDR source available
    None,
}

/// TLDR client for fetching command documentation.
///
/// The client provides async methods for looking up TLDR pages.
/// It prefers using a system-installed tldr CLI when available.
pub struct TldrClient {
    cache: Arc<RwLock<TldrCache>>,
    config: TldrConfig,
    cli_path: Option<PathBuf>,
    source: TldrSource,
}

impl TldrClient {
    /// Create a new client with default configuration.
    pub fn new() -> Result<Self, CacheError> {
        Self::with_config(TldrConfig::default())
    }

    /// Create a new client with custom configuration.
    pub fn with_config(config: TldrConfig) -> Result<Self, CacheError> {
        // Check for system tldr CLI
        let cli_path = if config.prefer_cli {
            Self::find_tldr_cli()
        } else {
            None
        };

        let source = if let Some(ref path) = cli_path {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("tldr")
                .to_string();
            TldrSource::Cli(name)
        } else {
            TldrSource::Cache
        };

        let mut cache = if let Some(ref dir) = config.cache_dir {
            TldrCache::with_dir(dir.clone())
        } else {
            TldrCache::new()?
        };

        if let Some(ref url) = config.archive_url {
            cache = cache.with_archive_url(url.clone());
        }

        cache = cache.with_expiry(config.cache_expiry);

        info!("TLDR client initialized with source: {:?}", source);

        Ok(Self {
            cache: Arc::new(RwLock::new(cache)),
            config,
            cli_path,
            source,
        })
    }

    /// Find an installed tldr CLI.
    fn find_tldr_cli() -> Option<PathBuf> {
        // Common tldr CLI names in order of preference
        const CLI_NAMES: &[&str] = &["tldr", "tealdeer"];

        for name in CLI_NAMES {
            if let Ok(path) = which::which(name) {
                // Verify it works
                if Command::new(&path).arg("--version").output().is_ok() {
                    debug!("Found tldr CLI: {}", path.display());
                    return Some(path);
                }
            }
        }

        None
    }

    /// Get a TLDR page for a command using CLI.
    fn get_page_from_cli(&self, command: &str) -> Option<TldrPage> {
        let cli_path = self.cli_path.as_ref()?;

        // Run tldr command with markdown output
        let output = Command::new(cli_path)
            .arg("--raw") // Get raw markdown (supported by tealdeer and tlrc)
            .arg(command)
            .output()
            .ok()?;

        if !output.status.success() {
            debug!("tldr CLI returned non-zero for: {}", command);
            return None;
        }

        let content = String::from_utf8(output.stdout).ok()?;
        if content.trim().is_empty() {
            return None;
        }

        // Parse the markdown output
        match TldrParser::parse(&content, Platform::current(), &self.config.language) {
            Ok(page) => Some(page),
            Err(e) => {
                warn!("Failed to parse tldr CLI output for {}: {}", command, e);
                None
            }
        }
    }

    /// Get a TLDR page for a command.
    ///
    /// This will:
    /// 1. Try the system tldr CLI if available
    /// 2. Fall back to the on-disk cache
    pub async fn get_page(&self, command: &str) -> Result<TldrPage, CacheError> {
        self.get_page_for_platform(command, Platform::current())
            .await
    }

    /// Get a TLDR page for a specific platform.
    pub async fn get_page_for_platform(
        &self,
        command: &str,
        platform: Platform,
    ) -> Result<TldrPage, CacheError> {
        let command = command.to_lowercase();

        // Try CLI first if available
        if self.cli_path.is_some() {
            if let Some(page) = self.get_page_from_cli(&command) {
                return Ok(page);
            }
            debug!("CLI lookup failed for {}, trying cache", command);
        }

        // Fall back to cache
        if self.config.auto_update {
            let is_valid = {
                let cache = self.cache.read().await;
                cache.is_valid().await
            };

            if !is_valid {
                debug!("Cache expired or missing, triggering update");
                if let Err(e) = self.update_cache().await {
                    warn!("Failed to update cache: {}. Will try with existing data.", e);
                }
            }
        }

        let mut cache = self.cache.write().await;
        cache
            .find_page_with_options(&command, platform, &self.config.language)
            .await
    }

    /// Get TLDR pages for multiple commands.
    ///
    /// Returns a map of command names to their pages.
    /// Commands that don't have pages are omitted from the result.
    pub async fn get_pages(&self, commands: &[String]) -> HashMap<String, TldrPage> {
        let mut result = HashMap::new();

        for command in commands {
            match self.get_page(command).await {
                Ok(page) => {
                    result.insert(command.clone(), page);
                }
                Err(CacheError::PageNotFound(_)) => {
                    debug!("No TLDR page found for: {}", command);
                }
                Err(e) => {
                    warn!("Error fetching TLDR for {}: {}", command, e);
                }
            }
        }

        result
    }

    /// Get context strings for multiple commands (for LLM prompts).
    ///
    /// Returns formatted TLDR content suitable for including in LLM prompts.
    pub async fn get_context_for_commands(&self, commands: &[String]) -> String {
        let pages = self.get_pages(commands).await;

        if pages.is_empty() {
            return String::new();
        }

        let mut context = String::from("=== TLDR Reference ===\n\n");

        for (command, page) in pages.iter() {
            context.push_str(&format!("## {}\n", command));
            context.push_str(&page.as_context());
            context.push_str("\n---\n\n");
        }

        context
    }

    /// Get the source of TLDR data.
    pub fn source(&self) -> &TldrSource {
        &self.source
    }

    /// Check if CLI is being used.
    pub fn is_using_cli(&self) -> bool {
        self.cli_path.is_some()
    }

    /// Update the cache from the remote source.
    ///
    /// Note: If using CLI, this updates the CLI's cache via `tldr --update`.
    pub async fn update_cache(&self) -> Result<(), CacheError> {
        // Try CLI update first
        if let Some(ref cli_path) = self.cli_path {
            debug!("Updating via tldr CLI");
            let status = Command::new(cli_path)
                .arg("--update")
                .status();

            match status {
                Ok(s) if s.success() => {
                    info!("TLDR cache updated via CLI");
                    return Ok(());
                }
                Ok(_) => warn!("tldr --update returned non-zero, falling back to internal update"),
                Err(e) => warn!("Failed to run tldr --update: {}", e),
            }
        }

        // Fall back to internal cache update
        info!("Updating TLDR cache");
        let mut cache = self.cache.write().await;
        cache.update().await
    }

    /// Clear the cache.
    pub async fn clear_cache(&self) -> Result<(), CacheError> {
        // Try CLI clear first
        if let Some(ref cli_path) = self.cli_path {
            let _ = Command::new(cli_path)
                .arg("--clear-cache")
                .status();
        }

        let mut cache = self.cache.write().await;
        cache.clear().await
    }

    /// Get cache information.
    pub async fn cache_info(&self) -> Result<CacheInfo, CacheError> {
        let cache = self.cache.read().await;
        cache.info().await
    }

    /// Check if the cache is valid.
    pub async fn is_cache_valid(&self) -> bool {
        // If using CLI, assume it manages its own cache
        if self.cli_path.is_some() {
            return true;
        }

        let cache = self.cache.read().await;
        cache.is_valid().await
    }

    /// List all available commands for the current platform.
    pub async fn list_commands(&self) -> Result<Vec<String>, CacheError> {
        self.list_commands_for_platform(Platform::current()).await
    }

    /// List all available commands for a specific platform.
    pub async fn list_commands_for_platform(
        &self,
        platform: Platform,
    ) -> Result<Vec<String>, CacheError> {
        // Try CLI list first
        if let Some(ref cli_path) = self.cli_path {
            let output = Command::new(cli_path)
                .arg("--list")
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    if let Ok(content) = String::from_utf8(out.stdout) {
                        let commands: Vec<String> = content
                            .lines()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !commands.is_empty() {
                            return Ok(commands);
                        }
                    }
                }
            }
        }

        // Fall back to cache
        let cache = self.cache.read().await;
        cache.list_commands(platform).await
    }

    /// Search for commands matching a pattern.
    pub async fn search(&self, pattern: &str) -> Result<Vec<String>, CacheError> {
        let all_commands = self.list_commands().await?;
        let pattern = pattern.to_lowercase();

        Ok(all_commands
            .into_iter()
            .filter(|cmd| cmd.contains(&pattern))
            .collect())
    }

    /// Get the client configuration.
    pub fn config(&self) -> &TldrConfig {
        &self.config
    }
}

impl Default for TldrClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default TldrClient")
    }
}

/// Builder for TldrClient configuration.
#[derive(Debug, Default)]
pub struct TldrClientBuilder {
    config: TldrConfig,
}

impl TldrClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the cache directory.
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.cache_dir = Some(path.into());
        self
    }

    /// Set the archive URL.
    pub fn archive_url(mut self, url: impl Into<String>) -> Self {
        self.config.archive_url = Some(url.into());
        self
    }

    /// Set the cache expiry duration.
    pub fn cache_expiry(mut self, expiry: Duration) -> Self {
        self.config.cache_expiry = expiry;
        self
    }

    /// Set the preferred language.
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.config.language = lang.into();
        self
    }

    /// Enable or disable auto-update.
    pub fn auto_update(mut self, enabled: bool) -> Self {
        self.config.auto_update = enabled;
        self
    }

    /// Set the maximum memory cache size.
    pub fn max_memory_cache(mut self, size: usize) -> Self {
        self.config.max_memory_cache = size;
        self
    }

    /// Enable or disable CLI preference.
    pub fn prefer_cli(mut self, prefer: bool) -> Self {
        self.config.prefer_cli = prefer;
        self
    }

    /// Build the TldrClient.
    pub fn build(self) -> Result<TldrClient, CacheError> {
        TldrClient::with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_client() -> (TldrClient, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let client = TldrClientBuilder::new()
            .cache_dir(temp_dir.path())
            .auto_update(false)
            .prefer_cli(false) // Use cache for predictable testing
            .build()
            .unwrap();
        (client, temp_dir)
    }

    fn populate_test_cache(temp_dir: &TempDir) {
        let pages_dir = temp_dir.path().join("pages").join("common");
        std::fs::create_dir_all(&pages_dir).unwrap();

        let git_page = r#"# git

> Distributed version control system.

- Clone a repository:

`git clone {{url}}`

- Show status:

`git status`
"#;
        std::fs::write(pages_dir.join("git.md"), git_page).unwrap();

        let curl_page = r#"# curl

> Transfer data from or to a server.

- Download a file:

`curl -O {{url}}`

- Make a POST request:

`curl -X POST {{url}}`
"#;
        std::fs::write(pages_dir.join("curl.md"), curl_page).unwrap();
    }

    #[tokio::test]
    async fn test_get_page() {
        let (client, temp_dir) = create_test_client();
        populate_test_cache(&temp_dir);

        let page = client.get_page("git").await.unwrap();
        assert_eq!(page.name, "git");
        assert_eq!(page.examples.len(), 2);
    }

    #[tokio::test]
    async fn test_get_pages() {
        let (client, temp_dir) = create_test_client();
        populate_test_cache(&temp_dir);

        let pages = client
            .get_pages(&["git".to_string(), "curl".to_string(), "nonexistent".to_string()])
            .await;

        assert_eq!(pages.len(), 2);
        assert!(pages.contains_key("git"));
        assert!(pages.contains_key("curl"));
        assert!(!pages.contains_key("nonexistent"));
    }

    #[tokio::test]
    async fn test_get_context_for_commands() {
        let (client, temp_dir) = create_test_client();
        populate_test_cache(&temp_dir);

        let context = client
            .get_context_for_commands(&["git".to_string()])
            .await;

        assert!(context.contains("TLDR Reference"));
        assert!(context.contains("git"));
        assert!(context.contains("Clone a repository"));
    }

    #[tokio::test]
    async fn test_list_commands() {
        let (client, temp_dir) = create_test_client();
        populate_test_cache(&temp_dir);

        let commands = client.list_commands().await.unwrap();
        assert!(commands.contains(&"git".to_string()));
        assert!(commands.contains(&"curl".to_string()));
    }

    #[tokio::test]
    async fn test_search() {
        let (client, temp_dir) = create_test_client();
        populate_test_cache(&temp_dir);

        let results = client.search("gi").await.unwrap();
        assert!(results.contains(&"git".to_string()));

        let results = client.search("curl").await.unwrap();
        assert!(results.contains(&"curl".to_string()));
    }

    #[tokio::test]
    async fn test_source_without_cli() {
        let (client, _temp_dir) = create_test_client();
        // With prefer_cli=false, should use Cache
        assert_eq!(client.source(), &TldrSource::Cache);
    }

    #[tokio::test]
    async fn test_builder() {
        let temp_dir = TempDir::new().unwrap();

        let client = TldrClientBuilder::new()
            .cache_dir(temp_dir.path())
            .language("es")
            .auto_update(false)
            .cache_expiry(Duration::from_secs(3600))
            .max_memory_cache(50)
            .prefer_cli(false)
            .build()
            .unwrap();

        assert_eq!(client.config().language, "es");
        assert!(!client.config().auto_update);
        assert!(!client.config().prefer_cli);
    }
}
