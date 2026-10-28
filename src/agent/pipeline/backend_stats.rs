//! Per-backend effectiveness stats — the feedback signal that lets best-of-N
//! ranking self-tune toward whichever backend's suggestions the user actually
//! accepts.
//!
//! Warp's "measure the effectiveness of each harness", at the backend layer: a
//! backend earns a **win only when its selected command was accepted by the
//! user** (executed, exit 0) — never when the scorer merely picked it.
//! Crediting the scorer's own choice would create a rich-get-richer feedback
//! loop; crediting real acceptance means a backend that keeps being picked but
//! keeps failing loses score and gets picked less.
//!
//! Persisted as a small JSON map under the caro data dir, alongside the
//! knowledge index. All disk access is best-effort: a corrupt or unwritable
//! stats file must never break command generation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Per-backend counters. `latency_ewma_ms` is an exponentially-weighted moving
/// average of accepted-command generation latency (informational for now).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BackendRecord {
    pub wins: u64,
    pub attempts: u64,
    pub latency_ewma_ms: f64,
}

/// Map of `backend label → record`, loaded from / saved to disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendStats {
    #[serde(default)]
    by_backend: HashMap<String, BackendRecord>,
}

/// Canonical on-disk location, alongside the knowledge index.
pub fn default_backend_stats_path() -> PathBuf {
    directories::ProjectDirs::from("sh", "caro", "caro")
        .map(|dirs| dirs.data_dir().join("backend_stats.json"))
        .unwrap_or_else(|| PathBuf::from("backend_stats.json"))
}

impl BackendStats {
    /// Load the stats from `path`, or return an empty set if it doesn't exist
    /// or can't be parsed (best-effort: a corrupt file never breaks ranking).
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                debug!("backend_stats: parse failed ({e}); starting empty");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Load from the default path.
    pub fn load_default() -> Self {
        Self::load(&default_backend_stats_path())
    }

    /// Laplace-smoothed acceptance rate for a backend, in `[0, 1]`. Returns the
    /// neutral prior **0.5** when there's no data, so an un-rated backend is
    /// neither rewarded nor penalized.
    pub fn success_score(&self, label: &str) -> f32 {
        match self.by_backend.get(label) {
            // Treat a semantically corrupt entry (wins > attempts, only reachable
            // via a hand-edited/garbled file) as unrated — the neutral 0.5 prior
            // — rather than promoting it to a perfect score and distorting ranking.
            Some(r) if r.wins <= r.attempts => {
                ((r.wins as f64 + 1.0) / (r.attempts as f64 + 2.0)) as f32
            }
            _ => 0.5,
        }
    }

    /// Record an outcome for `label`: every call is an attempt; `accepted`
    /// credits a win. `latency_ms` feeds the EWMA. In-memory only — call
    /// [`Self::save`] to persist, or [`Self::record_and_save`] for both.
    pub fn record(&mut self, label: &str, accepted: bool, latency_ms: u64) {
        let rec = self.by_backend.entry(label.to_string()).or_default();
        rec.attempts += 1;
        if accepted {
            rec.wins += 1;
        }
        // EWMA (alpha = 0.2); seed with the first sample.
        let sample = latency_ms as f64;
        rec.latency_ewma_ms = if rec.attempts == 1 {
            sample
        } else {
            0.2 * sample + 0.8 * rec.latency_ewma_ms
        };
    }

    /// Serialize to `path`, creating parent dirs as needed.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Read-modify-write a single outcome to the default path under an exclusive
    /// file lock (a sidecar `.lock` file) so concurrent caro processes don't
    /// clobber each other. Best-effort: any I/O error is logged and swallowed —
    /// these stats are advisory, never critical.
    pub fn record_and_save(label: &str, accepted: bool, latency_ms: u64) {
        if let Err(e) = Self::try_record_and_save(label, accepted, latency_ms) {
            debug!("backend_stats: record failed ({e}); skipping");
        }
    }

    fn try_record_and_save(label: &str, accepted: bool, latency_ms: u64) -> std::io::Result<()> {
        let path = default_backend_stats_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Lock a sidecar file for the critical section (avoids seek/truncate
        // gymnastics on the data file itself).
        let lock_file = std::fs::File::create(path.with_extension("lock"))?;
        let mut lock = fd_lock::RwLock::new(lock_file);
        let _guard = lock.write()?;

        let mut stats = Self::load(&path);
        stats.record(label, accepted, latency_ms);
        stats.save(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrated_backend_is_neutral_prior() {
        let stats = BackendStats::default();
        assert!((stats.success_score("anything") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn wins_raise_and_failures_lower_the_score() {
        let mut stats = BackendStats::default();
        for _ in 0..8 {
            stats.record("good", true, 100);
        }
        for _ in 0..8 {
            stats.record("bad", false, 100);
        }
        let good = stats.success_score("good");
        let bad = stats.success_score("bad");
        assert!(good > 0.5, "wins should push above the prior, got {good}");
        assert!(bad < 0.5, "failures should push below the prior, got {bad}");
        assert!(good > bad);
        // Bounded in [0, 1].
        assert!((0.0..=1.0).contains(&good) && (0.0..=1.0).contains(&bad));
    }

    #[test]
    fn save_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backend_stats.json");
        let mut stats = BackendStats::default();
        stats.record("mesh", true, 250);
        stats.record("mesh", false, 300);
        stats.save(&path).unwrap();

        let loaded = BackendStats::load(&path);
        assert_eq!(loaded.by_backend.get("mesh").unwrap().attempts, 2);
        assert_eq!(loaded.by_backend.get("mesh").unwrap().wins, 1);
        assert!((loaded.success_score("mesh") - stats.success_score("mesh")).abs() < 1e-6);
    }

    #[test]
    fn corrupt_entry_falls_back_to_neutral() {
        // wins > attempts is unreachable via record(); only a hand-edited or
        // garbled file produces it. It must read as neutral, never a perfect 1.0.
        let json = r#"{"by_backend":{"cheater":{"wins":99,"attempts":1,"latency_ewma_ms":0.0}}}"#;
        let stats: BackendStats = serde_json::from_str(json).unwrap();
        assert!((stats.success_score("cheater") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn missing_file_loads_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let stats = BackendStats::load(&path);
        assert!((stats.success_score("x") - 0.5).abs() < 1e-6);
    }
}
