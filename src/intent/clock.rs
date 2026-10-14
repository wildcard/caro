//! The single environmental input to generation: one clock read.

use std::time::{SystemTime, UNIX_EPOCH};

/// Source of the reference instant `T0` (whole Unix seconds, UTC).
pub trait Clock: Send + Sync {
    fn now_unix_secs(&self) -> Result<i64, String>;
}

/// Production clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_secs(&self) -> Result<i64, String> {
        let d = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("system clock is before 1970: {e}"))?;
        i64::try_from(d.as_secs()).map_err(|_| "system clock out of range".to_string())
    }
}

/// Fixed clock for tests; counts reads so tests can prove "read once".
#[derive(Debug)]
pub struct FixedClock {
    pub secs: i64,
    reads: std::sync::atomic::AtomicUsize,
}

impl FixedClock {
    pub fn new(secs: i64) -> Self {
        Self {
            secs,
            reads: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn reads(&self) -> usize {
        self.reads.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Clock for FixedClock {
    fn now_unix_secs(&self) -> Result<i64, String> {
        self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self.secs)
    }
}

impl Clock for std::sync::Arc<FixedClock> {
    fn now_unix_secs(&self) -> Result<i64, String> {
        (**self).now_unix_secs()
    }
}
