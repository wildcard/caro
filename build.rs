use std::env;
use std::fs;
use std::process::Command;

/// Try to get git info from the repository
fn get_git_info() -> Option<(String, String, String)> {
    // Try git rev-parse for short hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())?;

    // Get full hash
    let git_hash_full = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())?;

    // Get commit date
    let git_date = Command::new("git")
        .args(["log", "-1", "--format=%ci"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.split_whitespace().next().unwrap_or("").to_string())?;

    Some((git_hash, git_hash_full, git_date))
}

/// Try to get git info from .cargo_vcs_info.json (created by cargo publish)
fn get_cargo_vcs_info() -> Option<(String, String)> {
    let vcs_info = fs::read_to_string(".cargo_vcs_info.json").ok()?;

    // Parse the JSON manually to avoid serde dependency in build.rs
    // Format: {"git":{"sha1":"abc123..."},"path_in_vcs":""}
    let sha1_start = vcs_info.find("\"sha1\":")? + 8;
    let sha1_content = &vcs_info[sha1_start..];
    let sha1_end = sha1_content.find('"')?;
    let full_hash = sha1_content[..sha1_end].to_string();
    let short_hash = full_hash.chars().take(7).collect::<String>();

    Some((short_hash, full_hash))
}

fn main() {
    // Try to get git info from repository first, then fall back to .cargo_vcs_info.json
    let (git_hash, git_hash_full, git_date) = if let Some(info) = get_git_info() {
        info
    } else if let Some((short, full)) = get_cargo_vcs_info() {
        // When installed from crates.io, we have the hash but not the date
        (short, full, "crates.io".to_string())
    } else {
        // Complete fallback - no git info available
        (
            "source".to_string(),
            "source".to_string(),
            "source".to_string(),
        )
    };

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
