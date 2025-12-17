# ChromaDB Integration Design

## Overview

This document describes the architecture for integrating ChromaDB as a vector database to store and query system command documentation (man pages) for enhanced RAG-based command generation.

## Architecture Components

### 1. Indexing Module (`src/indexing/`)

**Purpose**: Extract and prepare man page content for vectorization

**Key Files**:
- `mod.rs` - Module interface and public API
- `man_parser.rs` - Parse man pages into structured sections
- `command_scanner.rs` - Discover installed commands
- `batch_indexer.rs` - Batch indexing for CI/CD

**Responsibilities**:
- Scan system for installed commands using `which` and `$PATH`
- Parse man pages using `man -P cat <command>` or direct file reading
- Extract structured sections: NAME, SYNOPSIS, DESCRIPTION, OPTIONS, EXAMPLES
- Collect metadata: command version, installation path, OS/distro
- Handle edge cases: commands without man pages, compressed man pages, multi-section pages

### 2. Vector Store Module (`src/vector_store/`)

**Purpose**: ChromaDB client wrapper with cmdai-specific abstractions

**Key Files**:
- `mod.rs` - Public API and client initialization
- `client.rs` - ChromaDB HTTP client wrapper
- `embeddings.rs` - Embedding generation using sentence transformers
- `query.rs` - RAG query interface

**Responsibilities**:
- Initialize ChromaDB server (embedded or remote)
- Manage collections per OS/distribution combination
- Generate embeddings using `all-MiniLM-L6-v2` model (22MB, 384 dimensions)
- Query interface: `query_commands(user_intent: &str, k: usize) -> Vec<CommandDoc>`
- Graceful degradation when ChromaDB unavailable

### 3. Distribution Detection (enhance `src/context/mod.rs`)

**Purpose**: Detect OS/distribution and determine appropriate index

**Enhancements to ExecutionContext**:
```rust
pub struct ExecutionContext {
    // ... existing fields ...

    /// Distribution identifier for index matching (e.g., "ubuntu-22.04", "macos-14")
    pub distro_id: String,

    /// Whether network is available for downloading indexes
    pub network_available: bool,

    /// Path to ChromaDB index directory
    pub chroma_db_path: PathBuf,
}
```

**Distribution ID Format**:
- Linux: `{distro}-{major_version}` (e.g., `ubuntu-22.04`, `debian-12`, `fedora-38`)
- macOS: `macos-{major_version}` (e.g., `macos-14`, `macos-13`)
- Windows: `windows-{version}` (e.g., `windows-11`, `windows-10`)

### 4. CLI Enhancements (`src/cli/mod.rs`)

**New Subcommands**:
```bash
# Initialize ChromaDB with current system commands
cmdai index init [--force] [--verbose]

# Refresh index with newly installed commands
cmdai index refresh [--incremental]

# Show index statistics and health
cmdai index status [--detailed]

# Download pre-built index for specific distribution
cmdai index download <distro-id> [--verify]

# Export index for air-gapped transfer
cmdai index export <output-path>

# Import pre-built index
cmdai index import <index-path> [--validate]

# Clear all indexes
cmdai index clear [--confirm]
```

### 5. RAG Integration (enhance `src/agent/mod.rs`)

**AgentLoop Enhancements**:
```rust
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    context: ExecutionContext,
    vector_store: Option<Arc<VectorStore>>,  // NEW: Optional ChromaDB
    max_iterations: usize,
    timeout: Duration,
}

impl AgentLoop {
    /// Query ChromaDB for relevant command documentation
    async fn get_command_context_from_db(
        &self,
        user_intent: &str,
        commands: &[String]
    ) -> HashMap<String, CommandDoc>;

    /// Fallback to --help when ChromaDB unavailable
    async fn get_command_context_fallback(
        &self,
        commands: &[String]
    ) -> HashMap<String, CommandInfo>;
}
```

**Prompt Enhancement**:
```
COMMAND DOCUMENTATION (from vector database):
- Command: grep
  Version: grep 3.7
  Description: File pattern searcher

  Relevant Options:
  -i, --ignore-case    Ignore case distinctions
  -r, --recursive      Read all files under each directory
  -n, --line-number    Prefix each line with line number

  Example Usage:
  grep -r "TODO" .     # Search for TODO in all files recursively
```

### 6. GitHub Actions (`.github/workflows/build-indexes.yml`)

**Matrix Strategy**:
```yaml
strategy:
  matrix:
    os-config:
      - os: ubuntu-22.04
        distro-id: ubuntu-22.04
      - os: ubuntu-20.04
        distro-id: ubuntu-20.04
      - os: macos-14
        distro-id: macos-14
      - os: macos-13
        distro-id: macos-13
```

**Workflow Steps**:
1. Set up environment for each OS
2. Install cmdai (release build)
3. Run `cmdai index init --batch-mode`
4. Package index as `.tar.gz`
5. Upload to GitHub Releases as `chroma-index-{distro-id}.tar.gz`
6. Generate checksums for verification

## Data Model

### ChromaDB Collection Schema

