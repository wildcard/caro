# Feature 006: Intelligent Prompt Generation & Multi-Agent Validation

**Status:** 📋 Ready for Implementation
**Priority:** P0 (Critical)
**Created:** 2025-11-28
**Owner:** Development Team

---

## 🎯 Executive Summary

This specification addresses critical failures in command generation discovered through session analysis. The system currently generates platform-incompatible commands (GNU flags on BSD systems), fails to understand user intent, and lacks validation infrastructure. This feature introduces a multi-agent system that:

- **Auto-detects** platform characteristics (OS, Unix flavor, tools)
- **Validates** commands against actual man pages before presenting to users
- **Clarifies** ambiguous requests through interactive dialogue
- **Adapts** prompts based on confidence scores and validation feedback
- **Learns** from community-contributed prompt templates

---

## 📊 Problem Analysis

### Session Failure Examples

```bash
# FAILURE 1: Platform-incompatible flags
$ cmdai "which files I can delete to de clutter my mac"
Generated: find / -type f | xargs ls -lh --sort=size
Error: ls: unrecognized option `--sort=size'
Issue: GNU-specific --sort flag doesn't exist on macOS BSD ls

# FAILURE 2: Inconsistent fallback
$ cmdai "which files I can delete to de clutter my macos sort by size"
Generated: echo 'Unable to generate command'
Issue: Reasonable request resulted in generic fallback

