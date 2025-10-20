// Execution module contract tests - THESE MUST FAIL INITIALLY (TDD)
// Tests validate the execution module API from specs/003-implement-core-infrastructure/contracts/execution-api.md

use std::env;
use std::path::PathBuf;
use std::time::Instant;

// Import types that will be implemented later
// NOTE: These imports will fail until we implement the actual execution module
use cmdai::execution::{ExecutionContext, ExecutionError, PlatformDetector, ShellDetector};
use cmdai::models::{Platform, ShellType};

#[test]
fn test_execution_context_capture() {
    // CONTRACT: ExecutionContext::capture() returns valid context
    let result = ExecutionContext::capture();

    assert!(result.is_ok(), "Context capture should succeed");

    let context = result.unwrap();

    // Verify all required fields are populated
    assert!(
        context.current_dir().is_absolute(),
        "Current dir should be absolute path"
    );
    assert!(
        !context.username().is_empty(),
        "Username should not be empty"
    );
    assert!(
        !context.hostname().is_empty(),
        "Hostname should not be empty"
    );

    // Shell type should be detected (or fallback to Sh)
    let shell = context.shell_type();
    assert!(
        matches!(
            shell,
            ShellType::Bash
                | ShellType::Zsh
                | ShellType::Fish
                | ShellType::PowerShell
                | ShellType::Cmd
                | ShellType::Sh
        ),
        "Should return valid shell type"
    );

    // Platform should be detected
    let platform = context.platform();
    assert!(
        matches!(
            platform,
            Platform::Linux | Platform::MacOS | Platform::Windows
        ),
        "Should return valid platform"
    );
}

#[test]
fn test_execution_context_new() {
    // CONTRACT: ExecutionContext::new() creates context with custom values
    let test_dir = PathBuf::from("/tmp/test");
    let result = ExecutionContext::new(test_dir.clone(), ShellType::Bash, Platform::Linux);

    assert!(
        result.is_ok(),
        "Context creation with custom values should succeed"
    );

    let context = result.unwrap();
    assert_eq!(context.current_dir(), test_dir.as_path());
    assert_eq!(context.shell_type(), ShellType::Bash);
    assert_eq!(context.platform(), Platform::Linux);
}

#[test]
fn test_context_filters_sensitive_env_vars() {
    // CONTRACT: capture() filters sensitive environment variables
    // Set a sensitive env var
    env::set_var("TEST_API_KEY", "secret123");
    env::set_var("TEST_HOME", "/home/user");

    let context = ExecutionContext::capture().unwrap();

    // Sensitive var should NOT be included
    assert!(
        !context.has_env_var("TEST_API_KEY"),
        "API_KEY should be filtered"
    );

    // Non-sensitive var should be included
    assert!(
        context.has_env_var("HOME") || context.has_env_var("PATH"),
        "Standard env vars should be included"
    );

    // Cleanup
    env::remove_var("TEST_API_KEY");
    env::remove_var("TEST_HOME");
}

#[test]
fn test_context_includes_essential_env_vars() {
    // CONTRACT: capture() includes essential environment variables
    let context = ExecutionContext::capture().unwrap();
    let env_vars = context.environment_vars();

    // Should include PATH, HOME, USER (or equivalents)
    let has_essential = env_vars.contains_key("PATH")
        || env_vars.contains_key("HOME")
        || env_vars.contains_key("USER")
        || env_vars.contains_key("USERNAME");

    assert!(
        has_essential,
        "Should include at least one essential env var"
    );

    // All keys should be non-empty
    for (key, value) in env_vars.iter() {
        assert!(!key.is_empty(), "Env var keys should not be empty");
        assert!(!value.is_empty(), "Env var values should not be empty");
    }
}

#[test]
fn test_shell_detector_uses_env_variable() {
    // CONTRACT: ShellDetector::detect_from_env() uses SHELL environment variable
    let detector = ShellDetector::new();

    // Get current SHELL value (if set)
    if let Ok(shell_path) = env::var("SHELL") {
        let detected = detector.detect_from_env();

        if shell_path.contains("bash") {
            assert_eq!(detected, Some(ShellType::Bash));
        } else if shell_path.contains("zsh") {
            assert_eq!(detected, Some(ShellType::Zsh));
        } else if shell_path.contains("fish") {
            assert_eq!(detected, Some(ShellType::Fish));
        }
        // Other shells or None if not recognized
    }
}

