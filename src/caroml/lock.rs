//! Lock file format (TOML) for CaroML — schema_version 2.
//!
//! The lock pairs each `DO` step with one or more **variants** keyed by
//! platform. Each variant carries the generated command, validations, a
//! track record, and lineage metadata. A separate `[[history]]` list is
//! the audit trail.
//!
//! Atomic writes use the `.lock.tmp → rename` convention also used by
//! [`crate::ai::store`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// The current supported schema version. Bumping this is a breaking change
/// to consumers; provide a `caro lock migrate` path before raising it.
pub const SCHEMA_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lock {
    pub schema_version: u32,
    #[serde(default)]
    pub meta: Meta,
    #[serde(default)]
    pub task: TaskMeta,
    /// One entry per `DO` step in the source `.caro` file.
    #[serde(default)]
    pub steps: Vec<Step>,
    /// Append-only generation lineage; ordered oldest-first.
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

impl Default for Lock {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            meta: Meta::default(),
            task: TaskMeta::default(),
            steps: Vec::new(),
            history: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-records
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Meta {
    /// Caro version that wrote this lock.
    #[serde(default)]
    pub caro_version: String,
    /// Path of the source `.caro` file relative to the project root.
    #[serde(default)]
    pub intent_path: String,
    /// SHA-256 hash of the parsed-and-normalized `.caro` AST.
    #[serde(default)]
    pub intent_hash: String,
    /// Platforms with at least one variant in this lock.
    #[serde(default)]
    pub supported_platforms: Vec<String>,
    /// Wall-clock timestamp of the last full regeneration.
    #[serde(default)]
    pub last_full_regen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TaskMeta {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub why: Option<String>,
    #[serde(default)]
    pub needs: Vec<String>,
    /// platform → list of `PREFER` items (e.g. `macos → [bsd-tools]`).
    #[serde(default)]
    pub prefers_by_platform: BTreeMap<String, Vec<String>>,
    /// platform → list of `AVOID` items.
    #[serde(default)]
    pub avoids_by_platform: BTreeMap<String, Vec<String>>,
    /// Resolved `LET` parameter values keyed by name.
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// 1-based source line number in the `.caro` file.
    pub line: usize,
    /// Intent text after `LET` substitution.
    pub intent: String,
    /// SHA-256 of the normalized step intent (for per-step staleness).
    pub intent_hash: String,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub variants: Vec<Variant>,
}

impl Step {
    /// The active variant for the given platform, if any.
    pub fn active_variant(&self, platform: &str) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|v| v.platform == platform && v.active)
    }

    /// All challenger variants for the given platform (active = false, not retired).
    pub fn challengers<'a>(&'a self, platform: &'a str) -> impl Iterator<Item = &'a Variant> + 'a {
        self.variants
            .iter()
            .filter(move |v| v.platform == platform && !v.active && v.retired_at.is_none())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    /// Platform identifier: "macos" / "linux" / "windows" / "posix".
    pub platform: String,
    /// True iff this variant is the chosen one for its platform.
    /// At most one active per platform per step.
    pub active: bool,
    /// Stable identifier for this generation; format `gen_<YYYY-MM-DD>_<plat>_<a|b|...>`.
    pub generation_id: String,
    /// The generated shell command for this step.
    pub command: String,
    /// Why the LLM chose this command (free-form, used for `caro why`).
    #[serde(default)]
    pub reasoning: String,
    /// Shell variables this command sets (informational; for downstream prompt context).
    #[serde(default)]
    pub exports: Vec<String>,
    /// Shell variables this command reads from prior steps.
    #[serde(default)]
    pub imports: Vec<String>,
    /// "safe" | "moderate" | "high" | "critical".
    pub risk_level: String,
    #[serde(default)]
    pub matched_patterns: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    /// 0.0–1.0 from the backend's confidence score.
    pub confidence: f32,
    /// Number of validation-loop iterations needed to settle on this command.
    #[serde(default = "default_iterations")]
    pub iterations: u32,
    #[serde(default)]
    pub validations: Vec<ValidationEntry>,
    pub generated_at: DateTime<Utc>,
    pub model: String,
    pub backend: String,
    /// Probed tool versions at generation time (informational for SoftExplore detection).
    #[serde(default)]
    pub tool_versions: BTreeMap<String, String>,
    /// Run history aggregate (project memory).
    #[serde(default)]
    pub track_record: TrackRecord,
    /// If set, this variant has been retired (no longer active or challenger).
    #[serde(default)]
    pub retired_at: Option<DateTime<Utc>>,
}

fn default_iterations() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationEntry {
    /// "safety" | "platform" | "secrets" | "side_effects" | …
    pub angle: String,
    /// "pass" | "warn" | "fail".
    pub result: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackRecord {
    #[serde(default)]
    pub runs: u64,
    #[serde(default)]
    pub succeeded: u64,
    #[serde(default)]
    pub failed: u64,
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub generation_id: String,
    /// Why this regeneration happened: "intent_hash_mismatch" | "cve_feed_update" | …
    pub trigger: String,
    pub caro_version: String,
    pub model: String,
    pub backend: String,
    pub platform: String,
    pub generated_at: DateTime<Utc>,
    pub intent_hash: String,
    #[serde(default)]
    pub cve_feed_rev: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Read / write
// ---------------------------------------------------------------------------

impl Lock {
    /// Parse a lock from a TOML string.
    pub fn read_str(toml_src: &str) -> Result<Self, LockError> {
        let lock: Lock = toml::from_str(toml_src).map_err(LockError::Parse)?;
        if lock.schema_version != SCHEMA_VERSION {
            return Err(LockError::UnsupportedSchema(lock.schema_version));
        }
        Ok(lock)
    }

    /// Serialize the lock to a TOML string.
    pub fn write_string(&self) -> Result<String, LockError> {
        toml::to_string_pretty(self).map_err(LockError::Serialize)
    }

    /// Read a lock file from disk.
    pub fn read_path(path: &Path) -> Result<Self, LockError> {
        let s = std::fs::read_to_string(path).map_err(LockError::Io)?;
        Self::read_str(&s)
    }

    /// Write the lock atomically: write to `<path>.tmp`, then rename.
    pub fn write_path(&self, path: &Path) -> Result<(), LockError> {
        let s = self.write_string()?;
        let tmp = path_with_tmp_suffix(path);
        std::fs::write(&tmp, s).map_err(LockError::Io)?;
        std::fs::rename(&tmp, path).map_err(LockError::Io)?;
        Ok(())
    }
}

fn path_with_tmp_suffix(path: &Path) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".tmp");
    s.into()
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum LockError {
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    Io(std::io::Error),
    UnsupportedSchema(u32),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "lock TOML parse error: {}", e),
            Self::Serialize(e) => write!(f, "lock TOML serialize error: {}", e),
            Self::Io(e) => write!(f, "lock IO error: {}", e),
            Self::UnsupportedSchema(v) => write!(
                f,
                "lock has schema_version {} but only {} is supported (use `caro lock migrate`)",
                v, SCHEMA_VERSION
            ),
        }
    }
}

