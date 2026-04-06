# ChromaFs-Style Virtual Filesystem for Documentation Navigation

## Summary

Add a virtual filesystem abstraction over caro's existing `caro_command_docs` collection, enabling path-based navigation (`list_path`, `read_doc`, `search_docs`) of indexed documentation alongside existing vector similarity search.

## Problem

Currently, caro's documentation retrieval is purely semantic:

- `find_similar_in(query, limit, QueryScope::Single(CollectionType::Docs))` returns the top-K most similar entries
- There is no way to deterministically fetch documentation for a known command name
- The agent loop cannot browse available documentation by source, section, or platform
- No structured hierarchy exists over the indexed content

This means when a user asks "show me the man page for find", the agent performs a fuzzy vector search instead of directly fetching `/docs/man/1/find`.

## Solution

Implement a `DocFs` trait that maps filesystem-like operations to metadata queries against the existing Docs collection. Enrich indexer output with path metadata so entries can be addressed by deterministic paths.

## Technical Approach

### Virtual Path Scheme

```
/docs/{source}/{section_or_platform}/{command_name}
```

| Source | Path Structure | Example |
|--------|---------------|---------|
| Man pages | `/docs/man/{section}/{command}` | `/docs/man/1/ls` |
| Tldr pages | `/docs/tldr/{platform}/{command}` | `/docs/tldr/common/tar` |
| Help output | `/docs/help/{command}` | `/docs/help/cargo` |
| GitHub docs | `/docs/github/{repo}/{path}` | `/docs/github/caro/README` |

### Architecture

```
┌──────────────────────────────────────────────────┐
│                   Agent Loop                      │
│                                                   │
│  ┌─────────────┐       ┌──────────────────────┐  │
│  │   DocFs     │──────>│   VectorBackend      │  │
│  │  .list()    │       │  (ChromaDB / LanceDB)│  │
│  │  .read()    │       │  caro_command_docs   │  │
│  │  .search()  │       └──────────────────────┘  │
│  └─────────────┘                                  │
│        │                                          │
│        │ Deterministic path lookup                │
│        │ + Vector similarity fallback             │
│        v                                          │
│  ┌─────────────────────────────────────────────┐  │
│  │  Command Generation Prompt                   │  │
│  │  (enriched with exact documentation)         │  │
│  └─────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

### Why Metadata-Based (Not Blob-Based)

Mintlify's ChromaFs stores a gzipped `__path_tree__` JSON blob as a sentinel key in the Chroma collection. This is optimal for their scale (millions of pages across thousands of documentation sites). For caro's scale (hundreds to low thousands of indexed docs), a simpler approach works:

| Aspect | Blob-Based (Mintlify) | Metadata-Based (Caro) |
|--------|----------------------|----------------------|
| Path tree storage | Gzipped JSON blob under sentinel key | Reconstructed from `source_path` metadata on indexed entries |
| Bootstrap cost | Single fetch + decompress | Metadata query (or cached) |
| Update strategy | Rebuild and re-store blob on index changes | Automatic—new entries carry their own path |
| Complexity | High (serialize/deserialize/cache management) | Low (metadata queries only) |
| Scale ceiling | Millions of pages | Thousands of pages (sufficient for CLI tool) |

### Dependencies

No new crate dependencies required. Uses existing:

- `chromadb` crate (metadata filtering via `where` expressions)
- `lancedb` crate (SQL-like WHERE clauses)
- Existing `Indexer` trait implementations
- Existing `VectorBackend` trait

## Implementation

### Phase 1: Path Metadata Enrichment

Update the three indexers to include `source_path` and `source_type` in entry metadata.

```rust
// In ManPageIndexer::index_one
metadata.insert("source_path".to_string(), format!("/docs/man/{}/{}", section, command));
metadata.insert("source_type".to_string(), "man".to_string());

// In TldrIndexer::index_one
metadata.insert("source_path".to_string(), format!("/docs/tldr/{}/{}", platform, command));
metadata.insert("source_type".to_string(), "tldr".to_string());

// In HelpIndexer::index_one
metadata.insert("source_path".to_string(), format!("/docs/help/{}", command));
metadata.insert("source_type".to_string(), "help".to_string());
```

**Migration**: Existing entries without `source_path` metadata are invisible to DocFs but continue to work with `find_similar_in()`. A re-index populates the new fields.

### Phase 2: DocFs Trait (`src/knowledge/docfs.rs`)

```rust
use anyhow::Result;
use async_trait::async_trait;

/// A single entry in a virtual documentation filesystem
#[derive(Debug, Clone)]
pub struct DocEntry {
    /// Full path (e.g., "/docs/man/1/ls")
    pub path: String,
    /// Whether this entry represents a directory (true) or a document (false)
    pub is_directory: bool,
    /// Number of children if this is a directory
    pub child_count: Option<usize>,
}

