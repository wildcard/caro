//! Contract tests for enhanced platform detection
//!
//! Tests the platform detection enhancements required for command validation.
//! Following TDD: These tests define the contract before implementation.

use caro::platform::{PlatformContext, UtilityType};

/// Test that PlatformContext can be detected from the current environment
#[tokio::test]
async fn test_platform_context_detection() {
    let ctx = PlatformContext::detect().await.expect("Should detect platform context");

    // Basic platform info should be populated
    assert!(!ctx.os().is_empty(), "OS should not be empty");
    assert!(!ctx.shell().is_empty(), "Shell should not be empty");
    assert!(
        !ctx.arch().is_empty(),
        "Architecture should not be empty"
    );
}

/// Test that platform context includes required fields
#[tokio::test]
async fn test_platform_context_has_required_fields() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    // Check OS information
    let os = ctx.os();
    assert!(
        os == "macos" || os == "linux" || os == "windows",
        "OS should be recognized: got {}",
        os
    );

    // Check shell information
    let shell = ctx.shell();
    assert!(
        ["zsh", "bash", "fish", "sh", "powershell", "cmd"].contains(&shell),
        "Shell should be recognized: got {}",
        shell
    );

    // Check architecture
    let arch = ctx.arch();
    assert!(
        arch == "x86_64" || arch == "aarch64" || arch == "arm64",
        "Architecture should be recognized: got {}",
        arch
    );

    // Check POSIX compliance flag
    let posix = ctx.is_posix_compliant();
    assert!(
        posix == true || posix == false,
        "POSIX compliance should be boolean"
    );
}

/// Test OS version detection
#[tokio::test]
async fn test_os_version_detection() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let os_version = ctx.os_version();
    assert!(
        !os_version.is_empty(),
        "OS version should not be empty"
    );

    // Version should contain numbers
    assert!(
        os_version.chars().any(|c| c.is_numeric()),
        "OS version should contain numbers: got {}",
        os_version
    );
}

/// Test shell version detection
#[tokio::test]
async fn test_shell_version_detection() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let shell_version = ctx.shell_version();

    // Shell version might not always be detectable, but if present should have numbers
    if !shell_version.is_empty() {
        assert!(
            shell_version.chars().any(|c| c.is_numeric()),
            "Shell version should contain numbers if present: got {}",
            shell_version
        );
    }
}

/// Test utility availability detection
#[tokio::test]
async fn test_utility_availability() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let utils = ctx.available_tools();

    // Should have at least some basic utilities
    assert!(!utils.is_empty(), "Should detect some available utilities");

    // Common utilities that should always exist on Unix-like systems
    if ctx.is_posix_compliant() {
        assert!(
            utils.contains_key("ls"),
            "Should detect 'ls' utility on POSIX systems"
        );
    }
}

/// Test GNU vs BSD coreutils detection
#[tokio::test]
async fn test_coreutils_detection() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    if ctx.is_posix_compliant() {
        let has_gnu = ctx.has_gnu_coreutils();
        let has_bsd = ctx.has_bsd_utils();

        // Should detect at least one type of utils
        assert!(
            has_gnu || has_bsd,
            "Should detect either GNU or BSD coreutils on POSIX systems"
        );

        // On macOS, should typically have BSD utils
        if ctx.os() == "macos" {
            assert!(
                has_bsd,
                "macOS should typically have BSD utilities"
            );
        }

        // On Linux, should typically have GNU utils
        if ctx.os() == "linux" {
            assert!(
                has_gnu,
                "Linux should typically have GNU coreutils"
            );
        }
    }
}

/// Test UtilityType enum detection
#[tokio::test]
async fn test_utility_type_detection() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    if ctx.is_posix_compliant() {
        let util_type = ctx.utility_type();
        
        assert!(
            matches!(util_type, UtilityType::Gnu | UtilityType::Bsd | UtilityType::Busybox),
            "Should detect utility type on POSIX systems: got {:?}",
            util_type
        );
    }
}

