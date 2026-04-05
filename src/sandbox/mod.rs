//! Nono sandbox integration
//!
//! Wraps command execution in a kernel-enforced security sandbox using
//! [nono](https://github.com/always-further/nono) when available.
//!
//! Nono applies OS-level restrictions (macOS Seatbelt / Linux Landlock) that
//! are structurally impossible to circumvent from within the sandboxed process,
//! providing an additional layer of protection beyond Caro's pattern-based
//! safety validation.

use std::process::Command;

/// Configuration for the Nono sandbox wrapper
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonoSandbox {
    /// Named security profile (e.g. "safe", "read-only"). If None, nono's
    /// defaults apply.
    pub profile: Option<String>,
    /// Enable file snapshot / rollback support
    pub rollback: bool,
}

impl Default for NonoSandbox {
    fn default() -> Self {
        Self {
            profile: None,
            rollback: false,
        }
    }
}

impl NonoSandbox {
    /// Create a new sandbox with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the nono security profile
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Enable snapshot / rollback support
    pub fn with_rollback(mut self) -> Self {
        self.rollback = true;
        self
    }

    /// Check whether the `nono` binary is available in PATH.
    pub fn is_available() -> bool {
        Command::new("nono")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
    }

    /// Build the nono wrapper argv prefix for a given inner shell command.
    ///
    /// Returns `(program, args)` ready for `std::process::Command::new(program)`.
    /// The caller should append the inner shell invocation arguments after these.
    ///
    /// # Example
    ///
    /// For `NonoSandbox { profile: Some("safe"), rollback: false }` and inner
    /// command `sh -c "ls"` the result is:
    ///
    /// ```text
    /// program = "nono"
    /// args    = ["run", "--profile", "safe", "--", "sh", "-c", "ls"]
    /// ```
    pub fn wrap(&self, shell_program: &str, shell_args: &[&str]) -> (String, Vec<String>) {
        let mut args: Vec<String> = vec!["run".to_string()];

        if let Some(ref profile) = self.profile {
            args.push("--profile".to_string());
            args.push(profile.clone());
        }

        if self.rollback {
            args.push("--rollback".to_string());
        }

        args.push("--".to_string());
        args.push(shell_program.to_string());
        for arg in shell_args {
            args.push(arg.to_string());
        }

        ("nono".to_string(), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_no_profile() {
        let sandbox = NonoSandbox::new();
        let (prog, args) = sandbox.wrap("sh", &["-c", "ls"]);
        assert_eq!(prog, "nono");
        assert_eq!(args, ["run", "--", "sh", "-c", "ls"]);
    }

    #[test]
    fn test_wrap_with_profile() {
        let sandbox = NonoSandbox::new().with_profile("safe");
        let (prog, args) = sandbox.wrap("sh", &["-c", "ls"]);
        assert_eq!(prog, "nono");
        assert_eq!(args, ["run", "--profile", "safe", "--", "sh", "-c", "ls"]);
    }

    #[test]
    fn test_wrap_with_rollback() {
        let sandbox = NonoSandbox::new().with_rollback();
        let (prog, args) = sandbox.wrap("bash", &["-c", "echo hi"]);
        assert_eq!(prog, "nono");
        assert_eq!(args, ["run", "--rollback", "--", "bash", "-c", "echo hi"]);
    }

    #[test]
    fn test_wrap_profile_and_rollback() {
        let sandbox = NonoSandbox::new().with_profile("read-only").with_rollback();
        let (prog, args) = sandbox.wrap("sh", &["-c", "cat file.txt"]);
        assert_eq!(prog, "nono");
        assert_eq!(
            args,
            ["run", "--profile", "read-only", "--rollback", "--", "sh", "-c", "cat file.txt"]
        );
    }
}
