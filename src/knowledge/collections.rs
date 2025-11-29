//! Collection management for different knowledge types

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Types of collections in the knowledge base
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionType {
    /// Command documentation (man, tldr, help)
    CommandDocs,
    /// Execution history
    ExecutionHistory,
    /// User preferences and patterns
    UserPreferences,
    /// Learned mistakes and corrections
    MistakesLearned,
    /// Project-specific context
    ProjectContext,
}

impl CollectionType {
    /// Get the collection name in ChromaDB
    pub fn collection_name(&self) -> &'static str {
        match self {
            Self::CommandDocs => "cmdai_command_docs",
            Self::ExecutionHistory => "cmdai_execution_history",
            Self::UserPreferences => "cmdai_user_preferences",
            Self::MistakesLearned => "cmdai_mistakes_learned",
            Self::ProjectContext => "cmdai_project_context",
        }
    }

    /// Get all collection types
    pub fn all() -> Vec<Self> {
        vec![
            Self::CommandDocs,
            Self::ExecutionHistory,
            Self::UserPreferences,
            Self::MistakesLearned,
            Self::ProjectContext,
        ]
    }
}

impl std::fmt::Display for CollectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.collection_name())
    }
}

/// Metadata for documents in collections
pub trait DocumentMetadata {
    /// Convert to HashMap for ChromaDB
    fn to_metadata(&self) -> HashMap<String, JsonValue>;

    /// Create from HashMap
    fn from_metadata(metadata: HashMap<String, JsonValue>) -> Result<Self, String>
    where
        Self: Sized;
}

/// Metadata for command documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDocMetadata {
    pub command_name: String,
    pub source_type: String, // "man", "tldr", "help"
    pub shell_type: Option<String>,
    pub last_updated: String,
    pub platform: String,
}

impl DocumentMetadata for CommandDocMetadata {
    fn to_metadata(&self) -> HashMap<String, JsonValue> {
        let mut map = HashMap::new();
        map.insert("command_name".to_string(), JsonValue::String(self.command_name.clone()));
        map.insert("source_type".to_string(), JsonValue::String(self.source_type.clone()));
        if let Some(shell) = &self.shell_type {
            map.insert("shell_type".to_string(), JsonValue::String(shell.clone()));
        }
        map.insert("last_updated".to_string(), JsonValue::String(self.last_updated.clone()));
        map.insert("platform".to_string(), JsonValue::String(self.platform.clone()));
        map
    }

    fn from_metadata(metadata: HashMap<String, JsonValue>) -> Result<Self, String> {
        Ok(Self {
            command_name: metadata
                .get("command_name")
                .and_then(|v| v.as_str())
                .ok_or("Missing command_name")?
                .to_string(),
            source_type: metadata
                .get("source_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing source_type")?
                .to_string(),
            shell_type: metadata.get("shell_type").and_then(|v| v.as_str()).map(String::from),
            last_updated: metadata
                .get("last_updated")
                .and_then(|v| v.as_str())
                .ok_or("Missing last_updated")?
                .to_string(),
            platform: metadata
                .get("platform")
                .and_then(|v| v.as_str())
                .ok_or("Missing platform")?
                .to_string(),
        })
    }
}

/// Metadata for execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistoryMetadata {
    pub timestamp: String,
    pub user_profile: String,
    pub success: bool,
    pub command: String,
    pub prompt: String,
    pub safety_level: String,
    pub shell_type: String,
}

impl DocumentMetadata for ExecutionHistoryMetadata {
    fn to_metadata(&self) -> HashMap<String, JsonValue> {
        let mut map = HashMap::new();
        map.insert("timestamp".to_string(), JsonValue::String(self.timestamp.clone()));
        map.insert("user_profile".to_string(), JsonValue::String(self.user_profile.clone()));
        map.insert("success".to_string(), JsonValue::Bool(self.success));
        map.insert("command".to_string(), JsonValue::String(self.command.clone()));
        map.insert("prompt".to_string(), JsonValue::String(self.prompt.clone()));
        map.insert("safety_level".to_string(), JsonValue::String(self.safety_level.clone()));
        map.insert("shell_type".to_string(), JsonValue::String(self.shell_type.clone()));
        map
    }

    fn from_metadata(metadata: HashMap<String, JsonValue>) -> Result<Self, String> {
        Ok(Self {
            timestamp: metadata
                .get("timestamp")
                .and_then(|v| v.as_str())
                .ok_or("Missing timestamp")?
                .to_string(),
            user_profile: metadata
                .get("user_profile")
                .and_then(|v| v.as_str())
                .ok_or("Missing user_profile")?
                .to_string(),
            success: metadata
                .get("success")
                .and_then(|v| v.as_bool())
                .ok_or("Missing success")?,
            command: metadata
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or("Missing command")?
                .to_string(),
            prompt: metadata
                .get("prompt")
                .and_then(|v| v.as_str())
                .ok_or("Missing prompt")?
                .to_string(),
            safety_level: metadata
                .get("safety_level")
                .and_then(|v| v.as_str())
                .ok_or("Missing safety_level")?
                .to_string(),
            shell_type: metadata
                .get("shell_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing shell_type")?
                .to_string(),
        })
    }
}

