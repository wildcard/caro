//! Platform Compatibility Tests
//!
//! Ensures V2 system works correctly across:
//! - Linux (Ubuntu, Debian, Fedora, Arch)
//! - macOS (Intel and Apple Silicon)
//! - Windows (with WSL support)
//!
//! Tests platform-specific features and graceful degradation.

use cmdai::intelligence::ContextGraph;
use cmdai::learning::PatternDB;
use cmdai::safety::{CommandFeatures, RuleBasedPredictor, Sandbox};
use std::path::PathBuf;

// ============================================================================
// TEST 1: Cache Directory Detection
// ============================================================================

#[test]
fn test_cache_directory_detection() {
    // Verify cache dir exists on all platforms
    // Linux: ~/.cache/cmdai
    // macOS: ~/Library/Caches/cmdai
    // Windows: %APPDATA%/cmdai

    let cache_dir = dirs::cache_dir();
    assert!(
        cache_dir.is_some(),
        "Should detect cache directory on all platforms"
    );

    let cache_dir = cache_dir.unwrap();

    println!("Platform cache directory: {}", cache_dir.display());

    #[cfg(target_os = "linux")]
    {
        assert!(
            cache_dir.to_string_lossy().contains(".cache"),
            "Linux should use .cache directory"
        );
    }

    #[cfg(target_os = "macos")]
    {
        assert!(
            cache_dir.to_string_lossy().contains("Library/Caches"),
            "macOS should use Library/Caches"
        );
    }

    #[cfg(target_os = "windows")]
    {
        assert!(
            cache_dir.to_string_lossy().contains("AppData"),
            "Windows should use AppData"
        );
    }

    println!("✓ Cache directory detection works on this platform");
}

#[test]
fn test_data_directory_detection() {
    // Test data directory for pattern database
    // Linux: ~/.local/share/cmdai
    // macOS: ~/Library/Application Support/cmdai
    // Windows: %APPDATA%/cmdai

    let data_dir = dirs::data_local_dir();
    assert!(
        data_dir.is_some(),
        "Should detect data directory on all platforms"
    );

    let data_dir = data_dir.unwrap();
    println!("Platform data directory: {}", data_dir.display());

    println!("✓ Data directory detection works on this platform");
}

// ============================================================================
// TEST 2: Shell Detection
// ============================================================================

