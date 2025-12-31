//! Plugin installation and enabling plans
//!
//! Provides installation plans for enabling plugins in various shell frameworks.

use crate::tips::automation::types::{
    InstallStep, InstallationPlan, MessageLevel, Prerequisite, RollbackPlan, VerificationStep,
};
use crate::tips::shell::TipsShellType;
use std::path::PathBuf;

/// Create a plan to enable an Oh My Zsh plugin
pub fn plugin_enable_plan(plugin: &str, description: &str) -> InstallationPlan {
    InstallationPlan::new(
        format!("{} Plugin", plugin),
        format!("Enable the {} plugin: {}", plugin, description),
    )
    .with_shells(vec![TipsShellType::Zsh])
    .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_prerequisite(Prerequisite::FileNotContains {
        path: PathBuf::from("~/.zshrc"),
        pattern: format!(" {} ", plugin),
    })
    .with_step(InstallStep::Backup {
        path: PathBuf::from("~/.zshrc"),
        label: format!("zshrc-pre-{}", plugin),
    })
    .with_step(InstallStep::EnableOmzPlugin {
        plugin: plugin.to_string(),
    })
    .with_step(InstallStep::Message {
        message: format!("{} plugin enabled!", plugin),
        level: MessageLevel::Success,
    })
    .with_verification(VerificationStep::FileContains {
        path: PathBuf::from("~/.zshrc"),
        pattern: plugin.to_string(),
    })
    .with_rollback(
        RollbackPlan::new()
            .with_backup(format!("zshrc-pre-{}", plugin))
            .with_message(format!("Restoring .zshrc before {} plugin", plugin)),
    )
}

/// Create a plan to install a custom Oh My Zsh plugin from GitHub
pub fn custom_plugin_install_plan(
    plugin_name: &str,
    repo_url: &str,
    description: &str,
) -> InstallationPlan {
    let plugin_path = format!("~/.oh-my-zsh/custom/plugins/{}", plugin_name);

    InstallationPlan::new(
        format!("{} Plugin", plugin_name),
        description.to_string(),
    )
    .with_shells(vec![TipsShellType::Zsh])
    .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_prerequisite(Prerequisite::CommandExists("git".to_string()))
    .with_prerequisite(Prerequisite::NotInstalled(PathBuf::from(&plugin_path)))
    .with_step(InstallStep::Message {
        message: format!("Installing {} plugin...", plugin_name),
        level: MessageLevel::Info,
    })
    .with_step(InstallStep::Run {
        command: format!("git clone --depth=1 {} {}", repo_url, plugin_path),
        description: format!("Clone {} plugin", plugin_name),
        continue_on_error: false,
    })
    .with_step(InstallStep::Backup {
        path: PathBuf::from("~/.zshrc"),
        label: format!("zshrc-pre-{}", plugin_name),
    })
    .with_step(InstallStep::EnableOmzPlugin {
        plugin: plugin_name.to_string(),
    })
    .with_step(InstallStep::Message {
        message: format!("{} plugin installed and enabled!", plugin_name),
        level: MessageLevel::Success,
    })
    .with_verification(VerificationStep::PathExists(PathBuf::from(&plugin_path)))
    .with_verification(VerificationStep::FileContains {
        path: PathBuf::from("~/.zshrc"),
        pattern: plugin_name.to_string(),
    })
    .with_rollback(
        RollbackPlan::new()
            .with_backup(format!("zshrc-pre-{}", plugin_name))
            .with_remove(PathBuf::from(&plugin_path))
            .with_message(format!("Removing {} plugin and restoring .zshrc", plugin_name)),
    )
}

/// Popular third-party plugins with installation info
pub fn popular_third_party_plugins() -> Vec<PluginInfo> {
    vec![
        PluginInfo {
            name: "zsh-autosuggestions",
            repo: "https://github.com/zsh-users/zsh-autosuggestions",
            description: "Fish-like autosuggestions for zsh",
            category: PluginCategory::Productivity,
        },
        PluginInfo {
            name: "zsh-syntax-highlighting",
            repo: "https://github.com/zsh-users/zsh-syntax-highlighting",
            description: "Fish shell like syntax highlighting for Zsh",
            category: PluginCategory::Productivity,
        },
        PluginInfo {
            name: "zsh-completions",
            repo: "https://github.com/zsh-users/zsh-completions",
            description: "Additional completion definitions for Zsh",
            category: PluginCategory::Completions,
        },
        PluginInfo {
            name: "fast-syntax-highlighting",
            repo: "https://github.com/zdharma-continuum/fast-syntax-highlighting",
            description: "Feature-rich syntax highlighting for Zsh",
            category: PluginCategory::Productivity,
        },
        PluginInfo {
            name: "zsh-history-substring-search",
            repo: "https://github.com/zsh-users/zsh-history-substring-search",
            description: "Fish-like history search",
            category: PluginCategory::Productivity,
        },
        PluginInfo {
            name: "fzf-tab",
            repo: "https://github.com/Aloxaf/fzf-tab",
            description: "Replace zsh's default completion selection menu with fzf",
            category: PluginCategory::Productivity,
        },
        PluginInfo {
            name: "you-should-use",
            repo: "https://github.com/MichaelAquilina/zsh-you-should-use",
            description: "Reminds you to use your aliases",
            category: PluginCategory::Productivity,
        },
        PluginInfo {
            name: "alias-tips",
            repo: "https://github.com/djui/alias-tips",
            description: "Help remembering shell aliases",
            category: PluginCategory::Productivity,
        },
    ]
}

