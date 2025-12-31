//! Core types for installation automation
//!
//! Defines installation plans, steps, prerequisites, and rollback procedures.

use crate::tips::shell::TipsShellType;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during installation
#[derive(Error, Debug)]
pub enum InstallationError {
    #[error("Prerequisite not met: {message}")]
    PrerequisiteNotMet { message: String },

    #[error("Step failed: {step}: {message}")]
    StepFailed { step: String, message: String },

    #[error("Command failed with exit code {code}: {message}")]
    CommandFailed { code: i32, message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("User cancelled installation")]
    UserCancelled,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Unsupported shell: {0:?}")]
    UnsupportedShell(TipsShellType),
}

/// An installation plan describing how to install something
#[derive(Debug, Clone)]
pub struct InstallationPlan {
    /// Name of what's being installed
    pub name: String,

    /// Detailed description
    pub description: String,

    /// Required conditions before installation
    pub prerequisites: Vec<Prerequisite>,

    /// Installation steps to execute
    pub steps: Vec<InstallStep>,

    /// Verification steps to confirm success
    pub verification: Vec<VerificationStep>,

    /// Optional rollback plan if installation fails
    pub rollback: Option<RollbackPlan>,

    /// Target shells this installation applies to
    pub shells: Vec<TipsShellType>,

    /// URL for more information
    pub url: Option<String>,
}

impl InstallationPlan {
    /// Create a new installation plan
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            prerequisites: Vec::new(),
            steps: Vec::new(),
            verification: Vec::new(),
            rollback: None,
            shells: Vec::new(),
            url: None,
        }
    }

    /// Add a prerequisite
    pub fn with_prerequisite(mut self, prereq: Prerequisite) -> Self {
        self.prerequisites.push(prereq);
        self
    }

    /// Add an installation step
    pub fn with_step(mut self, step: InstallStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add a verification step
    pub fn with_verification(mut self, verify: VerificationStep) -> Self {
        self.verification.push(verify);
        self
    }

    /// Set the rollback plan
    pub fn with_rollback(mut self, rollback: RollbackPlan) -> Self {
        self.rollback = Some(rollback);
        self
    }

    /// Set supported shells
    pub fn with_shells(mut self, shells: Vec<TipsShellType>) -> Self {
        self.shells = shells;
        self
    }

    /// Set the URL for more info
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Check if this plan is applicable to a shell
    pub fn applies_to_shell(&self, shell: TipsShellType) -> bool {
        self.shells.is_empty() || self.shells.contains(&shell)
    }

    /// Get a summary of the plan
    pub fn summary(&self) -> String {
        let steps_count = self.steps.len();
        let prereqs_count = self.prerequisites.len();
        format!(
            "{}: {} ({} prerequisites, {} steps)",
            self.name, self.description, prereqs_count, steps_count
        )
    }
}

/// A prerequisite condition that must be met
#[derive(Debug, Clone)]
pub enum Prerequisite {
    /// Specific shell type required
    ShellType(TipsShellType),

    /// A command must exist in PATH
    CommandExists(String),

    /// A path must NOT exist (to avoid reinstalling)
    NotInstalled(PathBuf),

    /// A path must exist
    PathExists(PathBuf),

    /// A file must contain a pattern
    FileContains {
        path: PathBuf,
        pattern: String,
    },

    /// A file must NOT contain a pattern
    FileNotContains {
        path: PathBuf,
        pattern: String,
    },

    /// Internet connectivity required
    InternetAccess,

    /// Running as specific user (not root, or must be root)
    NotRoot,
    MustBeRoot,

    /// Custom check with description
    Custom {
        description: String,
        check_command: String,
    },
}

impl Prerequisite {
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::ShellType(shell) => format!("Shell must be {:?}", shell),
            Self::CommandExists(cmd) => format!("Command '{}' must be in PATH", cmd),
            Self::NotInstalled(path) => {
                format!("Must not already be installed ({})", path.display())
            }
            Self::PathExists(path) => format!("Path must exist: {}", path.display()),
            Self::FileContains { path, pattern } => {
                format!("File {} must contain: {}", path.display(), pattern)
            }
            Self::FileNotContains { path, pattern } => {
                format!("File {} must NOT contain: {}", path.display(), pattern)
            }
            Self::InternetAccess => "Internet connectivity required".to_string(),
            Self::NotRoot => "Must not run as root".to_string(),
            Self::MustBeRoot => "Must run as root/sudo".to_string(),
            Self::Custom { description, .. } => description.clone(),
        }
    }
}

/// An installation step to execute
#[derive(Debug, Clone)]
pub enum InstallStep {
    /// Show message and ask for confirmation
    Confirmation { message: String },

    /// Show informational message
    Message {
        message: String,
        level: MessageLevel,
    },

    /// Create a backup of a file
    Backup { path: PathBuf, label: String },

    /// Run a shell command
    Run {
        command: String,
        description: String,
        /// Continue if this step fails
        continue_on_error: bool,
    },

    /// Add a line to a config file
    AddToConfig {
        path: PathBuf,
        content: String,
        /// Pattern to check if already present
        skip_if_contains: Option<String>,
    },

    /// Replace a pattern in a config file
    ReplaceInConfig {
        path: PathBuf,
        pattern: String,
        replacement: String,
    },

