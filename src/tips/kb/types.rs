//! Knowledge base data types
//!
//! Defines the structures for storing tips, aliases, and plugin information
//! in the community knowledge base.

use serde::{Deserialize, Serialize};

/// The main knowledge base container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Version of the knowledge base (semver)
    pub version: String,

    /// Unix timestamp of last update
    pub updated_at: i64,

    /// SHA256 checksum of the content (excluding this field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// All tips in the knowledge base
    pub tips: Vec<KbTip>,

    /// All aliases in the knowledge base
    pub aliases: Vec<KbAlias>,

    /// All plugins with their metadata
    pub plugins: Vec<KbPlugin>,
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            updated_at: 0,
            checksum: None,
            tips: Vec::new(),
            aliases: Vec::new(),
            plugins: Vec::new(),
        }
    }
}

impl KnowledgeBase {
    /// Create a new empty knowledge base
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a knowledge base with the specified version
    pub fn with_version(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            updated_at: chrono::Utc::now().timestamp(),
            ..Self::default()
        }
    }

    /// Add a tip to the knowledge base
    pub fn add_tip(&mut self, tip: KbTip) {
        self.tips.push(tip);
    }

    /// Add an alias to the knowledge base
    pub fn add_alias(&mut self, alias: KbAlias) {
        self.aliases.push(alias);
    }

    /// Add a plugin to the knowledge base
    pub fn add_plugin(&mut self, plugin: KbPlugin) {
        self.plugins.push(plugin);
    }

    /// Get the number of tips
    pub fn tip_count(&self) -> usize {
        self.tips.len()
    }

    /// Get the number of aliases
    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }

    /// Get the number of plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Find tips matching a pattern
    pub fn find_tips(&self, command: &str) -> Vec<&KbTip> {
        self.tips
            .iter()
            .filter(|tip| tip.matches(command))
            .collect()
    }

    /// Find an alias by name
    pub fn find_alias(&self, name: &str) -> Option<&KbAlias> {
        self.aliases.iter().find(|a| a.name == name)
    }

    /// Serialize to MessagePack bytes (uses named fields for compatibility)
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self)
    }

    /// Deserialize from MessagePack bytes
    pub fn from_msgpack(data: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(data)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// A tip from the knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbTip {
    /// Unique identifier for this tip
    pub id: String,

    /// Pattern to match against commands (regex or literal)
    pub pattern: String,

    /// Whether the pattern is a regex (true) or literal (false)
    #[serde(default)]
    pub is_regex: bool,

    /// The message to display to the user
    pub message: String,

    /// Category of the tip
    pub category: KbTipCategory,

    /// Shells this tip applies to (empty = all shells)
    #[serde(default)]
    pub shells: Vec<String>,

    /// Plugin required for this tip to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_plugin: Option<String>,

    /// Source cheatsheet this tip came from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Compiled regex pattern (not serialized)
    #[serde(skip)]
    compiled_pattern: Option<regex::Regex>,
}

impl KbTip {
    /// Create a new tip
    pub fn new(id: impl Into<String>, pattern: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            pattern: pattern.into(),
            is_regex: false,
            message: message.into(),
            category: KbTipCategory::General,
            shells: Vec::new(),
            requires_plugin: None,
            source: None,
            tags: Vec::new(),
            compiled_pattern: None,
        }
    }

    /// Set this tip as using regex pattern
    pub fn with_regex(mut self) -> Self {
        self.is_regex = true;
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: KbTipCategory) -> Self {
        self.category = category;
        self
    }

    /// Set the shells this tip applies to
    pub fn with_shells(mut self, shells: Vec<String>) -> Self {
        self.shells = shells;
        self
    }

    /// Set the required plugin
    pub fn with_required_plugin(mut self, plugin: impl Into<String>) -> Self {
        self.requires_plugin = Some(plugin.into());
        self
    }

    /// Set the source cheatsheet
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Check if this tip matches a command
    pub fn matches(&self, command: &str) -> bool {
        if self.is_regex {
            // Try to compile and match regex
            if let Ok(re) = regex::Regex::new(&self.pattern) {
                re.is_match(command)
            } else {
                false
            }
        } else {
            // Literal match
            command == self.pattern || command.starts_with(&format!("{} ", self.pattern))
        }
    }

    /// Check if this tip applies to a specific shell
    pub fn applies_to_shell(&self, shell: &str) -> bool {
        self.shells.is_empty() || self.shells.iter().any(|s| s.eq_ignore_ascii_case(shell))
    }
}