/// A search match within the documentation filesystem
#[derive(Debug, Clone)]
pub struct DocSearchResult {
    /// Path of the matching document
    pub path: String,
    /// The matched line or excerpt
    pub matched_text: String,
    /// Vector similarity score (0.0-1.0) if available
    pub similarity: Option<f32>,
}

/// Virtual filesystem interface for browsing indexed documentation
///
/// Maps filesystem-like operations to metadata queries against
/// the caro_command_docs collection in the VectorBackend.
#[async_trait]
pub trait DocFs: Send + Sync {
    /// List entries under a path prefix (ls equivalent)
    ///
    /// Returns immediate children of the given path. For "/docs/",
    /// returns ["man", "tldr", "help"]. For "/docs/man/",
    /// returns ["1", "5", "8"].
    async fn list_path(&self, path: &str) -> Result<Vec<DocEntry>>;

    /// Read the full document at a specific path (cat equivalent)
    ///
    /// Returns the concatenated content of all entries matching
    /// the exact source_path. Returns None if no document exists
    /// at that path.
    async fn read_doc(&self, path: &str) -> Result<Option<String>>;

    /// Search documents matching a pattern under a path prefix (grep equivalent)
    ///
    /// Uses a two-stage pipeline:
    /// 1. Coarse filter via vector DB where_document/metadata
    /// 2. Fine regex match on returned documents
    async fn search_docs(
        &self,
        pattern: &str,
        path_prefix: Option<&str>,
    ) -> Result<Vec<DocSearchResult>>;
}
```

### Phase 3: VectorDocFs Implementation

```rust
use std::sync::Arc;
use crate::knowledge::backends::VectorBackend;
use crate::knowledge::collections::{CollectionType, QueryScope};

pub struct VectorDocFs {
    backend: Arc<dyn VectorBackend>,
}

