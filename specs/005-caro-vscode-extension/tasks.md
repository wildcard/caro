# Tasks: Caro VS Code Extension

**Feature Branch**: `005-caro-vscode-extension`
**Created**: 2025-12-22
**Status**: Not Started

---

## Phase 1: Foundation

### Work Package 1.1: Rust Server Mode

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T1.1.1 | Add `--server` CLI flag to clap configuration | [ ] | |
| T1.1.2 | Create `src/server/mod.rs` with JsonRpcServer struct | [ ] | |
| T1.1.3 | Implement Content-Length message framing | [ ] | |
| T1.1.4 | Implement request parsing (JSON-RPC 2.0) | [ ] | |
| T1.1.5 | Implement response serialization | [ ] | |
| T1.1.6 | Create `initialize` RPC method | [ ] | |
| T1.1.7 | Create `generateCommand` RPC method | [ ] | |
| T1.1.8 | Create `shutdown` RPC method | [ ] | |
| T1.1.9 | Wire CommandGenerator to RPC handler | [ ] | |
| T1.1.10 | Wire SafetyValidator to RPC handler | [ ] | |
| T1.1.11 | Add unit tests for server module | [ ] | |
| T1.1.12 | Add integration test for full RPC flow | [ ] | |

**Acceptance Criteria:**
- `caro --server` starts and waits for stdin
- Valid JSON-RPC requests get responses
- `generateCommand` returns command with safety info
- Server exits cleanly on shutdown

---

### Work Package 1.2: Extension Scaffolding

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T1.2.1 | Initialize extension project (yo code / manual) | [ ] | |
| T1.2.2 | Configure TypeScript with strict mode | [ ] | |
| T1.2.3 | Set up ESLint and Prettier | [ ] | |
| T1.2.4 | Create package.json with basic contributions | [ ] | |
| T1.2.5 | Add Activity Bar view container for Caro | [ ] | |
| T1.2.6 | Create extension.ts with activate/deactivate | [ ] | |
| T1.2.7 | Implement CaroProcess class to spawn binary | [ ] | |
| T1.2.8 | Implement process lifecycle management | [ ] | |
| T1.2.9 | Add automatic restart on crash | [ ] | |
| T1.2.10 | Create CaroClient class for JSON-RPC | [ ] | |
| T1.2.11 | Implement request/response correlation | [ ] | |
| T1.2.12 | Add notification handler infrastructure | [ ] | |
| T1.2.13 | Configure settings for binary path | [ ] | |
| T1.2.14 | Add logging for debugging | [ ] | |

**Acceptance Criteria:**
- Extension activates without errors
- Caro process spawned on activation
- Logs show successful JSON-RPC initialize
- Crash triggers automatic restart

---

### Work Package 1.3: Chat Panel MVP

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T1.3.1 | Create WebView panel provider | [ ] | |
| T1.3.2 | Create chat HTML template | [ ] | |
| T1.3.3 | Style chat with VS Code theme variables | [ ] | |
| T1.3.4 | Implement message list rendering | [ ] | |
| T1.3.5 | Implement input field with send button | [ ] | |
| T1.3.6 | Implement command block component | [ ] | |
| T1.3.7 | Add syntax highlighting to command blocks | [ ] | |
| T1.3.8 | Implement safety level indicator | [ ] | |
| T1.3.9 | Add explanation accordion | [ ] | |
| T1.3.10 | Implement Copy button functionality | [ ] | |
| T1.3.11 | Implement Execute button | [ ] | |
| T1.3.12 | Connect WebView to extension via postMessage | [ ] | |
| T1.3.13 | Route messages to CaroClient | [ ] | |
| T1.3.14 | Handle streaming responses (if applicable) | [ ] | |
| T1.3.15 | Implement loading state | [ ] | |
| T1.3.16 | Implement error state | [ ] | |
| T1.3.17 | Create terminal for execution | [ ] | |
| T1.3.18 | Send command to terminal | [ ] | |
| T1.3.19 | Add keyboard shortcut to open panel | [ ] | |

**Acceptance Criteria:**
- Chat panel opens from Activity Bar
- User can type and send message
- Command appears with highlighting
- Safety level shown with color
- Copy copies to clipboard
- Execute runs in terminal

