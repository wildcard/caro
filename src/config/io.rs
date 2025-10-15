//! Configuration import/export module
//!
//! Provides functionality for exporting and importing configuration
//! with format conversion and validation.

use crate::config::{ConfigurationState, ValidationRules, ValidationReport};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Configuration I/O handler
#[derive(Debug)]
pub struct ConfigIO {
    /// Validation rules for import/export
    validation_rules: ValidationRules,
    
    /// Supported format converters
    converters: Vec<Box<dyn FormatConverter>>,
}

impl ConfigIO {
    /// Create new configuration I/O handler
    pub fn new() -> Self {
        Self {
            validation_rules: ValidationRules::new(),
            converters: vec![
                Box::new(TomlConverter),
                Box::new(JsonConverter),
                Box::new(YamlConverter),
            ],
        }
    }
    
    /// Create production I/O handler with strict validation
    pub fn production() -> Self {
        Self {
            validation_rules: ValidationRules::production(),
            converters: vec![
                Box::new(TomlConverter),
                Box::new(JsonConverter),
                Box::new(YamlConverter),
            ],
        }
    }
    
    /// Export configuration to file
    pub async fn export(
        &self,
        config: &ConfigurationState,
        path: &Path,
        format: ExportFormat,
    ) -> Result<ExportResult> {
        let start = std::time::Instant::now();
        
        // Validate configuration before export
        let validation = self.validation_rules.validate_config_state(config)?;
        if !validation.is_valid() {
            warn!("Exporting configuration with validation warnings");
        }
        
        // Convert to desired format
        let content = match format {
            ExportFormat::Toml => TomlConverter.serialize(config)?,
            ExportFormat::Json => JsonConverter.serialize(config)?,
            ExportFormat::Yaml => YamlConverter.serialize(config)?,
            ExportFormat::Shell => self.export_shell_env(config)?,
        };
        
        // Create backup if file exists
        let backup_path = if path.exists() {
            let backup = self.create_backup(path)?;
            Some(backup)
        } else {
            None
        };
        
        // Write to file
        fs::write(path, &content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        
        // Set appropriate permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(path, perms)?;
        }
        
        let result = ExportResult {
            path: path.to_path_buf(),
            format,
            size_bytes: content.len(),
            backup_path,
            validation_report: Some(validation),
            export_time: start.elapsed(),
        };
        
        info!("Configuration exported to {}: {} bytes", path.display(), result.size_bytes);
        Ok(result)
    }
    
    /// Import configuration from file
    pub async fn import(
        &self,
        path: &Path,
        format: Option<ImportFormat>,
    ) -> Result<ImportResult> {
        let start = std::time::Instant::now();
        
        // Check file exists and is readable
        if !path.exists() {
            anyhow::bail!("Configuration file not found: {}", path.display());
        }
        
        // Read file content
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        
        // Detect format if not specified
        let format = format.unwrap_or_else(|| self.detect_format(path, &content));
        
        // Parse configuration
        let config = match format {
            ImportFormat::Toml => TomlConverter.deserialize(&content)?,
            ImportFormat::Json => JsonConverter.deserialize(&content)?,
            ImportFormat::Yaml => YamlConverter.deserialize(&content)?,
            ImportFormat::Auto => self.auto_parse(&content)?,
        };
        
        // Validate imported configuration
        let validation = self.validation_rules.validate_config_state(&config)?;
        
        // Apply migrations if needed
        let migrated = self.apply_migrations(config)?;
        
        let result = ImportResult {
            config: migrated,
            source_path: path.to_path_buf(),
            format,
            validation_report: validation,
            import_time: start.elapsed(),
        };
        
        if result.validation_report.is_valid() {
            info!("Configuration imported successfully from {}", path.display());
        } else {
            warn!("Configuration imported with {} warnings", result.validation_report.warnings.len());
        }
        
        Ok(result)
    }
    
