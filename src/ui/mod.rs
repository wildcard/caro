//! Caro brand UI primitives — terminal mascot, risk badges, and a thinking
//! spinner. Together these deliver the brand at decision time inside the CLI.
//!
//! All components honor [`NO_COLOR`](https://no-color.org/) and degrade
//! gracefully when stdout is not a TTY. The spinner additionally suppresses
//! itself when verbose tracing is active (`RUST_LOG` is set, or the user
//! passed `--verbose`) so it does not fight log output on stderr.
//!
//! See [`palette`] for the brand RGB constants, [`kyaro`] for ASCII sprites,
//! [`risk_badge`] for SAFE/MODERATE/HIGH/CRITICAL formatting, and
//! [`spinner`] for the "Caro is thinking…" indicator.

pub mod kyaro;
pub mod palette;
pub mod risk_badge;
pub mod spinner;

pub use risk_badge::render as render_risk_badge;
pub use spinner::Spinner;
