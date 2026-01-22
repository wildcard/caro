# Implementation Plan: Caro VS Code Extension

**Feature Branch**: `005-caro-vscode-extension`
**Created**: 2025-12-22
**Status**: Planning
**Spec Reference**: [spec.md](./spec.md)
**Research Reference**: [research.md](./research.md)

---

## Overview

This plan details the implementation strategy for the Caro VS Code extension. The implementation is divided into 5 phases, each building on the previous to deliver incrementally usable functionality.

---

## Architecture Decisions

### AD-001: IPC Protocol
**Decision**: JSON-RPC 2.0 over stdio
**Rationale**:
- Same protocol as LSP (proven in VS Code ecosystem)
- Bidirectional communication
- Request/response correlation
- Notification support
- Language-agnostic

**Alternatives Considered**:
- gRPC: More complex, requires protobuf, overkill for this use case
- HTTP/REST: Higher latency, unnecessary network stack
- Unix sockets: Not cross-platform

### AD-002: Caro Server Mode
**Decision**: `caro --server` flag starts JSON-RPC server
**Rationale**:
- Single binary principle maintained
- Reuses all existing Caro logic
- Extension spawns process, no installation complexity

### AD-003: WebView for Chat
**Decision**: React-based WebView panel
**Rationale**:
- Rich UI capability
- Familiar development pattern
- Can share components with future GUI

### AD-004: Proactive Analysis Trigger
**Decision**: File open + file save events
**Rationale**:
- Balance responsiveness with performance
- User expectation: analysis on visible files
- Debounced to prevent excessive analysis

### AD-005: Local Model Default
**Decision**: Use embedded local model by default
**Rationale**:
- Privacy by design
- Offline capability
- No API keys required for basic usage

---

## Phase 1: Foundation

**Duration**: 3 weeks
**Goal**: Basic chat panel with command generation

### 1.1 Rust Core: Server Mode

**Tasks**:

1. **Add `--server` flag to CLI** (src/main.rs)
   - Parse new flag
   - Branch to server mode
   - Exit cleanly on stdin EOF

2. **Implement JSON-RPC server** (src/server/mod.rs)
   - Buffered stdin reader
   - Buffered stdout writer
   - Message framing (Content-Length headers, like LSP)
   - Request/response correlation

3. **Implement core RPC methods**
   - `initialize`: Handshake, capability exchange
   - `generateCommand`: Main command generation
   - `shutdown`: Clean exit

4. **Wire existing components**
   - Integrate CommandGenerator trait
   - Integrate SafetyValidator
   - Integrate ExecutionContext

**Deliverables**:
- `caro --server` responds to JSON-RPC requests
- `generateCommand` returns commands with safety info
- Unit tests for server module

### 1.2 VS Code Extension: Scaffolding

**Tasks**:

1. **Create extension project**
   - yo code generator or manual setup
   - TypeScript configuration
   - ESLint/Prettier setup

2. **Package.json contributions**
   - Activity bar view container
   - WebView panel contribution
   - Basic commands

3. **Extension activation**
   - Spawn `caro --server` process
   - Handle process lifecycle
   - Reconnection on crash

4. **JSON-RPC client**
   - TypeScript implementation
   - Request/response promises
   - Notification handling

**Deliverables**:
- Extension activates and spawns Caro
- Logs show successful connection
- Basic lifecycle management

### 1.3 Chat Panel: MVP

**Tasks**:

1. **WebView HTML/CSS**
   - Chat message list
   - Input field
   - Send button
   - VS Code theme integration

2. **WebView JavaScript**
   - Message posting to extension
   - Receiving responses
   - Rendering command blocks
   - Copy button functionality

3. **Extension message handling**
   - Route WebView messages to Caro client
   - Forward responses to WebView
   - Error display

4. **Command execution**
   - Terminal API integration
   - Create/reuse terminal instance
   - Send command to terminal

**Deliverables**:
- User can type prompt in chat
- Command appears with explanation
- Execute button runs in terminal
- Copy button works

---

## Phase 2: Context Integration

**Duration**: 2 weeks
**Goal**: Editor context awareness

### 2.1 Context Menu

**Tasks**:

1. **Register context menu items**
   - Submenu contribution
   - Command registrations
   - When-clause conditions

