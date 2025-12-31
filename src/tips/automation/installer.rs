//! Installation plan executor
//!
//! Executes installation plans step by step with user confirmation,
//! error handling, and rollback support.

use super::config_editor::ConfigEditor;
use super::shell_reload::ShellReload;
use super::types::{
    InstallStep, InstallationError, InstallationPlan, MessageLevel, Prerequisite, RollbackPlan,
    VerificationStep,
};
use crate::tips::shell::TipsShellType;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// Result of an installation operation
#[derive(Debug)]
pub enum InstallResult {
    /// Installation completed successfully
    Success {
        /// Messages from the installation
        messages: Vec<String>,
        /// Whether a shell reload is needed
        needs_reload: bool,
    },
    /// Installation was cancelled by user
    Cancelled,
    /// Installation failed (may have been rolled back)
    Failed {
        /// The error that occurred
        error: InstallationError,
        /// Whether rollback succeeded
        rolled_back: bool,
    },
    /// Dry run completed (no changes made)
    DryRun {
        /// Steps that would be executed
        steps: Vec<String>,
    },
}

impl InstallResult {
    /// Check if installation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

/// Installer for executing installation plans
pub struct Installer {
    /// Config editor for file modifications
    config_editor: ConfigEditor,
    /// Current shell type
    shell: Option<TipsShellType>,
    /// Whether to run in interactive mode
    interactive: bool,
    /// Whether to show verbose output
    verbose: bool,
}

impl Installer {
    /// Create a new installer
    pub fn new() -> Result<Self, InstallationError> {
        Ok(Self {
            config_editor: ConfigEditor::new()?,
            shell: None,
            interactive: true,
            verbose: false,
        })
    }

    /// Create an installer with a custom config editor
    pub fn with_config_editor(config_editor: ConfigEditor) -> Self {
        Self {
            config_editor,
            shell: None,
            interactive: true,
            verbose: false,
        }
    }

    /// Set the target shell
    pub fn with_shell(mut self, shell: TipsShellType) -> Self {
        self.shell = Some(shell);
        self
    }

    /// Disable interactive mode (auto-confirm)
    pub fn non_interactive(mut self) -> Self {
        self.interactive = false;
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Execute an installation plan
    pub fn execute(&mut self, plan: &InstallationPlan) -> InstallResult {
        let mut messages = Vec::new();
        let mut needs_reload = false;

        // Check shell compatibility
        if let Some(shell) = self.shell {
            if !plan.applies_to_shell(shell) {
                return InstallResult::Failed {
                    error: InstallationError::UnsupportedShell(shell),
                    rolled_back: false,
                };
            }
        }

        // Check prerequisites
        if let Err(e) = self.check_prerequisites(&plan.prerequisites) {
            return InstallResult::Failed {
                error: e,
                rolled_back: false,
            };
        }

        // Execute steps
        for (i, step) in plan.steps.iter().enumerate() {
            if self.verbose {
                messages.push(format!("[{}/{}] {}", i + 1, plan.steps.len(), step.description()));
            }

            match self.execute_step(step) {
                Ok(result) => {
                    if let Some(msg) = result.message {
                        messages.push(msg);
                    }
                    if result.needs_reload {
                        needs_reload = true;
                    }
                }
                Err(InstallationError::UserCancelled) => {
                    return InstallResult::Cancelled;
                }
                Err(e) => {
                    // Attempt rollback
                    let rolled_back = if let Some(ref rollback) = plan.rollback {
                        self.execute_rollback(rollback).is_ok()
                    } else {
                        false
                    };

                    return InstallResult::Failed {
                        error: e,
                        rolled_back,
                    };
                }
            }
        }

        // Run verification
        if let Err(e) = self.run_verification(&plan.verification) {
            let rolled_back = if let Some(ref rollback) = plan.rollback {
                self.execute_rollback(rollback).is_ok()
            } else {
                false
            };

            return InstallResult::Failed {
                error: e,
                rolled_back,
            };
        }

        messages.push(format!("{} installed successfully!", plan.name));

        InstallResult::Success {
            messages,
            needs_reload,
        }
    }

    /// Execute a dry run (show what would happen)
    pub fn execute_dry_run(&self, plan: &InstallationPlan) -> InstallResult {
        let mut steps = Vec::new();

        steps.push(format!("=== {} ===", plan.name));
        steps.push(plan.description.clone());
        steps.push(String::new());

        // Show prerequisites
        if !plan.prerequisites.is_empty() {
            steps.push("Prerequisites:".to_string());
            for prereq in &plan.prerequisites {
                steps.push(format!("  - {}", prereq.description()));
            }
            steps.push(String::new());
        }

        // Show steps
        steps.push("Steps:".to_string());
        for (i, step) in plan.steps.iter().enumerate() {
            steps.push(format!("  {}. {}", i + 1, self.describe_step(step)));
        }
        steps.push(String::new());

        // Show verification
        if !plan.verification.is_empty() {
            steps.push("Verification:".to_string());
            for verify in &plan.verification {
                steps.push(format!("  - {}", verify.description()));
            }
        }

        InstallResult::DryRun { steps }
    }

    /// Check all prerequisites
    fn check_prerequisites(&self, prerequisites: &[Prerequisite]) -> Result<(), InstallationError> {
        for prereq in prerequisites {
            self.check_prerequisite(prereq)?;
        }
        Ok(())
    }

    /// Check a single prerequisite
    fn check_prerequisite(&self, prereq: &Prerequisite) -> Result<(), InstallationError> {
        match prereq {
            Prerequisite::ShellType(shell) => {
                if let Some(current) = self.shell {
                    if current != *shell {
                        return Err(InstallationError::PrerequisiteNotMet {
                            message: format!(
                                "Shell must be {:?}, but current shell is {:?}",
                                shell, current
                            ),
                        });
                    }
                }
                Ok(())
            }

            Prerequisite::CommandExists(cmd) => {
                if Command::new("which").arg(cmd).output().is_ok() {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: format!("Command '{}' not found in PATH", cmd),
                    })
                }
            }

            Prerequisite::NotInstalled(path) => {
                let expanded = expand_tilde(path);
                if expanded.exists() {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: format!("Already installed at {}", expanded.display()),
                    })
                } else {
                    Ok(())
                }
            }

