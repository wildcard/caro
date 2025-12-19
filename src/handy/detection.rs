//! Handy.Computer detection and process management
//!
//! This module provides functionality to detect Handy.Computer installation
//! and running processes on the system.

use std::path::PathBuf;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

/// Status of Handy.Computer on the system
#[derive(Debug, Clone, PartialEq)]
pub enum HandyStatus {
    /// Handy is installed and running
    InstalledAndRunning {
        /// Process ID of the running Handy process
        pid: u32,
        /// Installation path
        install_path: PathBuf,
    },
    /// Handy is installed but not running
    InstalledNotRunning {
        /// Installation path
        install_path: PathBuf,
    },
    /// Handy is not installed
    NotInstalled,
}

impl HandyStatus {
    /// Check if Handy is available for use
    pub fn is_available(&self) -> bool {
        matches!(self, HandyStatus::InstalledAndRunning { .. })
    }

    /// Check if Handy is installed
    pub fn is_installed(&self) -> bool {
        !matches!(self, HandyStatus::NotInstalled)
    }

    /// Get the process ID if Handy is running
    pub fn pid(&self) -> Option<u32> {
        match self {
            HandyStatus::InstalledAndRunning { pid, .. } => Some(*pid),
            _ => None,
        }
    }

    /// Get the installation path if Handy is installed
    pub fn install_path(&self) -> Option<&PathBuf> {
        match self {
            HandyStatus::InstalledAndRunning { install_path, .. }
            | HandyStatus::InstalledNotRunning { install_path } => Some(install_path),
            HandyStatus::NotInstalled => None,
        }
    }
}

/// Detector for Handy.Computer installation and status
pub struct HandyDetector {
    system: System,
}

impl Default for HandyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl HandyDetector {
    /// Create a new Handy detector
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    /// Detect the current status of Handy.Computer
    pub fn detect(&mut self) -> HandyStatus {
        // Refresh process list
        self.system.refresh_processes();

        // Find Handy process
        let handy_pid = self.find_handy_process();

        // Find installation path
        let install_path = self.find_installation();

        match (handy_pid, install_path) {
            (Some(pid), Some(path)) => HandyStatus::InstalledAndRunning {
                pid,
                install_path: path,
            },
            (None, Some(path)) => HandyStatus::InstalledNotRunning { install_path: path },
            _ => HandyStatus::NotInstalled,
        }
    }

    /// Find the Handy process ID
    fn find_handy_process(&self) -> Option<u32> {
        // Look for processes named "Handy" or "com.pais.handy"
        for (pid, process) in self.system.processes() {
            let name = process.name().to_lowercase();
            if name.contains("handy") && !name.contains("handycomputer") {
                tracing::debug!("Found Handy process: {} (PID: {})", process.name(), pid);
                return Some(pid.as_u32());
            }
        }
        None
    }