    /// Create a directory
    CreateDir { path: PathBuf },

    /// Create/write a file
    WriteFile { path: PathBuf, content: String },

    /// Download a file
    Download {
        url: String,
        destination: PathBuf,
        checksum: Option<String>,
    },

    /// Enable an Oh My Zsh plugin
    EnableOmzPlugin { plugin: String },

    /// Enable a Prezto module
    EnablePreztoModule { module: String },

    /// Wait for user to press enter
    PauseForUser { message: String },
}

impl InstallStep {
    /// Get a short description of this step
    pub fn description(&self) -> &str {
        match self {
            Self::Confirmation { message } => message,
            Self::Message { message, .. } => message,
            Self::Backup { label, .. } => label,
            Self::Run { description, .. } => description,
            Self::AddToConfig { .. } => "Add configuration",
            Self::ReplaceInConfig { .. } => "Update configuration",
            Self::CreateDir { .. } => "Create directory",
            Self::WriteFile { .. } => "Write file",
            Self::Download { .. } => "Download file",
            Self::EnableOmzPlugin { plugin } => plugin,
            Self::EnablePreztoModule { module } => module,
            Self::PauseForUser { message } => message,
        }
    }
}

/// Message level for informational steps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MessageLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// A verification step to confirm installation success
#[derive(Debug, Clone)]
pub enum VerificationStep {
    /// Check that a path exists
    PathExists(PathBuf),

    /// Check that a path does NOT exist
    PathNotExists(PathBuf),

    /// Check that a file contains a pattern
    FileContains { path: PathBuf, pattern: String },

    /// Check that a command is now available
    CommandExists(String),

    /// Run a command and check exit code is 0
    CommandSucceeds(String),

    /// Custom verification with description
    Custom {
        description: String,
        command: String,
    },
}

impl VerificationStep {
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::PathExists(path) => format!("Verify {} exists", path.display()),
            Self::PathNotExists(path) => format!("Verify {} does not exist", path.display()),
            Self::FileContains { path, pattern } => {
                format!("Verify {} contains '{}'", path.display(), pattern)
            }
            Self::CommandExists(cmd) => format!("Verify '{}' command is available", cmd),
            Self::CommandSucceeds(cmd) => format!("Verify '{}' succeeds", cmd),
            Self::Custom { description, .. } => description.clone(),
        }
    }
}

/// A rollback plan for when installation fails
#[derive(Debug, Clone, Default)]
pub struct RollbackPlan {
    /// Backup labels to restore
    pub restore_backups: Vec<String>,

    /// Paths to remove
    pub remove_paths: Vec<PathBuf>,

    /// Commands to run for cleanup
    pub cleanup_commands: Vec<String>,

    /// Custom rollback message
    pub message: Option<String>,
}

impl RollbackPlan {
    /// Create a new empty rollback plan
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a backup to restore
    pub fn with_backup(mut self, label: impl Into<String>) -> Self {
        self.restore_backups.push(label.into());
        self
    }

    /// Add a path to remove
    pub fn with_remove(mut self, path: PathBuf) -> Self {
        self.remove_paths.push(path);
        self
    }

    /// Add a cleanup command
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.cleanup_commands.push(cmd.into());
        self
    }

    /// Set the rollback message
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Check if this rollback plan has anything to do
    pub fn is_empty(&self) -> bool {
        self.restore_backups.is_empty()
            && self.remove_paths.is_empty()
            && self.cleanup_commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_plan_creation() {
        let plan = InstallationPlan::new("Test", "Test installation")
            .with_prerequisite(Prerequisite::ShellType(TipsShellType::Zsh))
            .with_step(InstallStep::Message {
                message: "Starting".into(),
                level: MessageLevel::Info,
            });

        assert_eq!(plan.name, "Test");
        assert_eq!(plan.prerequisites.len(), 1);
        assert_eq!(plan.steps.len(), 1);
    }

    #[test]
    fn test_prerequisite_description() {
        let prereq = Prerequisite::CommandExists("curl".into());
        assert!(prereq.description().contains("curl"));
    }

    #[test]
    fn test_verification_description() {
        let verify = VerificationStep::PathExists("/test".into());
        assert!(verify.description().contains("/test"));
    }

    #[test]
    fn test_rollback_plan() {
        let rollback = RollbackPlan::new()
            .with_backup("zshrc")
            .with_remove("/path/to/remove".into());

        assert_eq!(rollback.restore_backups.len(), 1);
        assert_eq!(rollback.remove_paths.len(), 1);
        assert!(!rollback.is_empty());
    }

    #[test]
    fn test_plan_applies_to_shell() {
        let zsh_plan = InstallationPlan::new("Test", "Test").with_shells(vec![TipsShellType::Zsh]);

        assert!(zsh_plan.applies_to_shell(TipsShellType::Zsh));
        assert!(!zsh_plan.applies_to_shell(TipsShellType::Bash));

        // Empty shells means all shells
        let any_plan = InstallationPlan::new("Test", "Test");
        assert!(any_plan.applies_to_shell(TipsShellType::Zsh));
        assert!(any_plan.applies_to_shell(TipsShellType::Bash));
    }
}