#[test]
fn test_shell_detector_handles_variants() {
    // CONTRACT: ShellDetector handles multiple shell path variants
    let detector = ShellDetector::new();

    // Temporarily set SHELL to test variants
    let test_cases = vec![
        ("/bin/bash", ShellType::Bash),
        ("/usr/bin/bash", ShellType::Bash),
        ("bash", ShellType::Bash),
        ("/bin/zsh", ShellType::Zsh),
        ("/usr/bin/zsh", ShellType::Zsh),
        ("zsh", ShellType::Zsh),
        ("/usr/bin/fish", ShellType::Fish),
        ("fish", ShellType::Fish),
    ];

    for (shell_path, expected) in test_cases {
        env::set_var("SHELL", shell_path);
        let detected = detector.detect_from_env();

        assert_eq!(
            detected,
            Some(expected),
            "Failed to detect {} from path {}",
            expected,
            shell_path
        );
    }

    // Restore original SHELL
    if let Ok(original) = env::var("ORIGINAL_SHELL") {
        env::set_var("SHELL", original);
    }
}

#[test]
fn test_shell_detector_falls_back_to_sh() {
    // CONTRACT: ShellDetector::detect() falls back to Sh if detection fails
    let detector = ShellDetector::new();

    // Temporarily unset SHELL
    let original_shell = env::var("SHELL").ok();
    env::remove_var("SHELL");

    let detected = detector.detect();

    // Should fall back to Sh
    assert_eq!(
        detected,
        ShellType::Sh,
        "Should fallback to Sh when detection fails"
    );

    // Restore SHELL
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
}

#[test]
fn test_shell_detector_applies_override() {
    // CONTRACT: with_override() applies user configuration
    let detector = ShellDetector::new();

    // Auto-detection would return one value, override changes it
    let overridden = detector.with_override(Some(ShellType::Zsh));
    assert_eq!(overridden, ShellType::Zsh, "Should apply user override");

    // No override should use detection
    let auto_detected = detector.with_override(None);
    assert!(
        matches!(
            auto_detected,
            ShellType::Bash
                | ShellType::Zsh
                | ShellType::Fish
                | ShellType::PowerShell
                | ShellType::Cmd
                | ShellType::Sh
        ),
        "Should use auto-detection when no override"
    );
}

#[test]
fn test_platform_detector_returns_correct_platform() {
    // CONTRACT: PlatformDetector::detect() returns correct platform
    let platform = PlatformDetector::detect();

    assert!(
        matches!(
            platform,
            Platform::Linux | Platform::MacOS | Platform::Windows
        ),
        "Should detect valid platform"
    );

    // Verify against cfg! macros
    #[cfg(target_os = "linux")]
    assert_eq!(platform, Platform::Linux);

    #[cfg(target_os = "macos")]
    assert_eq!(platform, Platform::MacOS);

    #[cfg(target_os = "windows")]
    assert_eq!(platform, Platform::Windows);
}

#[test]
fn test_platform_is_posix() {
    // CONTRACT: is_posix() returns true for Linux/macOS, false for Windows
    let is_posix = PlatformDetector::is_posix();

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert!(is_posix, "Linux and macOS should be POSIX");

    #[cfg(target_os = "windows")]
    assert!(!is_posix, "Windows should not be POSIX");
}

#[test]
fn test_context_to_prompt_context() {
    // CONTRACT: to_prompt_context() serializes for LLM prompt
    let context = ExecutionContext::capture().unwrap();
    let prompt_text = context.to_prompt_context();

    // Should contain key information
    assert!(
        prompt_text.contains("directory") || prompt_text.contains("dir"),
        "Should mention current directory"
    );
    assert!(
        !prompt_text.is_empty(),
        "Prompt context should not be empty"
    );

    // Should be human-readable
    assert!(
        prompt_text.len() > 20,
        "Prompt context should be substantial"
    );
}

