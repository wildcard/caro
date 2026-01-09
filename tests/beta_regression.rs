//! Regression tests for v1.1.0-beta.1 fixes
//!
//! These tests verify fixes for the 5 P0 issues identified in beta testing:
//! - Issue #402: Telemetry notice on every command
//! - Issue #403: Telemetry cannot be disabled
//! - Issue #404: --output json produces invalid JSON
//! - Issue #405: Documentation mismatch
//! - Issue #406: Command quality 40% vs 95% target

use caro::backends::static_matcher::StaticMatcher;
use caro::backends::CommandGenerator;
use caro::models::CommandRequest;
use caro::models::ShellType;
use caro::prompts::CapabilityProfile;

/// Issue #406 Test 1: "show disk space by directory"
/// Should generate: du -h -d 1 (macOS) or du -h --max-depth=1 (Linux)
/// Was generating: ls -lh (incorrect)
#[tokio::test]
async fn test_disk_space_by_directory() {
    let profile = CapabilityProfile::for_platform(caro::prompts::ProfileType::Bsd);
    let matcher = StaticMatcher::new(profile);

    let request = CommandRequest::new("show disk space by directory", ShellType::Bash);

    let result = matcher.generate_command(&request).await;
    assert!(result.is_ok(), "Command generation should succeed");

    let cmd = result.unwrap();
    assert!(
        cmd.command.contains("du")
            && (cmd.command.contains("-d 1") || cmd.command.contains("--max-depth=1")),
        "Command should be 'du -h -d 1' or 'du -h --max-depth=1', got: {}",
        cmd.command
    );
}

/// Issue #406 Test 2: "find python files from last week"
/// Should generate: find . -name "*.py" -type f -mtime -7
/// Was generating: find . -name "*.py" -type f (missing -mtime -7)
#[tokio::test]
async fn test_python_files_from_last_week() {
    let profile = CapabilityProfile::for_platform(caro::prompts::ProfileType::Bsd);
    let matcher = StaticMatcher::new(profile);

    let request = CommandRequest::new("find python files from last week", ShellType::Bash);

    let result = matcher.generate_command(&request).await;
    assert!(result.is_ok(), "Command generation should succeed");

    let cmd = result.unwrap();
    assert!(
        cmd.command.contains("*.py") && cmd.command.contains("-mtime -7"),
        "Command should include '*.py' and '-mtime -7', got: {}",
        cmd.command
    );
}

/// Issue #406 Test 3: "list hidden files"
/// Should generate: ls -d .*
/// Was generating: ls -la (suboptimal)
#[tokio::test]
async fn test_list_hidden_files() {
    let profile = CapabilityProfile::for_platform(caro::prompts::ProfileType::Bsd);
    let matcher = StaticMatcher::new(profile);

    let request = CommandRequest::new("list hidden files", ShellType::Bash);

    let result = matcher.generate_command(&request).await;
    assert!(result.is_ok(), "Command generation should succeed");

    let cmd = result.unwrap();
    assert_eq!(
        cmd.command, "ls -d .*",
        "Command should be 'ls -d .*', got: {}",
        cmd.command
    );
}

/// Issue #406 Test 4: Verify "find python files modified last week" still works
/// This tests backward compatibility - the original phrasing should still match
#[tokio::test]
async fn test_python_files_modified_last_week() {
    let profile = CapabilityProfile::for_platform(caro::prompts::ProfileType::Bsd);
    let matcher = StaticMatcher::new(profile);

    let request = CommandRequest::new("find python files modified last week", ShellType::Bash);

    let result = matcher.generate_command(&request).await;
    assert!(result.is_ok(), "Command generation should succeed");

    let cmd = result.unwrap();
    assert!(
        cmd.command.contains("*.py") && cmd.command.contains("-mtime -7"),
        "Command should include '*.py' and '-mtime -7', got: {}",
        cmd.command
    );
}

/// Issue #406 Test 5: "show disk space by directory sorted" should still work
/// Verifies the more specific pattern still matches when "sorted" is present
#[tokio::test]
async fn test_disk_space_by_directory_sorted() {
    let profile = CapabilityProfile::for_platform(caro::prompts::ProfileType::Bsd);
    let matcher = StaticMatcher::new(profile);

    let request = CommandRequest::new("show disk space by directory sorted", ShellType::Bash);

    let result = matcher.generate_command(&request).await;
    assert!(result.is_ok(), "Command generation should succeed");

    let cmd = result.unwrap();
    assert!(
        cmd.command.contains("du") && cmd.command.contains("sort"),
        "Command should include 'du' and 'sort', got: {}",
        cmd.command
    );
}