/// Built-in Oh My Zsh plugins worth enabling
pub fn recommended_builtin_plugins() -> Vec<(&'static str, &'static str, PluginCategory)> {
    vec![
        ("git", "Git aliases and functions", PluginCategory::Development),
        ("docker", "Docker command completions", PluginCategory::Development),
        ("docker-compose", "Docker Compose completions", PluginCategory::Development),
        ("kubectl", "Kubernetes command completions", PluginCategory::Development),
        ("helm", "Helm command completions", PluginCategory::Development),
        ("npm", "npm command completions", PluginCategory::Development),
        ("yarn", "Yarn command completions", PluginCategory::Development),
        ("rust", "Rust and Cargo completions", PluginCategory::Development),
        ("python", "Python aliases", PluginCategory::Development),
        ("pip", "pip command completions", PluginCategory::Development),
        ("golang", "Go development aliases", PluginCategory::Development),
        ("aws", "AWS CLI completions", PluginCategory::Cloud),
        ("gcloud", "Google Cloud completions", PluginCategory::Cloud),
        ("terraform", "Terraform completions", PluginCategory::Cloud),
        ("sudo", "Press ESC twice to prefix previous command with sudo", PluginCategory::Productivity),
        ("copypath", "Copy current path to clipboard", PluginCategory::Productivity),
        ("copyfile", "Copy file content to clipboard", PluginCategory::Productivity),
        ("web-search", "Search engines from command line", PluginCategory::Productivity),
        ("extract", "One command to extract any archive", PluginCategory::Utility),
        ("z", "Jump to frequently used directories", PluginCategory::Navigation),
        ("autojump", "Shell extension for quick navigation", PluginCategory::Navigation),
        ("fzf", "Fuzzy finder integration", PluginCategory::Productivity),
        ("thefuck", "Auto-correct mistyped commands", PluginCategory::Productivity),
        ("colored-man-pages", "Colorized man pages", PluginCategory::Utility),
        ("command-not-found", "Suggest package installation", PluginCategory::Utility),
        ("safe-paste", "Prevent auto-execution on paste", PluginCategory::Safety),
        ("history", "History command aliases", PluginCategory::Productivity),
        ("dirhistory", "Navigate directories with alt+arrows", PluginCategory::Navigation),
    ]
}

/// Information about a plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: &'static str,
    /// Git repository URL
    pub repo: &'static str,
    /// Description
    pub description: &'static str,
    /// Category
    pub category: PluginCategory,
}

impl PluginInfo {
    /// Create an installation plan for this plugin
    pub fn installation_plan(&self) -> InstallationPlan {
        custom_plugin_install_plan(self.name, self.repo, self.description)
    }
}

/// Plugin categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCategory {
    /// Development tools
    Development,
    /// Cloud/infrastructure
    Cloud,
    /// Productivity enhancements
    Productivity,
    /// Command completions
    Completions,
    /// Navigation helpers
    Navigation,
    /// General utilities
    Utility,
    /// Safety/security
    Safety,
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "Development"),
            Self::Cloud => write!(f, "Cloud"),
            Self::Productivity => write!(f, "Productivity"),
            Self::Completions => write!(f, "Completions"),
            Self::Navigation => write!(f, "Navigation"),
            Self::Utility => write!(f, "Utility"),
            Self::Safety => write!(f, "Safety"),
        }
    }
}

/// Get plugins by category
pub fn plugins_by_category(category: PluginCategory) -> Vec<(&'static str, &'static str)> {
    recommended_builtin_plugins()
        .iter()
        .filter(|(_, _, cat)| *cat == category)
        .map(|(name, desc, _)| (*name, *desc))
        .collect()
}

