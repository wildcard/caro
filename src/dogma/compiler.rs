//! Shared YAML → CompiledRuleset compiler.
//!
//! This module is consumed from two places:
//!   1. `build.rs` — via `#[path = "src/dogma/compiler.rs"] mod compiler;`
//!      to compile `data/cve_rules/*.yaml` into a bincode blob at build time.
//!   2. Runtime (via `src/dogma/mod.rs`) — to expose the deserialized types
//!      to `src/safety/cve_patterns.rs`.
//!
//! It must therefore depend on nothing beyond `std` + `serde` + `serde_yaml`,
//! since `build.rs` has no access to the crate's own types or features.
//!
//! YAML shape accepted here is the one documented in
//! `data/cve_rules/README.md` and validated by `scripts/validate-cve-yaml.ts`.
//! `pattern: "TODO_NIMBLE_AUTHORED"` drafts are silently skipped so
//! not-yet-authored Nimble-sync PRs don't break the build.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Compiled, bincode-serialized CVE ruleset.
///
/// Written to `$OUT_DIR/cve_patterns.bin` by `build.rs` and loaded via
/// `include_bytes!` at runtime. The `metadata` block is surfaced in
/// `caro --version` via `CARO_CVE_RULE_COUNT` and friends.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompiledRuleset {
    pub patterns: Vec<CompiledPattern>,
    pub metadata: RulesetMetadata,
}

/// A single CVE-derived pattern after compilation.
///
/// Structurally equivalent to `crate::safety::DangerPattern` but defined
/// here independently so `build.rs` can construct it without pulling in
/// the runtime safety module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPattern {
    /// CVE ID (e.g. `CVE-2024-3094`)
    pub id: String,
    /// Advisory URL
    pub source: String,
    /// Disclosure date (ISO `YYYY-MM-DD`)
    pub disclosed: String,
    /// Lowercase risk level: `safe` / `moderate` / `high` / `critical`
    pub risk_level: String,
    /// Lowercase shell type or `None`: `bash` / `zsh` / `fish` / `sh` / `powershell` / `cmd`
    pub shell_specific: Option<String>,
    /// The compiled regex pattern string
    pub pattern: String,
    /// Human-readable description (shown in validator warnings)
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesetMetadata {
    pub rule_count: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub moderate_count: usize,
    pub safe_count: usize,
    pub skipped_drafts: usize,
}

/// Raw YAML shape — what lives in `data/cve_rules/CVE-*.yaml`.
///
/// `test_cases` is parsed but not embedded in the compiled ruleset — it's
/// only consulted at build time to emit `$OUT_DIR/cve_generated_tests.yaml`
/// for the eval framework.
#[derive(Debug, Clone, Deserialize)]
pub struct RawRule {
    pub id: String,
    pub source: String,
    pub disclosed: String,
    pub risk_level: String,
    #[serde(default)]
    pub shell_specific: Option<String>,
    pub pattern: String,
    pub description: String,
    #[serde(default)]
    pub test_cases: Vec<RawTestCase>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawTestCase {
    pub input: String,
    pub expected_behavior: String,
}

/// Errors produced by `compile_from_paths`.
#[derive(Debug)]
pub enum CompileError {
    Io(String),
    Yaml(String),
    InvalidRule(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(m) => write!(f, "IO: {}", m),
            CompileError::Yaml(m) => write!(f, "YAML: {}", m),
            CompileError::InvalidRule(m) => write!(f, "invalid rule: {}", m),
        }
    }
}

impl std::error::Error for CompileError {}

/// Sentinel used by the Nimble sync script to mark rules that haven't been
/// authored by the LLM yet. Skipped silently so weekly-cron PRs don't break
/// the build before the maintainer authors the pattern.
pub const DRAFT_SENTINEL: &str = "TODO_NIMBLE_AUTHORED";

/// Compile a list of CVE YAML rule files into a `CompiledRuleset`.
///
/// Silently skips rules whose `pattern` equals `DRAFT_SENTINEL`.
/// Returns an error if any other rule is malformed (bad YAML, unknown
/// `risk_level`, invalid regex, etc.) so a bad hand-authored rule fails
/// the build loudly rather than shipping a broken validator.
///
/// `compile_with_tests` is the variant used by `build.rs`. This thinner
/// entry point is kept as the canonical API for the Dogma runtime
/// (spec 006), which will load community rules without needing test
/// cases. Dead-code annotation reflects that the crate itself only uses
/// the test-bearing variant today.
#[allow(dead_code)]
pub fn compile_from_paths(paths: &[PathBuf]) -> Result<CompiledRuleset, CompileError> {
    let mut patterns = Vec::new();
    let mut meta = RulesetMetadata::default();

    for path in paths {
        let raw = read_rule_file(path)?;
        if raw.pattern == DRAFT_SENTINEL {
            meta.skipped_drafts += 1;
            continue;
        }
        validate_raw(&raw, path)?;
        bump_count(&raw.risk_level, &mut meta);
        patterns.push(CompiledPattern {
            id: raw.id,
            source: raw.source,
            disclosed: raw.disclosed,
            risk_level: raw.risk_level,
            shell_specific: raw.shell_specific,
            pattern: raw.pattern,
            description: raw.description,
        });
    }

    meta.rule_count = patterns.len();
    Ok(CompiledRuleset {
        patterns,
        metadata: meta,
    })
}

