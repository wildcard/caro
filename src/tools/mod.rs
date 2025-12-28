//! Advanced Tool Use Patterns for Shell Integration
//!
//! This module provides a comprehensive tool system for context-gathering,
//! validation, and discovery during command generation. Tools enable the
//! agent loop to gather additional context, validate assumptions, and
//! make informed decisions about command generation.
//!
//! # Architecture
//!
//! The tool system follows the function-calling paradigm common in modern LLMs:
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
//! │ Agent Loop  │────▶│ Tool Registry│────▶│ Tool Impls  │
//! └─────────────┘     └──────────────┘     └─────────────┘
//!       │                    │                    │
//!       │  ToolCall          │  invoke()          │  execute()
//!       ▼                    ▼                    ▼
//! ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
//! │ ToolResult  │◀────│   Dispatch   │◀────│   Result    │
//! └─────────────┘     └──────────────┘     └─────────────┘
//! ```
//!
//! # Tool Categories
//!
//! - **FileSystem**: Path validation, file existence, permissions, directory contents
//! - **Command**: Command discovery, version info, help text, availability
//! - **Context**: Environment variables, system info, shell capabilities
//! - **Validation**: Pre-execution assumption validation
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use caro::tools::{ToolRegistry, ToolCall};
//!
//! let registry = ToolRegistry::default();
//!
//! // Check if a file exists before generating a command that operates on it
//! let call = ToolCall::new("filesystem.exists")
//!     .with_param("path", "/etc/hosts");
//!
//! let result = registry.invoke(&call).await?;
//! if result.success {
//!     // File exists, proceed with command generation
//! }
//! ```

mod agent_integration;
mod command;
mod context;
mod filesystem;
mod registry;
mod validation;

pub use agent_integration::{
    ContextSummary, EnhancedCommandResult, PathCheckResult, PlatformFlags, ToolEnhancedAgent,
};
pub use command::CommandTool;
pub use context::ContextTool;
pub use filesystem::FileSystemTool;
pub use registry::{ToolRegistry, ToolRegistryBuilder};
pub use validation::ValidationTool;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core trait that all tools must implement.
///
/// Tools provide atomic, focused capabilities that can be invoked during
/// command generation to gather context or validate assumptions.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the unique name of this tool (e.g., "filesystem.exists")
    fn name(&self) -> &str;

    /// Returns a brief description of what this tool does
    fn description(&self) -> &str;

    /// Returns the schema for this tool's parameters
    fn parameters(&self) -> ToolParameters;

    /// Execute the tool with the given parameters
    async fn execute(&self, params: &ToolCallParams) -> ToolResult;

    /// Returns the category of this tool
    fn category(&self) -> ToolCategory;
}

/// Categories for organizing tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// File and directory operations
    FileSystem,
    /// Command discovery and inspection
    Command,
    /// System and environment context
    Context,
    /// Validation and checks
    Validation,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::FileSystem => write!(f, "filesystem"),
            ToolCategory::Command => write!(f, "command"),
            ToolCategory::Context => write!(f, "context"),
            ToolCategory::Validation => write!(f, "validation"),
        }
    }
}

/// Parameters schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    /// Required parameters
    pub required: Vec<ParameterDef>,
    /// Optional parameters
    pub optional: Vec<ParameterDef>,
}

impl ToolParameters {
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            optional: Vec::new(),
        }
    }

    pub fn with_required(mut self, name: &str, param_type: ParameterType, desc: &str) -> Self {
        self.required.push(ParameterDef {
            name: name.to_string(),
            param_type,
            description: desc.to_string(),
        });
        self
    }

    pub fn with_optional(mut self, name: &str, param_type: ParameterType, desc: &str) -> Self {
        self.optional.push(ParameterDef {
            name: name.to_string(),
            param_type,
            description: desc.to_string(),
        });
        self
    }
}

impl Default for ToolParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// Definition of a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
}

/// Types of parameters supported by tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Path,
    Boolean,
    Integer,
    StringArray,
}

/// Parameters passed to a tool call
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCallParams {
    params: HashMap<String, ParamValue>,
}

impl ToolCallParams {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn with_string(mut self, key: &str, value: impl Into<String>) -> Self {
        self.params.insert(key.to_string(), ParamValue::String(value.into()));
        self
    }

    pub fn with_path(mut self, key: &str, value: impl Into<String>) -> Self {
        self.params.insert(key.to_string(), ParamValue::Path(value.into()));
        self
    }

    pub fn with_bool(mut self, key: &str, value: bool) -> Self {
        self.params.insert(key.to_string(), ParamValue::Boolean(value));
        self
    }

    pub fn with_int(mut self, key: &str, value: i64) -> Self {
        self.params.insert(key.to_string(), ParamValue::Integer(value));
        self
    }

