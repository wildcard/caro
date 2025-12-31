//! Knowledge Base module for community tips and cheatsheets
//!
//! Provides storage, caching, and matching for community-contributed tips,
//! aliases, and plugin recommendations.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Knowledge Base Module                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌───────────┐  ┌───────────────────┐    │
//! │  │    Types     │  │   Cache   │  │     Matcher       │    │
//! │  │ KnowledgeBase│  │  Local    │  │  Pattern Match    │    │
//! │  │ KbTip, Alias │  │  Storage  │  │  Ranking          │    │
//! │  └──────────────┘  └───────────┘  └───────────────────┘    │
//! │         │                │                │                  │
//! │         └────────────────┼────────────────┘                  │
//! │                          ▼                                   │
//! │                  ┌───────────────┐                           │
//! │                  │   Processor   │                           │
//! │                  │   YAML → KB   │                           │
//! │                  └───────────────┘                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use caro::tips::kb::{KnowledgeBase, KbTip, KbAlias};
//!
//! let mut kb = KnowledgeBase::with_version("1.0.0");
//! kb.add_tip(KbTip::new("git-status", "git status", "Use `gst` instead!"));
//! kb.add_alias(KbAlias::new("gst", "git status"));
//! ```

pub mod cache;
pub mod matcher;
pub mod processor;
pub mod types;

pub use cache::{KbCache, KbCacheError};
pub use matcher::{KbMatcher, MatchResult};
pub use processor::{KbProcessor, ProcessorError};
pub use types::{
    Cheatsheet, CheatsheetAlias, CheatsheetPlugin, CheatsheetTip, KbAlias, KbPlugin, KbTip,
    KbTipCategory, KnowledgeBase,
};

/// Default KB cache directory name
pub const KB_CACHE_DIR: &str = "kb";

/// Default KB filename
pub const KB_FILENAME: &str = "caro-kb.msgpack";

/// KB checksum filename
pub const KB_CHECKSUM_FILENAME: &str = "caro-kb.sha256";
