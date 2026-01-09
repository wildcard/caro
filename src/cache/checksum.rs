//! Streaming checksum validation for downloaded files
//!
//! Provides efficient SHA256 hashing during download without re-reading files.

use sha2::{Digest, Sha256};

/// Streaming SHA256 hasher for validating downloaded files
///
/// Computes checksums incrementally as data is received, avoiding the need
/// to re-read large files after download completes.
///
/// # Example
/// ```no_run
/// use caro::cache::StreamingHasher;
///
/// let mut hasher = StreamingHasher::new();
/// hasher.update(b"chunk 1");
/// hasher.update(b"chunk 2");
/// let checksum = hasher.finalize();
/// assert_eq!(checksum.len(), 64); // Hex-encoded SHA256
/// ```
pub struct StreamingHasher {
    hasher: Sha256,
}

impl StreamingHasher {
    /// Create a new streaming hasher
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Update the hash with a chunk of data
    ///
    /// Call this method for each chunk received during streaming download.
    ///
    /// # Arguments
    /// * `data` - Byte slice to add to the hash
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize the hash and return the hex-encoded checksum
    ///
    /// Returns a 64-character lowercase hex string representing the SHA256 hash.
    ///
    /// # Returns
    /// Hex-encoded SHA256 checksum (64 characters)
    pub fn finalize(self) -> String {
        format!("{:x}", self.hasher.finalize())
    }
}

impl Default for StreamingHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_hasher_empty() {
        let hasher = StreamingHasher::new();
        let checksum = hasher.finalize();

        // SHA256 of empty string
        assert_eq!(
            checksum,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_streaming_hasher_single_chunk() {
        let mut hasher = StreamingHasher::new();
        hasher.update(b"hello world");
        let checksum = hasher.finalize();

        // SHA256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_streaming_hasher_multiple_chunks() {
        let mut hasher = StreamingHasher::new();
        hasher.update(b"hello");
        hasher.update(b" ");
        hasher.update(b"world");
        let checksum = hasher.finalize();

        // Should match single chunk "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_streaming_hasher_large_data() {
        let mut hasher = StreamingHasher::new();
        let data = vec![b'A'; 1024 * 1024]; // 1MB of 'A'
        hasher.update(&data);
        let checksum = hasher.finalize();

        // Verify it produces a valid 64-character hex string
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_default_constructor() {
        let hasher = StreamingHasher::default();
        let checksum = hasher.finalize();

        // Should match empty string hash
        assert_eq!(
            checksum,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_checksum_mismatch_detection() {
        let mut hasher1 = StreamingHasher::new();
        hasher1.update(b"hello world");
        let checksum1 = hasher1.finalize();

        let mut hasher2 = StreamingHasher::new();
        hasher2.update(b"goodbye world");
        let checksum2 = hasher2.finalize();

        assert_ne!(checksum1, checksum2);
    }
}