---

## Phase 2: Context Integration

### Work Package 2.1: Context Menu

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T2.1.1 | Add Caro submenu to editor context menu | [ ] | |
| T2.1.2 | Register "Ask Caro About Selection" command | [ ] | |
| T2.1.3 | Register "Generate Shell Command" command | [ ] | |
| T2.1.4 | Register "Check Script Safety" command | [ ] | |
| T2.1.5 | Add when-clause for selection-based commands | [ ] | |
| T2.1.6 | Add when-clause for script files | [ ] | |
| T2.1.7 | Implement getSelectedText helper | [ ] | |
| T2.1.8 | Implement getActiveFileInfo helper | [ ] | |
| T2.1.9 | Open chat panel from context commands | [ ] | |
| T2.1.10 | Pre-populate chat with context | [ ] | |

**Acceptance Criteria:**
- Right-click shows Caro submenu
- "Ask About Selection" appears when text selected
- "Check Safety" appears for .sh files
- Chat opens with context message

---

### Work Package 2.2: Context-Aware Generation

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T2.2.1 | Extend generateCommand RPC params with context | [ ] | |
| T2.2.2 | Add selectedText field | [ ] | |
| T2.2.3 | Add activeFile field | [ ] | |
| T2.2.4 | Add activeFileContent field (truncated) | [ ] | |
| T2.2.5 | Add cwd field | [ ] | |
| T2.2.6 | Update Rust handler to use context | [ ] | |
| T2.2.7 | Create file-type-aware system prompts | [ ] | |
| T2.2.8 | Optimize for Dockerfile context | [ ] | |
| T2.2.9 | Optimize for shell script context | [ ] | |
| T2.2.10 | Optimize for package.json context | [ ] | |
| T2.2.11 | Detect shell type from VS Code terminal | [ ] | |
| T2.2.12 | Pass shell type to generator | [ ] | |

**Acceptance Criteria:**
- Commands consider file context
- Dockerfile prompts get Docker-aware suggestions
- Shell scripts get POSIX-focused suggestions
- Shell type affects command syntax

---

## Phase 3: Proactive Analysis

### Work Package 3.1: Rust Proactive Analyzer

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T3.1.1 | Create `src/proactive/mod.rs` | [ ] | |
| T3.1.2 | Define Suggestion struct | [ ] | |
| T3.1.3 | Define SuggestionCategory enum | [ ] | |
| T3.1.4 | Define FileAnalysisResult struct | [ ] | |
| T3.1.5 | Implement file type detection | [ ] | |
| T3.1.6 | Create ProactiveAnalyzer struct | [ ] | |
| T3.1.7 | Implement tool recommendation rules | [ ] | |
| T3.1.8 | Implement security warning rules | [ ] | |
| T3.1.9 | Implement POSIX compliance rules | [ ] | |
| T3.1.10 | Implement performance tip rules | [ ] | |
| T3.1.11 | Add line/column tracking for suggestions | [ ] | |
| T3.1.12 | Create `analyzeFile` RPC method | [ ] | |
| T3.1.13 | Add suggested fix to suggestions | [ ] | |
| T3.1.14 | Unit tests for each rule category | [ ] | |
| T3.1.15 | Integration test for full analysis | [ ] | |

**Proactive Rules to Implement:**
- [ ] curl|bash pattern detection
- [ ] chmod 777 detection
- [ ] echo -e → printf suggestion
- [ ] Python JSON → jq suggestion
- [ ] Multiple grep → single grep|grep
- [ ] rm -rf on root paths
- [ ] sudo without justification
- [ ] Unquoted variables
- [ ] eval usage warning
- [ ] > /dev/null 2>&1 suggestion

**Acceptance Criteria:**
- `analyzeFile` returns structured suggestions
- 10+ rules implemented
- Suggestions include line numbers
- Fix suggestions included

---

