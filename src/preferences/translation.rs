//! Command translation module
//!
//! This module translates generated commands to match user preferences.
//! For example, translating `npm install` to `yarn add` when the project
//! uses Yarn, or suggesting `gst` instead of `git status` when the user
//! has that alias configured.

use super::{PackageManager, UserPreferences};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Result of translating a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedCommand {
    /// Original command
    pub original: String,

    /// Command adapted to user preferences
    pub translated: String,

    /// List of changes made
    pub changes: Vec<String>,

    /// Whether any translation was performed
    pub was_translated: bool,
}

impl TranslatedCommand {
    /// Create a new translated command (no changes)
    pub fn unchanged(command: String) -> Self {
        Self {
            original: command.clone(),
            translated: command,
            changes: vec![],
            was_translated: false,
        }
    }

    /// Create with translation
    pub fn with_translation(original: String, translated: String, changes: Vec<String>) -> Self {
        let was_translated = original != translated;
        Self {
            original,
            translated,
            changes,
            was_translated,
        }
    }
}

/// Command translator that adapts commands to user preferences
pub struct CommandTranslator;

impl CommandTranslator {
    /// Translate a command to match user preferences
    ///
    /// This performs the following translations:
    /// 1. Package manager translation (npm → yarn, etc.)
    /// 2. Alias suggestions (git status → gst)
    ///
    /// # Arguments
    ///
    /// * `command` - The command to translate
    /// * `preferences` - User preferences to translate against
    ///
    /// # Returns
    ///
    /// A TranslatedCommand with the result
    pub fn translate(command: &str, preferences: &UserPreferences) -> TranslatedCommand {
        let mut translated = command.to_string();
        let mut changes = Vec::new();

        // 1. Package manager translation
        if let Some(expected_pm) = &preferences.project.package_manager {
            if let Some((new_cmd, change)) = Self::translate_package_manager(&translated, expected_pm) {
                translated = new_cmd;
                changes.push(change);
            }
        }

        // 2. Alias translation (optional - suggest but don't force)
        // We'll add the alias suggestion as a change note but keep the full command
        if let Some(alias_cmd) = preferences.shell.find_alias_for_command(&translated) {
            if alias_cmd.len() < translated.len() {
                changes.push(format!(
                    "Alias available: '{}' instead of '{}'",
                    alias_cmd, translated
                ));
                // Optionally replace with alias
                // translated = alias_cmd;
            }
        }

        if changes.is_empty() {
            TranslatedCommand::unchanged(command.to_string())
        } else {
            debug!("Translated command: {} → {} ({:?})", command, translated, changes);
            TranslatedCommand::with_translation(command.to_string(), translated, changes)
        }
    }

    /// Translate package manager commands
    fn translate_package_manager(command: &str, expected: &PackageManager) -> Option<(String, String)> {
        // Map of package manager commands and their common operations
        let pm_commands = [
            ("npm install", "install"),
            ("npm i ", "install"),
            ("npm add ", "install"),
            ("yarn add ", "add"),
            ("yarn install", "install"),
            ("pnpm add ", "add"),
            ("pnpm install", "install"),
            ("npx ", "npx"),
            ("bunx ", "bunx"),
        ];

        for (prefix, op) in pm_commands {
            if command.starts_with(prefix) {
                // Detect which package manager is in the command
                let detected_pm = if prefix.starts_with("npm") || prefix.starts_with("npx") {
                    PackageManager::Npm
                } else if prefix.starts_with("yarn") {
                    PackageManager::Yarn
                } else if prefix.starts_with("pnpm") {
                    PackageManager::Pnpm
                } else if prefix.starts_with("bun") {
                    PackageManager::Bun
                } else {
                    continue;
                };

                // If it's already the expected package manager, no translation needed
                if detected_pm == *expected {
                    return None;
                }

                // Translate the command
                let args = command[prefix.len()..].trim();
                let translated = Self::build_translated_command(expected, op, args);

                return Some((
                    translated,
                    format!(
                        "Translated '{}' to '{}' (project uses {})",
                        detected_pm.command(),
                        expected.command(),
                        expected.name()
                    ),
                ));
            }
        }

        None
    }