2. **Get editor selection**
   - Active editor detection
   - Selection text extraction
   - Cursor position

3. **File context**
   - Active file path
   - File content (for analysis)
   - File type detection

4. **Pre-populate chat**
   - Pass selection to chat panel
   - Format context prompt
   - Auto-focus input

**Deliverables**:
- "Ask Caro About Selection" in context menu
- Selected text used as context
- Chat opens with context prefilled

### 2.2 Context in Generation

**Tasks**:

1. **Extend RPC protocol**
   - Add context field to request
   - Include cwd, file, selection
   - Include shell detection

2. **Rust core: context-aware prompts**
   - Use file content in system prompt
   - Detect file type for specialized prompts
   - Include platform context

**Deliverables**:
- Commands consider current file
- Better suggestions for Dockerfiles, scripts
- Shell-specific command syntax

---

## Phase 3: Proactive Analysis

**Duration**: 3 weeks
**Goal**: Background file analysis with suggestions

### 3.1 Rust Core: Proactive Analyzer

**Tasks**:

1. **File type detection**
   - Dockerfile, *.sh, *.bash, *.zsh
   - package.json (scripts section)
   - CI configs (.github/workflows)
   - Makefiles

2. **Rule engine for suggestions**
   - Tool recommendations (jq, awk, sed)
   - Security warnings (curl|bash, chmod 777)
   - POSIX compliance (echo -e → printf)
   - Performance tips (combine grep)

3. **AST parsing (optional)**
   - tree-sitter integration
   - Shell script parsing
   - Command extraction

4. **New RPC method: `analyzeFile`**
   - Accept file path and content
   - Return structured suggestions
   - Include fix actions

**Deliverables**:
- `analyzeFile` RPC method working
- 10+ rule patterns implemented
- Suggestions include line numbers

### 3.2 Extension: Diagnostics Provider

**Tasks**:

1. **DiagnosticCollection**
   - Create collection for Caro
   - Map suggestions to diagnostics
   - Severity mapping

2. **FileSystemWatcher**
   - Watch relevant file types
   - Debounce analysis calls
   - Respect settings

3. **Settings integration**
   - Enable/disable toggle
   - File type filters
   - Analysis frequency

**Deliverables**:
- Squiggly lines on issues
- Problems panel entries
- Settings control behavior

### 3.3 Extension: CodeLens Provider

**Tasks**:

1. **CodeLens for suggestions**
   - Show inline hints
   - "Caro: Consider using jq here"
   - Click to open chat

2. **Quick Fix actions**
   - Code action provider
   - Apply suggested fix
   - Open in chat for complex cases

**Deliverables**:
- Inline hints above code
- Click to apply fix
- Non-intrusive presentation

---

## Phase 4: Security Analysis

**Duration**: 2 weeks
**Goal**: Comprehensive security scanning

### 4.1 Extended Security Patterns

**Tasks**:

1. **Expand safety patterns**
   - Network exfiltration patterns
   - Encoded payload detection
   - Environment variable exposure
   - Credential patterns

2. **Severity classification**
   - Critical: immediate risk
   - High: potential damage
   - Medium: best practice violation
   - Low: suggestion

3. **Fix suggestions**
   - Safer alternatives
   - Explanations
   - Reference links

**Deliverables**:
- 20+ additional security patterns
- Severity-based presentation
- Actionable fixes

### 4.2 Security Report

**Tasks**:

1. **Workspace scan command**
   - Scan all relevant files
   - Aggregate findings
   - Generate report

2. **Report UI**
   - Summary view
   - Drill-down to files
   - Export capability

**Deliverables**:
- "Caro: Scan Workspace" command
- Summary of security issues
- Exportable report

---

## Phase 5: Polish & Release

**Duration**: 2 weeks
**Goal**: Production readiness

### 5.1 Performance Optimization

**Tasks**:

1. **Lazy loading**
   - Defer Caro spawn until needed
   - Lazy WebView initialization
   - Minimize activation impact

2. **Caching**
   - Cache analysis results
   - Invalidate on file change
   - Memory-bounded cache

3. **Streaming responses**
   - Stream command generation
   - Progressive UI updates
   - Cancel capability

