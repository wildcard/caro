//! Batch execution and parallel backend initialization.

use crate::dataset::TestCase;

/// Batch executor for grouping test cases
pub struct BatchExecutor {
    batch_size: usize,
}

impl BatchExecutor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    pub fn create_batches<'a>(&self, test_cases: &'a [TestCase]) -> Vec<&'a [TestCase]> {
        test_cases.chunks(self.batch_size).collect()
    }
}

/// Backend initializer for parallel pre-warming
pub struct BackendInitializer;

impl BackendInitializer {
    pub fn new() -> Self {
        Self
    }

    pub async fn prewarm_parallel(&self, _backends: &[String]) {
        // Simplified implementation for testing
        // In production, this would actually initialize backends in parallel
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

impl Default for BackendInitializer {
    fn default() -> Self {
        Self::new()
    }
}