#[test]
fn test_context_performance() {
    // CONTRACT: Context capture meets performance requirements (NFR-003: <50ms)
    let start = Instant::now();
    let _ = ExecutionContext::capture();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 50,
        "Context capture should be <50ms, took {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_context_has_env_var() {
    // CONTRACT: has_env_var() checks environment variable existence
    let context = ExecutionContext::capture().unwrap();

    // PATH should exist on all platforms
    assert!(
        context.has_env_var("PATH") || context.has_env_var("Path"),
        "PATH should be available"
    );

    // Non-existent var should return false
    assert!(
        !context.has_env_var("NONEXISTENT_VAR_12345"),
        "Non-existent var should return false"
    );
}

#[test]
fn test_context_get_env_var() {
    // CONTRACT: get_env_var() retrieves environment variable value
    let context = ExecutionContext::capture().unwrap();

    // Get PATH variable
    if let Some(path_value) = context.get_env_var("PATH") {
        assert!(!path_value.is_empty(), "PATH value should not be empty");
    }

    // Non-existent var returns None
    assert_eq!(
        context.get_env_var("NONEXISTENT_VAR_12345"),
        None,
        "Non-existent var should return None"
    );
}

#[test]
fn test_context_is_immutable() {
    // CONTRACT: ExecutionContext is immutable after capture
    let context = ExecutionContext::capture().unwrap();

    // All getters should return references or copies
    let _dir1 = context.current_dir();
    let _dir2 = context.current_dir();
    // Should be same reference (immutable)

    let _shell1 = context.shell_type();
    let _shell2 = context.shell_type();
    // Should be same value
}

#[test]
fn test_sensitive_data_filtering_patterns() {
    // CONTRACT: Environment variable filtering excludes sensitive patterns
    env::set_var("MY_API_KEY", "secret");
    env::set_var("AUTH_TOKEN", "token123");
    env::set_var("PASSWORD", "pass");
    env::set_var("SECRET_VALUE", "secret");
    env::set_var("AWS_SECRET_ACCESS_KEY", "aws_secret");
    env::set_var("NORMAL_VAR", "safe_value");

    let context = ExecutionContext::capture().unwrap();

    // Sensitive vars should be filtered
    assert!(
        !context.has_env_var("MY_API_KEY"),
        "API_KEY should be filtered"
    );
    assert!(
        !context.has_env_var("AUTH_TOKEN"),
        "TOKEN should be filtered"
    );
    assert!(
        !context.has_env_var("PASSWORD"),
        "PASSWORD should be filtered"
    );
    assert!(
        !context.has_env_var("SECRET_VALUE"),
        "SECRET should be filtered"
    );
    assert!(
        !context.has_env_var("AWS_SECRET_ACCESS_KEY"),
        "AWS secret should be filtered"
    );

    // Normal vars should be included
    assert!(
        context.has_env_var("NORMAL_VAR") || context.has_env_var("PATH"),
        "Normal vars should be included"
    );

    // Cleanup
    env::remove_var("MY_API_KEY");
    env::remove_var("AUTH_TOKEN");
    env::remove_var("PASSWORD");
    env::remove_var("SECRET_VALUE");
    env::remove_var("AWS_SECRET_ACCESS_KEY");
    env::remove_var("NORMAL_VAR");
}

#[test]
fn test_context_captured_at_timestamp() {
    // CONTRACT: captured_at() returns timestamp within reasonable range
    use chrono::Utc;

    let before = Utc::now();
    let context = ExecutionContext::capture().unwrap();
    let after = Utc::now();

    let captured = context.captured_at();

    assert!(
        captured >= before,
        "Captured time should be after or equal to before time"
    );
    assert!(
        captured <= after,
        "Captured time should be before or equal to after time"
    );
}

#[test]
fn test_invalid_current_directory_handling() {
    // CONTRACT: ExecutionContext handles invalid current directory
    // This test validates error handling when current_dir is inaccessible

    // Note: In normal operation, current_dir should always be accessible
    // This tests the error path if current directory is deleted after process starts

    let result = ExecutionContext::capture();

    // Should either succeed or return ExecutionError::CurrentDirNotAccessible
    match result {
        Ok(context) => {
            assert!(
                context.current_dir().exists()
                    || !context.current_dir().to_str().unwrap().is_empty(),
                "Current dir should be valid if capture succeeds"
            );
        }
        Err(ExecutionError::CurrentDirNotAccessible(_)) => {
            // Expected error for inaccessible directory
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
