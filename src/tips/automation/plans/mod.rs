//! Built-in installation plans
//!
//! Provides pre-configured installation plans for common shell enhancements.

mod ohmyzsh;
mod plugins;

pub use ohmyzsh::ohmyzsh_install_plan;
pub use plugins::plugin_enable_plan;

use super::types::{InstallationPlan, Prerequisite};
use crate::tips::shell::TipsShellType;

/// Get all available installation plans
pub fn all_plans() -> Vec<InstallationPlan> {
    let mut plans = Vec::new();

    // Oh My Zsh installation
    plans.push(ohmyzsh_install_plan());

    // Popular plugin enable plans
    plans.extend(popular_plugin_plans());

    plans
}

/// Get popular plugin installation plans
pub fn popular_plugin_plans() -> Vec<InstallationPlan> {
    vec![
        plugin_enable_plan("git", "Git aliases and functions"),
        plugin_enable_plan("docker", "Docker command completions"),
        plugin_enable_plan("kubectl", "Kubernetes command completions"),
        plugin_enable_plan("npm", "npm command completions"),
        plugin_enable_plan("rust", "Rust and Cargo completions"),
        plugin_enable_plan("python", "Python aliases"),
        plugin_enable_plan("golang", "Go development aliases"),
        plugin_enable_plan("aws", "AWS CLI completions"),
        plugin_enable_plan("gcloud", "Google Cloud completions"),
        plugin_enable_plan("terraform", "Terraform completions"),
    ]
}

/// Get plans applicable to a specific shell
pub fn plans_for_shell(shell: TipsShellType) -> Vec<InstallationPlan> {
    all_plans()
        .into_iter()
        .filter(|p| p.applies_to_shell(shell))
        .collect()
}

/// Find a plan by name
pub fn find_plan(name: &str) -> Option<InstallationPlan> {
    all_plans().into_iter().find(|p| {
        p.name.eq_ignore_ascii_case(name) || p.name.to_lowercase().replace(' ', "-") == name.to_lowercase()
    })
}

/// Check if a plan's prerequisites are met
pub fn check_plan_prerequisites(
    plan: &InstallationPlan,
    shell: Option<TipsShellType>,
) -> Vec<(Prerequisite, bool)> {
    plan.prerequisites
        .iter()
        .map(|prereq| {
            let met = match prereq {
                Prerequisite::ShellType(required) => {
                    shell.map_or(true, |s| s == *required)
                }
                Prerequisite::CommandExists(cmd) => {
                    std::process::Command::new("which")
                        .arg(cmd)
                        .output()
                        .map_or(false, |o| o.status.success())
                }
                Prerequisite::PathExists(path) => {
                    let expanded = expand_tilde(path);
                    expanded.exists()
                }
                Prerequisite::NotInstalled(path) => {
                    let expanded = expand_tilde(path);
                    !expanded.exists()
                }
                _ => true, // Can't easily check other prerequisites without execution
            };
            (prereq.clone(), met)
        })
        .collect()
}

/// Expand tilde in paths
fn expand_tilde(path: &std::path::Path) -> std::path::PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path_str[2..]);
        }
    } else if path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_plans() {
        let plans = all_plans();
        assert!(!plans.is_empty());
    }

    #[test]
    fn test_find_plan() {
        let plan = find_plan("Oh My Zsh");
        assert!(plan.is_some());
        assert_eq!(plan.unwrap().name, "Oh My Zsh");
    }

    #[test]
    fn test_find_plan_case_insensitive() {
        let plan = find_plan("oh-my-zsh");
        assert!(plan.is_some());
    }

    #[test]
    fn test_plans_for_shell() {
        let zsh_plans = plans_for_shell(TipsShellType::Zsh);
        assert!(!zsh_plans.is_empty());

        // Oh My Zsh should be in zsh plans
        assert!(zsh_plans.iter().any(|p| p.name == "Oh My Zsh"));
    }

    #[test]
    fn test_popular_plugin_plans() {
        let plans = popular_plugin_plans();
        assert!(!plans.is_empty());
        assert!(plans.iter().any(|p| p.name.contains("git")));
    }
}
