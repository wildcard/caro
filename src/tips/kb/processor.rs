//! Cheatsheet processor
//!
//! Parses YAML cheatsheet files and compiles them into a KnowledgeBase.

use super::types::{
    Cheatsheet, CheatsheetAlias, CheatsheetPlugin, CheatsheetTip, KbAlias, KbPlugin, KbTip,
    KnowledgeBase,
};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during processing
#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("Failed to read file {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse YAML in {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("Invalid cheatsheet format in {path}: {message}")]
    InvalidFormat { path: PathBuf, message: String },

    #[error("No cheatsheets found in directory")]
    NoCheatsheets,
}

/// Cheatsheet processor for building knowledge bases
pub struct KbProcessor {
    /// Source directory containing cheatsheet YAML files
    source_dir: PathBuf,

    /// Collected cheatsheets
    cheatsheets: Vec<(PathBuf, Cheatsheet)>,

    /// Whether to validate patterns
    validate_patterns: bool,
}

impl KbProcessor {
    /// Create a new processor for a source directory
    pub fn new(source_dir: impl AsRef<Path>) -> Self {
        Self {
            source_dir: source_dir.as_ref().to_path_buf(),
            cheatsheets: Vec::new(),
            validate_patterns: true,
        }
    }

    /// Disable pattern validation
    pub fn without_validation(mut self) -> Self {
        self.validate_patterns = false;
        self
    }

    /// Parse all cheatsheet files in the source directory
    pub fn parse_all(&mut self) -> Result<&mut Self, ProcessorError> {
        if !self.source_dir.exists() {
            return Err(ProcessorError::ReadError {
                path: self.source_dir.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Source directory not found",
                ),
            });
        }

