//! Suggestions module for generating tips
//!
//! Contains the alias suggester, display formatting, and tip types.

pub mod alias_suggester;
pub mod display;
pub mod types;

pub use alias_suggester::AliasSuggester;
pub use display::{print_tip, print_tip_box, DisplayStyle, TipDisplay};
pub use types::{SuggestionResult, Tip, TipAction, TipCategory};