    /// Export configuration as shell environment variables
    fn export_shell_env(&self, config: &ConfigurationState) -> Result<String> {
        let mut env_vars = Vec::new();
        
        // Export basic settings
        env_vars.push(format!("export CMDAI_BACKEND=\"{}\"", config.preferred_backend));
        env_vars.push(format!("export CMDAI_HISTORY_ENABLED=\"{}\"", config.history_enabled));
        env_vars.push(format!("export CMDAI_PRIVACY=\"{}\"", config.privacy_mode));
        env_vars.push(format!("export CMDAI_SAFETY=\"{}\"", config.safety_level));
        env_vars.push(format!("export CMDAI_STREAMING=\"{}\"", config.streaming_enabled));
        env_vars.push(format!("export CMDAI_COLOR=\"{}\"", config.color_output));
        env_vars.push(format!("export CMDAI_VERBOSITY=\"{}\"", config.verbosity_level));
        
        // Export retention policy
        env_vars.push(format!("export CMDAI_RETENTION_DAYS=\"{}\"", config.retention_policy.max_age_days));
        env_vars.push(format!("export CMDAI_RETENTION_ENTRIES=\"{}\"", config.retention_policy.max_entries));
        env_vars.push(format!("export CMDAI_RETENTION_SIZE_MB=\"{}\"", config.retention_policy.max_size_mb));
        
        // Export backend configurations
        for (name, backend) in &config.backend_configs {
            let prefix = format!("CMDAI_BACKEND_{}", name.to_uppercase());
            if let Some(endpoint) = &backend.endpoint {
                env_vars.push(format!("export {}=\"{}\"", prefix, endpoint));
            }
            if let Some(key) = &backend.api_key {
                env_vars.push(format!("export {}_KEY=\"{}\"", prefix, key));
            }
        }
        
        Ok(env_vars.join("\n"))
    }
    
    /// Create backup of existing configuration file
    fn create_backup(&self, path: &Path) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = path.with_extension(format!("bak.{}", timestamp));
        
        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;
        
        debug!("Created configuration backup at {}", backup_path.display());
        Ok(backup_path)
    }
    
    /// Detect configuration format from file extension or content
    fn detect_format(&self, path: &Path, content: &str) -> ImportFormat {
        // Check file extension
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("toml") => return ImportFormat::Toml,
                Some("json") => return ImportFormat::Json,
                Some("yaml") | Some("yml") => return ImportFormat::Yaml,
                _ => {}
            }
        }
        
        // Try to detect from content
        if content.trim().starts_with('{') {
            ImportFormat::Json
        } else if content.contains("---") || content.contains(": ") {
            ImportFormat::Yaml
        } else {
            ImportFormat::Toml
        }
    }
    
    /// Auto-parse configuration trying different formats
    fn auto_parse(&self, content: &str) -> Result<ConfigurationState> {
        // Try each converter
        for converter in &self.converters {
            if let Ok(config) = converter.deserialize(content) {
                debug!("Successfully parsed configuration as {}", converter.format_name());
                return Ok(config);
            }
        }
        
        anyhow::bail!("Failed to parse configuration in any supported format")
    }
    
    /// Apply migrations to imported configuration
    fn apply_migrations(&self, mut config: ConfigurationState) -> Result<ConfigurationState> {
        // Check version and apply necessary migrations
        // This would be extended with actual migration logic
        debug!("Checking for configuration migrations");
        
        // Example migration: ensure all required fields are set
        if config.fallback_chain.is_empty() {
            config.fallback_chain = vec![
                crate::backends::BackendType::VLlm,
                crate::backends::BackendType::Ollama,
                crate::backends::BackendType::Auto,
            ];
            debug!("Applied migration: set default fallback chain");
        }
        
        Ok(config)
    }
    
    /// Merge two configurations
    pub fn merge(
        &self,
        base: &ConfigurationState,
        overlay: &ConfigurationState,
        strategy: MergeStrategy,
    ) -> Result<ConfigurationState> {
        let merged = match strategy {
            MergeStrategy::ReplaceAll => overlay.clone(),
            MergeStrategy::PreferOverlay => self.merge_prefer_overlay(base, overlay)?,
            MergeStrategy::PreferBase => self.merge_prefer_base(base, overlay)?,
            MergeStrategy::Deep => self.deep_merge(base, overlay)?,
        };
        
        // Validate merged configuration
        let validation = self.validation_rules.validate_config_state(&merged)?;
        if !validation.is_valid() {
            warn!("Merged configuration has validation issues");
        }
        
        Ok(merged)
    }
    
    /// Merge preferring overlay values
    fn merge_prefer_overlay(
        &self,
        base: &ConfigurationState,
        overlay: &ConfigurationState,
    ) -> Result<ConfigurationState> {
        let mut merged = base.clone();
        
        // Override with overlay values
        merged.preferred_backend = overlay.preferred_backend.clone();
        merged.backend_configs.extend(overlay.backend_configs.clone());
        merged.fallback_chain = overlay.fallback_chain.clone();
        merged.history_enabled = overlay.history_enabled;
        merged.retention_policy = overlay.retention_policy.clone();
        merged.privacy_mode = overlay.privacy_mode;
        merged.auto_cleanup_days = overlay.auto_cleanup_days;
        merged.safety_level = overlay.safety_level;
        merged.confirmation_required = overlay.confirmation_required.clone();
        merged.custom_safety_patterns.extend(overlay.custom_safety_patterns.clone());
        merged.streaming_enabled = overlay.streaming_enabled;
        merged.color_output = overlay.color_output;
        merged.verbosity_level = overlay.verbosity_level;
        
        Ok(merged)
    }
    
    /// Merge preferring base values
    fn merge_prefer_base(
        &self,
        base: &ConfigurationState,
        overlay: &ConfigurationState,
    ) -> Result<ConfigurationState> {
        let mut merged = overlay.clone();
        
        // Keep base values where they exist
        for (key, value) in &base.backend_configs {
            merged.backend_configs.entry(key.clone()).or_insert(value.clone());
        }
        
        if !base.custom_safety_patterns.is_empty() {
            merged.custom_safety_patterns = base.custom_safety_patterns.clone();
        }
        
        Ok(merged)
    }
    
    /// Deep merge configurations
    fn deep_merge(
        &self,
        base: &ConfigurationState,
        overlay: &ConfigurationState,
    ) -> Result<ConfigurationState> {
        let mut merged = base.clone();
        
        // Deep merge backend configs
        for (key, value) in &overlay.backend_configs {
            merged.backend_configs.insert(key.clone(), value.clone());
        }
        
        // Merge arrays by combining unique values
        let mut patterns = base.custom_safety_patterns.clone();
        patterns.extend(overlay.custom_safety_patterns.clone());
        patterns.dedup();
        merged.custom_safety_patterns = patterns;
        
        // Use overlay for single values
        merged.preferred_backend = overlay.preferred_backend.clone();
        merged.history_enabled = overlay.history_enabled;
        merged.safety_level = overlay.safety_level;
        
        Ok(merged)
    }
}