    pub fn with_string_array(mut self, key: &str, values: Vec<String>) -> Self {
        self.params.insert(key.to_string(), ParamValue::StringArray(values));
        self
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.params.get(key) {
            Some(ParamValue::String(s) | ParamValue::Path(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.params.get(key) {
            Some(ParamValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.params.get(key) {
            Some(ParamValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn get_string_array(&self, key: &str) -> Option<&[String]> {
        match self.params.get(key) {
            Some(ParamValue::StringArray(arr)) => Some(arr),
            _ => None,
        }
    }
}

/// Value of a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    String(String),
    Path(String),
    Boolean(bool),
    Integer(i64),
    StringArray(Vec<String>),
}

/// A request to invoke a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// The name of the tool to invoke
    pub tool_name: String,
    /// Parameters to pass to the tool
    pub params: ToolCallParams,
}

impl ToolCall {
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            params: ToolCallParams::new(),
        }
    }

    pub fn with_param(mut self, key: &str, value: impl Into<String>) -> Self {
        self.params = self.params.with_string(key, value);
        self
    }

    pub fn with_path(mut self, key: &str, value: impl Into<String>) -> Self {
        self.params = self.params.with_path(key, value);
        self
    }

    pub fn with_bool(mut self, key: &str, value: bool) -> Self {
        self.params = self.params.with_bool(key, value);
        self
    }

    pub fn with_int(mut self, key: &str, value: i64) -> Self {
        self.params = self.params.with_int(key, value);
        self
    }

    pub fn with_string_array(mut self, key: &str, values: Vec<String>) -> Self {
        self.params = self.params.with_string_array(key, values);
        self
    }
}

/// Result of a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool executed successfully
    pub success: bool,
    /// The tool's output data
    pub data: ToolData,
    /// Optional error message if success is false
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

impl ToolResult {
    pub fn success(data: ToolData, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            data,
            error: None,
            execution_time_ms,
        }
    }

    pub fn error(message: impl Into<String>, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            data: ToolData::None,
            error: Some(message.into()),
            execution_time_ms,
        }
    }

    /// Extract a boolean value from the result
    pub fn as_bool(&self) -> Option<bool> {
        match &self.data {
            ToolData::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Extract a string value from the result
    pub fn as_string(&self) -> Option<&str> {
        match &self.data {
            ToolData::String(s) => Some(s),
            _ => None,
        }
    }

    /// Extract a map from the result
    pub fn as_map(&self) -> Option<&HashMap<String, String>> {
        match &self.data {
            ToolData::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Extract a list from the result
    pub fn as_list(&self) -> Option<&[String]> {
        match &self.data {
            ToolData::List(l) => Some(l),
            _ => None,
        }
    }
}

/// Data returned by a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolData {
    None,
    Boolean(bool),
    Integer(i64),
    String(String),
    List(Vec<String>),
    Map(HashMap<String, String>),
    /// Structured data for complex results
    Structured(StructuredData),
}

/// Structured data for complex tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub fields: HashMap<String, serde_json::Value>,
}

impl StructuredData {
    pub fn new(data_type: &str) -> Self {
        Self {
            data_type: data_type.to_string(),
            fields: HashMap::new(),
        }
    }

    pub fn with_field(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.to_string(), value.into());
        self
    }
}

/// Information about a registered tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub parameters: ToolParameters,
}

/// Error types for tool operations
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {name}")]
    NotFound { name: String },

    #[error("Missing required parameter: {param}")]
    MissingParameter { param: String },

    #[error("Invalid parameter type for {param}: expected {expected}")]
    InvalidParameterType { param: String, expected: String },

    #[error("Tool execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_builder() {
        let call = ToolCall::new("filesystem.exists")
            .with_path("path", "/etc/hosts")
            .with_bool("follow_symlinks", true);

        assert_eq!(call.tool_name, "filesystem.exists");
        assert_eq!(call.params.get_string("path"), Some("/etc/hosts"));
        assert_eq!(call.params.get_bool("follow_symlinks"), Some(true));
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success(ToolData::Boolean(true), 5);

        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("File not found", 2);

        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("File not found"));
    }

    #[test]
    fn test_tool_parameters_builder() {
        let params = ToolParameters::new()
            .with_required("path", ParameterType::Path, "The file path")
            .with_optional("recursive", ParameterType::Boolean, "Recurse into directories");

        assert_eq!(params.required.len(), 1);
        assert_eq!(params.optional.len(), 1);
        assert_eq!(params.required[0].name, "path");
    }

    #[test]
    fn test_structured_data() {
        let data = StructuredData::new("file_info")
            .with_field("size", 1024)
            .with_field("is_dir", false);

        assert_eq!(data.data_type, "file_info");
        assert_eq!(data.fields.get("size"), Some(&serde_json::json!(1024)));
    }
}