/// Create a plan to disable an Oh My Zsh plugin
pub fn plugin_disable_plan(plugin: &str) -> InstallationPlan {
    InstallationPlan::new(
        format!("Disable {} Plugin", plugin),
        format!("Remove the {} plugin from Oh My Zsh", plugin),
    )
    .with_shells(vec![TipsShellType::Zsh])
    .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_prerequisite(Prerequisite::FileContains {
        path: PathBuf::from("~/.zshrc"),
        pattern: plugin.to_string(),
    })
    .with_step(InstallStep::Backup {
        path: PathBuf::from("~/.zshrc"),
        label: format!("zshrc-pre-disable-{}", plugin),
    })
    .with_step(InstallStep::ReplaceInConfig {
        path: PathBuf::from("~/.zshrc"),
        pattern: format!(" {} ", plugin),
        replacement: " ".to_string(),
    })
    .with_step(InstallStep::ReplaceInConfig {
        path: PathBuf::from("~/.zshrc"),
        pattern: format!("({})", plugin),
        replacement: "()".to_string(),
    })
    .with_step(InstallStep::Message {
        message: format!("{} plugin disabled", plugin),
        level: MessageLevel::Success,
    })
}

/// Create a batch plan to enable multiple plugins
pub fn batch_plugin_enable_plan(plugins: &[&str]) -> InstallationPlan {
    let plugins_str = plugins.join(", ");

    let mut plan = InstallationPlan::new(
        "Enable Multiple Plugins",
        format!("Enable plugins: {}", plugins_str),
    )
    .with_shells(vec![TipsShellType::Zsh])
    .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_step(InstallStep::Backup {
        path: PathBuf::from("~/.zshrc"),
        label: "zshrc-pre-batch-plugins".to_string(),
    });

    for plugin in plugins {
        plan.steps.push(InstallStep::EnableOmzPlugin {
            plugin: (*plugin).to_string(),
        });
    }

    plan.steps.push(InstallStep::Message {
        message: format!("Enabled {} plugins: {}", plugins.len(), plugins_str),
        level: MessageLevel::Success,
    });

    plan.rollback = Some(
        RollbackPlan::new()
            .with_backup("zshrc-pre-batch-plugins")
            .with_message("Restoring .zshrc before plugin changes"),
    );

    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_enable_plan() {
        let plan = plugin_enable_plan("git", "Git aliases and functions");
        assert!(plan.name.contains("git"));
        assert!(plan.applies_to_shell(TipsShellType::Zsh));
        assert!(!plan.prerequisites.is_empty());
    }

    #[test]
    fn test_custom_plugin_plan() {
        let plan = custom_plugin_install_plan(
            "zsh-autosuggestions",
            "https://github.com/zsh-users/zsh-autosuggestions",
            "Fish-like autosuggestions",
        );
        assert!(plan.name.contains("zsh-autosuggestions"));
        assert!(plan.steps.iter().any(|s| matches!(s, InstallStep::Run { .. })));
    }

    #[test]
    fn test_popular_plugins() {
        let plugins = popular_third_party_plugins();
        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|p| p.name == "zsh-autosuggestions"));
    }

    #[test]
    fn test_recommended_builtin_plugins() {
        let plugins = recommended_builtin_plugins();
        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|(name, _, _)| *name == "git"));
    }

    #[test]
    fn test_plugins_by_category() {
        let dev_plugins = plugins_by_category(PluginCategory::Development);
        assert!(!dev_plugins.is_empty());
        assert!(dev_plugins.iter().any(|(name, _)| *name == "git"));
    }

    #[test]
    fn test_plugin_disable_plan() {
        let plan = plugin_disable_plan("docker");
        assert!(plan.name.contains("Disable"));
        assert!(plan.prerequisites.iter().any(|p| {
            matches!(p, Prerequisite::FileContains { pattern, .. } if pattern == "docker")
        }));
    }

    #[test]
    fn test_batch_plugin_plan() {
        let plan = batch_plugin_enable_plan(&["git", "docker", "kubectl"]);
        assert!(plan.name.contains("Multiple"));
        // Should have a step for each plugin
        let enable_steps: Vec<_> = plan
            .steps
            .iter()
            .filter(|s| matches!(s, InstallStep::EnableOmzPlugin { .. }))
            .collect();
        assert_eq!(enable_steps.len(), 3);
    }

    #[test]
    fn test_plugin_info_to_plan() {
        let plugin = &popular_third_party_plugins()[0];
        let plan = plugin.installation_plan();
        assert!(plan.name.contains(plugin.name));
    }
}