impl Default for ConfigIO {
    fn default() -> Self {
        Self::new()
    }
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Toml,
    Json,
    Yaml,
    Shell,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml => write!(f, "TOML"),
            Self::Json => write!(f, "JSON"),
            Self::Yaml => write!(f, "YAML"),
            Self::Shell => write!(f, "Shell"),
        }
    }
}

/// Import format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Toml,
    Json,
    Yaml,
    Auto,
}

impl std::fmt::Display for ImportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml => write!(f, "TOML"),
            Self::Json => write!(f, "JSON"),
            Self::Yaml => write!(f, "YAML"),
            Self::Auto => write!(f, "Auto-detect"),
        }
    }
}

/// Configuration merge strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Replace all values with overlay
    ReplaceAll,
    /// Prefer overlay values when present
    PreferOverlay,
    /// Prefer base values when present
    PreferBase,
    /// Deep merge with array concatenation
    Deep,
}

/// Export operation result
#[derive(Debug)]
pub struct ExportResult {
    /// Path where configuration was exported
    pub path: PathBuf,
    
    /// Format used for export
    pub format: ExportFormat,
    
    /// Size of exported file in bytes
    pub size_bytes: usize,
    
    /// Path to backup if created
    pub backup_path: Option<PathBuf>,
    
    /// Validation report
    pub validation_report: Option<ValidationReport>,
    
    /// Time taken to export
    pub export_time: std::time::Duration,
}

/// Import operation result
#[derive(Debug)]
pub struct ImportResult {
    /// Imported configuration
    pub config: ConfigurationState,
    
    /// Source file path
    pub source_path: PathBuf,
    
    /// Format detected/used
    pub format: ImportFormat,
    
    /// Validation report
    pub validation_report: ValidationReport,
    
    /// Time taken to import
    pub import_time: std::time::Duration,
}

/// Format converter trait
trait FormatConverter: Send + Sync + std::fmt::Debug {
    /// Serialize configuration to string
    fn serialize(&self, config: &ConfigurationState) -> Result<String>;
    
    /// Deserialize configuration from string
    fn deserialize(&self, content: &str) -> Result<ConfigurationState>;
    
    /// Format name for logging
    fn format_name(&self) -> &str;
}

/// TOML format converter
#[derive(Debug)]
struct TomlConverter;

impl FormatConverter for TomlConverter {
    fn serialize(&self, config: &ConfigurationState) -> Result<String> {
        toml::to_string_pretty(config)
            .context("Failed to serialize configuration to TOML")
    }
    
    fn deserialize(&self, content: &str) -> Result<ConfigurationState> {
        toml::from_str(content)
            .context("Failed to deserialize configuration from TOML")
    }
    
    fn format_name(&self) -> &str {
        "TOML"
    }
}

/// JSON format converter
#[derive(Debug)]
struct JsonConverter;

