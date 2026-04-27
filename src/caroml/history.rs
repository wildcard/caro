//! Per-user run journal at `~/.caro/state/<intent_hash_hex>/journal.jsonl`.
//!
//! Every `caro run` invocation appends a JSON line capturing what happened.
//! The journal is per-user / per-machine — it is **not** committed to the
//! project. Aggregate adopt decisions cross users via the lock's per-variant
//! `track_record`; the journal is the local input that drives those updates.
//!
//! The journal is append-only. Compaction / pruning is out of scope for v0.1.
//!
//! ## Why hash-based directory names
//!
//! Two tasks can have the same on-disk name (a project task and a global
//! library task with the same `name`), but their normalized intent hashes
//! differ. Keying the journal directory by intent_hash means the local
//! history follows the *intent*, not the file path.

use crate::caroml::lock::TrackRecord;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// One run captured into the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: DateTime<Utc>,
    pub intent_hash: String,
    pub variant_id: String,
    pub platform: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    /// SHA-256 hex digest of the first 4 KB of stderr (privacy default;
    /// see plan §"Privacy of the local journal").
    pub stderr_digest: String,
    /// Optional human-readable note; e.g. the failing step's intent.
    pub note: Option<String>,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("home directory not available")]
    NoHomeDir,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Compute the journal directory for `intent_hash`. Strips the `sha256:` prefix.
pub fn journal_dir(intent_hash: &str) -> Result<PathBuf, HistoryError> {
    let home = dirs::home_dir().ok_or(HistoryError::NoHomeDir)?;
    let key = intent_hash.strip_prefix("sha256:").unwrap_or(intent_hash);
    Ok(home.join(".caro").join("state").join(key))
}

/// The journal file path for `intent_hash`.
pub fn journal_path(intent_hash: &str) -> Result<PathBuf, HistoryError> {
    Ok(journal_dir(intent_hash)?.join("journal.jsonl"))
}

/// Append one entry to the journal (creates the directory + file if needed).
pub fn append(entry: &JournalEntry) -> Result<(), HistoryError> {
    let path = journal_path(&entry.intent_hash)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    append_to(&path, entry)
}

/// Append into a specific path (used by tests with tempdirs).
pub fn append_to(path: &Path, entry: &JournalEntry) -> Result<(), HistoryError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(entry)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

/// Read all entries for `intent_hash`. Returns empty Vec when the journal is missing.
pub fn read_all(intent_hash: &str) -> Result<Vec<JournalEntry>, HistoryError> {
    let path = journal_path(intent_hash)?;
    read_from(&path)
}

pub fn read_from(path: &Path) -> Result<Vec<JournalEntry>, HistoryError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let body = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: JournalEntry = serde_json::from_str(line)?;
        out.push(entry);
    }
    Ok(out)
}

/// Compute an aggregated [`TrackRecord`] for `(intent_hash, variant_id)` by
/// summing the journal entries.
pub fn aggregate(intent_hash: &str, variant_id: &str) -> Result<TrackRecord, HistoryError> {
    let entries = read_all(intent_hash)?;
    Ok(aggregate_entries(&entries, variant_id))
}

pub fn aggregate_entries(entries: &[JournalEntry], variant_id: &str) -> TrackRecord {
    let mut tr = TrackRecord::default();
    for e in entries {
        if e.variant_id != variant_id {
            continue;
        }
        tr.runs += 1;
        if e.exit_code == 0 {
            tr.succeeded += 1;
        } else {
            tr.failed += 1;
        }
        match tr.last_used {
            Some(t) if t >= e.timestamp => {}
            _ => tr.last_used = Some(e.timestamp),
        }
    }
    tr
}

/// Helper: digest the first 4 KB of `stderr` text. Used by the runner to
/// avoid storing arbitrary tool output verbatim.
pub fn stderr_digest(stderr: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = stderr.as_bytes();
    let cap = bytes.len().min(4096);
    let mut h = Sha256::new();
    h.update(&bytes[..cap]);
    let hex = h
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    format!("sha256:{}", hex)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn entry(variant: &str, exit: i32, ts_offset: i64) -> JournalEntry {
        JournalEntry {
            timestamp: Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap()
                + chrono::Duration::seconds(ts_offset),
            intent_hash: "sha256:abc".into(),
            variant_id: variant.into(),
            platform: "macos".into(),
            exit_code: exit,
            duration_ms: 100,
            stderr_digest: "sha256:0".into(),
            note: None,
        }
    }

    #[test]
    fn append_and_read_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal.jsonl");
        append_to(&path, &entry("gen_a", 0, 0)).unwrap();
        append_to(&path, &entry("gen_a", 1, 60)).unwrap();
        let entries = read_from(&path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].variant_id, "gen_a");
        assert_eq!(entries[0].exit_code, 0);
        assert_eq!(entries[1].exit_code, 1);
    }

    #[test]
    fn read_from_missing_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let entries = read_from(&dir.path().join("nope.jsonl")).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn aggregate_counts_runs_succeeded_failed() {
        let entries = vec![
            entry("gen_a", 0, 0),
            entry("gen_a", 0, 10),
            entry("gen_a", 1, 20),
            entry("gen_b", 0, 30), // different variant — ignored
        ];
        let tr = aggregate_entries(&entries, "gen_a");
        assert_eq!(tr.runs, 3);
        assert_eq!(tr.succeeded, 2);
        assert_eq!(tr.failed, 1);
        assert!(tr.last_used.is_some());
    }

    #[test]
    fn stderr_digest_truncates_to_4kb() {
        let big = "x".repeat(8192);
        let small = "x".repeat(4096);
        // Both produce the same digest because we only hash the first 4 KB.
        assert_eq!(stderr_digest(&big), stderr_digest(&small));
    }

    #[test]
    fn journal_dir_strips_sha256_prefix() {
        let dir = journal_dir("sha256:abc123").unwrap();
        assert!(dir.to_string_lossy().ends_with("/.caro/state/abc123"));
    }

    #[test]
    fn journal_dir_handles_missing_prefix() {
        let dir = journal_dir("def456").unwrap();
        assert!(dir.to_string_lossy().ends_with("/.caro/state/def456"));
    }
}
