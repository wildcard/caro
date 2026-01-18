//! Prompt versioning and management module.
//!
//! This module provides systematic prompt engineering capabilities including:
//! - Version control for prompts
//! - Template-based prompts with variable substitution
//! - Metadata tracking (author, date, model targets)

pub mod loader;
pub mod metadata;
pub mod registry;