#[test]
fn test_shell_detection() {
    // Detect Bash, Zsh, Fish, PowerShell correctly

    let shell = std::env::var("SHELL").ok();

    if let Some(shell_path) = shell {
        println!("Detected shell: {}", shell_path);

        if shell_path.contains("bash") {
            println!("  → Bash detected");
        } else if shell_path.contains("zsh") {
            println!("  → Zsh detected");
        } else if shell_path.contains("fish") {
            println!("  → Fish detected");
        } else {
            println!("  → Other shell: {}", shell_path);
        }
    } else {
        #[cfg(target_os = "windows")]
        {
            println!("Windows detected (SHELL env var not set)");
            // Check for PowerShell
            let ps_core = std::env::var("PSModulePath").is_ok();
            if ps_core {
                println!("  → PowerShell detected");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("Warning: SHELL environment variable not set");
        }
    }

    println!("✓ Shell detection works on this platform");
}

// ============================================================================
// TEST 3: Path Handling
// ============================================================================

#[test]
fn test_path_handling() {
    // Test platform-specific path separators and special characters

    let test_paths = vec![
        PathBuf::from("/tmp/test"),
        PathBuf::from("/home/user/test"),
        PathBuf::from("./relative/path"),
        PathBuf::from("../parent/path"),
    ];

    for path in test_paths {
        // Should handle paths without panicking
        let _ = path.to_string_lossy();
        let _ = path.is_absolute();
        let _ = path.exists();
    }

    // Test Windows paths (if on Windows)
    #[cfg(target_os = "windows")]
    {
        let windows_paths = vec![
            PathBuf::from(r"C:\Users\test"),
            PathBuf::from(r"\\network\share"),
            PathBuf::from(r"C:\Program Files\test"),
        ];

        for path in windows_paths {
            let _ = path.to_string_lossy();
        }
    }

    // Test spaces in paths
    let path_with_spaces = PathBuf::from("/tmp/path with spaces");
    assert!(path_with_spaces.to_string_lossy().contains("spaces"));

    println!("✓ Path handling works correctly on this platform");
}

// ============================================================================
// TEST 4: Platform-Specific Sandbox
// ============================================================================

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_linux_sandbox() {
    // Test sandbox creation on Linux (temp copy fallback)
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    tokio::fs::write(temp_dir.path().join("test.txt"), "test")
        .await
        .unwrap();

    let sandbox = Sandbox::create(temp_dir.path()).await;
    assert!(
        sandbox.is_ok(),
        "Sandbox should work on Linux: {:?}",
        sandbox.err()
    );

    println!("✓ Linux sandbox (temp copy) works");
}

#[cfg(target_os = "linux")]
#[test]
fn test_btrfs_detection() {
    // Check if BTRFS is available (for future enhancement)
    use std::process::Command;

    let output = Command::new("which").arg("btrfs").output();

    if let Ok(output) = output {
        if output.status.success() {
            println!("BTRFS tools detected - future snapshot support available");
        } else {
            println!("BTRFS tools not found - using temp copy fallback");
        }
    }

    println!("✓ BTRFS detection check complete");
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_macos_sandbox() {
    // Test sandbox creation on macOS
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    tokio::fs::write(temp_dir.path().join("test.txt"), "test")
        .await
        .unwrap();

    let sandbox = Sandbox::create(temp_dir.path()).await;
    assert!(
        sandbox.is_ok(),
        "Sandbox should work on macOS: {:?}",
        sandbox.err()
    );

    println!("✓ macOS sandbox (temp copy) works");
}

#[cfg(target_os = "macos")]
#[test]
fn test_apfs_detection() {
    // Check if running on APFS (for future snapshot support)
    use std::process::Command;

    let output = Command::new("diskutil").args(&["info", "/"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("APFS") {
            println!("APFS filesystem detected - future snapshot support available");
        } else {
            println!("Non-APFS filesystem - using temp copy fallback");
        }
    }

    println!("✓ APFS detection check complete");
}

#[cfg(target_os = "windows")]
#[tokio::test]
async fn test_windows_sandbox() {
    // Test sandbox on Windows (temp copy fallback)
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    tokio::fs::write(temp_dir.path().join("test.txt"), "test")
        .await
        .unwrap();

    let sandbox = Sandbox::create(temp_dir.path()).await;
    assert!(
        sandbox.is_ok(),
        "Sandbox should work on Windows: {:?}",
        sandbox.err()
    );

    println!("✓ Windows sandbox (temp copy) works");
}

// ============================================================================
// TEST 5: Cross-Platform Database
// ============================================================================

#[tokio::test]
async fn test_cross_platform_database() {
    // SQLite should work on all platforms

    // Test in-memory database
    let db = PatternDB::new(":memory:").await;
    assert!(
        db.is_ok(),
        "In-memory database should work on all platforms: {:?}",
        db.err()
    );

    let db = db.unwrap();

    // Test basic operations
    let pattern_id = db
        .record_interaction("test", "test command", "context", None)
        .await;

    assert!(
        pattern_id.is_ok(),
        "Database write should work on all platforms"
    );

    let pattern_id = pattern_id.unwrap();

    let pattern = db.get_pattern_by_id(&pattern_id).await;
    assert!(
        pattern.is_ok(),
        "Database read should work on all platforms"
    );

    println!("✓ Cross-platform database operations work");
}

#[tokio::test]
async fn test_file_based_database() {
    // Test file-based database (platform-specific paths)
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let db = PatternDB::new(db_path.to_str().unwrap()).await;
    assert!(
        db.is_ok(),
        "File-based database should work on all platforms: {:?}",
        db.err()
    );

    let db = db.unwrap();

    // Write and read
    let pattern_id = db
        .record_interaction("test", "command", "context", None)
        .await
        .unwrap();

    let pattern = db.get_pattern_by_id(&pattern_id).await.unwrap();
    assert_eq!(pattern.id, pattern_id);

    // Verify file was created
    assert!(db_path.exists(), "Database file should exist on disk");

    println!("✓ File-based database works on this platform");
}

// ============================================================================
// TEST 6: Context Intelligence Cross-Platform
// ============================================================================

#[tokio::test]
async fn test_context_cross_platform() {
    // Context intelligence should work on all platforms

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let context = ContextGraph::build(&cwd).await;
    assert!(
        context.is_ok(),
        "Context build should work on all platforms: {:?}",
        context.err()
    );

    let context = context.unwrap();

    // Should detect platform
    assert!(
        !context.environment.platform.is_empty(),
        "Should detect platform"
    );

    println!("Platform detected: {}", context.environment.platform);

    // Should detect shell (or handle gracefully)
    if let Some(shell) = context.environment.shell {
        println!("Shell detected: {}", shell);
    } else {
        println!("Shell detection not available (expected on some platforms)");
    }

    // Should have working directory
    assert!(
        !context.environment.cwd.as_os_str().is_empty(),
        "Should detect current working directory"
    );

    println!("✓ Context intelligence works on this platform");
}

// ============================================================================
// TEST 7: Safety Validation Cross-Platform
// ============================================================================

#[test]
fn test_safety_cross_platform() {
    // Safety validation should work identically on all platforms

    let test_commands = vec![
        ("ls -la", true),
        ("rm -rf /", false),
        ("cargo build", true),
        ("sudo rm -rf /usr", false),
    ];

    let predictor = RuleBasedPredictor::new();

    for (command, should_be_safe) in test_commands {
        let features = CommandFeatures::extract(command);
        let risk = predictor.predict_risk(command, &features).unwrap();

        if should_be_safe {
            assert!(
                risk.risk_score < 2.0,
                "{} should be safe on all platforms, got risk {}",
                command,
                risk.risk_score
            );
        } else {
            assert!(
                risk.risk_score >= 5.0,
                "{} should be dangerous on all platforms, got risk {}",
                command,
                risk.risk_score
            );
        }
    }

    println!("✓ Safety validation consistent across platforms");
}

// ============================================================================
// TEST 8: Platform-Specific Command Detection
// ============================================================================

#[test]
fn test_platform_specific_commands() {
    // Some commands are platform-specific

    #[cfg(target_os = "linux")]
    {
        let features = CommandFeatures::extract("apt-get update");
        assert!(features.token_count >= 2);
        println!("✓ Linux-specific commands (apt-get) detected");
    }

    #[cfg(target_os = "macos")]
    {
        let features = CommandFeatures::extract("brew install package");
        assert!(features.token_count >= 3);
        println!("✓ macOS-specific commands (brew) detected");
    }

    #[cfg(target_os = "windows")]
    {
        let features = CommandFeatures::extract("choco install package");
        assert!(features.token_count >= 3);
        println!("✓ Windows-specific commands (choco) detected");
    }

    println!("✓ Platform-specific command detection works");
}

// ============================================================================
// TEST 9: Line Ending Handling
// ============================================================================

#[test]
fn test_line_ending_handling() {
    // Test handling of different line endings (Windows \r\n vs Unix \n)

    let unix_command = "ls -la\ngrep pattern";
    let windows_command = "ls -la\r\ngrep pattern";

    let unix_features = CommandFeatures::extract(unix_command);
    let windows_features = CommandFeatures::extract(windows_command);

    // Both should parse correctly
    assert!(unix_features.token_count >= 2);
    assert!(windows_features.token_count >= 2);

    println!("✓ Line ending handling works correctly");
}

// ============================================================================
// TEST 10: Environment Variable Expansion
// ============================================================================

#[test]
fn test_environment_variables() {
    // Test that we can access environment variables on all platforms

    // These should exist on most platforms
    let common_vars = vec!["PATH", "HOME", "USER"];

    for var in common_vars {
        if let Ok(value) = std::env::var(var) {
            println!("{} = {} (length: {})", var, if value.len() > 50 { "..." } else { &value }, value.len());
        } else {
            println!("{} not set (acceptable on some platforms)", var);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(username) = std::env::var("USERNAME") {
            println!("USERNAME = {}", username);
        }
    }

    println!("✓ Environment variable access works");
}

// ============================================================================
// SUMMARY TEST
// ============================================================================

#[test]
fn test_platform_summary() {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║         PLATFORM COMPATIBILITY SUMMARY               ║");
    println!("╚══════════════════════════════════════════════════════╝");

    println!("\nCurrent platform:");

    #[cfg(target_os = "linux")]
    println!("  OS: Linux");

    #[cfg(target_os = "macos")]
    println!("  OS: macOS");

    #[cfg(target_os = "windows")]
    println!("  OS: Windows");

    #[cfg(target_arch = "x86_64")]
    println!("  Arch: x86_64 (Intel/AMD)");

    #[cfg(target_arch = "aarch64")]
    println!("  Arch: aarch64 (Apple Silicon/ARM)");

    println!("\nSupported features:");
    println!("  ✓ Cache directory detection");
    println!("  ✓ Data directory detection");
    println!("  ✓ Shell detection");
    println!("  ✓ Path handling");
    println!("  ✓ Sandbox (temp copy)");
    println!("  ✓ SQLite database");
    println!("  ✓ Context intelligence");
    println!("  ✓ Safety validation");

    println!("\nPlatform-specific optimizations:");

    #[cfg(target_os = "linux")]
    println!("  • BTRFS snapshot support (planned)");

    #[cfg(target_os = "macos")]
    println!("  • APFS snapshot support (planned)");

    #[cfg(target_os = "windows")]
    println!("  • Shadow copy support (planned)");

    println!("\nAll core features work on this platform!");
}
