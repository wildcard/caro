use std::env;
use std::process::Command;

fn main() {
    // Capture git commit hash (short)
    let git_hash = Command::new("git")
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
        .unwrap_or_else(|| "unknown".to_string());

    // Capture git commit date
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