/// Metadata for user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferenceMetadata {
    pub user_profile: String,
    pub preference_type: String,
    pub command_pattern: String,
    pub frequency: i64,
    pub last_used: String,
}

impl DocumentMetadata for UserPreferenceMetadata {
    fn to_metadata(&self) -> HashMap<String, JsonValue> {
        let mut map = HashMap::new();
        map.insert("user_profile".to_string(), JsonValue::String(self.user_profile.clone()));
        map.insert("preference_type".to_string(), JsonValue::String(self.preference_type.clone()));
        map.insert("command_pattern".to_string(), JsonValue::String(self.command_pattern.clone()));
        map.insert("frequency".to_string(), JsonValue::Number(self.frequency.into()));
        map.insert("last_used".to_string(), JsonValue::String(self.last_used.clone()));
        map
    }

    fn from_metadata(metadata: HashMap<String, JsonValue>) -> Result<Self, String> {
        Ok(Self {
            user_profile: metadata
                .get("user_profile")
                .and_then(|v| v.as_str())
                .ok_or("Missing user_profile")?
                .to_string(),
            preference_type: metadata
                .get("preference_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing preference_type")?
                .to_string(),
            command_pattern: metadata
                .get("command_pattern")
                .and_then(|v| v.as_str())
                .ok_or("Missing command_pattern")?
                .to_string(),
            frequency: metadata
                .get("frequency")
                .and_then(|v| v.as_i64())
                .ok_or("Missing frequency")?,
            last_used: metadata
                .get("last_used")
                .and_then(|v| v.as_str())
                .ok_or("Missing last_used")?
                .to_string(),
        })
    }
}

/// Metadata for learned mistakes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeMetadata {
    pub timestamp: String,
    pub user_profile: String,
    pub error_type: String,
    pub original_command: String,
    pub correction: Option<String>,
}

impl DocumentMetadata for MistakeMetadata {
    fn to_metadata(&self) -> HashMap<String, JsonValue> {
        let mut map = HashMap::new();
        map.insert("timestamp".to_string(), JsonValue::String(self.timestamp.clone()));
        map.insert("user_profile".to_string(), JsonValue::String(self.user_profile.clone()));
        map.insert("error_type".to_string(), JsonValue::String(self.error_type.clone()));
        map.insert("original_command".to_string(), JsonValue::String(self.original_command.clone()));
        if let Some(correction) = &self.correction {
            map.insert("correction".to_string(), JsonValue::String(correction.clone()));
        }
        map
    }

    fn from_metadata(metadata: HashMap<String, JsonValue>) -> Result<Self, String> {
        Ok(Self {
            timestamp: metadata
                .get("timestamp")
                .and_then(|v| v.as_str())
                .ok_or("Missing timestamp")?
                .to_string(),
            user_profile: metadata
                .get("user_profile")
                .and_then(|v| v.as_str())
                .ok_or("Missing user_profile")?
                .to_string(),
            error_type: metadata
                .get("error_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing error_type")?
                .to_string(),
            original_command: metadata
                .get("original_command")
                .and_then(|v| v.as_str())
                .ok_or("Missing original_command")?
                .to_string(),
            correction: metadata.get("correction").and_then(|v| v.as_str()).map(String::from),
        })
    }
}

/// Metadata for project context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContextMetadata {
    pub project_path: String,
    pub file_type: String,
    pub relevance_score: f64,
    pub last_indexed: String,
}

impl DocumentMetadata for ProjectContextMetadata {
    fn to_metadata(&self) -> HashMap<String, JsonValue> {
        let mut map = HashMap::new();
        map.insert("project_path".to_string(), JsonValue::String(self.project_path.clone()));
        map.insert("file_type".to_string(), JsonValue::String(self.file_type.clone()));
        map.insert(
            "relevance_score".to_string(),
            JsonValue::Number(serde_json::Number::from_f64(self.relevance_score).unwrap_or_else(|| serde_json::Number::from(0))),
        );
        map.insert("last_indexed".to_string(), JsonValue::String(self.last_indexed.clone()));
        map
    }

    fn from_metadata(metadata: HashMap<String, JsonValue>) -> Result<Self, String> {
        Ok(Self {
            project_path: metadata
                .get("project_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing project_path")?
                .to_string(),
            file_type: metadata
                .get("file_type")
                .and_then(|v| v.as_str())
                .ok_or("Missing file_type")?
                .to_string(),
            relevance_score: metadata
                .get("relevance_score")
                .and_then(|v| v.as_f64())
                .ok_or("Missing relevance_score")?,
            last_indexed: metadata
                .get("last_indexed")
                .and_then(|v| v.as_str())
                .ok_or("Missing last_indexed")?
                .to_string(),
        })
    }
}
