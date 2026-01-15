//! TLDR integration for Caro CLI.
//!
//! This module provides a TLDR client that leverages existing system-installed
//! TLDR clients (tealdeer, tlrc) when available, with a fallback to our minimal
//! implementation. This follows KISS/DRY principles by reusing maintained tools.
//!
//! # Overview
//!
//! The TLDR integration serves multiple purposes in Caro:
//!
//! 1. **Context enrichment**: TLDR pages provide concise command documentation
//!    that can be included in LLM prompts for better command generation.
//!
//! 2. **Explanation enhancement**: When explaining generated commands, TLDR
//!    examples provide additional context and common use cases.
//!
//! 3. **Command validation**: TLDR examples can help validate that generated
//!    commands follow common patterns and best practices.
//!
//! 4. **RAG integration**: TLDR pages can be indexed into a vector database
//!    for semantic search and retrieval during inference.
//!
//! # CLI-First Approach
//!
//! The client uses a CLI-first strategy:
//!
//! 1. **Primary**: If `tldr` CLI is installed (tealdeer, tlrc, etc.), use it
//! 2. **Fallback**: Use our minimal cache implementation when no CLI is available
//!
//! This minimizes maintenance burden since TLDR format changes are handled
//! by the upstream CLI projects.
//!
//! # Example
//!
//! ```no_run
//! use caro::tldr::{TldrClient, TldrClientBuilder};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client (auto-detects CLI if available)
//! let client = TldrClientBuilder::new()
//!     .language("en")
//!     .prefer_cli(true)  // Use system tldr if installed
//!     .build()?;
//!
//! // Check what source is being used
//! println!("Using source: {:?}", client.source());
//!
//! // Get a TLDR page for a command
//! let page = client.get_page("git").await?;
//! println!("Command: {}", page.name);
//! println!("Description: {}", page.description);
//!
//! for example in &page.examples {
//!     println!("- {}: {}", example.description, example.command);
//! }
//!
//! // Get context for multiple commands (for LLM prompts)
//! let context = client
//!     .get_context_for_commands(&["git".to_string(), "curl".to_string()])
//!     .await;
//! println!("{}", context);
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The module is organized into the following components:
//!
//! - [`types`]: Core data types for TLDR pages, examples, and platforms
//! - [`parser`]: Markdown parser for TLDR page format
//! - [`cache`]: Cache management for downloading and storing pages (fallback)
//! - [`client`]: High-level async client API with CLI detection
//!
//! # Recommended System TLDR Clients
//!
//! For best results, install one of these TLDR clients:
//!
//! - **tealdeer**: `cargo install tealdeer` or `brew install tealdeer`
//! - **tlrc**: `cargo install tlrc` or system package manager
//!
//! # Feature Flag
//!
//! The TLDR module is enabled by default. Remote cache updates (fallback mode)
//! require the `remote-backends` feature for HTTP support.

pub mod cache;
pub mod client;
pub mod parser;
pub mod types;

// Re-export main types for convenience
pub use cache::{CacheError, CacheInfo, TldrCache};
pub use client::{TldrClient, TldrClientBuilder, TldrConfig, TldrSource};
pub use parser::{ParseError, TldrParser};
pub use types::{Placeholder, Platform, TldrExample, TldrPage};

/// Provide TLDR context for a set of commands.
///
/// This is a convenience function that creates a temporary client
/// and fetches context for the given commands.
///
/// # Arguments
/// * `commands` - List of command names to fetch TLDR pages for
///
/// # Returns
/// A formatted string containing TLDR documentation for the commands,
/// suitable for inclusion in LLM prompts.
pub async fn get_tldr_context(commands: &[String]) -> String {
    match TldrClient::new() {
        Ok(client) => client.get_context_for_commands(commands).await,
        Err(e) => {
            tracing::warn!("Failed to create TLDR client: {}", e);
            String::new()
        }
    }
}

/// Check if TLDR pages are available in the cache.
pub async fn is_cache_available() -> bool {
    match TldrClient::new() {
        Ok(client) => client.is_cache_valid().await,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_cache(temp_dir: &TempDir) {
        let pages_dir = temp_dir.path().join("pages").join("common");
        std::fs::create_dir_all(&pages_dir).unwrap();

        std::fs::write(
            pages_dir.join("test.md"),
            "# test\n\n> A test command.\n\n- Run test:\n\n`test {{arg}}`\n",
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_module_integration() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_cache(&temp_dir);

        let client = TldrClientBuilder::new()
            .cache_dir(temp_dir.path())
            .auto_update(false)
            .build()
            .unwrap();

        // Test full workflow
        let page = client.get_page("test").await.unwrap();
        assert_eq!(page.name, "test");

        let context = client
            .get_context_for_commands(&["test".to_string()])
            .await;
        assert!(context.contains("test"));
    }
}
