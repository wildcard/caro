//! Progress tracking for command execution and inference
//!
//! Provides spinner and progress indication for long-running commands and inference.

use crate::tips::{format_tip_short, TipCollection};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Command execution progress tracker with spinner
///
/// Shows a spinner animation while a command is executing, providing visual
/// feedback to the user that work is happening.
///
/// # Example
/// ```no_run
/// use caro::execution::CommandProgress;
///
/// let progress = CommandProgress::new("ls -la");
/// progress.start();
/// // ... command executes ...
/// progress.finish_success(150);
/// ```
pub struct CommandProgress {
    bar: ProgressBar,
    command: String,
    start_time: Instant,
    running: Arc<AtomicBool>,
}

impl CommandProgress {
    /// Create a new progress tracker for a command
    ///
    /// # Arguments
    /// * `command` - The command being executed (for display)
    pub fn new(command: &str) -> Self {
        let bar = ProgressBar::new_spinner();

        // Configure spinner style with command context
        let style = ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("Valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

        bar.set_style(style);
        bar.enable_steady_tick(Duration::from_millis(80));

        Self {
            bar,
            command: truncate_command(command, 50),
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the progress spinner
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        self.bar.set_message(format!("Executing: {}", self.command));
    }

    /// Update the spinner message (useful for multi-step commands)
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Show elapsed time in the spinner
    pub fn tick_with_elapsed(&self) {
        let elapsed = self.start_time.elapsed();
        let seconds = elapsed.as_secs();
        let elapsed_str = if seconds < 60 {
            format!("{}s", seconds)
        } else {
            format!("{}m {}s", seconds / 60, seconds % 60)
        };
        self.bar
            .set_message(format!("Executing: {} ({})", self.command, elapsed_str));
    }

    /// Mark execution as successful and clean up
    ///
    /// # Arguments
    /// * `execution_time_ms` - How long the command took in milliseconds
    pub fn finish_success(&self, execution_time_ms: u64) {
        self.running.store(false, Ordering::SeqCst);
        let time_str = format_duration(execution_time_ms);
        self.bar
            .finish_with_message(format!("✓ Completed in {}", time_str));
    }

    /// Mark execution as failed and clean up
    ///
    /// # Arguments
    /// * `execution_time_ms` - How long the command took in milliseconds
    /// * `exit_code` - The exit code of the failed command
    pub fn finish_error(&self, execution_time_ms: u64, exit_code: i32) {
        self.running.store(false, Ordering::SeqCst);
        let time_str = format_duration(execution_time_ms);
        self.bar.finish_with_message(format!(
            "✗ Failed (exit code {}) in {}",
            exit_code, time_str
        ));
    }

    /// Mark execution as failed with a custom error message
    pub fn finish_with_error(&self, message: &str) {
        self.running.store(false, Ordering::SeqCst);
        self.bar
            .finish_with_message(format!("✗ Error: {}", message));
    }

    /// Get the elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if the progress is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get the underlying progress bar for custom operations
    pub fn bar(&self) -> &ProgressBar {
        &self.bar
    }
}

/// Truncate a command for display, adding ellipsis if needed
fn truncate_command(command: &str, max_len: usize) -> String {
    // Normalize whitespace first
    let normalized: String = command.split_whitespace().collect::<Vec<_>>().join(" ");

    if normalized.len() <= max_len {
        normalized
    } else {
        format!("{}...", &normalized[..max_len - 3])
    }
}

/// Format duration in human-readable form
fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        let secs = ms as f64 / 1000.0;
        format!("{:.1}s", secs)
    } else {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

/// Inference progress tracker with spinner and tips
///
/// Shows a spinner animation while inference is running, with rotating tips
/// to help users get better results from caro.
///
/// # Example
/// ```no_run
/// use caro::execution::InferenceProgress;
///
/// let progress = InferenceProgress::new();
/// progress.start();
/// // ... inference runs ...
/// progress.finish();
/// ```
pub struct InferenceProgress {
    bar: ProgressBar,
    start_time: Instant,
    tips: TipCollection,
    running: Arc<AtomicBool>,
    show_tips: bool,
}

impl InferenceProgress {
    /// Create a new inference progress tracker
    pub fn new() -> Self {
        Self::with_tips(true)
    }

    /// Create inference progress with or without tips
    pub fn with_tips(show_tips: bool) -> Self {
        let bar = ProgressBar::new_spinner();

        // Configure spinner style
        let style = ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("Valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

        bar.set_style(style);
        bar.enable_steady_tick(Duration::from_millis(80));

        Self {
            bar,
            start_time: Instant::now(),
            tips: TipCollection::new(),
            running: Arc::new(AtomicBool::new(false)),
            show_tips,
        }
    }

    /// Start the inference progress spinner
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        self.bar.set_message("Generating command...");
    }

    /// Start with a custom message
    pub fn start_with_message(&self, message: &str) {
        self.running.store(true, Ordering::SeqCst);
        self.bar.set_message(message.to_string());
    }

    /// Update the message to show a tip (call this periodically for long inference)
    pub fn show_tip(&self) {
        if self.show_tips {
            let tip = self.tips.random();
            let tip_text = format_tip_short(tip);
            self.bar
                .set_message(format!("Generating... Tip: {}", tip_text));
        }
    }

    /// Update with elapsed time
    pub fn tick_with_elapsed(&self) {
        let elapsed = self.start_time.elapsed();
        let seconds = elapsed.as_secs();
        let elapsed_str = if seconds < 60 {
            format!("{}s", seconds)
        } else {
            format!("{}m {}s", seconds / 60, seconds % 60)
        };
        self.bar
            .set_message(format!("Generating command... ({})", elapsed_str));
    }

    /// Finish the progress with success
    pub fn finish(&self) {
        self.running.store(false, Ordering::SeqCst);
        let elapsed = self.start_time.elapsed();
        let time_str = format_duration(elapsed.as_millis() as u64);
        self.bar
            .finish_with_message(format!("✓ Generated in {}", time_str));
    }

    /// Finish with an error message
    pub fn finish_with_error(&self, message: &str) {
        self.running.store(false, Ordering::SeqCst);
        self.bar
            .finish_with_message(format!("✗ Error: {}", message));
    }

    /// Get the elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if the progress is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get a random tip to show after generation completes
    pub fn get_tip(&self) -> String {
        format_tip_short(self.tips.random())
    }

    /// Get the underlying progress bar
    pub fn bar(&self) -> &ProgressBar {
        &self.bar
    }
}

impl Default for InferenceProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_progress_creation() {
        let progress = CommandProgress::new("ls -la");
        assert!(!progress.is_running());
    }

    #[test]
    fn test_command_progress_start() {
        let progress = CommandProgress::new("echo 'hello'");
        progress.start();
        assert!(progress.is_running());
    }

    #[test]
    fn test_command_progress_finish_success() {
        let progress = CommandProgress::new("echo 'test'");
        progress.start();
        progress.finish_success(100);
        assert!(!progress.is_running());
    }

    #[test]
    fn test_command_progress_finish_error() {
        let progress = CommandProgress::new("exit 1");
        progress.start();
        progress.finish_error(50, 1);
        assert!(!progress.is_running());
    }

    #[test]
    fn test_truncate_command_short() {
        assert_eq!(truncate_command("ls -la", 50), "ls -la");
    }

    #[test]
    fn test_truncate_command_long() {
        let long_cmd = "find . -name '*.rs' -type f -exec grep -l 'pattern' {} \\;";
        let truncated = truncate_command(long_cmd, 30);
        assert!(truncated.len() <= 30);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(999), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1000), "1.0s");
        assert_eq!(format_duration(2500), "2.5s");
        assert_eq!(format_duration(59999), "60.0s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60000), "1m 0s");
        assert_eq!(format_duration(125000), "2m 5s");
    }