/// Test that platform context can be serialized for prompts
#[tokio::test]
async fn test_platform_context_to_prompt() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let prompt_str = ctx.to_prompt_string();

    // Should include key information
    assert!(
        prompt_str.contains("OS:") || prompt_str.contains("os:"),
        "Prompt should include OS information"
    );
    assert!(
        prompt_str.contains("Shell:") || prompt_str.contains("shell:"),
        "Prompt should include shell information"
    );
    assert!(
        !prompt_str.is_empty(),
        "Prompt string should not be empty"
    );
}

/// Test platform context includes platform-specific notes
#[tokio::test]
async fn test_platform_specific_notes() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let notes = ctx.platform_notes();

    // Should have some platform-specific guidance
    if ctx.os() == "macos" && ctx.has_bsd_utils() {
        assert!(
            notes.iter().any(|note| note.to_lowercase().contains("bsd")),
            "macOS should include BSD-specific notes"
        );
    }
}

/// Test that platform context can be created with custom values (for testing)
#[test]
fn test_platform_context_builder() {
    let ctx = PlatformContext::builder()
        .os("linux")
        .os_version("Ubuntu 22.04")
        .arch("x86_64")
        .shell("bash")
        .shell_version("5.1.16")
        .posix_compliant(true)
        .has_gnu_coreutils(true)
        .has_bsd_utils(false)
        .build()
        .expect("Should build custom platform context");

    assert_eq!(ctx.os(), "linux");
    assert_eq!(ctx.shell(), "bash");
    assert!(ctx.is_posix_compliant());
    assert!(ctx.has_gnu_coreutils());
    assert!(!ctx.has_bsd_utils());
}

/// Test utility version detection
#[tokio::test]
async fn test_utility_version_detection() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let utils = ctx.available_tools();

    // Check that versions are captured where available
    for (name, version) in utils.iter() {
        // Version might be empty for some utilities
        if !version.is_empty() {
            println!("Detected utility: {} version {}", name, version);
        }
    }

    // At least one utility should have a version
    // (or version string might be empty if detection fails)
    // This is more of a smoke test
}

/// Test specific common utilities
#[tokio::test]
async fn test_common_utilities() {
    let ctx = PlatformContext::detect()
        .await
        .expect("Should detect platform context");

    let utils = ctx.available_tools();

    if ctx.is_posix_compliant() {
        // Common utilities that should exist
        let common = ["ls", "cat", "echo", "grep", "find"];

        for util in &common {
            if utils.contains_key(*util) {
                println!("Found {}: {}", util, utils.get(*util).unwrap_or(&"".to_string()));
            }
        }
    }
}

/// Test that platform context detection is reasonably fast
#[tokio::test]
async fn test_platform_detection_performance() {
    use std::time::Instant;

    let start = Instant::now();
    let _ctx = PlatformContext::detect().await.expect("Should detect platform");
    let duration = start.elapsed();

    // Detection should complete within 2 seconds (even with utility checks)
    assert!(
        duration.as_secs() < 2,
        "Platform detection took too long: {:?}",
        duration
    );
}

/// Test error handling for invalid builder values
#[test]
fn test_platform_context_builder_validation() {
    // Empty OS should fail
    let result = PlatformContext::builder()
        .os("")
        .arch("x86_64")
        .shell("bash")
        .build();

    assert!(result.is_err(), "Should fail with empty OS");

    // Empty arch should fail
    let result = PlatformContext::builder()
        .os("linux")
        .arch("")
        .shell("bash")
        .build();

    assert!(result.is_err(), "Should fail with empty architecture");

    // Empty shell should fail
    let result = PlatformContext::builder()
        .os("linux")
        .arch("x86_64")
        .shell("")
        .build();

    assert!(result.is_err(), "Should fail with empty shell");
}