    /// Build a translated command for the expected package manager
    fn build_translated_command(pm: &PackageManager, operation: &str, args: &str) -> String {
        match pm {
            PackageManager::Yarn => {
                match operation {
                    "install" if args.is_empty() => "yarn".to_string(),
                    "install" | "add" => format!("yarn add {}", args),
                    "npx" => format!("yarn dlx {}", args),
                    _ => format!("yarn {}", args),
                }
            }
            PackageManager::Npm => {
                match operation {
                    "add" | "install" if !args.is_empty() => format!("npm install {}", args),
                    "install" => "npm install".to_string(),
                    _ => format!("npm {}", args),
                }
            }
            PackageManager::Pnpm => {
                match operation {
                    "install" if args.is_empty() => "pnpm install".to_string(),
                    "install" | "add" => format!("pnpm add {}", args),
                    "npx" => format!("pnpm dlx {}", args),
                    _ => format!("pnpm {}", args),
                }
            }
            PackageManager::Bun => {
                match operation {
                    "install" if args.is_empty() => "bun install".to_string(),
                    "install" | "add" => format!("bun add {}", args),
                    "npx" => format!("bunx {}", args),
                    _ => format!("bun {}", args),
                }
            }
            _ => format!("{} {}", pm.command(), args),
        }
    }

    /// Suggest a command using user's aliases
    ///
    /// This doesn't translate but provides an alternative using aliases.
    pub fn suggest_alias(command: &str, preferences: &UserPreferences) -> Option<String> {
        preferences.shell.find_alias_for_command(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;
    use crate::preferences::{ProjectContext, ShellProfile};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_preferences(pm: PackageManager, aliases: HashMap<String, String>) -> UserPreferences {
        UserPreferences {
            project: ProjectContext {
                package_manager: Some(pm),
                build_tool: None,
                languages: vec![],
                root_path: PathBuf::from("/test"),
                detected_files: vec![],
            },
            shell: ShellProfile {
                aliases,
                exports: HashMap::new(),
                path_additions: vec![],
                shell_type: ShellType::Zsh,
                profile_files: vec![],
            },
            detected_at: Utc::now(),
            cache_key: "/test".to_string(),
        }
    }

    #[test]
    fn test_translate_npm_to_yarn() {
        let prefs = make_preferences(PackageManager::Yarn, HashMap::new());

        let result = CommandTranslator::translate("npm install lodash", &prefs);
        assert!(result.was_translated);
        assert_eq!(result.translated, "yarn add lodash");
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_translate_yarn_install_no_args() {
        let prefs = make_preferences(PackageManager::Npm, HashMap::new());

        let result = CommandTranslator::translate("yarn", &prefs);
        // "yarn" by itself shouldn't be translated since it doesn't start with our patterns
        assert!(!result.was_translated);
    }

    #[test]
    fn test_translate_npx_to_yarn_dlx() {
        let prefs = make_preferences(PackageManager::Yarn, HashMap::new());

        let result = CommandTranslator::translate("npx create-react-app my-app", &prefs);
        assert!(result.was_translated);
        assert_eq!(result.translated, "yarn dlx create-react-app my-app");
    }

    #[test]
    fn test_no_translation_when_correct() {
        let prefs = make_preferences(PackageManager::Yarn, HashMap::new());

        let result = CommandTranslator::translate("yarn add lodash", &prefs);
        assert!(!result.was_translated);
        assert_eq!(result.translated, "yarn add lodash");
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_alias_suggestion() {
        let mut aliases = HashMap::new();
        aliases.insert("gst".to_string(), "git status".to_string());

        let prefs = make_preferences(PackageManager::Npm, aliases);

        let result = CommandTranslator::translate("git status", &prefs);
        // Should have a change note about the alias
        assert!(!result.changes.is_empty());
        assert!(result.changes[0].contains("gst"));
    }

    #[test]
    fn test_translate_pnpm_to_bun() {
        let prefs = make_preferences(PackageManager::Bun, HashMap::new());

        let result = CommandTranslator::translate("pnpm add axios", &prefs);
        assert!(result.was_translated);
        assert_eq!(result.translated, "bun add axios");
    }

    #[test]
    fn test_suggest_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("gst".to_string(), "git status".to_string());
        aliases.insert("gco".to_string(), "git checkout".to_string());

        let prefs = make_preferences(PackageManager::Npm, aliases);

        let suggestion = CommandTranslator::suggest_alias("git status", &prefs);
        assert_eq!(suggestion, Some("gst".to_string()));

        let suggestion = CommandTranslator::suggest_alias("git checkout main", &prefs);
        assert_eq!(suggestion, Some("gco main".to_string()));
    }

    #[test]
    fn test_unchanged_command() {
        let unchanged = TranslatedCommand::unchanged("ls -la".to_string());

        assert!(!unchanged.was_translated);
        assert_eq!(unchanged.original, unchanged.translated);
        assert!(unchanged.changes.is_empty());
    }
}