**Collection Naming**: `cmdai_{os}_{distro_id}`
- Example: `cmdai_linux_ubuntu-22.04`
- Example: `cmdai_macos_14`

**Document Structure**:
```json
{
  "id": "grep_OPTIONS",
  "command": "grep",
  "section": "OPTIONS",
  "content": "Detailed options documentation...",
  "metadata": {
    "os": "linux",
    "distro": "ubuntu-22.04",
    "version": "grep (GNU grep) 3.7",
    "installed_path": "/usr/bin/grep",
    "last_indexed": "2025-12-17T12:00:00Z",
    "man_section": "1",
    "is_gnu": true,
    "command_type": "coreutil"
  }
}
```

**Embedding Strategy**:
- Model: `sentence-transformers/all-MiniLM-L6-v2`
- Dimensions: 384
- Size: ~22MB
- Inference: ~10ms per query on CPU
- Storage: Model downloaded to `~/.cache/cmdai/embeddings/`

### Index Storage

**Directory Structure**:
```
~/.cache/cmdai/
├── embeddings/
│   └── all-MiniLM-L6-v2/           # Sentence transformer model
├── chroma/
│   ├── cmdai_linux_ubuntu-22.04/   # ChromaDB collection
│   │   ├── chroma.sqlite3
│   │   └── index/
│   └── cmdai_macos_14/
│       ├── chroma.sqlite3
│       └── index/
└── indexes/
    ├── downloaded/
    │   ├── ubuntu-22.04.tar.gz     # Pre-built indexes
    │   └── macos-14.tar.gz
    └── checksums.txt
```

## Implementation Phases

### Phase 1: Core Indexing Infrastructure
- [ ] Man page parser with section extraction
- [ ] Command scanner for installed tools
- [ ] ChromaDB client wrapper
- [ ] Basic embedding generation

### Phase 2: CLI Integration
- [ ] `cmdai index init` command
- [ ] `cmdai index status` command
- [ ] Integration with main CLI workflow
- [ ] Graceful fallback when ChromaDB unavailable

### Phase 3: RAG Enhancement
- [ ] Integrate vector store into AgentLoop
- [ ] Query interface for command documentation
- [ ] Enhanced prompt templates with retrieved docs
- [ ] Performance benchmarking

### Phase 4: Distribution Support
- [ ] Distribution detection enhancement
- [ ] Pre-built index download logic
- [ ] Air-gapped mode handling
- [ ] Index import/export commands

### Phase 5: CI/CD Automation
- [ ] GitHub Actions workflow for index building
- [ ] Multi-platform matrix builds
- [ ] Release artifact uploading
- [ ] Checksum generation and verification

## Performance Requirements

- **Indexing Time**: < 60 seconds for ~500 common commands
- **Query Latency**: < 50ms for top-5 relevant docs
- **Storage Size**: < 100MB per distribution index
- **Embedding Model Load**: < 500ms on first use
- **Memory Overhead**: < 50MB when ChromaDB active

## Air-gapped Support Strategy

1. **Network Detection**:
   ```rust
   fn is_network_available() -> bool {
       // Try DNS lookup to common hosts
       // Timeout after 2 seconds
   }
   ```

2. **Fallback Chain**:
   - Try local ChromaDB index
   - Try downloaded pre-built index
   - Fall back to `--help` queries (current behavior)
   - Never block command generation

3. **Manual Index Transfer**:
   ```bash
   # On connected machine:
   cmdai index export /tmp/cmdai-index.tar.gz

   # Transfer to air-gapped machine, then:
   cmdai index import /tmp/cmdai-index.tar.gz
   ```

## Error Handling

- **ChromaDB Server Unavailable**: Fall back to --help queries, log warning
- **Embedding Model Missing**: Download on first use (if online), else disable RAG
- **Index Corruption**: Offer to rebuild, fall back to --help
- **Distribution Mismatch**: Warn user, allow manual selection
- **Out of Disk Space**: Clear old indexes, compact database

## Security Considerations

- **Man Page Injection**: Validate man page sources, sanitize content
- **Index Tampering**: Verify checksums for downloaded indexes
- **Arbitrary Code**: Never execute commands from indexed content
- **Privacy**: All processing local, no telemetry to ChromaDB servers

## Testing Strategy

- **Unit Tests**: Man parser, command scanner, ChromaDB client
- **Integration Tests**: End-to-end indexing workflow
- **Contract Tests**: ChromaDB API compatibility
- **Performance Tests**: Query latency, indexing throughput
- **Distribution Tests**: Verify indexes for each supported OS

## Metrics & Monitoring

Track via debug logs (optional telemetry):
- Index initialization time
- Query latency percentiles (p50, p95, p99)
- Cache hit rate for retrieved docs
- Fallback activation frequency
- Index size and document count

## Future Enhancements

- **Incremental Updates**: Only re-index changed commands
- **Custom Collections**: User-defined command collections
- **Context Caching**: Cache frequent queries
- **Multi-language Support**: Index non-English man pages
- **Semantic Search**: Natural language queries for command discovery
