use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Compile CVE rules at build time. Shared compiler lives at
// src/dogma/compiler.rs and is consumed here via #[path] include.
#[path = "src/dogma/compiler.rs"]
mod cve_compiler;

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

    // ── CVE + 0din ruleset compilation ────────────────────────────────────
    // Always produces a bincode blob (possibly empty when cve-rules is off,
    // or when no YAML rules exist yet). Runtime loader tolerates the empty case.
    let (cve_count, odin_count) = compile_cve_ruleset();
    println!("cargo:rustc-env=CARO_CVE_RULE_COUNT={}", cve_count);
    println!("cargo:rustc-env=CARO_ODIN_PROBE_COUNT={}", odin_count);
}

/// Compile `data/cve_rules/CVE-*.yaml` and `ODIN-*.yaml` into `$OUT_DIR/cve_patterns.bin`
/// (bincode blob read by `crate::safety::cve_patterns::CVE_COMPILED`).
///
/// Also emits `$OUT_DIR/cve_generated_tests.yaml` for the eval suite to
/// pick up. Returns `(cve_count, odin_count)` of compiled (non-draft) rules.
fn compile_cve_ruleset() -> (usize, usize) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    let bin_path = out_dir.join("cve_patterns.bin");
    let tests_path = out_dir.join("cve_generated_tests.yaml");
    let rules_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/cve_rules");

    println!("cargo:rerun-if-changed=data/cve_rules");

    let paths = match cve_compiler::discover_rule_files(&rules_dir) {
        Ok(p) => p,
        Err(e) => {
            println!("cargo:warning=CVE/0din rule discovery failed: {}", e);
            Vec::new()
        }
    };

    // Count CVE-* and ODIN-* paths separately for version output.
    let cve_path_count = paths
        .iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.starts_with("CVE-"))
        })
        .count();
    let _odin_path_count = paths.len() - cve_path_count;

    for p in &paths {
        println!("cargo:rerun-if-changed={}", p.display());
    }

    let (ruleset, tests) = match cve_compiler::compile_with_tests(&paths) {
        Ok(x) => x,
        Err(e) => panic!(
            "CVE/0din rule compile failed: {}. Fix data/cve_rules/*.yaml or remove the bad file.",
            e
        ),
    };

    let bytes = bincode::serialize(&ruleset)
        .expect("bincode serialize of CompiledRuleset must succeed (pure-serde types)");
    fs::write(&bin_path, bytes).expect("write cve_patterns.bin");

    // Emit test YAML for the eval framework.
    let mut yaml = String::from("metadata:\n  name: CVE+0din generated test cases\n  description: \"Auto-generated from data/cve_rules/*.yaml\"\n\ntest_cases:\n");
    for (rule_id, tc) in &tests {
        // Escape the input for YAML safety.
        let id = format!("{}-{}", rule_id, sanitize(&tc.input));
        let esc_input = tc.input.replace('\\', "\\\\").replace('"', "\\\"");
        let esc_behavior = tc.expected_behavior.replace('"', "\\\"");
        let category = if rule_id.starts_with("ODIN-") { "0din" } else { "cve" };
        yaml.push_str(&format!(
            "  - id: \"{id}\"\n    input: \"{inp}\"\n    expected_behavior: \"{beh}\"\n    category: \"{cat}\"\n    risk_level: \"critical\"\n",
            id = id,
            inp = esc_input,
            beh = esc_behavior,
            cat = category,
        ));
    }
    fs::write(&tests_path, yaml).expect("write cve_generated_tests.yaml");

    // Return compiled counts; use path counts as proxy for draft-skipped rules are rare.
    let total = ruleset.patterns.len();
    let odin_compiled = total.saturating_sub(cve_path_count);
    let cve_compiled = total - odin_compiled;
    (cve_compiled, odin_compiled)
}

/// Derive a stable suffix from a test-case input for the generated test id.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(24)
        .collect()
}
