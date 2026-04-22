//! JSON-file session store for the AI feature.
//!
//! A single JSON file at `<db_path>` holds an ordered list of [`AiSession`].
//! We intentionally avoid `rusqlite` here: sessions are small, ordered, and
//! ephemeral — a flat file keeps the on-disk format debuggable by humans and
//! doesn't pull the whole SQL surface into a hot CLI path.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::session::AiSession;

/// Errors from the session store.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("no home/data directory available")]
    NoDataDir,
}

/// Default store path: `$XDG_DATA_HOME/caro/ai_sessions.json`.
pub fn default_store_path() -> Result<PathBuf, StoreError> {
    dirs::data_dir()
        .ok_or(StoreError::NoDataDir)
        .map(|d| d.join("caro").join("ai_sessions.json"))
}

/// The on-disk document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoreFile {
    /// Monotonically increasing next-id counter.
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    sessions: Vec<AiSession>,
}

/// Filesystem-backed append-only session store.
pub struct SessionStore {
    path: PathBuf,
    data: StoreFile,
}

impl SessionStore {
    /// Open or create a store at `path`.
    pub fn open(path: PathBuf) -> Result<Self, StoreError> {
        let data = if path.exists() {
            let raw = fs::read_to_string(&path)?;
            if raw.trim().is_empty() {
                StoreFile::default()
            } else {
                serde_json::from_str(&raw)?
            }
        } else {
            StoreFile::default()
        };
        Ok(Self { path, data })
    }

    /// Return the most recent session whose `last_at` is within `minutes` minutes of now.
    pub fn resume_recent(&self, minutes: u32) -> Option<&AiSession> {
        let now = Utc::now();
        self.data
            .sessions
            .iter()
            .filter(|s| s.is_recent(minutes, now))
            .max_by_key(|s| s.last_at)
    }

    /// Create a new session with a fresh id, persist it, and return a clone.
    pub fn create(&mut self, shell: impl Into<String>) -> Result<AiSession, StoreError> {
        self.data.next_id += 1;
        let sess = AiSession::new(self.data.next_id, shell);
        self.data.sessions.push(sess.clone());
        self.flush()?;
        Ok(sess)
    }

    /// Upsert an existing session in-place (by id) and persist.
    pub fn upsert(&mut self, session: &AiSession) -> Result<(), StoreError> {
        if let Some(existing) = self.data.sessions.iter_mut().find(|s| s.id == session.id) {
            *existing = session.clone();
        } else {
            self.data.sessions.push(session.clone());
            if session.id > self.data.next_id {
                self.data.next_id = session.id;
            }
        }
        self.flush()
    }

    /// Get a session by id.
    pub fn get(&self, id: u64) -> Option<&AiSession> {
        self.data.sessions.iter().find(|s| s.id == id)
    }

    fn flush(&self) -> Result<(), StoreError> {
        ensure_parent(&self.path)?;
        let tmp = self.path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(&self.data)?;
        fs::write(&tmp, text)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

fn ensure_parent(p: &Path) -> std::io::Result<()> {
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::session::Turn;
    use chrono::Duration;
    use tempfile::tempdir;

    #[test]
    fn create_persist_reload() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ai_sessions.json");

        let mut store = SessionStore::open(path.clone()).unwrap();
        let mut s = store.create("bash").unwrap();
        s.push(Turn::user("list files"));
        s.push(Turn::assistant("ls -la", "list all"));
        store.upsert(&s).unwrap();

        let store2 = SessionStore::open(path).unwrap();
        let reloaded = store2.get(s.id).unwrap();
        assert_eq!(reloaded.turns.len(), 2);
        assert_eq!(reloaded.last_command(), Some("ls -la"));
    }

    #[test]
    fn resume_recent_respects_ttl() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("s.json");
        let mut store = SessionStore::open(path).unwrap();

        // Stale session (pretend it was last touched two hours ago).
        let mut old = store.create("bash").unwrap();
        old.last_at = Utc::now() - Duration::minutes(120);
        store.upsert(&old).unwrap();

        // Resuming at 60-minute TTL skips the stale session.
        assert!(store.resume_recent(60).is_none());

        // Fresh session → resumable.
        let fresh = store.create("bash").unwrap();
        let resumed = store.resume_recent(60).unwrap();
        assert_eq!(resumed.id, fresh.id);
    }

    #[test]
    fn open_handles_empty_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.json");
        std::fs::write(&path, "").unwrap();
        let store = SessionStore::open(path).unwrap();
        assert!(store.resume_recent(60).is_none());
    }
}
