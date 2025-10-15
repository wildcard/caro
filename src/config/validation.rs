//! Configuration validation module
//!
//! Provides comprehensive validation for user configuration with
//! production-ready error messages and migration support.

use crate::config::{ConfigurationState, BackendConfig, RetentionPolicy, 
                    PrivacyLevel, VerbosityLevel};
use crate::backends::BackendType;
use crate::models::{SafetyLevel, RiskLevel};
use crate::models::{UserConfiguration, LogLevel, ShellType};
use anyhow::{Result, Context};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Configuration validation rules engine
#[derive(Debug, Clone)]
pub struct ValidationRules {
    /// Strict mode for production environments
    pub strict_mode: bool,
    
    /// Allow deprecated configuration keys
    pub allow_deprecated: bool,
    
    /// Custom validation rules
    pub custom_rules: Vec<Box<dyn ValidationRule>>,
    
    /// Performance constraints
    pub performance_constraints: PerformanceConstraints,
    
    /// Security requirements
    pub security_requirements: SecurityRequirements,
}

impl ValidationRules {
    /// Create new validation rules with defaults
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            allow_deprecated: true,
            custom_rules: Vec::new(),
            performance_constraints: PerformanceConstraints::default(),
            security_requirements: SecurityRequirements::default(),
        }
    }
    
    /// Create production validation rules
    pub fn production() -> Self {
        Self {
            strict_mode: true,
            allow_deprecated: false,
            custom_rules: Vec::new(),
            performance_constraints: PerformanceConstraints::production(),
            security_requirements: SecurityRequirements::production(),
        }
    }
    
    /// Validate a UserConfiguration
    pub fn validate_user_config(&self, config: &UserConfiguration) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate safety level
        self.validate_safety_level(&config.safety_level, &mut report)?;
        
        // Validate log level
        self.validate_log_level(&config.log_level, &mut report)?;
        
        // Validate shell type
        if let Some(shell) = &config.default_shell {
            self.validate_shell_type(shell, &mut report)?;
        }
        
        // Validate cache settings
        self.validate_cache_settings(config.cache_max_size_gb, &mut report)?;
        
        // Validate model settings
        if let Some(model) = &config.default_model {
            self.validate_model_name(model, &mut report)?;
        }
        
        // Validate color settings
        self.validate_color_settings(config.enable_colors, &mut report)?;
        
        // Validate history settings
        self.validate_history_settings(config.enable_history, config.history_max_entries, &mut report)?;
        
        // Check for deprecations
        if !self.allow_deprecated {
            self.check_deprecated_settings(config, &mut report)?;
        }
        
        // Apply custom rules
        for rule in &self.custom_rules {
            rule.validate_user_config(config, &mut report)?;
        }
        
        Ok(report)
    }
    
    /// Validate a ConfigurationState
    pub fn validate_config_state(&self, state: &ConfigurationState) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate backend configuration
        self.validate_backend_config(&state.preferred_backend, &state.backend_configs, &mut report)?;
        
        // Validate fallback chain
        self.validate_fallback_chain(&state.fallback_chain, &mut report)?;
        
        // Validate retention policy
        self.validate_retention_policy(&state.retention_policy, &mut report)?;
        
        // Validate privacy level
        self.validate_privacy_level(&state.privacy_mode, &mut report)?;
        
        // Validate safety configuration
        self.validate_safety_config(&state.safety_level, &state.confirmation_required, &mut report)?;
        
        // Validate custom patterns
        self.validate_custom_patterns(&state.custom_safety_patterns, &mut report)?;
        
        // Validate UI settings
        self.validate_ui_settings(state.streaming_enabled, state.color_output, &state.verbosity_level, &mut report)?;
        
        // Check performance constraints
        if self.strict_mode {
            self.check_performance_constraints(state, &mut report)?;
        }
        
        // Check security requirements
        if self.strict_mode {
            self.check_security_requirements(state, &mut report)?;
        }
        
        // Apply custom rules
        for rule in &self.custom_rules {
            rule.validate_config_state(state, &mut report)?;
        }
        
        Ok(report)
    }
    
    /// Validate safety level
    fn validate_safety_level(&self, level: &SafetyLevel, report: &mut ValidationReport) -> Result<()> {
        match level {
            SafetyLevel::Minimal if self.strict_mode => {
                report.add_warning("Minimal safety level not recommended for production");
            }
            SafetyLevel::Interactive => {
                report.add_info("Interactive safety requires user confirmation");
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Validate log level
    fn validate_log_level(&self, level: &LogLevel, report: &mut ValidationReport) -> Result<()> {
        match level {
            LogLevel::Trace if self.strict_mode => {
                report.add_warning("Trace logging may impact performance");
            }
            LogLevel::Silent => {
                report.add_warning("Silent logging makes debugging difficult");
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Validate shell type
    fn validate_shell_type(&self, shell: &ShellType, report: &mut ValidationReport) -> Result<()> {
        // Check shell availability
        let shell_cmd = match shell {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::Sh => "sh",
            ShellType::Powershell => "powershell",
            ShellType::Cmd => "cmd",
        };
        
        if !self.is_shell_available(shell_cmd) {
            report.add_error(format!("Shell '{}' not available on system", shell_cmd));
        }
        
        Ok(())
    }
    
    /// Check if shell is available
    fn is_shell_available(&self, shell: &str) -> bool {
        std::process::Command::new("which")
            .arg(shell)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    /// Validate cache settings
    fn validate_cache_settings(&self, size_gb: f64, report: &mut ValidationReport) -> Result<()> {
        if size_gb < 0.1 {
            report.add_error("Cache size too small (minimum 0.1 GB)");
        } else if size_gb > 100.0 {
            report.add_warning("Cache size very large (>100 GB)");
        }
        
        // Check available disk space
        if let Ok(available) = self.get_available_disk_space() {
            let required = (size_gb * 1024.0 * 1024.0 * 1024.0) as u64;
            if required > available {
                report.add_error(format!(
                    "Insufficient disk space for cache (required: {} GB, available: {} GB)",
                    size_gb,
                    available / (1024 * 1024 * 1024)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get available disk space
    fn get_available_disk_space(&self) -> Result<u64> {
        // This would use platform-specific code to check disk space
        // For now, return a dummy value
        Ok(100 * 1024 * 1024 * 1024) // 100 GB
    }
    
    /// Validate model name
    fn validate_model_name(&self, model: &str, report: &mut ValidationReport) -> Result<()> {
        // Check model format
        if model.is_empty() {
            report.add_error("Model name cannot be empty");
        } else if model.len() > 256 {
            report.add_error("Model name too long (max 256 characters)");
        }
        
        // Validate model path if it's a file path
        if model.starts_with('/') || model.starts_with("./") {
            let path = Path::new(model);
            if !path.exists() {
                report.add_error(format!("Model file not found: {}", model));
            }
        }
        
        Ok(())
    }
    
    /// Validate color settings
    fn validate_color_settings(&self, enabled: bool, report: &mut ValidationReport) -> Result<()> {
        if enabled && std::env::var("NO_COLOR").is_ok() {
            report.add_warning("Colors enabled but NO_COLOR environment variable is set");
        }
        Ok(())
    }
    
    /// Validate history settings
    fn validate_history_settings(&self, enabled: bool, max_entries: usize, report: &mut ValidationReport) -> Result<()> {
        if enabled {
            if max_entries < 10 {
                report.add_warning("History max entries very low (<10)");
            } else if max_entries > 1000000 {
                report.add_warning("History max entries very high (>1M)");
            }
        }
        Ok(())
    }
    
    /// Check for deprecated settings
    fn check_deprecated_settings(&self, _config: &UserConfiguration, report: &mut ValidationReport) -> Result<()> {
        // Check for deprecated patterns
        // This would be extended as settings are deprecated
        Ok(())
    }
    
    /// Validate backend configuration
    fn validate_backend_config(
        &self,
        preferred: &BackendType,
        configs: &HashMap<String, BackendConfig>,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Check preferred backend has configuration
        let backend_key = preferred.to_string().to_lowercase();
        if !configs.contains_key(&backend_key) && *preferred != BackendType::Auto {
            report.add_warning(format!("No configuration for preferred backend: {}", preferred));
        }
        
        // Validate each backend config
        for (name, config) in configs {
            self.validate_single_backend_config(name, config, report)?;
        }
        
        Ok(())
    }
    
    /// Validate single backend configuration
    fn validate_single_backend_config(
        &self,
        name: &str,
        config: &BackendConfig,
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Validate endpoint URL
        if let Some(endpoint) = &config.endpoint {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                report.add_error(format!("Invalid endpoint URL for {}: {}", name, endpoint));
            }
        }
        
        // Validate timeout
        if config.timeout_ms < 100 {
            report.add_error(format!("Timeout too short for {} (min 100ms)", name));
        } else if config.timeout_ms > 300000 {
            report.add_warning(format!("Timeout very long for {} (>5 minutes)", name));
        }
        
        // Validate max retries
        if config.max_retries > 10 {
            report.add_warning(format!("High retry count for {} (>10)", name));
        }
        
        // Validate model path
        if let Some(model_path) = &config.model_path {
            if !Path::new(model_path).exists() {
                report.add_error(format!("Model path not found for {}: {}", name, model_path));
            }
        }
        
        Ok(())
    }
    
    /// Validate fallback chain
    fn validate_fallback_chain(&self, chain: &[BackendType], report: &mut ValidationReport) -> Result<()> {
        if chain.is_empty() {
            report.add_warning("Empty fallback chain");
        }
        
        // Check for duplicates
        let mut seen = HashSet::new();
        for backend in chain {
            if !seen.insert(backend) {
                report.add_warning(format!("Duplicate backend in fallback chain: {}", backend));
            }
        }
        
        // Warn if Auto is not last
        if chain.contains(&BackendType::Auto) && chain.last() != Some(&BackendType::Auto) {
            report.add_warning("Auto backend should be last in fallback chain");
        }
        
        Ok(())
    }
    
    /// Validate retention policy
    fn validate_retention_policy(&self, policy: &RetentionPolicy, report: &mut ValidationReport) -> Result<()> {
        if policy.max_entries < 10 {
            report.add_warning("Very low max entries in retention policy (<10)");
        }
        
        if policy.max_age_days < 1 {
            report.add_error("Invalid max age days (must be >= 1)");
        } else if policy.max_age_days > 3650 {
            report.add_warning("Very long retention period (>10 years)");
        }
        
        if policy.max_size_mb < 1 {
            report.add_error("Invalid max size (must be >= 1 MB)");
        } else if policy.max_size_mb > 10000 {
            report.add_warning("Very large retention size (>10 GB)");
        }
        
        Ok(())
    }
    
    /// Validate privacy level
    fn validate_privacy_level(&self, level: &PrivacyLevel, report: &mut ValidationReport) -> Result<()> {
        match level {
            PrivacyLevel::Telemetry if self.strict_mode => {
                report.add_info("Telemetry enabled - ensure compliance with privacy policies");
            }
            PrivacyLevel::Paranoid => {
                report.add_info("Paranoid mode - some features may be limited");
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Validate safety configuration
    fn validate_safety_config(
        &self,
        level: &SafetyLevel,
        confirmation: &[RiskLevel],
        report: &mut ValidationReport,
    ) -> Result<()> {
        // Check consistency
        if *level == SafetyLevel::Strict && confirmation.is_empty() {
            report.add_warning("Strict safety level but no confirmation requirements");
        }
        
        if *level == SafetyLevel::Minimal && !confirmation.is_empty() {
            report.add_info("Minimal safety with confirmations - consider raising safety level");
        }
        
        Ok(())
    }
    
    /// Validate custom safety patterns
    fn validate_custom_patterns(&self, patterns: &[String], report: &mut ValidationReport) -> Result<()> {
        for pattern in patterns {
            // Try to compile as regex
            match regex::Regex::new(pattern) {
                Ok(_) => {}
                Err(e) => {
                    report.add_error(format!("Invalid regex pattern: {} - {}", pattern, e));
                }
            }
        }
        
        if patterns.len() > 100 {
            report.add_warning("Large number of custom patterns (>100) may impact performance");
        }
        
        Ok(())
    }
    
    /// Validate UI settings
    fn validate_ui_settings(
        &self,
        streaming: bool,
        colors: bool,
        verbosity: &VerbosityLevel,
        report: &mut ValidationReport,
    ) -> Result<()> {
        if streaming && *verbosity == VerbosityLevel::Silent {
            report.add_warning("Streaming enabled but verbosity is silent");
        }
        
        if colors && std::env::var("TERM").unwrap_or_default() == "dumb" {
            report.add_warning("Colors enabled but terminal doesn't support them");
        }
        
        Ok(())
    }
    
    /// Check performance constraints
    fn check_performance_constraints(&self, state: &ConfigurationState, report: &mut ValidationReport) -> Result<()> {
        // Check startup time constraints
        if state.backend_configs.len() > 5 {
            report.add_warning("Many backend configs may slow startup (>5)");
        }
        
        // Check inference time constraints
        for (name, config) in &state.backend_configs {
            if config.timeout_ms < self.performance_constraints.min_inference_timeout_ms {
                report.add_warning(format!(
                    "Backend {} timeout below recommended minimum ({} ms)",
                    name, self.performance_constraints.min_inference_timeout_ms
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check security requirements
    fn check_security_requirements(&self, state: &ConfigurationState, report: &mut ValidationReport) -> Result<()> {
        // Check safety level
        if state.safety_level == SafetyLevel::Minimal && self.security_requirements.require_safety_validation {
            report.add_error("Security policy requires higher safety level");
        }
        
        // Check privacy
        if state.privacy_mode == PrivacyLevel::Telemetry && self.security_requirements.disallow_telemetry {
            report.add_error("Security policy disallows telemetry");
        }
        
        // Check history
        if state.history_enabled && self.security_requirements.disallow_history {
            report.add_error("Security policy disallows history storage");
        }
        
        Ok(())
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation report containing errors, warnings, and info
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// Critical errors that must be fixed
    pub errors: Vec<String>,
    
    /// Warnings that should be reviewed
    pub warnings: Vec<String>,
    
    /// Informational messages
    pub info: Vec<String>,
    
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ValidationReport {
    /// Create new validation report
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Add an error
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
    
    /// Add info
    pub fn add_info(&mut self, msg: impl Into<String>) {
        self.info.push(msg.into());
    }
    
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    /// Get severity level
    pub fn severity(&self) -> ValidationSeverity {
        if !self.errors.is_empty() {
            ValidationSeverity::Error
        } else if !self.warnings.is_empty() {
            ValidationSeverity::Warning
        } else {
            ValidationSeverity::Ok
        }
    }
    
    /// Format report for display
    pub fn format(&self) -> String {
        let mut output = Vec::new();
        
        if !self.errors.is_empty() {
            output.push(format!("❌ {} Errors:", self.errors.len()));
            for error in &self.errors {
                output.push(format!("   • {}", error));
            }
        }
        
        if !self.warnings.is_empty() {
            output.push(format!("⚠️  {} Warnings:", self.warnings.len()));
            for warning in &self.warnings {
                output.push(format!("   • {}", warning));
            }
        }
        
        if !self.info.is_empty() {
            output.push(format!("ℹ️  {} Info:", self.info.len()));
            for info in &self.info {
                output.push(format!("   • {}", info));
            }
        }
        
        if output.is_empty() {
            output.push("✅ Configuration valid".to_string());
        }
        
        output.join("\n")
    }
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Ok,
    Warning,
    Error,
}

/// Performance constraints for validation
#[derive(Debug, Clone)]
pub struct PerformanceConstraints {
    /// Maximum startup time in milliseconds
    pub max_startup_time_ms: u64,
    
    /// Maximum inference time in milliseconds
    pub max_inference_time_ms: u64,
    
    /// Minimum inference timeout in milliseconds
    pub min_inference_timeout_ms: u64,
    
    /// Maximum memory usage in MB
    pub max_memory_usage_mb: u64,
}

impl Default for PerformanceConstraints {
    fn default() -> Self {
        Self {
            max_startup_time_ms: 100,
            max_inference_time_ms: 2000,
            min_inference_timeout_ms: 1000,
            max_memory_usage_mb: 500,
        }
    }
}

impl PerformanceConstraints {
    /// Production constraints
    pub fn production() -> Self {
        Self {
            max_startup_time_ms: 100,
            max_inference_time_ms: 2000,
            min_inference_timeout_ms: 2000,
            max_memory_usage_mb: 250,
        }
    }
}

/// Security requirements for validation
#[derive(Debug, Clone)]
pub struct SecurityRequirements {
    /// Require safety validation
    pub require_safety_validation: bool,
    
    /// Disallow telemetry
    pub disallow_telemetry: bool,
    
    /// Disallow history storage
    pub disallow_history: bool,
    
    /// Require encrypted storage
    pub require_encryption: bool,
}

impl Default for SecurityRequirements {
    fn default() -> Self {
        Self {
            require_safety_validation: true,
            disallow_telemetry: false,
            disallow_history: false,
            require_encryption: false,
        }
    }
}

impl SecurityRequirements {
    /// Production security requirements
    pub fn production() -> Self {
        Self {
            require_safety_validation: true,
            disallow_telemetry: false,
            disallow_history: false,
            require_encryption: true,
        }
    }
}

/// Custom validation rule trait
pub trait ValidationRule: Send + Sync + std::fmt::Debug {
    /// Validate UserConfiguration
    fn validate_user_config(&self, config: &UserConfiguration, report: &mut ValidationReport) -> Result<()>;
    
    /// Validate ConfigurationState
    fn validate_config_state(&self, state: &ConfigurationState, report: &mut ValidationReport) -> Result<()>;
    
    /// Rule name
    fn name(&self) -> &str;
}

/// Configuration migration for updating old configs
#[derive(Debug)]
pub struct ConfigMigration {
    /// Source version
    pub from_version: String,
    
    /// Target version
    pub to_version: String,
    
    /// Migration function
    pub migrate: Box<dyn Fn(&mut toml::Value) -> Result<()>>,
}

impl ConfigMigration {
    /// Apply migration
    pub fn apply(&self, config: &mut toml::Value) -> Result<()> {
        debug!("Applying migration from {} to {}", self.from_version, self.to_version);
        (self.migrate)(config)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_rules_creation() {
        let rules = ValidationRules::new();
        assert!(!rules.strict_mode);
        assert!(rules.allow_deprecated);
        
        let prod_rules = ValidationRules::production();
        assert!(prod_rules.strict_mode);
        assert!(!prod_rules.allow_deprecated);
    }
    
    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        
        assert!(report.is_valid());
        assert_eq!(report.severity(), ValidationSeverity::Ok);
        
        report.add_warning("Test warning");
        assert!(report.is_valid());
        assert_eq!(report.severity(), ValidationSeverity::Warning);
        
        report.add_error("Test error");
        assert!(!report.is_valid());
        assert_eq!(report.severity(), ValidationSeverity::Error);
        
        report.add_info("Test info");
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.info.len(), 1);
    }
    
    #[test]
    fn test_report_formatting() {
        let mut report = ValidationReport::new();
        report.add_error("Critical error");
        report.add_warning("Minor issue");
        report.add_info("Note");
        
        let formatted = report.format();
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("⚠️"));
        assert!(formatted.contains("ℹ️"));
        assert!(formatted.contains("Critical error"));
        assert!(formatted.contains("Minor issue"));
        assert!(formatted.contains("Note"));
    }
    
    #[test]
    fn test_performance_constraints() {
        let default_constraints = PerformanceConstraints::default();
        assert_eq!(default_constraints.max_startup_time_ms, 100);
        assert_eq!(default_constraints.max_inference_time_ms, 2000);
        
        let prod_constraints = PerformanceConstraints::production();
        assert_eq!(prod_constraints.max_memory_usage_mb, 250);
        assert_eq!(prod_constraints.min_inference_timeout_ms, 2000);
    }
    
    #[test]
    fn test_security_requirements() {
        let default_security = SecurityRequirements::default();
        assert!(default_security.require_safety_validation);
        assert!(!default_security.disallow_telemetry);
        
        let prod_security = SecurityRequirements::production();
        assert!(prod_security.require_safety_validation);
        assert!(prod_security.require_encryption);
    }
    
    #[test]
    fn test_cache_validation() {
        let rules = ValidationRules::new();
        let mut report = ValidationReport::new();
        
        // Test invalid cache size
        rules.validate_cache_settings(0.05, &mut report).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("too small"));
        
        // Test warning for large cache
        let mut report = ValidationReport::new();
        rules.validate_cache_settings(150.0, &mut report).unwrap();
        assert!(!report.warnings.is_empty());
        assert!(report.warnings[0].contains("very large"));
        
        // Test valid cache size
        let mut report = ValidationReport::new();
        rules.validate_cache_settings(1.0, &mut report).unwrap();
        assert!(report.errors.is_empty());
        assert!(report.warnings.is_empty());
    }
    
    #[test]
    fn test_pattern_validation() {
        let rules = ValidationRules::new();
        let mut report = ValidationReport::new();
        
        // Valid patterns
        let valid = vec![
            r"rm -rf /".to_string(),
            r"sudo \w+".to_string(),
        ];
        rules.validate_custom_patterns(&valid, &mut report).unwrap();
        assert!(report.errors.is_empty());
        
        // Invalid pattern
        let invalid = vec![
            r"[invalid(".to_string(),
        ];
        rules.validate_custom_patterns(&invalid, &mut report).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("Invalid regex"));
    }
}