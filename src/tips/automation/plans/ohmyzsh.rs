//! Oh My Zsh installation plan
//!
//! Provides a complete installation plan for Oh My Zsh with safety checks,
//! backups, and rollback support.

use crate::tips::automation::types::{
    InstallStep, InstallationPlan, MessageLevel, Prerequisite, RollbackPlan, VerificationStep,
};
use crate::tips::shell::TipsShellType;
use std::path::PathBuf;

/// Create the Oh My Zsh installation plan
pub fn ohmyzsh_install_plan() -> InstallationPlan {
    InstallationPlan::new(
        "Oh My Zsh",
        "A delightful community-driven framework for managing your zsh configuration",
    )
    .with_url("https://ohmyz.sh")
    .with_shells(vec![TipsShellType::Zsh])
    // Prerequisites
    .with_prerequisite(Prerequisite::ShellType(TipsShellType::Zsh))
    .with_prerequisite(Prerequisite::CommandExists("git".to_string()))
    .with_prerequisite(Prerequisite::CommandExists("curl".to_string()))
    .with_prerequisite(Prerequisite::NotInstalled(PathBuf::from("~/.oh-my-zsh")))
    .with_prerequisite(Prerequisite::NotRoot)
    .with_prerequisite(Prerequisite::InternetAccess)
    // Steps
    .with_step(InstallStep::Message {
        message: "Installing Oh My Zsh - a zsh configuration framework".to_string(),
        level: MessageLevel::Info,
    })
    .with_step(InstallStep::Confirmation {
        message: "This will modify your ~/.zshrc. Continue?".to_string(),
    })
    .with_step(InstallStep::Backup {
        path: PathBuf::from("~/.zshrc"),
        label: "zshrc-pre-omz".to_string(),
    })
    .with_step(InstallStep::Message {
        message: "Downloading and installing Oh My Zsh...".to_string(),
        level: MessageLevel::Info,
    })
    .with_step(InstallStep::Run {
        command: r#"sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended"#.to_string(),
        description: "Install Oh My Zsh".to_string(),
        continue_on_error: false,
    })
    .with_step(InstallStep::Message {
        message: "Oh My Zsh installed!".to_string(),
        level: MessageLevel::Success,
    })
    // Verification
    .with_verification(VerificationStep::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_verification(VerificationStep::PathExists(PathBuf::from(
        "~/.oh-my-zsh/oh-my-zsh.sh",
    )))
    .with_verification(VerificationStep::FileContains {
        path: PathBuf::from("~/.zshrc"),
        pattern: "ZSH_THEME".to_string(),
    })
    // Rollback
    .with_rollback(
        RollbackPlan::new()
            .with_backup("zshrc-pre-omz")
            .with_remove(PathBuf::from("~/.oh-my-zsh"))
            .with_message("Removing Oh My Zsh and restoring original .zshrc"),
    )
}

/// Create a plan to install Oh My Zsh with specific theme and plugins
pub fn ohmyzsh_custom_install_plan(theme: &str, plugins: &[&str]) -> InstallationPlan {
    let mut plan = ohmyzsh_install_plan();

    // Add step to set theme
    plan.steps.push(InstallStep::ReplaceInConfig {
        path: PathBuf::from("~/.zshrc"),
        pattern: "ZSH_THEME=\"robbyrussell\"".to_string(),
        replacement: format!("ZSH_THEME=\"{}\"", theme),
    });

    // Add steps to enable plugins
    if !plugins.is_empty() {
        let plugins_str = plugins.join(" ");
        plan.steps.push(InstallStep::ReplaceInConfig {
            path: PathBuf::from("~/.zshrc"),
            pattern: "plugins=(git)".to_string(),
            replacement: format!("plugins=({})", plugins_str),
        });
    }

    plan
}

/// Create a plan to update Oh My Zsh
pub fn ohmyzsh_update_plan() -> InstallationPlan {
    InstallationPlan::new("Oh My Zsh Update", "Update Oh My Zsh to the latest version")
        .with_shells(vec![TipsShellType::Zsh])
        .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
        .with_step(InstallStep::Run {
            command: "cd ~/.oh-my-zsh && git pull".to_string(),
            description: "Update Oh My Zsh".to_string(),
            continue_on_error: false,
        })
        .with_step(InstallStep::Message {
            message: "Oh My Zsh updated successfully!".to_string(),
            level: MessageLevel::Success,
        })
}

/// Create a plan to uninstall Oh My Zsh
pub fn ohmyzsh_uninstall_plan() -> InstallationPlan {
    InstallationPlan::new(
        "Uninstall Oh My Zsh",
        "Remove Oh My Zsh and restore original .zshrc",
    )
    .with_shells(vec![TipsShellType::Zsh])
    .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_step(InstallStep::Confirmation {
        message: "This will remove Oh My Zsh. Are you sure?".to_string(),
    })
    .with_step(InstallStep::Backup {
        path: PathBuf::from("~/.zshrc"),
        label: "zshrc-pre-uninstall".to_string(),
    })
    .with_step(InstallStep::Run {
        command: "uninstall_oh_my_zsh 2>/dev/null || rm -rf ~/.oh-my-zsh".to_string(),
        description: "Remove Oh My Zsh".to_string(),
        continue_on_error: true,
    })
    .with_step(InstallStep::Run {
        command: r#"if [ -f ~/.zshrc.pre-oh-my-zsh ]; then mv ~/.zshrc.pre-oh-my-zsh ~/.zshrc; fi"#
            .to_string(),
        description: "Restore original .zshrc".to_string(),
        continue_on_error: true,
    })
    .with_verification(VerificationStep::PathNotExists(PathBuf::from(
        "~/.oh-my-zsh",
    )))
}

/// Create a plan to install a custom Oh My Zsh theme
pub fn ohmyzsh_theme_install_plan(theme_name: &str, theme_repo: &str) -> InstallationPlan {
    let theme_path = format!("~/.oh-my-zsh/custom/themes/{}", theme_name);

    InstallationPlan::new(
        format!("{} Theme", theme_name),
        format!("Install the {} theme for Oh My Zsh", theme_name),
    )
    .with_shells(vec![TipsShellType::Zsh])
    .with_prerequisite(Prerequisite::PathExists(PathBuf::from("~/.oh-my-zsh")))
    .with_prerequisite(Prerequisite::CommandExists("git".to_string()))
    .with_step(InstallStep::Run {
        command: format!("git clone --depth=1 {} {}", theme_repo, theme_path),
        description: format!("Clone {} theme", theme_name),
        continue_on_error: false,
    })
    .with_step(InstallStep::ReplaceInConfig {
        path: PathBuf::from("~/.zshrc"),
        pattern: "ZSH_THEME=".to_string(),
        replacement: format!("ZSH_THEME=\"{}\"", theme_name),
    })
    .with_verification(VerificationStep::PathExists(PathBuf::from(&theme_path)))
}

/// Popular Oh My Zsh themes with their installation info
pub fn popular_themes() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "powerlevel10k",
            "https://github.com/romkatv/powerlevel10k.git",
            "A fast, flexible and feature-rich theme",
        ),
        (
            "spaceship",
            "https://github.com/spaceship-prompt/spaceship-prompt.git",
            "A minimalistic, powerful and customizable prompt",
        ),
        (
            "dracula",
            "https://github.com/dracula/zsh.git",
            "Dark theme for zsh",
        ),
        (
            "agnoster",
            "", // Built-in
            "A powerline-inspired theme (built-in)",
        ),
        (
            "robbyrussell",
            "", // Built-in, default
            "The default Oh My Zsh theme",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ohmyzsh_plan_creation() {
        let plan = ohmyzsh_install_plan();
        assert_eq!(plan.name, "Oh My Zsh");
        assert!(!plan.prerequisites.is_empty());
        assert!(!plan.steps.is_empty());
        assert!(!plan.verification.is_empty());
        assert!(plan.rollback.is_some());
    }

    #[test]
    fn test_ohmyzsh_applies_to_zsh() {
        let plan = ohmyzsh_install_plan();
        assert!(plan.applies_to_shell(TipsShellType::Zsh));
        assert!(!plan.applies_to_shell(TipsShellType::Bash));
    }

    #[test]
    fn test_custom_install_plan() {
        let plan = ohmyzsh_custom_install_plan("agnoster", &["git", "docker", "kubectl"]);
        assert!(plan.steps.iter().any(|s| {
            matches!(s, InstallStep::ReplaceInConfig { replacement, .. } if replacement.contains("agnoster"))
        }));
    }

    #[test]
    fn test_update_plan() {
        let plan = ohmyzsh_update_plan();
        assert_eq!(plan.name, "Oh My Zsh Update");
        assert!(plan.steps.iter().any(
            |s| matches!(s, InstallStep::Run { command, .. } if command.contains("git pull"))
        ));
    }

    #[test]
    fn test_uninstall_plan() {
        let plan = ohmyzsh_uninstall_plan();
        assert!(plan.verification.iter().any(|v| {
            matches!(v, VerificationStep::PathNotExists(p) if p.to_string_lossy().contains(".oh-my-zsh"))
        }));
    }

    #[test]
    fn test_popular_themes() {
        let themes = popular_themes();
        assert!(!themes.is_empty());
        assert!(themes.iter().any(|(name, _, _)| *name == "powerlevel10k"));
    }
}
