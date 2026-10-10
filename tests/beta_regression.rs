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

/// Content search scoped to a file type: "search for TODO in all python files"
/// Should generate: grep -rn 'TODO' --include='*.py' .
/// Was generating: find . -name "*.py" -type f (lists files, never searches them)
#[tokio::test]
async fn test_search_todo_in_python_files_searches_contents() {
    for (platform, query) in [
        (
            caro::prompts::ProfileType::Bsd,
            "search for TODO in all python files",
        ),
        (
            caro::prompts::ProfileType::GnuLinux,
            "search for TODO in all python files",
        ),
        (
            caro::prompts::ProfileType::GnuLinux,
            "find TODOs in python files",
        ),
        (
            caro::prompts::ProfileType::GnuLinux,
            "grep for TODO in .py files",
        ),
        (
            caro::prompts::ProfileType::GnuLinux,
            "search python files for TODO",
        ),
    ] {
        let matcher = StaticMatcher::new(CapabilityProfile::for_platform(platform));
        let request = CommandRequest::new(query, ShellType::Bash);

        let cmd = matcher
            .generate_command(&request)
            .await
            .expect("Command generation should succeed");
        assert_eq!(
            cmd.command, "grep -rn 'TODO' --include='*.py' .",
            "query {:?} should search .py file contents for TODO",
            query
        );
    }
}

/// Guard for the neighbouring pattern: listing python files is still a find.
#[tokio::test]
async fn test_find_all_python_files_still_lists_files() {
    let matcher = StaticMatcher::new(CapabilityProfile::ubuntu());
    for query in ["find all python files", "find python files"] {
        let request = CommandRequest::new(query, ShellType::Bash);
        let cmd = matcher
            .generate_command(&request)
            .await
            .expect("Command generation should succeed");
        assert_eq!(
            cmd.command, r#"find . -name "*.py" -type f"#,
            "query {:?}",
            query
        );
    }
}

/// Issue #1516: "search for <term> in python files" must search file contents for
/// that term. It used to return `find . -name "*.py" -type f` for any term but TODO.
#[tokio::test]
async fn test_search_for_term_in_python_files_greps_contents() {
    let matcher = StaticMatcher::new(CapabilityProfile::ubuntu());
    for (query, expected) in [
        (
            "search for FIXME in all python files",
            "grep -rn 'FIXME' --include='*.py' .",
        ),
        (
            "search for print in python files",
            "grep -rn 'print' --include='*.py' .",
        ),
        (
            "grep for import os in .py files",
            "grep -rn 'import os' --include='*.py' .",
        ),
        (
            "search python files for print",
            "grep -rn 'print' --include='*.py' .",
        ),
        (
            "find python files containing requests",
            "grep -rn 'requests' --include='*.py' .",
        ),
        (
            "search for print statements in python files",
            "grep -rn 'print' --include='*.py' .",
        ),
        (
            "search for \"api_key\" in python files",
            "grep -rn 'api_key' --include='*.py' .",
        ),
        (
            "Search for TODO comments in Python files",
            "grep -rn 'TODO' --include='*.py' .",
        ),
    ] {
        let request = CommandRequest::new(query, ShellType::Bash);
        let cmd = matcher
            .generate_command(&request)
            .await
            .expect("Command generation should succeed");
        assert_eq!(cmd.command, expected, "query {:?}", query);
    }
}

/// Issue #1516 guard: listing phrasings must keep producing a file listing.
#[tokio::test]
async fn test_python_file_listing_phrasings_unchanged() {
    let matcher = StaticMatcher::new(CapabilityProfile::ubuntu());
    for query in [
        "locate .py files",
        "find files ending in .py",
        "search for python files",
    ] {
        let request = CommandRequest::new(query, ShellType::Bash);
        let cmd = matcher
            .generate_command(&request)
            .await
            .expect("Command generation should succeed");
        assert_eq!(
            cmd.command, r#"find . -name "*.py" -type f"#,
            "query {:?}",
            query
        );
    }
}
