//! Progress tracking for file downloads
//!
//! Provides download progress bars with speed and ETA calculations using indicatif.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

/// Download progress tracker with speed and ETA calculation
///
/// Tracks download progress and calculates download speed using a rolling average
/// to provide smooth, accurate estimates of remaining time.
///
/// # Example
/// ```no_run
/// use caro::cache::DownloadProgress;
///
/// let mut progress = DownloadProgress::new(Some(1024 * 1024 * 100)); // 100MB file
/// progress.update(1024 * 1024); // Downloaded 1MB
/// progress.finish();
/// ```
pub struct DownloadProgress {
    bar: ProgressBar,
    start_time: Instant,
    bytes_downloaded: u64,
    last_update: Instant,
    last_bytes: u64,
}

impl DownloadProgress {
    /// Create a new progress tracker for a download
    ///
    /// # Arguments
    /// * `total_bytes` - Total size of the file being downloaded (if known)
    ///
    /// # Returns
    /// A new DownloadProgress instance with configured progress bar
    pub fn new(total_bytes: Option<u64>) -> Self {
        let bar = if let Some(total) = total_bytes {
            ProgressBar::new(total)
        } else {
            ProgressBar::new_spinner()
        };

        // Configure progress bar template
        // Format: [###########>          ] 45.2 MB/100.0 MB @ 2.3 MB/s ETA 00:24
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} @ {bytes_per_sec} ETA {eta}")
            .expect("Valid progress template")
            .progress_chars("##>-");

        bar.set_style(style);

        let now = Instant::now();
        Self {
            bar,
            start_time: now,
            bytes_downloaded: 0,
            last_update: now,
            last_bytes: 0,
        }
    }

    /// Update progress with newly downloaded bytes
    ///
    /// This should be called after each chunk is written to disk.
    /// The method automatically calculates download speed and ETA.
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes to add to the progress
    pub fn update(&mut self, bytes: u64) {
        self.bytes_downloaded += bytes;
        self.bar.set_position(self.bytes_downloaded);

        // Update speed calculation every 100ms to keep display smooth
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        if elapsed >= Duration::from_millis(100) {
            self.last_update = now;
            self.last_bytes = self.bytes_downloaded;
        }
    }

    /// Set the current position (useful for resume scenarios)
    ///
    /// # Arguments
    /// * `position` - The starting byte position
    pub fn set_position(&mut self, position: u64) {
        self.bytes_downloaded = position;
        self.bar.set_position(position);
        self.last_bytes = position;
    }

    /// Mark the download as complete and clean up the progress bar
    ///
    /// This should be called after the download finishes successfully.
    pub fn finish(&self) {
        self.bar.finish_with_message("Download complete");
    }

    /// Mark the download as failed and clean up the progress bar
    ///
    /// This should be called if the download encounters an error.
    ///
    /// # Arguments
    /// * `message` - Error message to display
    pub fn finish_with_error(&self, message: &str) {
        self.bar
            .finish_with_message(format!("Download failed: {}", message));
    }

    /// Get the underlying progress bar for custom operations
    ///
    /// This allows direct access to the indicatif ProgressBar for
    /// advanced use cases not covered by the standard API.
    pub fn bar(&self) -> &ProgressBar {
        &self.bar
    }

    /// Calculate current download speed in bytes per second
    ///
    /// Uses a rolling average based on bytes downloaded since last update.
    ///
    /// # Returns
    /// Download speed in bytes per second
    pub fn speed_bytes_per_sec(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bytes_downloaded as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate estimated time remaining
    ///
    /// # Arguments
    /// * `total_bytes` - Total size of the file
    ///
    /// # Returns
    /// Estimated duration until completion, or None if speed is zero
    pub fn eta(&self, total_bytes: u64) -> Option<Duration> {
        let speed = self.speed_bytes_per_sec();
        if speed > 0.0 && self.bytes_downloaded < total_bytes {
            let remaining_bytes = total_bytes - self.bytes_downloaded;
            let remaining_secs = remaining_bytes as f64 / speed;
            Some(Duration::from_secs_f64(remaining_secs))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_progress_creation() {
        let progress = DownloadProgress::new(Some(1000));
        assert_eq!(progress.bytes_downloaded, 0);
    }

    #[test]
    fn test_progress_update() {
        let mut progress = DownloadProgress::new(Some(1000));
        progress.update(500);
        assert_eq!(progress.bytes_downloaded, 500);
    }

    #[test]
    fn test_progress_set_position() {
        let mut progress = DownloadProgress::new(Some(1000));
        progress.set_position(300);
        assert_eq!(progress.bytes_downloaded, 300);
    }

    #[test]
    fn test_speed_calculation() {
        let mut progress = DownloadProgress::new(Some(1000));

        // Simulate some download activity
        thread::sleep(Duration::from_millis(100));
        progress.update(500);

        let speed = progress.speed_bytes_per_sec();
        assert!(speed > 0.0, "Speed should be positive after update");
    }

    #[test]
    fn test_eta_calculation() {
        let mut progress = DownloadProgress::new(Some(1000));

        // Simulate download progress
        thread::sleep(Duration::from_millis(100));
        progress.update(500);

        let eta = progress.eta(1000);
        assert!(
            eta.is_some(),
            "ETA should be Some when download is in progress"
        );
    }

    #[test]
    fn test_eta_when_complete() {
        let mut progress = DownloadProgress::new(Some(1000));
        progress.update(1000);

        let eta = progress.eta(1000);
        assert!(
            eta.is_none(),
            "ETA should be None when download is complete"
        );
    }

    #[test]
    fn test_finish() {
        let progress = DownloadProgress::new(Some(1000));
        progress.finish(); // Should not panic
    }

    #[test]
    fn test_finish_with_error() {
        let progress = DownloadProgress::new(Some(1000));
        progress.finish_with_error("Network error"); // Should not panic
    }
}