impl VectorDocFs {
    pub fn new(backend: Arc<dyn VectorBackend>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl DocFs for VectorDocFs {
    async fn list_path(&self, path: &str) -> Result<Vec<DocEntry>> {
        // 1. Query all entries in Docs collection with source_path
        //    starting with the given path prefix
        // 2. Extract the next path component after the prefix
        // 3. Deduplicate to unique components
        // 4. For each unique component, determine if it's a directory
        //    (has children) or a leaf document
        // 5. Return sorted DocEntry list
        todo!()
    }

    async fn read_doc(&self, path: &str) -> Result<Option<String>> {
        // 1. Query entries where source_path == path exactly
        // 2. If no results, return None
        // 3. Sort entries by chunk_index (if present) or insertion order
        // 4. Concatenate document text from all matching entries
        // 5. Return the full document string
        todo!()
    }

    async fn search_docs(
        &self,
        pattern: &str,
        path_prefix: Option<&str>,
    ) -> Result<Vec<DocSearchResult>> {
        // Stage 1 (coarse): Use find_similar_in with the pattern as query
        //   - Scope to CollectionType::Docs
        //   - If path_prefix is set, add metadata filter for source_path prefix
        // Stage 2 (fine): For each candidate result:
        //   - Apply regex/substring match against document text
        //   - Filter out false positives from vector similarity
        // Return matching results with path and matched text
        todo!()
    }
}
```

### Phase 4: Agent Loop Integration

Integrate `DocFs` as a supplementary context source in the agent loop:

1. When the agent loop identifies a specific command name in the user's query, use `read_doc()` for deterministic content
2. When exploring what documentation is available, use `list_path()` to discover indexed commands
3. Fall back to `find_similar_in(Documentation)` for fuzzy/conceptual queries where no specific command is identified
4. Priority: exact path match > metadata-filtered search > pure vector similarity

### Phase 5: CLI Surface (Optional / Future)

```bash
# List available documentation sources
caro docs ls /docs/

# List man page sections
caro docs ls /docs/man/

# Read a specific man page
caro docs cat /docs/man/1/ls

# Search across all documentation
caro docs grep "recursive" /docs/

# Search within a specific source
caro docs grep "install" /docs/tldr/
```

This phase is deferred and tracked separately.

## Files to Modify

| File | Change |
|------|--------|
| `src/knowledge/docfs.rs` | **NEW**: `DocFs` trait, `DocEntry`, `DocSearchResult`, `VectorDocFs` |
| `src/knowledge/mod.rs` | Add `pub mod docfs;` and re-exports |
| `src/knowledge/indexers/man.rs` | Add `source_path` and `source_type` metadata |
| `src/knowledge/indexers/tldr.rs` | Add `source_path` and `source_type` metadata |
| `src/knowledge/indexers/help.rs` | Add `source_path` and `source_type` metadata |
| `src/knowledge/indexers/github.rs` | Add `source_path` and `source_type` metadata |
| `src/knowledge/collections.rs` | Potentially add `DocFs`-aware query helpers |
| `src/agent/mod.rs` | Integrate DocFs as context source (Phase 4) |

## Feature Flags

No new feature flag required. `DocFs` uses the existing `chromadb` feature gate and works with LanceDB by default, since it depends on `VectorBackend` (which abstracts over both).

## Success Criteria

1. `list_path("/docs/")` returns top-level sources: `man`, `tldr`, `help`
2. `list_path("/docs/man/")` returns section directories: `1`, `5`, `8`, etc.
3. `read_doc("/docs/man/1/ls")` returns full ls man page content
4. `read_doc("/docs/nonexistent")` returns `None`
5. `search_docs("recursive", Some("/docs/"))` returns entries for grep, find, ls, etc.
6. `search_docs("install", None)` searches all documentation
7. All indexers produce correct `source_path` metadata after re-index
8. Path-based operations complete in <10ms for typical indexed doc sets
9. All existing tests continue to pass (backward compatible)
10. Re-indexing with path metadata is backward compatible (old entries still work via vector search)

## Test Cases

```rust
#[cfg(test)]
mod tests {
    // Phase 1: Metadata enrichment
    #[test]
    fn man_indexer_produces_source_path() {
        // Index a man page, verify source_path = "/docs/man/{section}/{cmd}"
    }

    #[test]
    fn tldr_indexer_produces_source_path() {
        // Index a tldr page, verify source_path = "/docs/tldr/{platform}/{cmd}"
    }

    #[test]
    fn help_indexer_produces_source_path() {
        // Index help output, verify source_path = "/docs/help/{cmd}"
    }

    // Phase 2-3: DocFs operations
    #[tokio::test]
    async fn list_path_root_returns_sources() {
        // list_path("/docs/") -> ["help", "man", "tldr"]
    }

    #[tokio::test]
    async fn list_path_man_returns_sections() {
        // list_path("/docs/man/") -> ["1", "5", "8"]
    }

    #[tokio::test]
    async fn read_doc_existing_returns_content() {
        // read_doc("/docs/man/1/ls") -> Some("ls - list directory contents...")
    }

    #[tokio::test]
    async fn read_doc_nonexistent_returns_none() {
        // read_doc("/docs/man/1/nonexistent") -> None
    }

    #[tokio::test]
    async fn search_docs_with_prefix_filters_correctly() {
        // search_docs("recursive", Some("/docs/man/")) only searches man pages
    }

    #[tokio::test]
    async fn search_docs_without_prefix_searches_all() {
        // search_docs("recursive", None) searches all doc sources
    }
}
```

## Open Questions

1. **Path tree caching**: Should `list_path()` results be cached in memory after first call? The metadata query is fast for caro's scale, but caching would eliminate repeated queries during a single agent loop invocation.

2. **Chunk ordering**: Current indexers do not emit a `chunk_index` metadata field. For `read_doc()` to correctly order multi-chunk documents, should we add `chunk_index` in Phase 1, or rely on insertion order?

3. **CLI surface timing**: Should `caro docs ls/cat/grep` be part of the initial implementation or deferred to a separate spec? Phase 5 is marked optional.

4. **Staleness handling**: When docs are re-indexed, old entries with outdated `source_path` metadata may linger. Should `DocFs` filter by index timestamp, or should re-indexing clear old entries first?

5. **Backend-specific filter syntax**: ChromaDB uses `$contains`/`$eq` in `where` expressions; LanceDB uses SQL-like WHERE clauses. The `VectorBackend` trait may need a `find_by_metadata()` method to abstract over this difference. Should this be added to `VectorBackend` or handled inside `VectorDocFs` with backend-specific logic?

## References

- `docs/research/CHROMAFS_VIRTUAL_FILESYSTEM_ANALYSIS.md` — Full technical analysis of ChromaFs pattern
- `docs/adr/ADR-015-chromafs-virtual-filesystem.md` — Architecture decision record
- `kitty-specs/028-issue-166-add/spec.md` — Original knowledge index specification (Issue #166)
- `src/knowledge/backends/mod.rs` — `VectorBackend` trait definition
- `src/knowledge/collections.rs` — `CollectionType::Docs`, `QueryScope` definitions
- `src/knowledge/backends/chromadb.rs` — ChromaDB backend implementation
- Mintlify ChromaFs technical deep dive (user-provided, April 2026)