impl FormatConverter for JsonConverter {
    fn serialize(&self, config: &ConfigurationState) -> Result<String> {
        serde_json::to_string_pretty(config)
            .context("Failed to serialize configuration to JSON")
    }
    
    fn deserialize(&self, content: &str) -> Result<ConfigurationState> {
        serde_json::from_str(content)
            .context("Failed to deserialize configuration from JSON")
    }
    
    fn format_name(&self) -> &str {
        "JSON"
    }
}

/// YAML format converter
#[derive(Debug)]
struct YamlConverter;

impl FormatConverter for YamlConverter {
    fn serialize(&self, config: &ConfigurationState) -> Result<String> {
        serde_yaml::to_string(config)
            .context("Failed to serialize configuration to YAML")
    }
    
    fn deserialize(&self, content: &str) -> Result<ConfigurationState> {
        serde_yaml::from_str(content)
            .context("Failed to deserialize configuration from YAML")
    }
    
    fn format_name(&self) -> &str {
        "YAML"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_format_detection() {
        let io = ConfigIO::new();
        
        // Test file extension detection
        assert_eq!(
            io.detect_format(Path::new("config.toml"), ""),
            ImportFormat::Toml
        );
        assert_eq!(
            io.detect_format(Path::new("config.json"), ""),
            ImportFormat::Json
        );
        assert_eq!(
            io.detect_format(Path::new("config.yaml"), ""),
            ImportFormat::Yaml
        );
        
        // Test content detection
        assert_eq!(
            io.detect_format(Path::new("config"), "{\"key\": \"value\"}"),
            ImportFormat::Json
        );
        assert_eq!(
            io.detect_format(Path::new("config"), "key: value"),
            ImportFormat::Yaml
        );
        assert_eq!(
            io.detect_format(Path::new("config"), "key = \"value\""),
            ImportFormat::Toml
        );
    }
    
    #[test]
    fn test_toml_converter() {
        let converter = TomlConverter;
        let config = ConfigurationState::default();
        
        // Test round-trip
        let serialized = converter.serialize(&config).unwrap();
        let deserialized = converter.deserialize(&serialized).unwrap();
        
        assert_eq!(config.preferred_backend, deserialized.preferred_backend);
        assert_eq!(config.history_enabled, deserialized.history_enabled);
    }
    
    #[test]
    fn test_json_converter() {
        let converter = JsonConverter;
        let config = ConfigurationState::default();
        
        // Test round-trip
        let serialized = converter.serialize(&config).unwrap();
        let deserialized = converter.deserialize(&serialized).unwrap();
        
        assert_eq!(config.preferred_backend, deserialized.preferred_backend);
        assert_eq!(config.history_enabled, deserialized.history_enabled);
    }
    
    #[test]
    fn test_yaml_converter() {
        let converter = YamlConverter;
        let config = ConfigurationState::default();
        
        // Test round-trip
        let serialized = converter.serialize(&config).unwrap();
        let deserialized = converter.deserialize(&serialized).unwrap();
        
        assert_eq!(config.preferred_backend, deserialized.preferred_backend);
        assert_eq!(config.history_enabled, deserialized.history_enabled);
    }
    
    #[tokio::test]
    async fn test_export_import_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let io = ConfigIO::new();
        let config = ConfigurationState::default();
        
        // Export configuration
        let export_result = io.export(&config, &config_path, ExportFormat::Toml).await.unwrap();
        assert!(config_path.exists());
        assert!(export_result.size_bytes > 0);
        
        // Import configuration
        let import_result = io.import(&config_path, None).await.unwrap();
        assert_eq!(config.preferred_backend, import_result.config.preferred_backend);
    }
    
    #[test]
    fn test_merge_strategies() {
        let io = ConfigIO::new();
        
        let mut base = ConfigurationState::default();
        base.history_enabled = true;
        base.auto_cleanup_days = 30;
        
        let mut overlay = ConfigurationState::default();
        overlay.history_enabled = false;
        overlay.auto_cleanup_days = 60;
        
        // Test replace all
        let merged = io.merge(&base, &overlay, MergeStrategy::ReplaceAll).unwrap();
        assert_eq!(merged.history_enabled, overlay.history_enabled);
        assert_eq!(merged.auto_cleanup_days, overlay.auto_cleanup_days);
        
        // Test prefer overlay
        let merged = io.merge(&base, &overlay, MergeStrategy::PreferOverlay).unwrap();
        assert_eq!(merged.history_enabled, overlay.history_enabled);
        
        // Test prefer base
        let merged = io.merge(&base, &overlay, MergeStrategy::PreferBase).unwrap();
        assert_eq!(merged.auto_cleanup_days, overlay.auto_cleanup_days);
    }
}