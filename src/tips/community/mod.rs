//! Community features for cheatsheet contribution and sharing
//!
//! Enables users to contribute their aliases and tips to the community
//! knowledge base with proper validation and attribution.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Community Module                           │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌───────────────┐  ┌─────────────────┐  │
//! │  │   Schema     │  │   Exporter    │  │   Contributor   │  │
//! │  │  Validator   │  │  Local->YAML  │  │   Attribution   │  │
//! │  └──────────────┘  └───────────────┘  └─────────────────┘  │
//! │         │                 │                   │             │
//! │         └─────────────────┼───────────────────┘             │
//! │                           ▼                                  │
//! │                   ┌───────────────┐                          │
//! │                   │   Submission  │                          │
//! │                   │   Formatter   │                          │
//! │                   └───────────────┘                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use caro::tips::community::{CheatsheetExporter, ExportOptions, SchemaValidator};
//! use caro::tips::shell::TipsShellType;
//!
//! // Export local aliases to cheatsheet format
//! let exporter = CheatsheetExporter::new(TipsShellType::Zsh);
//! if let Ok(yaml) = exporter.to_yaml(&ExportOptions::default()) {
//!     // Validate before submission
//!     let validator = SchemaValidator::new();
//!     let result = validator.validate_yaml(&yaml);
//!     println!("Valid: {}", result.is_valid());
//! }
//! ```

pub mod contributor;
pub mod exporter;
pub mod schema;
pub mod submission;

pub use contributor::{Contributor, ContributorAttribution};
pub use exporter::{CheatsheetExporter, ExportOptions};
pub use schema::{SchemaValidator, ValidationError, ValidationResult};
pub use submission::{Submission, SubmissionFormat, SubmissionStatus};
