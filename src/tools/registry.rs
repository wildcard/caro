//! Tool Registry - Central registry for tool discovery and invocation
//!
//! Provides a unified interface for registering, discovering, and invoking
//! tools. Supports both synchronous and asynchronous tool execution with
//! caching for improved performance in multi-turn flows.

use super::{
    CommandTool, ContextTool, FileSystemTool, Tool, ToolCall, ToolCategory,
    ToolError, ToolInfo, ToolResult, ValidationTool,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// LRU cache for tool results
struct ResultCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    ttl_seconds: u64,
}

#[derive(Clone)]
struct CacheEntry {
    result: ToolResult,
    timestamp: std::time::SystemTime,
}

impl ResultCache {
    fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            ttl_seconds,
        }
    }

    fn get(&self, key: &str) -> Option<ToolResult> {
        if let Some(entry) = self.entries.get(key) {
            let age = entry
                .timestamp
                .elapsed()
                .map(|d| d.as_secs())
                .unwrap_or(u64::MAX);

            if age < self.ttl_seconds {
                return Some(entry.result.clone());
            }
        }
        None
    }

    fn insert(&mut self, key: String, result: ToolResult) {
        // Evict oldest entries if at capacity
        if self.entries.len() >= self.max_size {
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            }
        }

        self.entries.insert(
            key,
            CacheEntry {
                result,
                timestamp: std::time::SystemTime::now(),
            },
        );
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Central registry for all tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    cache: RwLock<ResultCache>,
    /// Enable result caching
    caching_enabled: bool,
    /// Track tool usage for telemetry
    usage_stats: RwLock<HashMap<String, UsageStats>>,
}

#[derive(Debug, Clone, Default)]
struct UsageStats {
    invocation_count: u64,
    total_time_ms: u64,
    error_count: u64,
    cache_hits: u64,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            cache: RwLock::new(ResultCache::new(100, 300)), // 100 entries, 5 min TTL
            caching_enabled: true,
            usage_stats: RwLock::new(HashMap::new()),
        };

        // Register default tools
        registry.register(Arc::new(FileSystemTool::new()));
        registry.register(Arc::new(CommandTool::new()));
        registry.register(Arc::new(ContextTool::new()));
        registry.register(Arc::new(ValidationTool::new()));

        registry
    }
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            cache: RwLock::new(ResultCache::new(100, 300)),
            caching_enabled: true,
            usage_stats: RwLock::new(HashMap::new()),
        }
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        info!("Registering tool: {}", name);
        self.tools.insert(name, tool);
    }

    /// Unregister a tool
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.remove(name)
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// List all registered tools
    pub fn list(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
                category: t.category(),
                parameters: t.parameters(),
            })
            .collect()
    }

    /// List tools by category
    pub fn list_by_category(&self, category: ToolCategory) -> Vec<ToolInfo> {
        self.tools
            .values()
            .filter(|t| t.category() == category)
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
                category: t.category(),
                parameters: t.parameters(),
            })
            .collect()
    }

    /// Enable or disable result caching
    pub fn set_caching(&mut self, enabled: bool) {
        self.caching_enabled = enabled;
    }

    /// Clear the result cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Generate a cache key for a tool call
    fn cache_key(call: &ToolCall) -> String {
        format!(
            "{}:{}",
            call.tool_name,
            serde_json::to_string(&call.params).unwrap_or_default()
        )
    }

    /// Invoke a tool by name with parameters
    pub async fn invoke(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        let start = Instant::now();

        // Check cache first
        if self.caching_enabled {
            let cache_key = Self::cache_key(call);
            if let Some(cached) = self.cache.read().await.get(&cache_key) {
                debug!("Cache hit for tool: {}", call.tool_name);

                // Update stats
                {
                    let mut stats = self.usage_stats.write().await;
                    let entry = stats.entry(call.tool_name.clone()).or_default();
                    entry.cache_hits += 1;
                }

                return Ok(cached);
            }
        }

        // Find the tool
        let tool = self.tools.get(&call.tool_name).ok_or_else(|| ToolError::NotFound {
            name: call.tool_name.clone(),
        })?;

        debug!("Invoking tool: {}", call.tool_name);

        // Execute the tool
        let result = tool.execute(&call.params).await;

        let elapsed = start.elapsed().as_millis() as u64;

        // Update stats
        {
            let mut stats = self.usage_stats.write().await;
            let entry = stats.entry(call.tool_name.clone()).or_default();
            entry.invocation_count += 1;
            entry.total_time_ms += elapsed;
            if !result.success {
                entry.error_count += 1;
            }
        }

        // Cache successful results
        if self.caching_enabled && result.success {
            let cache_key = Self::cache_key(call);
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Invoke multiple tools in parallel
    pub async fn invoke_batch(&self, calls: Vec<ToolCall>) -> Vec<Result<ToolResult, ToolError>> {
        let futures: Vec<_> = calls.iter().map(|call| self.invoke(call)).collect();

        futures::future::join_all(futures).await
    }

    /// Get usage statistics for all tools
    pub async fn get_stats(&self) -> HashMap<String, (u64, u64, u64, u64)> {
        let stats = self.usage_stats.read().await;
        stats
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    (
                        v.invocation_count,
                        v.total_time_ms,
                        v.error_count,
                        v.cache_hits,
                    ),
                )
            })
            .collect()
    }

    /// Reset usage statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.usage_stats.write().await;
        stats.clear();
    }

    /// Search for tools matching a query
    pub fn search(&self, query: &str) -> Vec<ToolInfo> {
        let query_lower = query.to_lowercase();

        self.tools
            .values()
            .filter(|t| {
                t.name().to_lowercase().contains(&query_lower)
                    || t.description().to_lowercase().contains(&query_lower)
            })
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
                category: t.category(),
                parameters: t.parameters(),
            })
            .collect()
    }
}

