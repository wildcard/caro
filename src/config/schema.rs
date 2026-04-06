//! Configuration schema validation

use crate::models::ConfigSchema;
use std::collections::HashMap;

/// Schema validator for configuration files
pub struct SchemaValidator;

impl SchemaValidator {
    /// Validate a configuration value against the schema
    pub fn validate(_schema: &ConfigSchema, _value: &toml::Value) -> Result<(), String> {
        // Placeholder for schema validation logic
        // In a full implementation, this would validate:
        // - Required fields are present
        // - Field types match expected types
        // - Values are within valid ranges
        Ok(())
    }
}

impl Default for ConfigSchema {
    fn default() -> Self {
        let mut known_keys = HashMap::new();
        known_keys.insert(
            "general.safety_level".to_string(),
            "SafetyLevel enum".to_string(),
        );
        known_keys.insert(
            "general.default_shell".to_string(),
            "ShellType enum".to_string(),
        );
        known_keys.insert("general.default_model".to_string(), "String".to_string());
        known_keys.insert("logging.log_level".to_string(), "LogLevel enum".to_string());
        known_keys.insert("logging.log_rotation_days".to_string(), "u32".to_string());
        known_keys.insert("cache.max_size_gb".to_string(), "u64".to_string());

        // Safety configuration keys
        known_keys.insert(
            "safety.trusted_paths".to_string(),
            "Vec<String>".to_string(),
        );
        known_keys.insert(
            "safety.trusted_domains".to_string(),
            "Vec<String>".to_string(),
        );
        known_keys.insert(
            "safety.trusted_commands".to_string(),
            "Vec<String>".to_string(),
        );
        known_keys.insert(
            "safety.semantic_safety".to_string(),
            "bool".to_string(),
        );

        let deprecated_keys = HashMap::new();
        // Example: deprecated_keys.insert("old.key".to_string(), "new.key".to_string());

        Self {
            known_sections: vec![
                "general".to_string(),
                "logging".to_string(),
                "cache".to_string(),
                "safety".to_string(),
            ],
            known_keys,
            deprecated_keys,
        }
    }
}
