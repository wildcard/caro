//! Deterministic regular-file selection (`--backend select-files`, GNU/Linux).
//!
//! ```text
//! request bytes + Clock ──> RequestFacts ──verify──> VerifiedIntent ──compile──> argv ──render──> shell string
//! ```
//!
//! No model, network, file read or process spawn happens during generation.
//! The only environmental input is one clock read. The one process spawn
//! (`find --version`) happens once, when the backend is selected
//! ([`find_identity`]).
//!
//! Scope (M1): one root, regular files, optionally one exact name or one
//! extension, optionally one rolling modification window `(T0 - W, T0]`.
//! The command ends with `-print0` (NUL-delimited, machine-readable).

pub mod check;
pub mod clock;
pub mod compile;
pub mod facts;
pub mod find_identity;
pub mod generator;

pub use generator::SelectFilesGenerator;

/// Backend name used on the command line and in provenance.
pub const BACKEND_NAME: &str = "select-files";

/// First words of the first explanation line; the CLI uses it to show the
/// output-framing note even when the explanation itself is hidden.
pub const OUTPUT_FRAMING_LABEL: &str = "output-framing: nul-separated";
