# WP03: Knowledge Base System

**Work Package**: WP03
**Status**: planned
**Priority**: medium
**Estimated Effort**: 4-5 days
**Depends On**: WP02

## Objective

Build the knowledge base system for storing and distributing community tips, including cheatsheet processing pipeline and local caching.

## Tasks

### T3.1: KB Data Types
- [ ] Create `src/tips/kb/types.rs`
- [ ] Define `KnowledgeBase` struct with version, checksum
- [ ] Define `KbTip` struct with pattern, message, category
- [ ] Define `KbAlias` struct for community aliases
- [ ] Define `KbPlugin` struct for plugin metadata
- [ ] Implement serde for MessagePack serialization

### T3.2: Cheatsheet YAML Format
- [ ] Create `kb/cheatsheets/schema.json` JSON Schema
- [ ] Define YAML format for cheatsheets
- [ ] Create example cheatsheets (git, docker, kubectl)
- [ ] Document contribution format

### T3.3: KB Processor Binary
- [ ] Create `src/bin/kb-processor.rs`
- [ ] Parse YAML cheatsheets from directory
- [ ] Validate against schema
- [ ] Compile to MessagePack binary format
- [ ] Generate checksum file
- [ ] Output versioned artifact

### T3.4: KB Cache Module
- [ ] Create `src/tips/kb/cache.rs`
- [ ] Store KB in `~/.cache/caro/kb/`
- [ ] Implement cache invalidation (version check)
- [ ] Support offline mode (use cached KB)
- [ ] Handle missing/corrupt cache gracefully

### T3.5: KB Updater Module
- [ ] Create `src/tips/kb/updater.rs`
- [ ] Fetch KB from GitHub releases
- [ ] Verify checksum before using
- [ ] Implement incremental updates (if possible)
- [ ] Background update (non-blocking)

### T3.6: KB Matcher Module
- [ ] Create `src/tips/kb/matcher.rs`
- [ ] Compile tip patterns to regex on load
- [ ] Match commands against KB patterns
- [ ] Score and rank matching tips
- [ ] Integrate with TipsEngine

### T3.7: GitHub Actions Workflow
- [ ] Create `.github/workflows/kb-build.yml`
- [ ] Trigger on cheatsheet changes
- [ ] Build KB artifact
- [ ] Upload to GitHub releases
- [ ] Version tagging

### T3.8: Unit and Integration Tests
- [ ] Test YAML parsing
- [ ] Test KB serialization/deserialization
- [ ] Test pattern matching
- [ ] Test cache operations
- [ ] Test update flow (mocked)

## Acceptance Criteria

- [ ] Cheatsheets compile to versioned KB artifact
- [ ] KB fetched and cached locally
- [ ] Tips from KB appear in suggestions
- [ ] Offline mode works with cached KB
- [ ] GitHub Actions builds KB on push
- [ ] Checksum verification prevents tampering

## Technical Notes

**Cheatsheet Format**:
```yaml
# kb/cheatsheets/git.yaml
name: Git Productivity
version: 1.0.0
author: caro-team
shell: [zsh, bash]

aliases:
  - name: gst
    expansion: git status
    plugin: git

tips:
  - id: git-status-alias
    pattern: "^git status$"
    message: "Use `gst` for faster git status"
    category: alias_shortcut
    shell: [zsh]
    requires_plugin: git
```

**KB Binary Format (MessagePack)**:
```rust
#[derive(Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub version: String,
    pub updated_at: i64,  // Unix timestamp
    pub checksum: String,
    pub tips: Vec<KbTip>,
    pub aliases: Vec<KbAlias>,
}
```

**GitHub Actions**:
```yaml
- name: Build KB
  run: cargo run --release --bin kb-processor -- \
         --input kb/cheatsheets/ \
         --output kb/processed/caro-kb.msgpack

- name: Create Release
  uses: softprops/action-gh-release@v1
  with:
    files: kb/processed/caro-kb.msgpack
    tag_name: kb-v${{ steps.version.outputs.version }}
```

## Dependencies

- `rmp-serde` for MessagePack
- `serde_yaml` for YAML parsing
- `sha2` for checksums
- `reqwest` for HTTP fetching

## Files to Create

```
src/tips/kb/
├── mod.rs
├── types.rs
├── cache.rs
├── updater.rs
└── matcher.rs

src/bin/
└── kb-processor.rs

kb/
├── cheatsheets/
│   ├── schema.json
│   ├── git.yaml
│   ├── docker.yaml
│   └── kubectl.yaml
└── processed/
    └── .gitkeep

.github/workflows/
└── kb-build.yml
```