    #[test]
    fn test_elapsed_time() {
        let progress = CommandProgress::new("sleep 0.1");
        progress.start();
        std::thread::sleep(Duration::from_millis(100));
        assert!(progress.elapsed() >= Duration::from_millis(100));
    }

    // InferenceProgress tests
    #[test]
    fn test_inference_progress_creation() {
        let progress = InferenceProgress::new();
        assert!(!progress.is_running());
    }

    #[test]
    fn test_inference_progress_start() {
        let progress = InferenceProgress::new();
        progress.start();
        assert!(progress.is_running());
    }

    #[test]
    fn test_inference_progress_finish() {
        let progress = InferenceProgress::new();
        progress.start();
        progress.finish();
        assert!(!progress.is_running());
    }

    #[test]
    fn test_inference_progress_finish_with_error() {
        let progress = InferenceProgress::new();
        progress.start();
        progress.finish_with_error("test error");
        assert!(!progress.is_running());
    }

    #[test]
    fn test_inference_progress_with_tips_disabled() {
        let progress = InferenceProgress::with_tips(false);
        progress.start();
        progress.show_tip(); // Should not panic even with tips disabled
        assert!(progress.is_running());
    }

    #[test]
    fn test_inference_progress_get_tip() {
        let progress = InferenceProgress::new();
        let tip = progress.get_tip();
        assert!(!tip.is_empty());
    }

    #[test]
    fn test_inference_progress_default() {
        let progress = InferenceProgress::default();
        assert!(!progress.is_running());
    }
}