/// Builder for constructing a ToolRegistry with custom configuration
pub struct ToolRegistryBuilder {
    tools: Vec<Arc<dyn Tool>>,
    cache_size: usize,
    cache_ttl: u64,
    caching_enabled: bool,
    include_defaults: bool,
}

impl Default for ToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistryBuilder {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            cache_size: 100,
            cache_ttl: 300,
            caching_enabled: true,
            include_defaults: true,
        }
    }

    /// Add a custom tool
    pub fn with_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Set cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }

    /// Set cache TTL in seconds
    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache_ttl = ttl_seconds;
        self
    }

    /// Enable or disable caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.caching_enabled = enabled;
        self
    }

    /// Include or exclude default tools
    pub fn with_defaults(mut self, include: bool) -> Self {
        self.include_defaults = include;
        self
    }

    /// Build the registry
    pub fn build(self) -> ToolRegistry {
        let mut registry = ToolRegistry {
            tools: HashMap::new(),
            cache: RwLock::new(ResultCache::new(self.cache_size, self.cache_ttl)),
            caching_enabled: self.caching_enabled,
            usage_stats: RwLock::new(HashMap::new()),
        };

        // Register default tools if requested
        if self.include_defaults {
            registry.register(Arc::new(FileSystemTool::new()));
            registry.register(Arc::new(CommandTool::new()));
            registry.register(Arc::new(ContextTool::new()));
            registry.register(Arc::new(ValidationTool::new()));
        }

        // Register custom tools
        for tool in self.tools {
            registry.register(tool);
        }

        registry
    }
}

