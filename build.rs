use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Read git commit SHA from .cargo_vcs_info.json (created by crates.io during publish)
fn read_cargo_vcs_info() -> Option<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let vcs_info_path = Path::new(&manifest_dir).join(".cargo_vcs_info.json");

    let content = fs::read_to_string(vcs_info_path).ok()?;

    // Parse the JSON manually to avoid adding serde as a build dependency
    // Format: {"git":{"sha1":"abc123..."}}
    let sha1_start = content.find("\"sha1\":")? + 8; // Skip past "sha1":"
    let sha1_content = &content[sha1_start..];
    let sha1_end = sha1_content.find('"')?;
    let sha1 = &sha1_content[..sha1_end];

    if !sha1.is_empty() && sha1.len() >= 7 {
        Some(sha1.to_string())
    } else {
        None
    }
}

fn main() {
    // Try git first, then fall back to .cargo_vcs_info.json (for crates.io installs)
    let git_from_command = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let cargo_vcs_sha = read_cargo_vcs_info();

    // Capture git commit hash (short)
    let git_hash = git_from_command
        .clone()
        .or_else(|| cargo_vcs_sha.as_ref().map(|s| s[..7.min(s.len())].to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    // Capture git commit hash (full)
    let git_hash_full = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| cargo_vcs_sha.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Capture git commit date
    // Note: .cargo_vcs_info.json doesn't include the date, so we use a placeholder
    // for crates.io installs. The date shown will be "source" to indicate this.
    let git_date = Command::new("git")
        .args(["log", "-1", "--format=%ci"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            // If we got SHA from cargo_vcs_info but no git date, use "source" as indicator
            if cargo_vcs_sha.is_some() {
                Some("source".to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Use git commit date as build date (more stable and reproducible)
    let build_date = git_date.clone();

    // Capture rustc version
    let rustc_version = Command::new("rustc")
        .args(["--version"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .and_then(|s| {
            // Extract just the version number like "1.92.0"
            s.split_whitespace().nth(1).map(|v| v.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Capture target triple
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    // Capture build profile
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());

    // Check if this is a release binary from GitHub Actions
    let is_release = env::var("CARO_RELEASE").is_ok();

    // Set environment variables for use in the code
    println!("cargo:rustc-env=CARO_GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=CARO_GIT_HASH_FULL={}", git_hash_full);
    println!("cargo:rustc-env=CARO_GIT_DATE={}", git_date);
    println!("cargo:rustc-env=CARO_BUILD_DATE={}", build_date);
    println!("cargo:rustc-env=CARO_RUSTC_VERSION={}", rustc_version);
    println!("cargo:rustc-env=CARO_TARGET={}", target);
    println!("cargo:rustc-env=CARO_BUILD_PROFILE={}", profile);
    println!(
        "cargo:rustc-env=CARO_RELEASE={}",
        if is_release { "1" } else { "0" }
    );

    // Rebuild if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
