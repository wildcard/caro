# ADR-015: ChromaFs-Style Virtual Filesystem for Documentation Navigation

**Status**: Proposed

**Date**: 2026-04-06

**Authors**: Caro Maintainers

**Target**: Community

## Context

Caro has a mature ChromaDB integration (Phase 1-5 complete) with five specialized collections. The `caro_command_docs` collection (`CollectionType::Docs`) stores indexed documentation from three sources: man pages, tldr pages, and command help output. These are populated by `ManPageIndexer`, `TldrIndexer`, and `HelpIndexer` respectively.

Current documentation retrieval is exclusively vector-similarity-based via `find_similar_in(query, limit, QueryScope::Single(CollectionType::Docs))`. This has two limitations:

1. **No deterministic lookup**: When the agent loop knows a specific command name (e.g., user asks "how do I use find?"), it still performs a semantic similarity search rather than a direct fetch
2. **No structured navigation**: The agent cannot browse available documentation by source, section, or platform—it can only search by textual similarity

Mintlify's ChromaFs pattern demonstrates a proven production approach: map UNIX filesystem semantics (`ls`, `cat`, `grep`) to vector database queries via a virtual filesystem interface. At Mintlify's scale (30,000+ conversations/day), this replaced both traditional RAG (top-K chunk injection) and real sandbox containers ($70k+/year), achieving ~100ms session creation with equivalent answer quality.

See `docs/research/CHROMAFS_VIRTUAL_FILESYSTEM_ANALYSIS.md` for the full technical analysis.

## Decision

Adopt a lightweight ChromaFs-inspired virtual filesystem layer over the existing `caro_command_docs` collection. Implement a `DocFs` trait providing three operations:

- **`list_path(path)`** — directory listing (ls equivalent): query entries by metadata prefix, deduplicate to unique path components
- **`read_doc(path)`** — full document retrieval (cat equivalent): fetch all entries matching an exact `source_path`, concatenate in order
- **`search_docs(pattern, path_prefix)`** — combined metadata filter + regex search (grep equivalent): use `where_document` for coarse candidate generation, fine regex for verification

Store path metadata in Docs collection entries (`source_path`, `source_type` fields). Reconstruct the directory tree from metadata queries rather than storing a separate `__path_tree__` blob.

## Rationale

- **Builds on existing infrastructure**: No new collections, backends, or dependencies needed. DocFs wraps the existing `VectorBackend` trait
- **Enables hybrid retrieval**: Path-based deterministic lookup for known commands + vector similarity for fuzzy/conceptual queries
- **Lightweight adaptation**: Mintlify's full ChromaFs includes a bash interpreter, gzipped path tree blobs, and Redis caching. Caro needs only the filesystem abstraction layer since its scale (hundreds of indexed docs) doesn't require those optimizations
- **Proven pattern**: ChromaFs powers 30,000+ conversations/day at Mintlify with equivalent quality to real sandbox approaches
- **Natural fit for documentation**: Man pages, tldr pages, and help text have inherent hierarchical structure (source → section → command) that maps cleanly to filesystem paths

## Consequences

### Benefits

- Agent loop can deterministically fetch documentation for known command names in <10ms
- Structured navigation enables the agent to discover available documentation by browsing
- Foundation for future user-facing `caro docs` CLI subcommands
- Read-only by design—no mutation complexity, no state management

### Trade-offs

- Additional `source_path` and `source_type` metadata fields on every Docs entry (minimal storage overhead)
- Path conventions must be maintained consistently across all indexers
- Adds an abstraction layer between the agent loop and raw vector search

### Risks

- **Over-engineering for current CLI scope** → Mitigate by keeping DocFs trait minimal (3 methods only); defer CLI surface and agent integration to separate phases
- **Path tree reconstruction latency for large doc sets** → Mitigate with optional in-memory cache; caro's doc count (hundreds) is well within metadata query performance
- **Marginal quality improvement** → Mitigate by measuring agent loop quality with and without DocFs before committing to deep integration

## Alternatives Considered

### Alternative 1: Full ChromaFs Port

- Description: Port the complete approach including gzipped path-tree blob (`__path_tree__`), chunk reassembly with `chunk_index` ordering, and Redis-backed grep cache
- Pros: Proven at Mintlify's scale; handles millions of pages efficiently
- Cons: Over-engineered for caro's embedded CLI context; requires sentinel-key storage patterns; TypeScript-oriented design doesn't translate cleanly to Rust trait system

### Alternative 2: Status Quo (Pure Vector Search)

- Description: Continue using `find_similar_in(QueryScope::Single(CollectionType::Docs))` for all documentation retrieval
- Pros: No new code; already working
- Cons: No deterministic lookup by command name; no structured navigation; agent cannot browse available docs; potential quality gap when exact content is needed but similarity search returns adjacent entries

### Alternative 3: Real Filesystem Cache

- Description: Materialize all indexed docs to `~/.config/caro/docs/` as real files on disk
- Pros: Standard filesystem tools work natively; no abstraction layer needed
- Cons: Duplicates data (vector DB + filesystem); sync/staleness issues on doc updates; large disk footprint for complete man page sets; loses vector search capability for the materialized content

### Alternative 4: Bubblewrap Sandbox (ADR-010)

- Description: Use bubblewrap (bwrap) process isolation to give the agent real filesystem access to docs
- Pros: Full POSIX semantics; real `grep`, `find`, etc.
- Cons: 1-5s setup latency per invocation; process isolation overhead; overkill for read-only documentation access

## Implementation Notes

- **Phase 1**: Enrich indexer output with `source_path` metadata field (e.g., `/docs/man/1/ls`, `/docs/tldr/common/tar`, `/docs/help/cargo`)
- **Phase 2**: Implement `DocFs` trait and `VectorDocFs` struct in `src/knowledge/docfs.rs`
- **Phase 3**: Integrate `DocFs` into the agent loop as supplementary context source
- **Phase 4**: (Optional) Expose as `caro docs ls/cat/grep` CLI subcommands
- See `kitty-specs/045-chromafs-virtual-filesystem/spec.md` for the full implementation specification

## Success Metrics

- **Deterministic lookup latency**: `read_doc()` by path completes in <10ms for indexed commands
- **Path listing correctness**: `list_path()` returns accurate entries for all indexed source types
- **No regression**: Agent loop documentation retrieval latency does not increase vs current approach
- **Test coverage**: All DocFs operations covered by unit and integration tests
- **Quality signal**: Agent loop generates more accurate commands when DocFs provides exact doc content (measured via eval suite)

## Business Implications

- Differentiates caro's documentation-aware command generation from competitors relying on pure RAG
- Foundation for enterprise documentation integration features (custom doc sources, access-controlled namespaces)
- Demonstrates architectural maturity for the knowledge subsystem

## References

- `docs/research/CHROMAFS_VIRTUAL_FILESYSTEM_ANALYSIS.md` — Full technical analysis
- `docs/adr/ADR-010-bubblewrap-sandbox-execution.md` — Contrasting real sandbox approach
- `kitty-specs/028-issue-166-add/spec.md` — Original knowledge index specification
- `kitty-specs/045-chromafs-virtual-filesystem/spec.md` — Implementation specification
- `src/knowledge/backends/mod.rs` — `VectorBackend` trait (DocFs wraps this)
- `src/knowledge/collections.rs` — `CollectionType::Docs`, `QueryScope` definitions
- Mintlify ChromaFs technical deep dive (user-provided, April 2026)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-04-06 | Caro Maintainers | Initial draft |