**Deliverables**:
- < 100ms activation
- < 2s command generation
- Responsive UI

### 5.2 Error Handling

**Tasks**:

1. **Graceful degradation**
   - Handle missing Caro binary
   - Handle crash/restart
   - Offline mode messaging

2. **User-friendly errors**
   - Clear error messages
   - Suggested actions
   - Help links

**Deliverables**:
- No silent failures
- Helpful error UI
- Recovery guidance

### 5.3 Documentation

**Tasks**:

1. **README**
   - Installation guide
   - Feature overview
   - Configuration reference

2. **Marketplace listing**
   - Description
   - Screenshots
   - Changelog

3. **In-extension help**
   - Welcome page
   - Feature tour
   - Tips

**Deliverables**:
- Complete documentation
- Marketplace-ready listing
- User onboarding

### 5.4 Testing

**Tasks**:

1. **Unit tests**
   - Extension components
   - RPC client
   - Providers

2. **Integration tests**
   - Extension ↔ Caro communication
   - Full workflows
   - Edge cases

3. **Manual testing**
   - macOS, Linux, Windows
   - Various VS Code versions
   - Performance profiling

**Deliverables**:
- 80%+ code coverage
- Cross-platform validation
- Performance benchmarks

---

## Dependencies Graph

```
Phase 1.1 (Rust Server) ─────┐
                              ├─→ Phase 1.3 (Chat MVP)
Phase 1.2 (Extension) ────────┘
                                        │
                                        ▼
                              Phase 2.1 (Context Menu)
                                        │
                                        ▼
                              Phase 2.2 (Context in Gen)
                                        │
                                        ▼
Phase 3.1 (Rust Analyzer) ────┐
                               ├─→ Phase 3.2 (Diagnostics)
                               │            │
                               │            ▼
                               └──→ Phase 3.3 (CodeLens)
                                            │
                                            ▼
                              Phase 4.1 (Security Patterns)
                                            │
                                            ▼
                              Phase 4.2 (Security Report)
                                            │
                                            ▼
                              Phase 5 (Polish & Release)
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Caro binary distribution | Medium | High | Bundle binary or require pre-install |
| WebView security restrictions | Low | Medium | Follow VS Code best practices |
| Performance on large workspaces | Medium | Medium | Debounce, filter, pagination |
| Cross-platform issues | Medium | Medium | CI testing on all platforms |
| VS Code API changes | Low | Low | Pin VS Code version, test updates |

---

## Success Criteria

### Phase 1 Complete When:
- [ ] User can open chat panel
- [ ] User can generate command from prompt
- [ ] User can execute command in terminal
- [ ] Extension handles Caro crashes gracefully

### Phase 2 Complete When:
- [ ] Context menu shows Caro options
- [ ] Selection used as context
- [ ] File type improves suggestions

### Phase 3 Complete When:
- [ ] Proactive analysis runs on Dockerfiles
- [ ] Suggestions appear as diagnostics
- [ ] Quick fixes apply changes

### Phase 4 Complete When:
- [ ] Security patterns detect risky commands
- [ ] Workspace scan available
- [ ] Report exportable

### Phase 5 Complete When:
- [ ] Activation < 100ms
- [ ] All platforms tested
- [ ] Documentation complete
- [ ] Marketplace-ready

---

## Resource Requirements

### Development
- 1 TypeScript/VS Code extension developer
- 1 Rust developer (for server mode)
- 0.5 UX designer (for WebView)

### Testing
- macOS test machine
- Linux test machine (VM acceptable)
- Windows test machine (VM acceptable)

### Infrastructure
- VS Code Marketplace publisher account
- CI/CD for multi-platform builds
- Documentation hosting

---

## Open Items for Stakeholder Decision

1. **Binary Distribution Strategy**
   - Option A: Require user to install `caro` separately
   - Option B: Bundle `caro` binary with extension
   - Option C: Offer both (download if missing)

2. **Telemetry**
   - What metrics to collect (opt-in)?
   - Privacy policy requirements?

3. **Branding**
   - Icon design
   - Color scheme
   - Marketplace category

4. **Pricing Model** (future)
   - Free forever?
   - Premium features?
   - Enterprise tier?

---

*Plan Version: 1.0*
*Last Updated: 2025-12-22*