### Work Package 3.2: Extension Diagnostics

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T3.2.1 | Create DiagnosticCollection for Caro | [ ] | |
| T3.2.2 | Map Suggestion to Diagnostic | [ ] | |
| T3.2.3 | Map severity levels | [ ] | |
| T3.2.4 | Create FileSystemWatcher for target files | [ ] | |
| T3.2.5 | Define file patterns to watch | [ ] | |
| T3.2.6 | Debounce analysis calls | [ ] | |
| T3.2.7 | Trigger analysis on file open | [ ] | |
| T3.2.8 | Trigger analysis on file save | [ ] | |
| T3.2.9 | Clear diagnostics on file close | [ ] | |
| T3.2.10 | Add setting: enableProactiveAnalysis | [ ] | |
| T3.2.11 | Add setting: proactiveFilePatterns | [ ] | |
| T3.2.12 | Respect settings in watcher | [ ] | |

**Acceptance Criteria:**
- Diagnostics appear for analyzed files
- Problems panel shows Caro issues
- Squiggly lines on problematic code
- Settings control behavior

---

### Work Package 3.3: CodeLens Provider

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T3.3.1 | Create CodeLensProvider class | [ ] | |
| T3.3.2 | Register for target file types | [ ] | |
| T3.3.3 | Map suggestions to CodeLens | [ ] | |
| T3.3.4 | Format CodeLens title | [ ] | |
| T3.3.5 | Link CodeLens to command | [ ] | |
| T3.3.6 | Implement "Open in Chat" command | [ ] | |
| T3.3.7 | Create CodeActionProvider class | [ ] | |
| T3.3.8 | Provide Quick Fix for suggestions | [ ] | |
| T3.3.9 | Apply text replacement fix | [ ] | |
| T3.3.10 | Test non-intrusive presentation | [ ] | |

**Acceptance Criteria:**
- CodeLens shows above problematic lines
- Click opens chat with context
- Quick Fix applies suggested change
- UI is non-intrusive

---

## Phase 4: Security Analysis

### Work Package 4.1: Extended Security Patterns

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T4.1.1 | Audit existing 52 safety patterns | [ ] | |
| T4.1.2 | Add network exfiltration patterns | [ ] | |
| T4.1.3 | Add encoded payload detection | [ ] | |
| T4.1.4 | Add environment exposure patterns | [ ] | |
| T4.1.5 | Add credential leak patterns | [ ] | |
| T4.1.6 | Add privilege escalation patterns | [ ] | |
| T4.1.7 | Define severity classification | [ ] | |
| T4.1.8 | Add severity to all patterns | [ ] | |
| T4.1.9 | Create fix suggestions for each pattern | [ ] | |
| T4.1.10 | Add reference links to explanations | [ ] | |
| T4.1.11 | Test patterns against real scripts | [ ] | |

**Security Patterns to Add:**
- [ ] base64 decode + eval
- [ ] wget -O - | sh
- [ ] printenv to log/file
- [ ] AWS/GCP credential patterns
- [ ] SSH key exposure
- [ ] Password in script
- [ ] setuid/setgid changes
- [ ] iptables disable
- [ ] SELinux disable
- [ ] Firewall manipulation

**Acceptance Criteria:**
- 20+ additional security patterns
- All patterns have severity
- All patterns have fixes
- Zero false positives on benign scripts

---

### Work Package 4.2: Security Report

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T4.2.1 | Create `scanWorkspace` RPC method | [ ] | |
| T4.2.2 | Implement file discovery | [ ] | |
| T4.2.3 | Implement batch analysis | [ ] | |
| T4.2.4 | Aggregate findings by severity | [ ] | |
| T4.2.5 | Create "Scan Workspace" command | [ ] | |
| T4.2.6 | Create report WebView panel | [ ] | |
| T4.2.7 | Design summary view | [ ] | |
| T4.2.8 | Implement drill-down to files | [ ] | |
| T4.2.9 | Add export to JSON | [ ] | |
| T4.2.10 | Add export to Markdown | [ ] | |
| T4.2.11 | Add progress indicator | [ ] | |

**Acceptance Criteria:**
- "Caro: Scan Workspace" in command palette
- Report shows summary by severity
- Click navigates to file
- Export works

---

## Phase 5: Polish & Release

