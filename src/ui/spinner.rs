//! "Caro is thinking…" spinner shown while the LLM generates a command.
//!
//! The spinner intentionally suppresses itself in three environments where
//! it would corrupt other output:
//!
//! * `NO_COLOR` is set — respects the cross-tool convention (see
//!   <https://no-color.org>).
//! * `RUST_LOG` is set — `tracing-subscriber` writes structured logs to
//!   stderr, and the spinner draw target would interleave with them.
//! * stdout is not a TTY — piping or redirecting must produce clean output.
//! * The caller passed `--verbose` — same reasoning as `RUST_LOG`.
//!
//! In every suppressed mode the constructor still emits a plain text
//! "caro is thinking…" line so users have feedback.
//!
//! The spinner finishes-and-clears on `Drop` so callers cannot accidentally
//! leave it running after a panic or early `?` return. Call [`Spinner::stop`]
//! explicitly to consume the spinner gracefully.

use std::io::IsTerminal;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::ui::palette::CARO_YELLOW_400;

const TICK_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const DEFAULT_MESSAGE: &str = "caro is thinking…";

/// A brand-styled spinner that tucks itself away cleanly.
pub struct Spinner {
    bar: Option<ProgressBar>,
}

impl Spinner {
    /// Start the spinner with the default "caro is thinking…" message.
    ///
    /// `verbose` should reflect the user's `--verbose` flag so the spinner
    /// can yield to log output when active.
    pub fn thinking(verbose: bool) -> Self {
        Self::with_message(DEFAULT_MESSAGE, verbose)
    }

    /// Start the spinner with a custom message.
    pub fn with_message(message: &str, verbose: bool) -> Self {
        if should_suppress(verbose) {
            // Print a one-shot status line so the user still sees feedback.
            eprintln!("  {message}");
            return Self { bar: None };
        }

        let bar = ProgressBar::new_spinner();
        bar.set_draw_target(ProgressDrawTarget::stdout());

        let (r, g, b) = CARO_YELLOW_400;
        let template = format!("  {{spinner:.color({r},{g},{b})}} {{msg}}");
        let style = ProgressStyle::with_template(&template)
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(TICK_FRAMES);

        bar.set_style(style);
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));

        Self { bar: Some(bar) }
    }

    /// Stop the spinner and clear its line.
    pub fn stop(mut self) {
        if let Some(bar) = self.bar.take() {
            bar.finish_and_clear();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if let Some(bar) = self.bar.take() {
            bar.finish_and_clear();
        }
    }
}

fn should_suppress(verbose: bool) -> bool {
    if verbose {
        return true;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return true;
    }
    if std::env::var_os("RUST_LOG").is_some() {
        return true;
    }
    !std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_does_not_panic() {
        // We construct the same format used at runtime; verify it parses.
        let (r, g, b) = CARO_YELLOW_400;
        let template = format!("  {{spinner:.color({r},{g},{b})}} {{msg}}");
        assert!(ProgressStyle::with_template(&template).is_ok());
    }

    #[test]
    fn drop_does_not_panic_when_inactive() {
        // When suppressed, Spinner has no underlying ProgressBar; Drop must
        // still be a no-op.
        let s = Spinner { bar: None };
        drop(s);
    }
}
