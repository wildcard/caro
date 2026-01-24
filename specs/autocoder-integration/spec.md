# Autocoder Integration Specification

**Created**: January 13, 2026
**Status**: Proposal
**Priority**: High - Development Velocity Enhancement

## Executive Summary

This proposal outlines how to integrate [Autocoder](https://github.com/leonvanzyl/autocoder) into caro's development workflow to enable **autonomous multi-session feature development**. Autocoder is a long-running AI development agent built on the Claude Agent SDK that can progressively build features across sessions while maintaining state through SQLite persistence.

## Problem Statement

Current caro development has sophisticated AI-assisted tooling (25+ Claude agents, Spec-Kitty, Kittify missions), but suffers from:

1. **Session-bound context**: Each Claude Code session starts fresh; accumulated understanding is lost
2. **Manual orchestration**: Developers must manually invoke agents and track progress
3. **No visibility dashboard**: Real-time progress monitoring requires terminal watching
4. **Single-session features**: Complex features spanning multiple days require re-context loading

## Proposed Solution

Integrate Autocoder to provide:

| Capability | Current State | With Autocoder |
|------------|--------------|----------------|
| Session persistence | Manual handoffs via `.claude/memory/` | Automatic SQLite-based state |
| Feature tracking | File-based Spec-Kitty tasks | Database with prioritization |
| Progress visibility | Terminal output | React web dashboard |
| Autonomous execution | Manual agent invocation | Long-running agent loops |
| Multi-session work | Context re-loading | Seamless continuation |

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CARO DEVELOPMENT WORKFLOW                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐     ┌─────────────────┐     ┌─────────────────┐   │
│  │ Spec-Kitty  │────▶│   AUTOCODER     │────▶│  Claude Code    │   │
│  │ Feature     │     │   ORCHESTRATOR  │     │  Agents         │   │
│  │ Specs       │     │                 │     │  (25+ agents)   │   │
│  └─────────────┘     │  ┌───────────┐  │     └─────────────────┘   │
│                      │  │ SQLite DB │  │                            │
│  ┌─────────────┐     │  │ Features  │  │     ┌─────────────────┐   │
│  │ /spec-kitty │     │  │ Progress  │  │     │  Web Dashboard  │   │
│  │ Commands    │────▶│  │ Sessions  │  │────▶│  localhost:5173 │   │
│  └─────────────┘     │  └───────────┘  │     └─────────────────┘   │
│                      │                 │                            │
│  ┌─────────────┐     │  MCP TOOLS:     │     ┌─────────────────┐   │
│  │ GitHub      │     │  • feature_get  │     │  Git + CI/CD    │   │
│  │ Issues      │────▶│  • feature_next │────▶│  Progress       │   │
│  └─────────────┘     │  • feature_done │     │  Commits        │   │
│                      └─────────────────┘     └─────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Integration Phases

### Phase 1: Local Development Enhancement (Quick Wins)

**Timeline**: 1-2 weeks
**Effort**: Low
**Impact**: Medium

#### 1.1 Install Autocoder as Development Tool
```bash
# Clone into tools directory
git clone https://github.com/leonvanzyl/autocoder.git tools/autocoder

# Configure for caro project
cd tools/autocoder
cp .env.example .env
# Set ANTHROPIC_API_KEY or use existing Claude Code auth
```

#### 1.2 Create Spec-Kitty → Autocoder Bridge Script
Build a bridge that converts Spec-Kitty `tasks.md` files into Autocoder's SQLite format:

```python
# tools/spec-to-autocoder.py
"""
Converts .specify/tasks.md → autocoder SQLite features
Enables autonomous execution of spec-kitty planned tasks
"""
```

#### 1.3 Add Web Dashboard Launch Command
```toml
# .claude/commands/autocoder-dashboard.md
Launch the Autocoder web dashboard for monitoring autonomous development progress.
Run: cd tools/autocoder && ./start_ui.sh
```

### Phase 2: MCP Integration (Medium Term)

**Timeline**: 3-4 weeks
**Effort**: Medium
**Impact**: High

#### 2.1 Create Autocoder MCP Server for Caro

Expose autocoder's feature management as MCP tools callable from Claude Code:

```rust
// src/mcp/autocoder_bridge.rs
/// MCP tools for autocoder integration
pub struct AutocoderMcpServer {
    db: SqliteConnection,
}

impl AutocoderMcpServer {
    /// Get next prioritized feature to implement
    pub async fn feature_get_next(&self) -> Result<Feature>;

    /// Mark current feature as complete
    pub async fn feature_mark_done(&self, id: FeatureId) -> Result<()>;

    /// Get progress statistics
    pub async fn feature_get_stats(&self) -> Result<ProgressStats>;

    /// Skip current feature (requeue with lower priority)
    pub async fn feature_skip(&self, id: FeatureId) -> Result<()>;
}
```

#### 2.2 Bidirectional Sync with Spec-Kitty

```yaml
# .claude/mcp/autocoder-sync.yaml
sync_rules:
  - source: ".specify/{feature}/tasks.md"
    target: "autocoder.db:features"
    direction: bidirectional

  - source: "autocoder.db:progress"
    target: ".specify/{feature}/kanban.md"
    direction: autocoder → spec-kitty
```

### Phase 3: Deep Integration (Long Term)

**Timeline**: 6-8 weeks
**Effort**: High
**Impact**: Very High

#### 3.1 Custom Autocoder Agent for Caro

Create a caro-specialized autocoder that understands:
- Rust/cargo conventions
- Safety-first development practices
- TDD requirements from constitution
- Platform-specific builds (MLX, CPU, remote backends)

```python
# tools/autocoder/agents/caro_specialist.py
class CaroSpecialistAgent:
    """
    Specialized agent for caro development that:
    - Runs cargo check/test after each change
    - Validates safety patterns before commit
    - Ensures MSRV compliance (Rust 1.83)
    - Runs evaluation harness for LLM changes
    """
```

#### 3.2 Webhook Integration with GitHub

Connect autocoder progress to GitHub:
- Auto-update issue status when features complete
- Create PRs when feature branches are ready
- Post progress comments on related issues

```bash
# Set N8N webhook for GitHub integration
export PROGRESS_N8N_WEBHOOK_URL="https://n8n.caro.dev/webhook/autocoder-progress"
```

#### 3.3 Voice Synthesis Integration (v2.0.0 Synergy)

Leverage autocoder's long-running capability for v2.0.0 voice synthesis research:
- Autonomous experimentation with TTS models
- Multi-day training runs with progress tracking
- Dashboard showing synthesis quality metrics

## Workflow Examples

### Example 1: Autonomous Feature Implementation

```bash
# 1. Define feature with spec-kitty
/spec-kitty.specify "Add JSON Schema validation for config files"

# 2. Plan implementation
/spec-kitty.plan

# 3. Generate tasks
/spec-kitty.tasks

# 4. Launch autonomous execution
./tools/autocoder/start.sh

# 5. Monitor progress at http://localhost:5173
# Agent autonomously:
#   - Picks highest priority task
#   - Implements with TDD
#   - Commits changes
#   - Marks complete
#   - Moves to next task
```

### Example 2: Multi-Day Feature Development

```bash
# Day 1: Initialize feature
/spec-kitty.specify "Implement Karo distributed intelligence"
/spec-kitty.plan
/spec-kitty.tasks

# Launch autocoder (can run overnight)
./tools/autocoder/start.sh --session-name karo-distributed

# Day 2: Check progress
curl http://localhost:5173/api/stats
# {"completed": 12, "remaining": 8, "in_progress": "T015"}

# Continue from where it left off
./tools/autocoder/start.sh --session-name karo-distributed --resume
```

### Example 3: Roadmap Item Automation

Convert ROADMAP.md items into autocoder features:

```bash
# Parse roadmap and create features
python tools/roadmap-to-features.py ROADMAP.md

# Launch autonomous work on v1.2.0 items
./tools/autocoder/start.sh --milestone v1.2.0
```

## Benefits Analysis

### Development Velocity

| Metric | Current | With Autocoder | Improvement |
|--------|---------|----------------|-------------|
| Context switching cost | ~15 min/session | ~0 (persistent) | 100% |
| Feature tracking overhead | Manual updates | Automatic | 80% reduction |
| Overnight development | None | Autonomous | New capability |
| Multi-day feature time | Re-context each day | Continuous | 40% faster |

### Quality Assurance

Autocoder's security controls align with caro's safety-first philosophy:
- Filesystem sandboxing (project directory only)
- Bash command allowlist (safe operations only)
- Git-based progress tracking (audit trail)
- Per-commit verification (tests must pass)

### Developer Experience

1. **Web Dashboard**: Visual progress tracking vs terminal scrollback
2. **Pause/Resume**: Interrupt and continue without context loss
3. **Feature Queue**: Prioritized backlog automatically managed
4. **Session Reports**: Summary of what was accomplished

## Technical Considerations

### Security

Autocoder runs with the same permissions as Claude Code. Caro's existing security model applies:
- Safety validation on all generated commands
- No execution without user confirmation (interactive mode)
- Audit logging of all operations

### Resource Usage

| Resource | Autocoder Requirement | Caro Impact |
|----------|----------------------|-------------|
| API calls | Per-task (~5-20/feature) | Standard Claude usage |
| Disk | SQLite DB (~10MB) | Negligible |
| Memory | ~200MB (Python + React) | Acceptable |
| Network | Claude API + optional N8N | Standard |

### Licensing

Autocoder is AGPL-3.0 licensed. For internal development tooling (not distributed with caro), this is acceptable. If distributing, would need separate licensing consideration.

## Success Metrics

### Phase 1 Success Criteria
- [ ] Autocoder installed and running locally
- [ ] Spec-kitty tasks successfully imported
- [ ] Dashboard accessible at localhost:5173
- [ ] At least one feature completed autonomously

### Phase 2 Success Criteria
- [ ] MCP tools callable from Claude Code
- [ ] Bidirectional sync working
- [ ] GitHub webhook integration functional
- [ ] 50% reduction in context-switching overhead

### Phase 3 Success Criteria
- [ ] Caro-specific agent operational
- [ ] Multi-day features completing autonomously
- [ ] Integration with v2.0.0 research workflows
- [ ] 3x improvement in feature completion velocity

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| API cost increase | Medium | Monitor usage; set budget limits |
| Quality regression | High | Maintain CI gates; require human review |
| Context drift | Medium | Regular sync points; session summaries |
| Dependency on external project | Medium | Fork and maintain if needed |

## Next Steps

1. **Immediate**: Clone autocoder and test with a small feature
2. **This week**: Create spec-to-autocoder bridge script
3. **Next sprint**: Implement MCP integration (Phase 2.1)
4. **v1.2.0 timeframe**: Full integration (Phase 3)

## References

- [Autocoder GitHub](https://github.com/leonvanzyl/autocoder)
- [Claude Agent SDK](https://docs.anthropic.com/en/docs/claude-agent-sdk)
- [Caro Roadmap](../ROADMAP.md)
- [Spec-Kitty Documentation](../.specify/)
- [MCP Integration Issue](../.github/first-time-issues/06-mcp-claude-code-integration.md)