impl std::error::Error for LockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(e) => Some(e),
            Self::Serialize(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::UnsupportedSchema(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_variant() -> Variant {
        Variant {
            platform: "macos".into(),
            active: true,
            generation_id: "gen_2026-04-26_macos_a".into(),
            command: "files=$(find /var/log -type f -name '*.log')".into(),
            reasoning: "Use BSD find with -type f / -name; capture into a shell variable.".into(),
            exports: vec!["files".into()],
            imports: Vec::new(),
            risk_level: "safe".into(),
            matched_patterns: Vec::new(),
            warnings: Vec::new(),
            confidence: 0.94,
            iterations: 1,
            validations: vec![ValidationEntry {
                angle: "safety".into(),
                result: "pass".into(),
                note: None,
            }],
            generated_at: Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap(),
            model: "smollm-1.7b".into(),
            backend: "embedded".into(),
            tool_versions: {
                let mut m = BTreeMap::new();
                m.insert("find".into(), "BSD-stock-macos14".into());
                m
            },
            track_record: TrackRecord {
                runs: 42,
                succeeded: 42,
                failed: 0,
                last_used: Some(Utc.with_ymd_and_hms(2026, 4, 25, 8, 30, 0).unwrap()),
            },
            retired_at: None,
        }
    }

    fn sample_lock() -> Lock {
        Lock {
            schema_version: SCHEMA_VERSION,
            meta: Meta {
                caro_version: "1.4.0".into(),
                intent_path: "tasks/cleanup-logs.caro".into(),
                intent_hash: "sha256:a7b3c1".into(),
                supported_platforms: vec!["macos".into(), "linux".into()],
                last_full_regen: Some(Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap()),
            },
            task: TaskMeta {
                title: "Clean up old log files".into(),
                why: Some("Free disk space".into()),
                needs: vec!["sudo".into()],
                prefers_by_platform: {
                    let mut m = BTreeMap::new();
                    m.insert("macos".into(), vec!["bsd-tools".into()]);
                    m
                },
                avoids_by_platform: BTreeMap::new(),
                params: {
                    let mut m = BTreeMap::new();
                    m.insert("path".into(), "/var/log".into());
                    m.insert("days".into(), "30".into());
                    m
                },
            },
            steps: vec![Step {
                line: 12,
                intent: "find log files in /var/log".into(),
                intent_hash: "sha256:step1".into(),
                notes: vec!["prefer single-pass find".into()],
                variants: vec![sample_variant()],
            }],
            history: vec![HistoryEntry {
                generation_id: "gen_2026-04-26_macos_a".into(),
                trigger: "intent_hash_mismatch".into(),
                caro_version: "1.4.0".into(),
                model: "smollm-1.7b".into(),
                backend: "embedded".into(),
                platform: "macos".into(),
                generated_at: Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap(),
                intent_hash: "sha256:a7b3c1".into(),
                cve_feed_rev: Some("2026-04-26".into()),
                notes: Some("initial gen".into()),
            }],
        }
    }

    #[test]
    fn round_trip_via_toml() {
        let lock = sample_lock();
        let s = lock.write_string().unwrap();
        let back = Lock::read_str(&s).unwrap();
        assert_eq!(lock, back);
    }

    #[test]
    fn defaults_compile_and_serialize() {
        let lock = Lock::default();
        let s = lock.write_string().unwrap();
        let back = Lock::read_str(&s).unwrap();
        assert_eq!(lock, back);
    }

    #[test]
    fn rejects_unsupported_schema() {
        let mut lock = sample_lock();
        lock.schema_version = 99;
        let s = lock.write_string().unwrap();
        match Lock::read_str(&s) {
            Err(LockError::UnsupportedSchema(99)) => {}
            other => panic!("expected UnsupportedSchema(99), got {:?}", other),
        }
    }

    #[test]
    fn active_variant_lookup() {
        let lock = sample_lock();
        let step = &lock.steps[0];
        assert!(step.active_variant("macos").is_some());
        assert!(step.active_variant("linux").is_none());
    }

    #[test]
    fn challengers_filter() {
        let mut lock = sample_lock();
        let mut challenger = sample_variant();
        challenger.active = false;
        challenger.generation_id = "gen_2026-04-26_macos_b".into();
        lock.steps[0].variants.push(challenger);
        let count = lock.steps[0].challengers("macos").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn write_path_atomic_rename(/* tmpfs not always available; use tempdir */) {
        let dir = tempdir_or_skip();
        let path = dir.path().join("sample.lock");
        let lock = sample_lock();
        lock.write_path(&path).unwrap();
        let back = Lock::read_path(&path).unwrap();
        assert_eq!(lock, back);
        // The .tmp file should not remain after a successful rename.
        let tmp = super::path_with_tmp_suffix(&path);
        assert!(!tmp.exists());
    }

    fn tempdir_or_skip() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir creation should succeed")
    }
}