            Prerequisite::PathExists(path) => {
                let expanded = expand_tilde(path);
                if expanded.exists() {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: format!("Required path does not exist: {}", expanded.display()),
                    })
                }
            }

            Prerequisite::FileContains { path, pattern } => {
                let expanded = expand_tilde(path);
                if self.config_editor.contains(&expanded, pattern)? {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: format!(
                            "File {} does not contain required pattern",
                            expanded.display()
                        ),
                    })
                }
            }

            Prerequisite::FileNotContains { path, pattern } => {
                let expanded = expand_tilde(path);
                if !expanded.exists() || !self.config_editor.contains(&expanded, pattern)? {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: format!(
                            "File {} already contains pattern '{}'",
                            expanded.display(),
                            pattern
                        ),
                    })
                }
            }

            Prerequisite::InternetAccess => {
                // Simple check: try to reach a reliable host
                let result = Command::new("ping")
                    .args(["-c", "1", "-W", "2", "1.1.1.1"])
                    .output();

                if result.is_ok() && result.unwrap().status.success() {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: "No internet connectivity detected".to_string(),
                    })
                }
            }

            Prerequisite::NotRoot => {
                // Check if running as root using the id command
                let output = Command::new("id").arg("-u").output();
                let is_root = output
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
                    .unwrap_or(false);

                if !is_root {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: "Must not run as root".to_string(),
                    })
                }
            }

            Prerequisite::MustBeRoot => {
                // Check if running as root using the id command
                let output = Command::new("id").arg("-u").output();
                let is_root = output
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
                    .unwrap_or(false);

                if is_root {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: "Must run as root/sudo".to_string(),
                    })
                }
            }

            Prerequisite::Custom {
                description,
                check_command,
            } => {
                let result = Command::new("sh").arg("-c").arg(check_command).output();

                if result.is_ok() && result.unwrap().status.success() {
                    Ok(())
                } else {
                    Err(InstallationError::PrerequisiteNotMet {
                        message: description.clone(),
                    })
                }
            }
        }
    }

    /// Execute a single installation step
    fn execute_step(&mut self, step: &InstallStep) -> Result<StepResult, InstallationError> {
        match step {
            InstallStep::Confirmation { message } => {
                if self.interactive {
                    if !self.prompt_confirm(message)? {
                        return Err(InstallationError::UserCancelled);
                    }
                }
                Ok(StepResult::default())
            }

            InstallStep::Message { message, level } => {
                let prefix = match level {
                    MessageLevel::Info => "[INFO]",
                    MessageLevel::Success => "[OK]",
                    MessageLevel::Warning => "[WARN]",
                    MessageLevel::Error => "[ERROR]",
                };
                Ok(StepResult {
                    message: Some(format!("{} {}", prefix, message)),
                    needs_reload: false,
                })
            }

            InstallStep::Backup { path, label } => {
                let expanded = expand_tilde(path);
                self.config_editor.backup(&expanded, label)?;
                Ok(StepResult {
                    message: Some(format!("Backed up {} as '{}'", expanded.display(), label)),
                    needs_reload: false,
                })
            }

            InstallStep::Run {
                command,
                description,
                continue_on_error,
            } => {
                let output = Command::new("sh").arg("-c").arg(command).output()?;

                if !output.status.success() && !continue_on_error {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(InstallationError::CommandFailed {
                        code: output.status.code().unwrap_or(-1),
                        message: format!("{}: {}", description, stderr.trim()),
                    });
                }

                Ok(StepResult {
                    message: Some(description.clone()),
                    needs_reload: false,
                })
            }

            InstallStep::AddToConfig {
                path,
                content,
                skip_if_contains,
            } => {
                let expanded = expand_tilde(path);
                let added = self.config_editor.add_line_if_missing(
                    &expanded,
                    content,
                    skip_if_contains.as_deref(),
                )?;

                Ok(StepResult {
                    message: if added {
                        Some(format!("Added to {}", expanded.display()))
                    } else {
                        Some(format!("Already in {}", expanded.display()))
                    },
                    needs_reload: added,
                })
            }

            InstallStep::ReplaceInConfig {
                path,
                pattern,
                replacement,
            } => {
                let expanded = expand_tilde(path);
                let replaced = self.config_editor.replace_pattern(&expanded, pattern, replacement)?;

                Ok(StepResult {
                    message: if replaced {
                        Some(format!("Updated {}", expanded.display()))
                    } else {
                        Some(format!("Pattern not found in {}", expanded.display()))
                    },
                    needs_reload: replaced,
                })
            }

            InstallStep::CreateDir { path } => {
                let expanded = expand_tilde(path);
                if !expanded.exists() {
                    fs::create_dir_all(&expanded)?;
                    Ok(StepResult {
                        message: Some(format!("Created directory {}", expanded.display())),
                        needs_reload: false,
                    })
                } else {
                    Ok(StepResult {
                        message: Some(format!("Directory already exists: {}", expanded.display())),
                        needs_reload: false,
                    })
                }
            }

            InstallStep::WriteFile { path, content } => {
                let expanded = expand_tilde(path);

                // Ensure parent directory exists
                if let Some(parent) = expanded.parent() {
                    fs::create_dir_all(parent)?;
                }

                self.config_editor.write(&expanded, content)?;
                Ok(StepResult {
                    message: Some(format!("Wrote {}", expanded.display())),
                    needs_reload: false,
                })
            }

            InstallStep::Download {
                url,
                destination,
                checksum,
            } => {
                let expanded = expand_tilde(destination);

                // Use curl for downloading
                let mut args = vec!["-fsSL", "-o"];
                let dest_str = expanded.to_string_lossy();
                args.push(&dest_str);
                args.push(url);

                let output = Command::new("curl").args(&args).output()?;

                if !output.status.success() {
                    return Err(InstallationError::StepFailed {
                        step: "Download".to_string(),
                        message: format!("Failed to download {}", url),
                    });
                }

                // Verify checksum if provided
                if let Some(expected) = checksum {
                    let output = Command::new("sha256sum").arg(&expanded).output()?;
                    let actual = String::from_utf8_lossy(&output.stdout);
                    let actual_hash = actual.split_whitespace().next().unwrap_or("");

                    if actual_hash != expected {
                        fs::remove_file(&expanded)?;
                        return Err(InstallationError::StepFailed {
                            step: "Download".to_string(),
                            message: "Checksum verification failed".to_string(),
                        });
                    }
                }

                Ok(StepResult {
                    message: Some(format!("Downloaded {}", expanded.display())),
                    needs_reload: false,
                })
            }

            InstallStep::EnableOmzPlugin { plugin } => {
                // Add plugin to OMZ plugins array in .zshrc
                let zshrc = expand_tilde(Path::new("~/.zshrc"));

                // Read current content
                let content = if zshrc.exists() {
                    self.config_editor.read(&zshrc)?
                } else {
                    return Err(InstallationError::StepFailed {
                        step: "Enable OMZ Plugin".to_string(),
                        message: "~/.zshrc not found".to_string(),
                    });
                };

                // Check if already enabled
                if content.contains(&format!("plugins=(*{}*)", plugin))
                    || content.contains(&format!(" {} ", plugin))
                    || content.contains(&format!(" {})", plugin))
                    || content.contains(&format!("({})", plugin))
                    || content.contains(&format!("({} ", plugin))
                {
                    return Ok(StepResult {
                        message: Some(format!("Plugin '{}' already enabled", plugin)),
                        needs_reload: false,
                    });
                }

                // Find plugins=(...) line and add plugin
                let new_content = if let Some(start) = content.find("plugins=(") {
                    let end = content[start..].find(')').map(|i| start + i);
                    if let Some(end_pos) = end {
                        let before = &content[..end_pos];
                        let after = &content[end_pos..];
                        // Insert plugin before closing paren
                        format!("{} {}{}", before, plugin, after)
                    } else {
                        // Malformed plugins line, append new one
                        format!("{}\nplugins=({})\n", content, plugin)
                    }
                } else {
                    // No plugins line, add one
                    format!("{}\nplugins=({})\n", content, plugin)
                };

                self.config_editor.write(&zshrc, &new_content)?;

                Ok(StepResult {
                    message: Some(format!("Enabled Oh My Zsh plugin: {}", plugin)),
                    needs_reload: true,
                })
            }

            InstallStep::EnablePreztoModule { module } => {
                let zpreztorc = expand_tilde(Path::new("~/.zpreztorc"));

                // Similar logic for prezto modules
                let content = if zpreztorc.exists() {
                    self.config_editor.read(&zpreztorc)?
                } else {
                    return Err(InstallationError::StepFailed {
                        step: "Enable Prezto Module".to_string(),
                        message: "~/.zpreztorc not found".to_string(),
                    });
                };

                if content.contains(&format!("'{}'", module)) {
                    return Ok(StepResult {
                        message: Some(format!("Module '{}' already enabled", module)),
                        needs_reload: false,
                    });
                }

                // Append to module list
                let new_content = format!("{}\nzstyle ':prezto:load' pmodule '{}'\n", content, module);
                self.config_editor.write(&zpreztorc, &new_content)?;

                Ok(StepResult {
                    message: Some(format!("Enabled Prezto module: {}", module)),
                    needs_reload: true,
                })
            }

            InstallStep::PauseForUser { message } => {
                if self.interactive {
                    println!("{}", message);
                    println!("Press Enter to continue...");
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                }
                Ok(StepResult::default())
            }
        }
    }

    /// Run verification steps
    fn run_verification(&self, steps: &[VerificationStep]) -> Result<(), InstallationError> {
        for step in steps {
            self.verify_step(step)?;
        }
        Ok(())
    }

    /// Verify a single step
    fn verify_step(&self, step: &VerificationStep) -> Result<(), InstallationError> {
        match step {
            VerificationStep::PathExists(path) => {
                let expanded = expand_tilde(path);
                if expanded.exists() {
                    Ok(())
                } else {
                    Err(InstallationError::VerificationFailed(format!(
                        "Path does not exist: {}",
                        expanded.display()
                    )))
                }
            }

            VerificationStep::PathNotExists(path) => {
                let expanded = expand_tilde(path);
                if !expanded.exists() {
                    Ok(())
                } else {
                    Err(InstallationError::VerificationFailed(format!(
                        "Path should not exist: {}",
                        expanded.display()
                    )))
                }
            }

            VerificationStep::FileContains { path, pattern } => {
                let expanded = expand_tilde(path);
                if self.config_editor.contains(&expanded, pattern)? {
                    Ok(())
                } else {
                    Err(InstallationError::VerificationFailed(format!(
                        "File {} does not contain expected pattern",
                        expanded.display()
                    )))
                }
            }

            VerificationStep::CommandExists(cmd) => {
                let output = Command::new("which").arg(cmd).output();
                if output.is_ok() && output.unwrap().status.success() {
                    Ok(())
                } else {
                    Err(InstallationError::VerificationFailed(format!(
                        "Command '{}' not found",
                        cmd
                    )))
                }
            }

            VerificationStep::CommandSucceeds(cmd) => {
                let output = Command::new("sh").arg("-c").arg(cmd).output()?;
                if output.status.success() {
                    Ok(())
                } else {
                    Err(InstallationError::VerificationFailed(format!(
                        "Command '{}' failed",
                        cmd
                    )))
                }
            }

            VerificationStep::Custom {
                description,
                command,
            } => {
                let output = Command::new("sh").arg("-c").arg(command).output()?;
                if output.status.success() {
                    Ok(())
                } else {
                    Err(InstallationError::VerificationFailed(description.clone()))
                }
            }
        }
    }

    /// Execute rollback plan
    fn execute_rollback(&mut self, rollback: &RollbackPlan) -> Result<(), InstallationError> {
        if let Some(ref msg) = rollback.message {
            eprintln!("Rolling back: {}", msg);
        }

        // Restore backups
        for label in &rollback.restore_backups {
            if let Err(e) = self.config_editor.restore_backup(label) {
                eprintln!("Warning: Failed to restore backup '{}': {}", label, e);
            }
        }

        // Remove paths
        for path in &rollback.remove_paths {
            let expanded = expand_tilde(path);
            if expanded.exists() {
                if expanded.is_dir() {
                    let _ = fs::remove_dir_all(&expanded);
                } else {
                    let _ = fs::remove_file(&expanded);
                }
            }
        }

        // Run cleanup commands
        for cmd in &rollback.cleanup_commands {
            let _ = Command::new("sh").arg("-c").arg(cmd).output();
        }

        Ok(())
    }

    /// Prompt for user confirmation
    fn prompt_confirm(&self, message: &str) -> Result<bool, InstallationError> {
        print!("{} [y/N] ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    }

    /// Get a description of what a step does
    fn describe_step(&self, step: &InstallStep) -> String {
        match step {
            InstallStep::Confirmation { message } => format!("Confirm: {}", message),
            InstallStep::Message { message, level } => format!("[{:?}] {}", level, message),
            InstallStep::Backup { path, label } => {
                format!("Backup {} as '{}'", path.display(), label)
            }
            InstallStep::Run {
                command,
                description,
                ..
            } => format!("{}: `{}`", description, command),
            InstallStep::AddToConfig { path, content, .. } => {
                format!("Add to {}: {}", path.display(), content)
            }
            InstallStep::ReplaceInConfig {
                path,
                pattern,
                replacement,
            } => format!(
                "Replace in {}: '{}' -> '{}'",
                path.display(),
                pattern,
                replacement
            ),
            InstallStep::CreateDir { path } => format!("Create directory: {}", path.display()),
            InstallStep::WriteFile { path, .. } => format!("Write file: {}", path.display()),
            InstallStep::Download { url, destination, .. } => {
                format!("Download {} to {}", url, destination.display())
            }
            InstallStep::EnableOmzPlugin { plugin } => {
                format!("Enable Oh My Zsh plugin: {}", plugin)
            }
            InstallStep::EnablePreztoModule { module } => {
                format!("Enable Prezto module: {}", module)
            }
            InstallStep::PauseForUser { message } => format!("Pause: {}", message),
        }
    }

    /// Get shell reload helper
    pub fn shell_reload(&self) -> Option<ShellReload> {
        self.shell.map(ShellReload::new)
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::new().expect("Failed to create Installer")
    }
}

/// Result of executing a single step
#[derive(Debug, Default)]
struct StepResult {
    /// Message to display
    message: Option<String>,
    /// Whether shell reload is needed after this step
    needs_reload: bool,
}

/// Expand ~ to home directory
fn expand_tilde(path: &Path) -> std::path::PathBuf {
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
    use tempfile::TempDir;

    fn create_test_installer() -> (Installer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_editor = ConfigEditor::with_backup_dir(temp_dir.path().join("backups"))
            .without_atomic_writes();
        let installer = Installer::with_config_editor(config_editor).non_interactive();
        (installer, temp_dir)
    }

    #[test]
    fn test_dry_run() {
        let (installer, _temp_dir) = create_test_installer();

        let plan = InstallationPlan::new("Test", "Test installation")
            .with_step(InstallStep::Message {
                message: "Starting".into(),
                level: MessageLevel::Info,
            });

        let result = installer.execute_dry_run(&plan);
        assert!(matches!(result, InstallResult::DryRun { .. }));

        if let InstallResult::DryRun { steps } = result {
            assert!(!steps.is_empty());
            assert!(steps.iter().any(|s| s.contains("Test")));
        }
    }

    #[test]
    fn test_execute_message_step() {
        let (mut installer, _temp_dir) = create_test_installer();

        let plan = InstallationPlan::new("Test", "Test installation").with_step(InstallStep::Message {
            message: "Test message".into(),
            level: MessageLevel::Success,
        });

        let result = installer.execute(&plan);
        assert!(result.is_success());
    }

    #[test]
    fn test_execute_add_to_config() {
        let (mut installer, temp_dir) = create_test_installer();

        // Create a test config file
        let config_path = temp_dir.path().join("test.conf");
        fs::write(&config_path, "# Test config\n").unwrap();

        let plan =
            InstallationPlan::new("Test", "Test installation").with_step(InstallStep::AddToConfig {
                path: config_path.clone(),
                content: "export FOO=bar".into(),
                skip_if_contains: Some("FOO=".into()),
            });

        let result = installer.execute(&plan);
        assert!(result.is_success());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("export FOO=bar"));
    }

    #[test]
    fn test_prerequisite_path_exists() {
        let (installer, temp_dir) = create_test_installer();

        // Test with existing path
        let existing = temp_dir.path().join("existing");
        fs::write(&existing, "").unwrap();

        let prereq = Prerequisite::PathExists(existing.clone());
        assert!(installer.check_prerequisite(&prereq).is_ok());

        // Test with non-existing path
        let missing = temp_dir.path().join("missing");
        let prereq = Prerequisite::PathExists(missing);
        assert!(installer.check_prerequisite(&prereq).is_err());
    }

    #[test]
    fn test_prerequisite_not_installed() {
        let (installer, temp_dir) = create_test_installer();

        // Should pass when path doesn't exist
        let missing = temp_dir.path().join("missing");
        let prereq = Prerequisite::NotInstalled(missing);
        assert!(installer.check_prerequisite(&prereq).is_ok());

        // Should fail when path exists
        let existing = temp_dir.path().join("existing");
        fs::write(&existing, "").unwrap();
        let prereq = Prerequisite::NotInstalled(existing);
        assert!(installer.check_prerequisite(&prereq).is_err());
    }

    #[test]
    fn test_verification_path_exists() {
        let (installer, temp_dir) = create_test_installer();

        let existing = temp_dir.path().join("existing");
        fs::write(&existing, "content").unwrap();

        let step = VerificationStep::PathExists(existing);
        assert!(installer.verify_step(&step).is_ok());

        let missing = temp_dir.path().join("missing");
        let step = VerificationStep::PathExists(missing);
        assert!(installer.verify_step(&step).is_err());
    }

    #[test]
    fn test_create_directory() {
        let (mut installer, temp_dir) = create_test_installer();

        let new_dir = temp_dir.path().join("new_dir");
        let plan =
            InstallationPlan::new("Test", "Test").with_step(InstallStep::CreateDir { path: new_dir.clone() });

        let result = installer.execute(&plan);
        assert!(result.is_success());
        assert!(new_dir.exists());
    }

    #[test]
    fn test_write_file() {
        let (mut installer, temp_dir) = create_test_installer();

        let file_path = temp_dir.path().join("new_file.txt");
        let plan = InstallationPlan::new("Test", "Test").with_step(InstallStep::WriteFile {
            path: file_path.clone(),
            content: "Hello, World!".into(),
        });

        let result = installer.execute(&plan);
        assert!(result.is_success());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_backup_and_rollback() {
        let (mut installer, temp_dir) = create_test_installer();

        // Create a file to backup
        let config_file = temp_dir.path().join("config");
        fs::write(&config_file, "original content").unwrap();

        let plan = InstallationPlan::new("Test", "Test")
            .with_step(InstallStep::Backup {
                path: config_file.clone(),
                label: "config-backup".into(),
            })
            .with_step(InstallStep::WriteFile {
                path: config_file.clone(),
                content: "new content".into(),
            })
            .with_rollback(
                RollbackPlan::new()
                    .with_backup("config-backup")
                    .with_message("Restoring original config"),
            );

        let result = installer.execute(&plan);
        assert!(result.is_success());

        // Verify content was changed
        let content = fs::read_to_string(&config_file).unwrap();
        assert_eq!(content, "new content");
    }

    #[test]
    fn test_expand_tilde() {
        let home = dirs::home_dir().unwrap();

        let expanded = expand_tilde(Path::new("~/.zshrc"));
        assert_eq!(expanded, home.join(".zshrc"));

        let no_tilde = expand_tilde(Path::new("/etc/config"));
        assert_eq!(no_tilde, std::path::PathBuf::from("/etc/config"));
    }
}
