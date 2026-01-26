//! Version information module for Caro
//!
//! This module provides version info captured at build time from git and rustc.
//! It supports both basic (scriptable) and verbose (human-friendly) output formats.

use std::fmt;

/// Version information captured at build time
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: &'static str,
    pub git_hash: &'static str,
    pub git_hash_full: &'static str,
    pub git_date: &'static str,
    pub build_date: &'static str,
    pub rustc_version: &'static str,
    pub target: &'static str,
    pub build_profile: &'static str,
    pub release_flag: &'static str,
}

impl VersionInfo {
    /// Get the singleton version info
    pub fn get() -> &'static VersionInfo {
        &VERSION_INFO
    }

    /// Generate short version string (scriptable, single line)
    ///
    /// Format: `caro 1.0.2 (x86_64-unknown-linux-gnu)`
    /// Follows standard CLI conventions (bash, rustc, etc.)
    pub fn short(&self) -> String {
        format!("caro {} ({})", self.version, self.target)
    }

    /// Generate long version string (verbose, Caro's voice)
    ///
    /// Includes build details and Caro's personality
    pub fn long(&self) -> String {
        format!(
            "Hey! I'm Caro v{} ({} {})\n\
             \n\
             I turn your thoughts into shell commands using local LLMs.\n\
             \n\
             Build details:\n\
             {}\
             \n\
             Ready to help!",
            self.version,
            self.git_hash,
            self.git_date,
            self.build_details()
        )
    }

    /// Generate formatted build details section
    fn build_details(&self) -> String {
        format!(
            "  commit:     {}\n\
             \x20 built:      {}\n\
             \x20 host:       {}\n\
             \x20 rustc:      {}\n\
             \x20 build-type: {}",
            self.git_hash_full,
            self.build_date,
            self.target,
            self.rustc_version,
            self.build_type()
        )
    }

    /// Check if this is an official release build
    pub fn is_release(&self) -> bool {
        self.release_flag == "1"
    }

    /// Determine build type from profile and release flag
    ///
    /// Returns one of:
    /// - "dev (local build)" - for debug builds or missing git info
    /// - "source (cargo install)" - for release builds from source
    /// - "binary (official release)" - for GitHub release builds
    pub fn build_type(&self) -> &'static str {
        if self.is_release() {
            "binary (official release)"
        } else if self.build_profile == "debug" || self.git_hash == "unknown" {
            "dev (local build)"
        } else {
            "source (cargo install)"
        }
    }
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short())
    }
}

/// Static version information captured at build time
static VERSION_INFO: VersionInfo = VersionInfo {
    version: env!("CARGO_PKG_VERSION"),
    git_hash: env!("CARO_GIT_HASH"),
    git_hash_full: env!("CARO_GIT_HASH_FULL"),
    git_date: env!("CARO_GIT_DATE"),
    build_date: env!("CARO_BUILD_DATE"),
    rustc_version: env!("CARO_RUSTC_VERSION"),
    target: env!("CARO_TARGET"),
    build_profile: env!("CARO_BUILD_PROFILE"),
    release_flag: env!("CARO_RELEASE"),
};

/// Get version info
pub fn info() -> &'static VersionInfo {
    VersionInfo::get()
}

/// Get short version string (for clap version template)
pub fn short() -> String {
    VERSION_INFO.short()
}

/// Get long version string (for verbose output)
pub fn long() -> String {
    VERSION_INFO.long()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_exists() {
        let info = VersionInfo::get();
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_short_version_format() {
        let short = short();
        assert!(short.starts_with("caro "));
        assert!(short.contains(env!("CARGO_PKG_VERSION")));
        // Verify platform/target triple is included
        let info = VersionInfo::get();
        assert!(short.contains(info.target), "Version should include target triple");
    }

    #[test]
    fn test_long_version_has_personality() {
        let long = long();
        assert!(long.contains("Hey! I'm Caro"));
        assert!(long.contains("Ready to help!"));
        assert!(long.contains("Build details:"));
    }

    #[test]
    fn test_build_type_logic() {
        let info = VersionInfo::get();
        let build_type = info.build_type();

        // Should be one of the three types
        assert!(
            build_type == "dev (local build)"
                || build_type == "source (cargo install)"
                || build_type == "binary (official release)"
        );
    }
}