    /// Find Handy installation path
    fn find_installation(&self) -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            self.find_installation_macos()
        }

        #[cfg(target_os = "linux")]
        {
            self.find_installation_linux()
        }

        #[cfg(target_os = "windows")]
        {
            self.find_installation_windows()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }

    #[cfg(target_os = "macos")]
    fn find_installation_macos(&self) -> Option<PathBuf> {
        use std::fs;

        // Check common macOS installation paths
        let possible_paths = vec![
            PathBuf::from("/Applications/Handy.app"),
            PathBuf::from(format!("{}/Applications/Handy.app", std::env::var("HOME").ok()?)),
        ];

        for path in possible_paths {
            if path.exists() {
                tracing::debug!("Found Handy installation at: {:?}", path);
                return Some(path);
            }
        }

        // Also check config directory as indicator
        if let Some(config_dir) = dirs::config_dir() {
            let handy_config = config_dir.join("com.pais.handy");
            if handy_config.exists() {
                tracing::debug!("Found Handy config directory: {:?}", handy_config);
                // Config exists but we don't know the app path
                // Return the config dir as indicator
                return Some(handy_config);
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn find_installation_linux(&self) -> Option<PathBuf> {

        // Check common Linux installation paths
        let possible_paths = vec![
            PathBuf::from("/usr/local/bin/handy"),
            PathBuf::from("/usr/bin/handy"),
            PathBuf::from(format!("{}/.local/bin/handy", std::env::var("HOME").ok()?)),
        ];

        for path in possible_paths {
            if path.exists() {
                tracing::debug!("Found Handy installation at: {:?}", path);
                return Some(path);
            }
        }

        // Also check config directory
        if let Some(config_dir) = dirs::config_dir() {
            let handy_config = config_dir.join("com.pais.handy");
            if handy_config.exists() {
                tracing::debug!("Found Handy config directory: {:?}", handy_config);
                return Some(handy_config);
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn find_installation_windows(&self) -> Option<PathBuf> {
        use std::fs;

        // Check common Windows installation paths
        let program_files = std::env::var("ProgramFiles").ok()?;
        let local_app_data = std::env::var("LOCALAPPDATA").ok()?;

        let possible_paths = vec![
            PathBuf::from(format!("{}/Handy/Handy.exe", program_files)),
            PathBuf::from(format!("{}/Programs/Handy/Handy.exe", local_app_data)),
        ];

        for path in possible_paths {
            if path.exists() {
                tracing::debug!("Found Handy installation at: {:?}", path);
                return Some(path);
            }
        }

        // Check config directory
        if let Some(config_dir) = dirs::config_dir() {
            let handy_config = config_dir.join("com.pais.handy");
            if handy_config.exists() {
                tracing::debug!("Found Handy config directory: {:?}", handy_config);
                return Some(handy_config);
            }
        }

        None
    }
}

/// Quick check if Handy is available
pub fn is_handy_available() -> bool {
    let mut detector = HandyDetector::new();
    detector.detect().is_available()
}

/// Quick check if Handy is installed
pub fn is_handy_installed() -> bool {
    let mut detector = HandyDetector::new();
    detector.detect().is_installed()
}

/// Get the current Handy status
pub fn get_handy_status() -> HandyStatus {
    let mut detector = HandyDetector::new();
    detector.detect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handy_detector_creation() {
        let detector = HandyDetector::new();
        assert!(detector.system.processes().len() > 0);
    }

    #[test]
    fn test_handy_status_methods() {
        let status = HandyStatus::NotInstalled;
        assert!(!status.is_available());
        assert!(!status.is_installed());
        assert!(status.pid().is_none());
        assert!(status.install_path().is_none());

        let install_path = PathBuf::from("/Applications/Handy.app");
        let status = HandyStatus::InstalledNotRunning {
            install_path: install_path.clone(),
        };
        assert!(!status.is_available());
        assert!(status.is_installed());
        assert!(status.pid().is_none());
        assert_eq!(status.install_path(), Some(&install_path));

        let status = HandyStatus::InstalledAndRunning {
            pid: 1234,
            install_path: install_path.clone(),
        };
        assert!(status.is_available());
        assert!(status.is_installed());
        assert_eq!(status.pid(), Some(1234));
        assert_eq!(status.install_path(), Some(&install_path));
    }

    #[test]
    fn test_detect_handy() {
        let mut detector = HandyDetector::new();
        let status = detector.detect();

        // This test will vary based on whether Handy is actually installed
        match status {
            HandyStatus::InstalledAndRunning { pid, install_path } => {
                println!("Handy is running with PID: {}", pid);
                println!("Installed at: {:?}", install_path);
            }
            HandyStatus::InstalledNotRunning { install_path } => {
                println!("Handy is installed but not running");
                println!("Installed at: {:?}", install_path);
            }
            HandyStatus::NotInstalled => {
                println!("Handy is not installed");
            }
        }
    }
}
