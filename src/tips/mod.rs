//! Tips module for intelligent command suggestions
//!
//! Provides contextual "Did you know?" tips and alias suggestions based on
//! shell configuration, installed plugins, and community knowledge base.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    Tips Module                       │
//! ├─────────────────────────────────────────────────────┤
//! │  ┌──────────┐  ┌─────────────┐  ┌──────────────┐   │
//! │  │  Engine  │──│ Suggestions │──│    Config    │   │
//! │  └──────────┘  └─────────────┘  └──────────────┘   │
//! │       │                                             │
//! │       ▼                                             │
//! │  ┌─────────────────────────────────────────────┐   │
//! │  │           Shell Intelligence                 │   │
//! │  │  ┌──────────┐ ┌──────────┐ ┌────────────┐   │   │
//! │  │  │ Detector │ │  Alias   │ │  Plugin    │   │   │
//! │  │  │          │ │  Parser  │ │  Detector  │   │   │
//! │  │  └──────────┘ └──────────┘ └────────────┘   │   │
//! │  └─────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! # Modules
//!
//! - [`shell`] - Shell detection, alias parsing, plugin detection
//! - [`suggestions`] - Tip generation and display
//! - [`config`] - Tips configuration
//! - [`engine`] - Main tips engine orchestrator
//!
//! # Example
//!
//! ```no_run
//! use caro::tips::{TipsEngine, TipsConfig};
//!
//! // Create a tips engine with default config
//! let mut engine = TipsEngine::new();
//!
//! // Suggest a tip for a command
//! if let caro::tips::SuggestionResult::Found(tip) = engine.suggest("git status") {
//!     engine.display(&tip);
//! }
//! ```

pub mod config;
pub mod engine;
pub mod kb;
pub mod shell;
pub mod suggestions;

// Re-export shell types
pub use shell::{Alias, AliasParser, AliasSource, PluginDetector, PluginManager, ShellIntelligence, TipsShellType};

// Re-export knowledge base types
pub use kb::{
    Cheatsheet, KbAlias, KbCache, KbCacheError, KbMatcher, KbPlugin, KbProcessor, KbTip,
    KbTipCategory, KnowledgeBase, MatchResult, ProcessorError,
};

// Re-export config types
pub use config::{TipCategories, TipFrequency, TipsConfig};

// Re-export engine types
pub use engine::{SessionStats, TipsEngine, TipsSession};

// Re-export suggestion types
pub use suggestions::{
    AliasSuggester, DisplayStyle, SuggestionResult, Tip, TipAction, TipCategory, TipDisplay,
    print_tip, print_tip_box,
};
