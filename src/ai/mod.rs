//! Interactive AI feature — Atuin-AI-inspired conversational command generation.
//!
//! Public surface:
//! - [`privacy::build_context`] — construct the opt-in context string sent to the LLM.
//! - [`session`] — in-memory session + turn types.
//! - [`store`] — append/resume session persistence on disk (JSON file).
//! - [`shell_init`] — bash/zsh/fish integration scripts with a `?` keybinding.
//! - [`run_once`] — single-turn AI command generation honoring sessions and privacy.
//!
//! The feature flows every generated command through the existing
//! [`crate::safety::SafetyValidator`] — there is no safety bypass.

pub mod privacy;
pub mod session;
pub mod shell_init;
pub mod store;

pub mod runner;

pub use runner::{build_validator, run_once, AiInvocation, AiOutcome};