### Work Package 5.1: Performance

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T5.1.1 | Defer Caro spawn to first use | [ ] | |
| T5.1.2 | Lazy initialize WebView | [ ] | |
| T5.1.3 | Measure activation time | [ ] | |
| T5.1.4 | Optimize to < 100ms | [ ] | |
| T5.1.5 | Implement analysis result cache | [ ] | |
| T5.1.6 | Bound cache memory usage | [ ] | |
| T5.1.7 | Invalidate cache on file change | [ ] | |
| T5.1.8 | Implement response streaming | [ ] | |
| T5.1.9 | Add cancel capability | [ ] | |
| T5.1.10 | Profile memory usage | [ ] | |

**Acceptance Criteria:**
- Activation < 100ms
- Command generation < 2s
- Memory < 100MB baseline
- UI remains responsive

---

### Work Package 5.2: Error Handling

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T5.2.1 | Handle missing Caro binary | [ ] | |
| T5.2.2 | Provide install instructions | [ ] | |
| T5.2.3 | Handle Caro crash gracefully | [ ] | |
| T5.2.4 | Show reconnecting state | [ ] | |
| T5.2.5 | Handle RPC timeout | [ ] | |
| T5.2.6 | Handle malformed response | [ ] | |
| T5.2.7 | Create user-friendly error messages | [ ] | |
| T5.2.8 | Add "Get Help" links | [ ] | |
| T5.2.9 | Log errors for debugging | [ ] | |

**Acceptance Criteria:**
- No silent failures
- All errors have user message
- Recovery paths documented
- Debug info available

---

### Work Package 5.3: Documentation

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T5.3.1 | Write README.md | [ ] | |
| T5.3.2 | Document installation | [ ] | |
| T5.3.3 | Document features | [ ] | |
| T5.3.4 | Document configuration | [ ] | |
| T5.3.5 | Add screenshots | [ ] | |
| T5.3.6 | Create CHANGELOG.md | [ ] | |
| T5.3.7 | Write Marketplace description | [ ] | |
| T5.3.8 | Create demo GIF | [ ] | |
| T5.3.9 | Create welcome walkthrough | [ ] | |
| T5.3.10 | Add in-app tips | [ ] | |

**Acceptance Criteria:**
- README complete
- Marketplace listing ready
- Users can self-serve

---

### Work Package 5.4: Testing

| ID | Task | Status | Assignee |
|----|------|--------|----------|
| T5.4.1 | Set up test framework | [ ] | |
| T5.4.2 | Unit tests: CaroClient | [ ] | |
| T5.4.3 | Unit tests: Providers | [ ] | |
| T5.4.4 | Unit tests: Message handling | [ ] | |
| T5.4.5 | Integration test: Full workflow | [ ] | |
| T5.4.6 | Test on macOS | [ ] | |
| T5.4.7 | Test on Linux | [ ] | |
| T5.4.8 | Test on Windows | [ ] | |
| T5.4.9 | Test VS Code 1.85 | [ ] | |
| T5.4.10 | Test VS Code latest | [ ] | |
| T5.4.11 | Performance benchmark | [ ] | |
| T5.4.12 | Document test results | [ ] | |

**Acceptance Criteria:**
- 80%+ code coverage
- All platforms pass
- Performance meets targets

---

## Task Summary

| Phase | Work Package | Tasks | Status |
|-------|-------------|-------|--------|
| 1 | Server Mode | 12 | Not Started |
| 1 | Extension Scaffolding | 14 | Not Started |
| 1 | Chat Panel MVP | 19 | Not Started |
| 2 | Context Menu | 10 | Not Started |
| 2 | Context-Aware Generation | 12 | Not Started |
| 3 | Proactive Analyzer | 15 | Not Started |
| 3 | Extension Diagnostics | 12 | Not Started |
| 3 | CodeLens Provider | 10 | Not Started |
| 4 | Security Patterns | 11 | Not Started |
| 4 | Security Report | 11 | Not Started |
| 5 | Performance | 10 | Not Started |
| 5 | Error Handling | 9 | Not Started |
| 5 | Documentation | 10 | Not Started |
| 5 | Testing | 12 | Not Started |
| **Total** | | **167** | |

---

*Tasks Version: 1.0*
*Last Updated: 2025-12-22*