# FAILURE 3: Weak intent understanding
$ cmdai "clean up disk space"
Generated: rm -rf /tmp/*  # Dangerous!
Issue: Didn't ask clarifying questions about location/criteria
```

### Root Causes

1. **No Platform Context**: Prompt doesn't inform LLM about BSD vs GNU utilities
2. **No Validation Layer**: Commands presented without checking flag compatibility
3. **Poor Ambiguity Detection**: System doesn't recognize when clarification is needed
4. **Static Prompts**: No adaptation based on validation failures

---

## ✨ Solution Overview

### Multi-Agent Architecture

```
User Request
     ↓
[Orchestrator] ──→ Detect ambiguity → [Clarification Agent]
     ↓                                        ↓
     ├──→ Select flow                   Ask questions
     ↓                                        ↓
[Generator] ←──────────────── Enhanced context with answers
     ↓
Generate command (with platform context)
     ↓
[Validation Agent] ──→ Check flags against man pages
     ↓                        ↓
     ├─── PASS ────→ Present to user
     ↓
     └─── FAIL ────→ [Feedback Agent] ──→ Regenerate with feedback
                            ↓
                     (Max 3 retries, then escalate prompt)
```

### Key Innovations

1. **Platform-Aware Generation**
   - Auto-detects macOS → selects BSD prompt template
   - Injects available tools and platform context into prompt
   - Validates against actual system man pages

2. **Intelligent Validation**
   - Parses generated commands into tools/flags/args
   - Validates each flag against man page cache
   - Detects forbidden patterns (e.g., GNU flags on BSD)
   - Provides specific suggestions: "Use `-S` instead of `--sort`"

3. **Adaptive Clarification**
   - Ambiguity score triggers interactive questions
   - "Which location? (home/specific/system)"
   - "What criteria? (size/age/duplicates)"
   - "Action? (list/trash/delete)"

4. **Multi-Turn Refinement**
   - Generation → Validation → Feedback → Regenerate
   - Escalates to detailed prompt after 3 failures
   - Confidence scoring at each stage

5. **Community Extensibility**
   - Prompt templates as external TOML files
   - Users can create custom templates
   - Community can contribute better prompts

---

## 📁 Specification Files

| File | Purpose |
|------|---------|
| [spec.md](./spec.md) | Complete technical specification with requirements, success metrics, and data models |
| [architecture.md](./architecture.md) | System architecture, component design, agent communication protocols, and performance optimization |
| [tasks.md](./tasks.md) | Phased implementation plan with 15 detailed tasks following TDD methodology |
| [test_cases.md](./test_cases.md) | Comprehensive test scenarios derived from session analysis (TC-001 through TC-010) |

---

## 🚀 Implementation Phases

### Phase 1: Foundation (Weeks 1-2)
- Platform detection system
- Configuration management
- Man page parser & analyzer
- Validation agent
- Integration tests

**Deliverables:**
- Auto-detected platform config
- Man page cache (~50 common tools)
- Flag validation working

### Phase 2: Multi-Agent System (Weeks 3-4)
- Orchestrator agent
- Prompt template system
- Clarification agent
- Feedback agent
- Multi-turn generation loop

**Deliverables:**
- Single-shot generation
- Multi-turn retry with feedback
- Interactive clarification
- Confidence scoring

### Phase 3: Community Features (Weeks 5-6)
- External template loader
- Cross-platform generation
- Community template examples
- Documentation
- End-to-end testing

**Deliverables:**
- Custom templates supported
- Cross-platform mode working
- Complete documentation
- All E2E tests passing

---

## 📈 Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Platform compatibility | 60% | 99% |
| Single-shot success | 60% | 85% |
| User intent accuracy | 50% | 90% |
| Validation overhead | N/A | <100ms |
| Cold start time | 2s | <5s |

---

## 🧪 Test Coverage

### Comprehensive Test Suite

- **10 Test Cases** from session analysis (TC-001 through TC-010)
- **Unit Tests**: 95%+ coverage for all agents
- **Integration Tests**: Phase 1, Phase 2, Cross-platform
- **Contract Tests**: Agent interfaces
- **Performance Tests**: Benchmarks for critical paths
- **Regression Tests**: Known failure scenarios

See [test_cases.md](./test_cases.md) for detailed test scenarios.

---

## 💡 Usage Examples

### Before (Current Behavior)
```bash
$ cmdai "list files sorted by size"
Command: ls -lh --sort=size
Error: ls: unrecognized option `--sort=size'
```

### After (With This Feature)
```bash
$ cmdai "list files sorted by size"

[Orchestrator] Analyzing request...
[Generator] Using BSD template for macOS...
[Validator] Checking command...

Command: ls -lhS
Explanation: Generated using BSD backend
✓ Validated against BSD ls 8.3
✓ Confidence: 0.95
```

### Clarification Example
```bash
$ cmdai "clean up disk space"

🤔 Clarification needed (ambiguity: 0.82)

1. Which location?
   a) Home directory (~)
   b) Specific folder
   c) System-wide (requires sudo)

2. What criteria?
   a) Large files (>100MB)
   b) Old files (>30 days)
   c) Duplicate files

3. Action preference?
   a) List only (safe)
   b) Move to trash
   c) Permanent delete

Your choice: 1a 2a 3a

Command: find ~ -type f -size +100M -mtime +30 -ls
Explanation: Lists large, old files in home directory
✓ Confidence: 0.92
```

---

## 🔧 Configuration

### Auto-Generated Config

```toml
# ~/.config/cmdai/config.toml

[platform]
os = "macos"
unix_flavor = "bsd"
shell = "zsh"
arch = "aarch64"

[generation]
confidence_threshold = 0.6
enable_multi_turn = true
enable_clarification = true
max_retries = 3

[prompts]
base_template = "base-bsd.toml"
fallback_templates = ["detailed-bsd.toml"]
template_dir = "~/.config/cmdai/prompts"
```

### User Overrides

```bash
# Override platform for cross-platform generation
cmdai config set platform.target_os linux
cmdai config set platform.target_unix_flavor gnu

# Use custom template
cmdai config set prompts.base_template custom-macos.toml

# Adjust confidence threshold
cmdai config set generation.confidence_threshold 0.7
```

---

## 🛠️ Development Approach

### Test-Driven Development (TDD)

All tasks follow strict TDD:

1. **Red**: Write failing tests first
2. **Green**: Implement minimal code to pass
3. **Refactor**: Improve while maintaining green

Example from TASK-001:
```rust
// 1. Write test (RED)
#[test]
fn test_detects_macos_platform() {
    let platform = PlatformDetector::detect();
    assert_eq!(platform.os, Platform::MacOS);
    assert_eq!(platform.unix_flavor, UnixFlavor::BSD);
}

// 2. Implement (GREEN)
fn detect_os() -> Platform {
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
}

// 3. Refactor if needed
```

---

## 📚 Next Steps

1. **Review** this specification and architecture
2. **Approve** implementation approach
3. **Begin** TASK-001 (Platform Detection)
4. **Follow** TDD methodology throughout
5. **Track** progress in tasks.md

---

## 🤝 Contributing

### For Developers

- See [tasks.md](./tasks.md) for detailed implementation tasks
- Follow TDD approach (tests first)
- Ensure 90%+ test coverage
- Run benchmarks for performance-critical code

### For Community

- Create custom prompt templates in `~/.config/cmdai/prompts/`
- Share templates via GitHub discussions
- Report issues with specific platform/tool combinations
- Contribute to prompt improvements

---

## 📖 Related Documentation

- [CLAUDE.md](../../CLAUDE.md) - Project overview
- [Spec 001](../001-create-a-comprehensive/) - Safety validation
- [Spec 003](../003-hugging-face-model-caching/) - Model caching
- [Spec 004](../004-implement-ollama-and/) - Backend implementations

---

## 📝 Session Context

This specification was developed from an interactive session analyzing real command generation failures. The user experienced:

- GNU-specific commands failing on macOS
- Inconsistent fallback behavior
- Poor user intent understanding
- No validation of platform compatibility

These failures informed the design of the multi-agent validation system and adaptive prompt engineering approach.

---

**Status:** 📋 Ready for Implementation
**Estimated Duration:** 6 weeks (3 phases)
**Team Size:** 2-3 developers
**Methodology:** TDD, Spec-Driven Development
