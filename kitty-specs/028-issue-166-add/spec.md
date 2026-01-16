# Issue #166: Add Local Knowledge Index for Machine Context

## Summary

Add a local vector database to index machine context (command history, directory patterns, successful commands) for improved command generation through semantic similarity search.

## Problem

Currently, Caro generates commands without awareness of:
- User's past successful commands
- Patterns specific to this machine
- Previously learned corrections from the agentic loop
- Project-specific command patterns

This means the LLM often suggests commands that the user has previously corrected or doesn't match their environment.

## Solution

Integrate **LanceDB** (Rust-native embedded vector database) with **FastEmbed** (Rust embedding library) to:

1. **Index successful commands** - Store commands that the user accepted/executed
2. **Index corrections** - Learn from agentic loop refinements
3. **Semantic search** - Find similar past commands for context
4. **Enhance prompts** - Include relevant past examples in command generation

## Technical Approach

### Why LanceDB (not ChromaDB)

| Feature | LanceDB | ChromaDB |
|---------|---------|----------|
| Language | Pure Rust | Python (Rust core) |
| Deployment | Embedded, no server | Requires Python runtime |
| CLI Integration | Native | FFI/subprocess needed |
| Binary Size | Minimal | Large Python deps |

LanceDB is the better choice for a Rust CLI tool.

### Dependencies

```toml
[dependencies]
lancedb = "0.23"           # Embedded vector database
fastembed = "4"            # Embedding generation (ONNX-based)
```

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Caro CLI                            │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │  KnowledgeIndex │    │  AgentLoop                  │ │
│  │  ├─ embedder    │◄──►│  ├─ generator               │ │
│  │  ├─ db          │    │  ├─ knowledge (new)         │ │
│  │  └─ cache       │    │  └─ safety                  │ │
│  └─────────────────┘    └─────────────────────────────┘ │
│           │                                             │
│           ▼                                             │
│  ┌─────────────────────────────────────────────────────┐│
│  │  ~/.config/caro/knowledge/                          ││
│  │  ├─ vectors.lance     (LanceDB data)                ││
│  │  └─ embeddings.onnx   (cached model)                ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

## Implementation

### New Module: `src/knowledge/mod.rs`

```rust
pub struct KnowledgeIndex {
    db: lancedb::Database,
    embedder: fastembed::TextEmbedding,
}

impl KnowledgeIndex {
    /// Open or create knowledge index
    pub async fn open(path: &Path) -> Result<Self>;

    /// Index a successful command execution
    pub async fn record_success(
        &self,
        request: &str,      // Natural language request
        command: &str,      // Generated command
        context: &str,      // Directory/project context
    ) -> Result<()>;

    /// Index a correction/refinement
    pub async fn record_correction(
        &self,
        original: &str,     // What was first generated
        corrected: &str,    // What user accepted
        feedback: &str,     // Why it was wrong
    ) -> Result<()>;

    /// Find similar past commands
    pub async fn find_similar(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<KnowledgeEntry>>;
}

pub struct KnowledgeEntry {
    pub request: String,
    pub command: String,
    pub context: String,
    pub similarity: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### Integration Point: `AgentLoop`

In `src/agent/mod.rs`, add knowledge context to command generation:

```rust
impl AgentLoop {
    pub async fn generate_initial(&self) -> Result<GeneratedCommand> {
        // Query knowledge index for similar past commands
        let similar = self.knowledge.find_similar(&self.request, 3).await?;

        // Include in prompt if relevant
        let knowledge_context = if !similar.is_empty() {
            format!("\n\nRelevant past commands:\n{}",
                similar.iter()
                    .map(|e| format!("- \"{}\": `{}`", e.request, e.command))
                    .collect::<Vec<_>>()
                    .join("\n"))
        } else {
            String::new()
        };

        // Add to existing context
        // ...
    }

    pub async fn execute(&self, command: &GeneratedCommand) -> Result<()> {
        // After successful execution, record to knowledge index
        self.knowledge.record_success(
            &self.request,
            &command.command,
            &self.directory_context.to_context_string(),
        ).await?;
    }
}
```

### Data Storage Schema

LanceDB table schema (using Arrow):

| Column | Type | Description |
|--------|------|-------------|
| id | String | UUID |
| request | String | Natural language request |
| command | String | Shell command |
| context | String | Directory/project context |
| embedding | FixedSizeList<Float32, 384> | Vector embedding |
| entry_type | String | "success" or "correction" |
| timestamp | Timestamp | When recorded |
| original_command | String? | For corrections only |
| feedback | String? | For corrections only |

### Embedding Model

Use `sentence-transformers/all-MiniLM-L6-v2`:
- Small: ~80MB
- Fast: <10ms per embedding
- Dimension: 384
- Good for short text (commands)

The model is downloaded on first use to `~/.config/caro/models/`.

## Feature Flags

```toml
[features]
default = ["knowledge"]
knowledge = ["lancedb", "fastembed"]
```

Users can opt out if they don't want the extra dependencies:
```bash
cargo install caro --no-default-features
```

## Privacy Considerations

1. **Local only** - All data stays on user's machine
2. **No cloud** - No telemetry or upload of command history
3. **Opt-in recording** - Only records when commands are executed
4. **Clear command** - `caro knowledge clear` to delete all data

## CLI Commands

```bash
# View knowledge stats
caro knowledge stats

# Search knowledge index
caro knowledge search "find files"

# Clear all knowledge
caro knowledge clear

# Export knowledge (for backup/sharing)
caro knowledge export knowledge.json
```

## Success Criteria

1. Knowledge index opens/creates in <100ms
2. Embedding generation <50ms per query
3. Similarity search <20ms for 10k entries
4. Command generation improved for repeated patterns
5. All existing tests continue to pass
6. New tests for knowledge module

## Files to Modify

- `Cargo.toml` - Add lancedb, fastembed dependencies
- `src/knowledge/mod.rs` - New knowledge index module
- `src/knowledge/schema.rs` - Arrow schema definitions
- `src/knowledge/embedder.rs` - Embedding wrapper
- `src/agent/mod.rs` - Integrate knowledge context
- `src/cli/mod.rs` - Add knowledge subcommands
- `src/lib.rs` - Export knowledge types

## Test Cases

1. Create new knowledge index in temp directory
2. Record command success and query back
3. Record correction and verify storage
4. Similarity search returns relevant results
5. Empty index returns empty results
6. Large index (10k entries) performs acceptably
7. Concurrent access is safe
8. Index survives process restart

## Open Questions

1. **Embedding model choice** - MiniLM-L6 vs BGE-small?
2. **Cache policy** - How long to keep old entries?
3. **Similarity threshold** - What score is "relevant"?
4. **Max context entries** - How many past examples to include?

## References

- [LanceDB Rust docs](https://docs.rs/lancedb)
- [FastEmbed Rust docs](https://docs.rs/fastembed)
- [Sentence Transformers](https://www.sbert.net/)