/// Convenience functions for common tool operations
impl ToolRegistry {
    /// Check if a file exists
    pub async fn file_exists(&self, path: &str) -> Result<bool, ToolError> {
        let call = ToolCall::new("filesystem")
            .with_param("operation", "exists")
            .with_path("path", path);

        let result = self.invoke(&call).await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// Check if a command is available
    pub async fn command_available(&self, command: &str) -> Result<bool, ToolError> {
        let call = ToolCall::new("command")
            .with_param("operation", "available")
            .with_param("command", command);

        let result = self.invoke(&call).await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// Validate a command for safety
    pub async fn validate_command(&self, command: &str) -> Result<ToolResult, ToolError> {
        let call = ToolCall::new("validation")
            .with_param("operation", "validate")
            .with_param("command", command);

        self.invoke(&call).await
    }

    /// Get full system context
    pub async fn get_context(&self) -> Result<ToolResult, ToolError> {
        let call = ToolCall::new("context").with_param("operation", "full");

        self.invoke(&call).await
    }

    /// Get command risk score
    pub async fn get_risk_score(&self, command: &str) -> Result<i64, ToolError> {
        let call = ToolCall::new("validation")
            .with_param("operation", "risk_score")
            .with_param("command", command);

        let result = self.invoke(&call).await?;
        match result.data {
            super::ToolData::Integer(score) => Ok(score),
            _ => Ok(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_default() {
        let registry = ToolRegistry::default();

        // Check default tools are registered
        assert!(registry.get("filesystem").is_some());
        assert!(registry.get("command").is_some());
        assert!(registry.get("context").is_some());
        assert!(registry.get("validation").is_some());
    }

    #[tokio::test]
    async fn test_registry_list() {
        let registry = ToolRegistry::default();
        let tools = registry.list();

        assert_eq!(tools.len(), 4);
    }

    #[tokio::test]
    async fn test_registry_invoke() {
        let registry = ToolRegistry::default();

        let call = ToolCall::new("filesystem")
            .with_param("operation", "exists")
            .with_path("path", "/tmp");

        let result = registry.invoke(&call).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_registry_not_found() {
        let registry = ToolRegistry::default();

        let call = ToolCall::new("nonexistent");
        let result = registry.invoke(&call).await;

        assert!(result.is_err());
        if let Err(ToolError::NotFound { name }) = result {
            assert_eq!(name, "nonexistent");
        }
    }

    #[tokio::test]
    async fn test_registry_caching() {
        let registry = ToolRegistry::default();

        let call = ToolCall::new("filesystem")
            .with_param("operation", "exists")
            .with_path("path", "/tmp");

        // First call
        let _ = registry.invoke(&call).await;

        // Second call should hit cache
        let _ = registry.invoke(&call).await;

        let stats = registry.get_stats().await;
        if let Some((invocations, _, _, cache_hits)) = stats.get("filesystem") {
            assert_eq!(*invocations, 1); // Only one actual invocation
            assert_eq!(*cache_hits, 1); // One cache hit
        }
    }

    #[tokio::test]
    async fn test_registry_batch() {
        let registry = ToolRegistry::default();

        let calls = vec![
            ToolCall::new("filesystem")
                .with_param("operation", "exists")
                .with_path("path", "/tmp"),
            ToolCall::new("command")
                .with_param("operation", "available")
                .with_param("command", "ls"),
        ];

        let results = registry.invoke_batch(calls).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[tokio::test]
    async fn test_registry_builder() {
        let registry = ToolRegistryBuilder::new()
            .with_cache_size(50)
            .with_cache_ttl(60)
            .with_defaults(true)
            .build();

        assert!(registry.get("filesystem").is_some());
    }

    #[tokio::test]
    async fn test_convenience_file_exists() {
        let registry = ToolRegistry::default();

        let exists = registry.file_exists("/tmp").await;
        assert!(exists.is_ok());
        assert!(exists.unwrap());
    }

    #[tokio::test]
    async fn test_convenience_command_available() {
        let registry = ToolRegistry::default();

        let available = registry.command_available("ls").await;
        assert!(available.is_ok());
        assert!(available.unwrap());
    }

    #[tokio::test]
    async fn test_convenience_validate() {
        let registry = ToolRegistry::default();

        let result = registry.validate_command("ls -la").await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_registry_search() {
        let registry = ToolRegistry::default();

        let results = registry.search("file");
        assert!(!results.is_empty());
        assert!(results.iter().any(|t| t.name == "filesystem"));
    }

    #[tokio::test]
    async fn test_risk_score_convenience() {
        let registry = ToolRegistry::default();

        let safe_score = registry.get_risk_score("echo hello").await;
        assert!(safe_score.is_ok());
        assert_eq!(safe_score.unwrap(), 0);

        let dangerous_score = registry.get_risk_score("rm -rf /").await;
        assert!(dangerous_score.is_ok());
        assert_eq!(dangerous_score.unwrap(), 100);
    }
}
