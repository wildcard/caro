//! Shell and platform detection utilities

use crate::models::{Platform, ShellType};

/// Platform detection utilities
pub struct PlatformDetector;

impl PlatformDetector {
    /// Detect the current platform
    pub fn detect() -> Platform {
        Platform::detect()
    }

    /// Check if the current platform is POSIX-compliant
    pub fn is_posix() -> bool {
        Self::detect().is_posix()
    }

    /// Check if the current platform is Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// Check if the current platform is macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Check if the current platform is Linux
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }
}

/// Shell detection utilities
pub struct ShellDetector {
    override_shell: Option<ShellType>,
}

impl Default for ShellDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellDetector {
    /// Create a new ShellDetector
    pub fn new() -> Self {
        Self {
            override_shell: None,
        }
    }

    /// Detect the current shell from environment (instance method with fallback)
    pub fn detect(&self) -> ShellType {
        if let Some(override_shell) = self.override_shell {
            return override_shell;
        }

        let detected = ShellType::detect();
        if matches!(detected, ShellType::Unknown) {
            ShellType::Sh
        } else {
            detected
        }
    }

    /// Detect shell from environment, returns None if not recognized
    pub fn detect_from_env(&self) -> Option<ShellType> {
        if let Some(override_shell) = self.override_shell {
            return Some(override_shell);
        }

        let detected = ShellType::detect();
        if matches!(detected, ShellType::Unknown) {
            None
        } else {
            Some(detected)
        }
    }

    /// Set an override shell type
    pub fn with_override(&self, shell: Option<ShellType>) -> ShellType {
        shell.unwrap_or_else(|| self.detect())
    }

    /// Detect shell with fallback (static method)
    pub fn detect_with_fallback(fallback: ShellType) -> ShellType {
        let detected = ShellType::detect();
        if matches!(detected, ShellType::Unknown) {
            fallback
        } else {
            detected
        }
    }

    /// Check if the detected shell is POSIX-compatible (static method)
    pub fn is_posix_shell() -> bool {
        ShellType::detect().is_posix()
    }

    /// Get the shell from an environment variable or detect
    pub fn from_env_or_detect(env_var: &str) -> ShellType {
        if let Ok(shell_str) = std::env::var(env_var) {
            if let Ok(shell) = shell_str.parse::<ShellType>() {
                return shell;
            }
        }
        ShellType::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detector() {
        let platform = PlatformDetector::detect();
        assert!(matches!(
            platform,
            Platform::Linux | Platform::MacOS | Platform::Windows
        ));
    }

    #[test]
    fn test_shell_detector() {
        let detector = ShellDetector::new();
        let shell = detector.detect();
        // Should detect some shell type (at least Unknown as fallback)
        assert!(matches!(
            shell,
            ShellType::Bash
                | ShellType::Zsh
                | ShellType::Fish
                | ShellType::Sh
                | ShellType::PowerShell
                | ShellType::Cmd
                | ShellType::Unknown
        ));
    }

    #[test]
    fn test_shell_detector_with_fallback() {
        let shell = ShellDetector::detect_with_fallback(ShellType::Bash);
        // Should never return Unknown when fallback is provided
        assert!(!matches!(shell, ShellType::Unknown));
    }

    #[test]
    fn test_platform_checks() {
        // At least one should be true
        assert!(
            PlatformDetector::is_linux()
                || PlatformDetector::is_macos()
                || PlatformDetector::is_windows()
        );
    }
}