/// Category of a knowledge base tip
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KbTipCategory {
    /// Alias shortcut suggestion
    AliasShortcut,
    /// Plugin recommendation
    PluginRecommendation,
    /// Best practice advice
    BestPractice,
    /// Safety warning
    SafetyTip,
    /// Performance tip
    Performance,
    /// General tip
    #[default]
    General,
}

impl std::fmt::Display for KbTipCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AliasShortcut => write!(f, "Alias"),
            Self::PluginRecommendation => write!(f, "Plugin"),
            Self::BestPractice => write!(f, "Best Practice"),
            Self::SafetyTip => write!(f, "Safety"),
            Self::Performance => write!(f, "Performance"),
            Self::General => write!(f, "Tip"),
        }
    }
}

/// An alias from the knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbAlias {
    /// The alias name (e.g., "gst")
    pub name: String,

    /// The command it expands to (e.g., "git status")
    pub expansion: String,

    /// Description of what this alias does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Plugin that provides this alias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,

    /// Shells this alias is available for
    #[serde(default)]
    pub shells: Vec<String>,

    /// Source cheatsheet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl KbAlias {
    /// Create a new alias
    pub fn new(name: impl Into<String>, expansion: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            expansion: expansion.into(),
            description: None,
            plugin: None,
            shells: Vec::new(),
            source: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the plugin
    pub fn with_plugin(mut self, plugin: impl Into<String>) -> Self {
        self.plugin = Some(plugin.into());
        self
    }

    /// Calculate characters saved by using this alias
    pub fn chars_saved(&self) -> i32 {
        self.expansion.len() as i32 - self.name.len() as i32
    }
}

/// A plugin in the knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbPlugin {
    /// Plugin name (e.g., "git")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Plugin manager(s) this plugin is for
    pub managers: Vec<String>,

    /// Installation command/instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_command: Option<String>,

    /// URL for more information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Source cheatsheet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl KbPlugin {
    /// Create a new plugin entry
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            managers: Vec::new(),
            install_command: None,
            url: None,
            source: None,
        }
    }

    /// Set the plugin managers
    pub fn with_managers(mut self, managers: Vec<String>) -> Self {
        self.managers = managers;
        self
    }

    /// Set the install command
    pub fn with_install_command(mut self, cmd: impl Into<String>) -> Self {
        self.install_command = Some(cmd.into());
        self
    }
}

/// A cheatsheet source file (YAML format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cheatsheet {
    /// Name of the cheatsheet
    pub name: String,

    /// Version of the cheatsheet
    #[serde(default = "default_version")]
    pub version: String,

    /// Author(s)
    #[serde(default)]
    pub authors: Vec<String>,

    /// Shells this cheatsheet applies to
    #[serde(default)]
    pub shells: Vec<String>,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Aliases defined in this cheatsheet
    #[serde(default)]
    pub aliases: Vec<CheatsheetAlias>,

    /// Tips defined in this cheatsheet
    #[serde(default)]
    pub tips: Vec<CheatsheetTip>,

    /// Plugins referenced in this cheatsheet
    #[serde(default)]
    pub plugins: Vec<CheatsheetPlugin>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl Default for Cheatsheet {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            version: default_version(),
            authors: Vec::new(),
            shells: Vec::new(),
            description: None,
            aliases: Vec::new(),
            tips: Vec::new(),
            plugins: Vec::new(),
        }
    }
}