/// Compile and also return the raw test-case list for emission to
/// `$OUT_DIR/cve_generated_tests.yaml`. Only used from `build.rs`.
pub fn compile_with_tests(
    paths: &[PathBuf],
) -> Result<(CompiledRuleset, Vec<(String, RawTestCase)>), CompileError> {
    let mut patterns = Vec::new();
    let mut meta = RulesetMetadata::default();
    let mut tests = Vec::new();

    for path in paths {
        let raw = read_rule_file(path)?;
        if raw.pattern == DRAFT_SENTINEL {
            meta.skipped_drafts += 1;
            continue;
        }
        validate_raw(&raw, path)?;
        bump_count(&raw.risk_level, &mut meta);
        for tc in &raw.test_cases {
            tests.push((raw.id.clone(), tc.clone()));
        }
        patterns.push(CompiledPattern {
            id: raw.id,
            source: raw.source,
            disclosed: raw.disclosed,
            risk_level: raw.risk_level,
            shell_specific: raw.shell_specific,
            pattern: raw.pattern,
            description: raw.description,
        });
    }

    meta.rule_count = patterns.len();
    Ok((
        CompiledRuleset {
            patterns,
            metadata: meta,
        },
        tests,
    ))
}

/// Discover `data/cve_rules/CVE-*.yaml` files under a given repo root.
/// Used from `build.rs`. Excludes `EXAMPLE-TEMPLATE.yaml` by naming convention
/// (it doesn't start with `CVE-`).
pub fn discover_rule_files(rules_dir: &Path) -> Result<Vec<PathBuf>, CompileError> {
    let mut out = Vec::new();
    if !rules_dir.exists() {
        return Ok(out);
    }
    let entries = std::fs::read_dir(rules_dir)
        .map_err(|e| CompileError::Io(format!("read_dir {}: {}", rules_dir.display(), e)))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with("CVE-") && name.ends_with(".yaml") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn read_rule_file(path: &Path) -> Result<RawRule, CompileError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("{}: {}", path.display(), e)))?;
    serde_yaml::from_str::<RawRule>(&content)
        .map_err(|e| CompileError::Yaml(format!("{}: {}", path.display(), e)))
}

fn validate_raw(raw: &RawRule, path: &Path) -> Result<(), CompileError> {
    let lower = raw.risk_level.to_ascii_lowercase();
    if !matches!(lower.as_str(), "safe" | "moderate" | "high" | "critical") {
        return Err(CompileError::InvalidRule(format!(
            "{}: risk_level must be one of [safe, moderate, high, critical], got {:?}",
            path.display(),
            raw.risk_level
        )));
    }
    if lower != raw.risk_level {
        return Err(CompileError::InvalidRule(format!(
            "{}: risk_level must be lowercase, got {:?}",
            path.display(),
            raw.risk_level
        )));
    }
    if let Some(shell) = raw.shell_specific.as_deref() {
        if !matches!(shell, "bash" | "zsh" | "fish" | "sh" | "powershell" | "cmd") {
            return Err(CompileError::InvalidRule(format!(
                "{}: shell_specific must be one of [bash,zsh,fish,sh,powershell,cmd], got {:?}",
                path.display(),
                shell
            )));
        }
    }
    Ok(())
}

fn bump_count(risk: &str, meta: &mut RulesetMetadata) {
    match risk {
        "critical" => meta.critical_count += 1,
        "high" => meta.high_count += 1,
        "moderate" => meta.moderate_count += 1,
        "safe" => meta.safe_count += 1,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(risk: &str, pattern: &str) -> String {
        format!(
            "id: CVE-2024-0001\n\
             source: https://example.test/x\n\
             disclosed: '2024-01-01'\n\
             risk_level: {risk}\n\
             pattern: \"{pattern}\"\n\
             description: test\n\
             test_cases:\n  - input: foo\n    expected_behavior: Block\n  - input: bar\n    expected_behavior: Allow\n"
        )
    }

    #[test]
    fn rejects_uppercase_risk_level() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), rule("Critical", "foo.*bar")).unwrap();
        let err = compile_from_paths(&[tmp.path().to_path_buf()]).unwrap_err();
        assert!(matches!(err, CompileError::InvalidRule(_)), "got {:?}", err);
    }

    #[test]
    fn accepts_lowercase_risk_level() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), rule("critical", "foo.*bar")).unwrap();
        let rs = compile_from_paths(&[tmp.path().to_path_buf()]).unwrap();
        assert_eq!(rs.patterns.len(), 1);
        assert_eq!(rs.metadata.critical_count, 1);
        assert_eq!(rs.metadata.rule_count, 1);
    }

    #[test]
    fn skips_draft_sentinel() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), rule("critical", "TODO_NIMBLE_AUTHORED")).unwrap();
        let rs = compile_from_paths(&[tmp.path().to_path_buf()]).unwrap();
        assert_eq!(rs.patterns.len(), 0);
        assert_eq!(rs.metadata.skipped_drafts, 1);
    }
}