        let entries: Vec<_> = fs::read_dir(&self.source_dir)
            .map_err(|e| ProcessorError::ReadError {
                path: self.source_dir.clone(),
                source: e,
            })?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "yaml" || ext == "yml")
                    .unwrap_or(false)
            })
            .collect();

        if entries.is_empty() {
            return Err(ProcessorError::NoCheatsheets);
        }

        for entry in entries {
            let path = entry.path();
            let cheatsheet = self.parse_file(&path)?;
            self.cheatsheets.push((path, cheatsheet));
        }

        Ok(self)
    }

    /// Parse a single cheatsheet file
    pub fn parse_file(&self, path: &Path) -> Result<Cheatsheet, ProcessorError> {
        let content = fs::read_to_string(path).map_err(|e| ProcessorError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let cheatsheet: Cheatsheet =
            serde_yaml::from_str(&content).map_err(|e| ProcessorError::ParseError {
                path: path.to_path_buf(),
                source: e,
            })?;

        // Validate if enabled
        if self.validate_patterns {
            self.validate_cheatsheet(path, &cheatsheet)?;
        }

        Ok(cheatsheet)
    }

    /// Validate a cheatsheet
    fn validate_cheatsheet(
        &self,
        path: &Path,
        cheatsheet: &Cheatsheet,
    ) -> Result<(), ProcessorError> {
        // Validate tip patterns
        for tip in &cheatsheet.tips {
            if tip.is_regex {
                if let Err(e) = regex::Regex::new(&tip.pattern) {
                    return Err(ProcessorError::InvalidFormat {
                        path: path.to_path_buf(),
                        message: format!(
                            "Invalid regex pattern '{}' in tip '{}': {}",
                            tip.pattern, tip.id, e
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Build the knowledge base from parsed cheatsheets
    pub fn build(&self, version: impl Into<String>) -> KnowledgeBase {
        let mut kb = KnowledgeBase::with_version(version);

        for (path, cheatsheet) in &self.cheatsheets {
            let source_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Add tips
            for tip in &cheatsheet.tips {
                let kb_tip = self.convert_tip(tip, &source_name, &cheatsheet.shells);
                kb.add_tip(kb_tip);
            }

            // Add aliases
            for alias in &cheatsheet.aliases {
                let kb_alias = self.convert_alias(alias, &source_name, &cheatsheet.shells);
                kb.add_alias(kb_alias);
            }

            // Add plugins
            for plugin in &cheatsheet.plugins {
                let kb_plugin = self.convert_plugin(plugin, &source_name);
                kb.add_plugin(kb_plugin);
            }
        }

        kb
    }

    /// Convert a cheatsheet tip to a KB tip
    fn convert_tip(&self, tip: &CheatsheetTip, source: &str, default_shells: &[String]) -> KbTip {
        let mut kb_tip = KbTip::new(&tip.id, &tip.pattern, &tip.message)
            .with_category(tip.category)
            .with_source(source.to_string());

        if tip.is_regex {
            kb_tip = kb_tip.with_regex();
        }

        // Use tip shells if specified, otherwise use cheatsheet default
        let shells = if tip.shells.is_empty() {
            default_shells.to_vec()
        } else {
            tip.shells.clone()
        };
        if !shells.is_empty() {
            kb_tip = kb_tip.with_shells(shells);
        }

        if let Some(ref plugin) = tip.requires_plugin {
            kb_tip = kb_tip.with_required_plugin(plugin.clone());
        }

        kb_tip.tags = tip.tags.clone();
        kb_tip
    }

    /// Convert a cheatsheet alias to a KB alias
    fn convert_alias(
        &self,
        alias: &CheatsheetAlias,
        source: &str,
        default_shells: &[String],
    ) -> KbAlias {
        let mut kb_alias = KbAlias::new(&alias.name, &alias.expansion);
        kb_alias.source = Some(source.to_string());
        kb_alias.shells = default_shells.to_vec();

        if let Some(ref desc) = alias.description {
            kb_alias = kb_alias.with_description(desc.clone());
        }

        if let Some(ref plugin) = alias.plugin {
            kb_alias = kb_alias.with_plugin(plugin.clone());
        }

        kb_alias
    }

    /// Convert a cheatsheet plugin to a KB plugin
    fn convert_plugin(&self, plugin: &CheatsheetPlugin, source: &str) -> KbPlugin {
        let mut kb_plugin =
            KbPlugin::new(&plugin.name, &plugin.description).with_managers(plugin.managers.clone());
        kb_plugin.source = Some(source.to_string());

        if let Some(ref cmd) = plugin.install_command {
            kb_plugin = kb_plugin.with_install_command(cmd.clone());
        }

        kb_plugin.url = plugin.url.clone();
        kb_plugin
    }

    /// Get the number of parsed cheatsheets
    pub fn cheatsheet_count(&self) -> usize {
        self.cheatsheets.len()
    }

    /// Get processing statistics
    pub fn stats(&self) -> ProcessorStats {
        let mut stats = ProcessorStats::default();

        for (_, cheatsheet) in &self.cheatsheets {
            stats.cheatsheets += 1;
            stats.tips += cheatsheet.tips.len();
            stats.aliases += cheatsheet.aliases.len();
            stats.plugins += cheatsheet.plugins.len();
        }

        stats
    }
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    pub cheatsheets: usize,
    pub tips: usize,
    pub aliases: usize,
    pub plugins: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cheatsheet_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create a test cheatsheet
        let cheatsheet = r#"
name: Test Cheatsheet
version: 1.0.0
authors:
  - test-author
shells:
  - zsh
  - bash

aliases:
  - name: gst
    expansion: git status
    description: Git status shortcut
    plugin: git
  - name: ll
    expansion: ls -la

tips:
  - id: git-status-tip
    pattern: git status
    message: Use `gst` instead of `git status`
    category: alias_shortcut
    requires_plugin: git
  - id: git-push-force-tip
    pattern: ^git push -f
    is_regex: true
    message: Consider using --force-with-lease
    category: safety_tip

plugins:
  - name: git
    description: Git integration for Oh My Zsh
    managers:
      - ohmyzsh
    install_command: omz plugin enable git
    url: https://github.com/ohmyzsh/ohmyzsh/tree/master/plugins/git
"#;

        fs::write(temp_dir.path().join("test.yaml"), cheatsheet).unwrap();
        temp_dir
    }

    #[test]
    fn test_parse_cheatsheet() {
        let temp_dir = create_test_cheatsheet_dir();
        let mut processor = KbProcessor::new(temp_dir.path());

        processor.parse_all().expect("parse_all");
        assert_eq!(processor.cheatsheet_count(), 1);
    }

    #[test]
    fn test_build_kb() {
        let temp_dir = create_test_cheatsheet_dir();
        let mut processor = KbProcessor::new(temp_dir.path());

        processor.parse_all().expect("parse_all");
        let kb = processor.build("1.0.0");

        assert_eq!(kb.version, "1.0.0");
        assert_eq!(kb.tip_count(), 2);
        assert_eq!(kb.alias_count(), 2);
        assert_eq!(kb.plugin_count(), 1);
    }

    #[test]
    fn test_tip_conversion() {
        let temp_dir = create_test_cheatsheet_dir();
        let mut processor = KbProcessor::new(temp_dir.path());

        processor.parse_all().expect("parse_all");
        let kb = processor.build("1.0.0");

        // Find the git-status-tip
        let tip = kb.tips.iter().find(|t| t.id == "git-status-tip");
        assert!(tip.is_some());
        let tip = tip.unwrap();
        assert_eq!(tip.pattern, "git status");
        assert!(!tip.is_regex);
        assert_eq!(tip.requires_plugin, Some("git".to_string()));
    }

    #[test]
    fn test_regex_tip() {
        let temp_dir = create_test_cheatsheet_dir();
        let mut processor = KbProcessor::new(temp_dir.path());

        processor.parse_all().expect("parse_all");
        let kb = processor.build("1.0.0");

        let tip = kb.tips.iter().find(|t| t.id == "git-push-force-tip");
        assert!(tip.is_some());
        let tip = tip.unwrap();
        assert!(tip.is_regex);
    }

    #[test]
    fn test_alias_conversion() {
        let temp_dir = create_test_cheatsheet_dir();
        let mut processor = KbProcessor::new(temp_dir.path());

        processor.parse_all().expect("parse_all");
        let kb = processor.build("1.0.0");

        let alias = kb.find_alias("gst");
        assert!(alias.is_some());
        let alias = alias.unwrap();
        assert_eq!(alias.expansion, "git status");
        assert_eq!(alias.plugin, Some("git".to_string()));
    }

    #[test]
    fn test_stats() {
        let temp_dir = create_test_cheatsheet_dir();
        let mut processor = KbProcessor::new(temp_dir.path());

        processor.parse_all().expect("parse_all");
        let stats = processor.stats();

        assert_eq!(stats.cheatsheets, 1);
        assert_eq!(stats.tips, 2);
        assert_eq!(stats.aliases, 2);
        assert_eq!(stats.plugins, 1);
    }

    #[test]
    fn test_invalid_regex() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_cheatsheet = r#"
name: Invalid
version: 1.0.0

tips:
  - id: bad-regex
    pattern: "[invalid regex"
    is_regex: true
    message: This should fail
    category: general
"#;

        fs::write(temp_dir.path().join("invalid.yaml"), invalid_cheatsheet).unwrap();

        let mut processor = KbProcessor::new(temp_dir.path());
        let result = processor.parse_all();

        assert!(matches!(result, Err(ProcessorError::InvalidFormat { .. })));
    }

    #[test]
    fn test_no_cheatsheets() {
        let temp_dir = TempDir::new().unwrap();
        let mut processor = KbProcessor::new(temp_dir.path());

        let result = processor.parse_all();
        assert!(matches!(result, Err(ProcessorError::NoCheatsheets)));
    }
}