/// An alias in a cheatsheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheatsheetAlias {
    pub name: String,
    pub expansion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
}

/// A tip in a cheatsheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheatsheetTip {
    pub id: String,
    pub pattern: String,
    #[serde(default)]
    pub is_regex: bool,
    pub message: String,
    #[serde(default)]
    pub category: KbTipCategory,
    #[serde(default)]
    pub shells: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_plugin: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A plugin in a cheatsheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheatsheetPlugin {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub managers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_creation() {
        let kb = KnowledgeBase::new();
        assert_eq!(kb.version, "1.0.0");
        assert!(kb.tips.is_empty());
        assert!(kb.aliases.is_empty());
    }

    #[test]
    fn test_add_tip() {
        let mut kb = KnowledgeBase::new();
        let tip = KbTip::new("test-tip", "git status", "Use gst instead!");
        kb.add_tip(tip);
        assert_eq!(kb.tip_count(), 1);
    }

    #[test]
    fn test_add_alias() {
        let mut kb = KnowledgeBase::new();
        let alias = KbAlias::new("gst", "git status");
        kb.add_alias(alias);
        assert_eq!(kb.alias_count(), 1);
    }

    #[test]
    fn test_tip_matching() {
        let tip = KbTip::new("test", "git status", "test message");
        assert!(tip.matches("git status"));
        assert!(tip.matches("git status -s"));
        assert!(!tip.matches("git commit"));
    }

    #[test]
    fn test_regex_tip_matching() {
        let tip = KbTip::new("test", "^git (status|st)$", "test message").with_regex();
        assert!(tip.matches("git status"));
        assert!(tip.matches("git st"));
        assert!(!tip.matches("git commit"));
    }

    #[test]
    fn test_alias_chars_saved() {
        let alias = KbAlias::new("gst", "git status");
        assert_eq!(alias.chars_saved(), 7); // 10 - 3 = 7
    }

    #[test]
    fn test_msgpack_roundtrip() {
        let mut kb = KnowledgeBase::with_version("1.0.0");
        kb.add_tip(KbTip::new("tip1", "git status", "Use gst!"));
        kb.add_alias(KbAlias::new("gst", "git status"));

        let bytes = kb.to_msgpack().expect("serialize");
        let restored = KnowledgeBase::from_msgpack(&bytes).expect("deserialize");

        assert_eq!(restored.version, kb.version);
        assert_eq!(restored.tip_count(), kb.tip_count());
        assert_eq!(restored.alias_count(), kb.alias_count());
    }

    #[test]
    fn test_json_roundtrip() {
        let mut kb = KnowledgeBase::with_version("1.0.0");
        kb.add_tip(KbTip::new("tip1", "git status", "Use gst!"));

        let json = kb.to_json().expect("serialize");
        let restored = KnowledgeBase::from_json(&json).expect("deserialize");

        assert_eq!(restored.version, kb.version);
        assert_eq!(restored.tip_count(), kb.tip_count());
    }

    #[test]
    fn test_tip_category_display() {
        assert_eq!(format!("{}", KbTipCategory::AliasShortcut), "Alias");
        assert_eq!(format!("{}", KbTipCategory::SafetyTip), "Safety");
    }

    #[test]
    fn test_tip_applies_to_shell() {
        let tip = KbTip::new("test", "cmd", "msg").with_shells(vec!["zsh".to_string()]);
        assert!(tip.applies_to_shell("zsh"));
        assert!(tip.applies_to_shell("ZSH"));
        assert!(!tip.applies_to_shell("bash"));

        // Empty shells means applies to all
        let general_tip = KbTip::new("test", "cmd", "msg");
        assert!(general_tip.applies_to_shell("bash"));
        assert!(general_tip.applies_to_shell("zsh"));
    }
}
