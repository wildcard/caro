# PRD-001: Intelligent Clarification System

**Product Requirements Document**

| Field | Value |
|-------|-------|
| **PRD ID** | PRD-001 |
| **Status** | Draft |
| **Created** | January 2026 |
| **Authors** | Caro Product Team |
| **Priority** | P1 - High |
| **Target Release** | v1.3.0 |

---

## Executive Summary

### Problem Statement

Caro currently provides an unhelpful response when it cannot confidently translate a user query to a shell command:

```
Command: echo 'Please clarify your request'
```

This creates a frustrating user experience with no actionable guidance. Users must guess what went wrong and blindly retry, leading to abandonment and poor product perception.

**Real Example:**
```
PS C:\Users\User> caro how to reload powershell profile?
Command:
  echo 'Please clarify your request'
```

The user wanted `. $PROFILE` (PowerShell's profile reload command) but got no help achieving their goal.

### Proposed Solution

Implement an **Intelligent Clarification System** that:
1. Analyzes WHY a query is ambiguous using LLM reasoning
2. Generates targeted, answerable clarification questions
3. Enhances the original query with user responses
4. Retries command generation with improved context
5. Provides alternatives when the request is outside caro's expertise

### Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| "Please clarify" rate | 15% | <5% | Responses containing clarification fallback |
| First-try success rate | 60% | 80% | Commands that work on first generation |
| Cross-platform success | 20% | 75% | PowerShell/CMD queries resolved correctly |
| User retry rate | 40% | 15% | Users who rephrase after failure |
| Net Promoter Score | 35 | 60 | User satisfaction survey |

---

## User Stories

### US-001: Platform-Aware Clarification

**As a** Windows PowerShell user
**I want** caro to recognize when I'm asking about PowerShell
**So that** I get the correct PowerShell command instead of a POSIX equivalent

**Acceptance Criteria:**
- [x] Query containing "powershell" triggers PowerShell mode
- [x] System generates `. $PROFILE` for profile reload
- [x] Alternative commands shown for other shells
- [x] Platform detection explained in response

**Example:**
```
$ caro how to reload powershell profile

Detected: PowerShell query

Command: . $PROFILE

Alternatives:
  bash:  source ~/.bashrc
  zsh:   source ~/.zshrc
```

---

### US-002: Interactive Clarification for Ambiguous Queries

**As a** user with an ambiguous request
**I want** caro to ask specific questions
**So that** I can provide context and get the right command

**Acceptance Criteria:**
- [ ] System detects ambiguity score > 0.7
- [ ] Generates 1-3 focused clarification questions
- [ ] Questions have clear, selectable options
- [ ] User can answer with single keystrokes (1a, 2b)
- [ ] Enhanced query generates accurate command

**Example:**
```
$ caro clean up disk space

I need a bit more context:

1. What type of cleanup?
   [a] Find large files (manual review)
   [b] Delete temp/cache files
   [c] Find duplicates
   [d] Show disk breakdown

2. Which location?
   [a] Current directory
   [b] Home directory
   [c] System-wide

Your choice (e.g., "1a 2b"): 1b

Command: find ~ -type f -size +100M -exec ls -lh {} \;
```

---

### US-003: Transparent Reasoning

**As a** power user
**I want** to understand why caro is uncertain
**So that** I can rephrase my query more effectively

**Acceptance Criteria:**
- [ ] Reasoning available via `--verbose` or `--debug` flag
- [ ] Shows what caro understood vs. what's unclear
- [ ] Displays confidence score
- [ ] Lists ambiguity factors

**Example:**
```
$ caro --verbose "deploy my app"

Reasoning:
  Understood: User wants to deploy an application
  Unclear:
    - Deployment target (local, cloud, container?)
    - Application type (web, CLI, service?)
    - Deployment method (docker, k8s, rsync?)
  Confidence: 0.25
  Ambiguity Type: missing_context
```

---

### US-004: Graceful Limitations

**As a** user asking about unsupported domains
**I want** caro to explain its limitations
**So that** I know to seek help elsewhere

**Acceptance Criteria:**
- [ ] System detects queries outside its expertise
- [ ] Explains what it can and cannot help with
- [ ] Suggests related queries it can handle
- [ ] Points to external resources when appropriate

**Example:**
```
$ caro write a python script to analyze logs

This request involves writing code, which is outside my expertise.

I can help with:
  [1] Find log files matching a pattern
  [2] Search for errors in log files
  [3] Count occurrences of a pattern
  [4] Tail log files in real-time

For Python scripting, try:
  - GitHub Copilot
  - ChatGPT
  - Your favorite IDE
```

---

### US-005: Clarification Memory

**As a** returning user
**I want** caro to remember my platform preferences
**So that** I don't have to specify my shell every time

**Acceptance Criteria:**
- [ ] Platform preference stored in config file
- [ ] Preference can be set via `caro config set shell powershell`
- [ ] Preference overridable per-query with `--shell` flag
- [ ] Clear indication when using stored preference

**Example:**
```
$ caro config set shell powershell
Shell preference set to: PowerShell

$ caro reload profile
Using stored preference: PowerShell

Command: . $PROFILE
```

---

## Functional Requirements

### FR-001: Ambiguity Detection

| Requirement | Description | Priority |
|-------------|-------------|----------|
| FR-001.1 | Calculate ambiguity score (0.0-1.0) for every query | P1 |
| FR-001.2 | Classify ambiguity type (platform, scope, action, context, domain, safety) | P1 |
| FR-001.3 | Detect platform hints in query text (powershell, bash, zsh, etc.) | P1 |
| FR-001.4 | Extract key intent keywords for analysis | P2 |
| FR-001.5 | Identify safety-critical requests early | P1 |

### FR-002: Clarification Question Generation

| Requirement | Description | Priority |
|-------------|-------------|----------|
| FR-002.1 | Generate 1-3 questions maximum per interaction | P1 |
| FR-002.2 | Each question has 2-4 selectable options | P1 |
| FR-002.3 | Options map directly to command variants | P1 |
| FR-002.4 | Allow freeform input for complex cases | P2 |
| FR-002.5 | Show detected hints with questions | P2 |

### FR-003: Query Enhancement

| Requirement | Description | Priority |
|-------------|-------------|----------|
| FR-003.1 | Combine original query with clarification answers | P1 |
| FR-003.2 | Override platform when explicitly answered | P1 |
| FR-003.3 | Retry generation with enhanced context | P1 |
| FR-003.4 | Limit to 2 clarification rounds maximum | P1 |
| FR-003.5 | Fall back to helpful error after max rounds | P1 |

### FR-004: User Interface

| Requirement | Description | Priority |
|-------------|-------------|----------|
| FR-004.1 | Display questions in clear, boxed format | P1 |
| FR-004.2 | Support single-keystroke answers (1a, 2b) | P1 |
| FR-004.3 | Show "Detected: X" hint when platform identified | P1 |
| FR-004.4 | Display alternatives for cross-platform commands | P2 |
| FR-004.5 | Support `--no-clarify` flag to skip questions | P2 |

### FR-005: Configuration

| Requirement | Description | Priority |
|-------------|-------------|----------|
| FR-005.1 | Store shell preference in ~/.config/caro/config.toml | P2 |
| FR-005.2 | `caro config set shell <shell>` command | P2 |
| FR-005.3 | `caro config get shell` command | P2 |
| FR-005.4 | Per-query override with `--shell <shell>` flag | P2 |
| FR-005.5 | `caro config set clarify.enabled <bool>` | P3 |

---

## Non-Functional Requirements

### NFR-001: Performance

| Requirement | Description | Target |
|-------------|-------------|--------|
| NFR-001.1 | Ambiguity analysis latency | <200ms |
| NFR-001.2 | Question generation latency | <500ms |
| NFR-001.3 | Total clarification round-trip | <2s |
| NFR-001.4 | Enhanced query generation | <2s |

### NFR-002: Reliability

| Requirement | Description | Target |
|-------------|-------------|--------|
| NFR-002.1 | Clarification flow completion rate | >95% |
| NFR-002.2 | Graceful degradation on LLM failure | 100% |
| NFR-002.3 | No infinite clarification loops | 100% |

### NFR-003: Usability

| Requirement | Description | Target |
|-------------|-------------|--------|
| NFR-003.1 | Questions require <10 words to answer | >90% |
| NFR-003.2 | Single interaction clarification | >70% |
| NFR-003.3 | Clear error messages on failure | 100% |

---

## Technical Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CLARIFICATION SYSTEM                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐    ┌──────────────────┐    ┌───────────────┐  │
│  │ Ambiguity        │───▶│ Question         │───▶│ Query         │  │
│  │ Analyzer         │    │ Generator        │    │ Enhancer      │  │
│  │                  │    │                  │    │               │  │
│  │ • Score calc     │    │ • Template-based │    │ • Combine Q+A │  │
│  │ • Type classify  │    │ • LLM-generated  │    │ • Platform    │  │
│  │ • Platform detect│    │ • Validate opts  │    │   override    │  │
│  └──────────────────┘    └──────────────────┘    └───────────────┘  │
│           │                       │                      │           │
│           └───────────────────────┴──────────────────────┘           │
│                                   │                                   │
│                                   ▼                                   │
│                    ┌──────────────────────────┐                      │
│                    │ Clarification UI         │                      │
│                    │                          │                      │
│                    │ • Terminal renderer      │                      │
│                    │ • Input handler          │                      │
│                    │ • State machine          │                      │
│                    └──────────────────────────┘                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Query ──▶ Static Matcher ──(no match)──▶ LLM Generation
                                                   │
                                                   ▼
                                            Confidence Check
                                                   │
                          ┌────────────────────────┴────────────────────┐
                          │                                             │
                    High (>0.7)                                   Low (<0.7)
                          │                                             │
                          ▼                                             ▼
                   Return Command                          Ambiguity Analyzer
                                                                   │
                                                                   ▼
                                                          Question Generator
                                                                   │
                                                                   ▼
                                                          Clarification UI
                                                                   │
                                                                   ▼
                                                          User Answers
                                                                   │
                                                                   ▼
                                                          Query Enhancer
                                                                   │
                                                                   ▼
                                                          LLM Generation (retry)
                                                                   │
                                                                   ▼
                                                          Return Command
```

### Key Files to Create/Modify

| File | Change Type | Description |
|------|-------------|-------------|
| `src/clarification/mod.rs` | New | Module root |
| `src/clarification/analyzer.rs` | New | Ambiguity analysis |
| `src/clarification/questions.rs` | New | Question generation |
| `src/clarification/enhancer.rs` | New | Query enhancement |
| `src/clarification/ui.rs` | New | Terminal UI |
| `src/agent/mod.rs` | Modify | Integrate clarification flow |
| `src/prompts/smollm_prompt.rs` | Modify | Add reasoning output format |
| `src/config.rs` | Modify | Add shell preference |
| `src/backends/static_matcher.rs` | Modify | Add PowerShell patterns |

---

## Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Deliverables:**
1. Ambiguity analyzer with platform detection
2. Basic question templates for common ambiguity types
3. Simple terminal UI for clarification
4. Integration into agent loop

**Success Criteria:**
- [ ] Platform detection works for "powershell", "bash", "zsh", "fish"
- [ ] Clarification UI displays correctly in terminal
- [ ] At least 5 ambiguity scenarios have question templates
- [ ] End-to-end flow works for profile reload example

### Phase 2: LLM Integration (Week 3-4)

**Deliverables:**
1. LLM-based reasoning output format
2. Dynamic question generation from reasoning
3. Query enhancement with LLM context
4. Confidence-based routing

**Success Criteria:**
- [ ] Reasoning JSON output parsed correctly
- [ ] LLM generates relevant clarification questions
- [ ] Enhanced queries produce better commands
- [ ] Confidence threshold tuned for optimal routing

### Phase 3: Polish & Config (Week 5-6)

**Deliverables:**
1. Shell preference configuration
2. `--verbose` reasoning output
3. `--no-clarify` flag
4. Graceful limitations handling
5. Comprehensive testing

**Success Criteria:**
- [ ] All config commands work correctly
- [ ] Verbose mode shows full reasoning
- [ ] No-clarify mode skips to best guess
- [ ] Outside-expertise queries handled gracefully
- [ ] >90% test coverage on clarification module

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LLM reasoning quality varies | High | Medium | Fallback to template questions |
| Clarification loops | Medium | High | Max 2 rounds, force exit after |
| User abandonment during clarification | Medium | Medium | Single-keystroke answers, skip option |
| Platform detection false positives | Low | Medium | Explicit override option |
| Latency increase | High | Medium | Stream questions, cache patterns |

---

## Appendix A: PowerShell Profile Commands

Common PowerShell profile operations for reference:

| Operation | Command |
|-----------|---------|
| Reload current user profile | `. $PROFILE` |
| Reload all profiles | `& $PROFILE.CurrentUserAllHosts` |
| View profile path | `$PROFILE` |
| Edit profile | `notepad $PROFILE` |
| Check if profile exists | `Test-Path $PROFILE` |
| Create profile | `New-Item -Path $PROFILE -Type File -Force` |

---

## Appendix B: Clarification Question Templates

### Platform Ambiguity

```yaml
question_id: shell_type
question: "Which shell are you using?"
options:
  - key: a
    label: PowerShell
    maps_to: powershell_variant
  - key: b
    label: bash
    maps_to: bash_variant
  - key: c
    label: zsh
    maps_to: zsh_variant
  - key: d
    label: Other
    freeform: true
```

### Scope Ambiguity

```yaml
question_id: target_location
question: "Where should I look?"
options:
  - key: a
    label: Current directory
    maps_to: "."
  - key: b
    label: Home directory
    maps_to: "~"
  - key: c
    label: Specific path
    freeform: true
```

### Action Ambiguity

```yaml
question_id: cleanup_type
question: "What kind of cleanup?"
options:
  - key: a
    label: Find large files
    maps_to: find_large
  - key: b
    label: Delete temp files
    maps_to: delete_temp
  - key: c
    label: Show usage breakdown
    maps_to: show_usage
```

---

## Appendix C: Competitive Analysis

| Feature | caro (current) | caro (proposed) | GitHub Copilot CLI | tldr |
|---------|----------------|-----------------|-------------------|------|
| Clarification questions | No | Yes | No | No |
| Platform detection | Partial | Yes | Yes | No |
| Reasoning transparency | No | Yes | No | No |
| Cross-platform commands | No | Yes | Yes | Yes |
| Configuration memory | No | Yes | Yes | No |

---

*This PRD was authored in January 2026 for caro v1.3.0 planning.*
