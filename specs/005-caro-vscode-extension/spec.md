# Feature Specification: Caro VS Code Extension

**Feature Branch**: `005-caro-vscode-extension`
**Created**: 2025-12-22
**Status**: Draft
**Input**: User description: "VS Code extension interface for Caro with chat panel, context integration, and proactive analysis"

## Execution Flow (main)
```
1. Parse user description from Input
   → Feature: VS Code extension providing accessible interface to Caro core
2. Extract key concepts from description
   → Actors: Developers, DevOps engineers, SREs, infrastructure engineers
   → Actions: Generate commands, analyze files, receive proactive suggestions
   → Data: Natural language prompts, shell commands, file content, safety assessments
   → Constraints: Non-duplicative architecture, local-first, opt-in proactive features
3. For each unclear aspect:
   → IPC protocol: JSON-RPC 2.0 over stdio
   → UI framework: WebView with message passing
   → Proactive scope: Opt-in with file type filtering
4. Fill User Scenarios & Testing section
5. Generate Functional Requirements
6. Identify Key Entities
7. Run Review Checklist (pending)
8. Return: SUCCESS (spec ready for planning)
```

---

## Quick Guidelines
- Focus on WHAT users need and WHY
- Avoid HOW to implement (no tech stack, APIs, code structure)
- Written for business stakeholders, not developers

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a developer working in VS Code, I want to generate shell commands from natural language within my IDE, so I can stay in flow without switching to documentation or terminal experimentation.

### Acceptance Scenarios

1. **Chat Panel Generation**
   - **Given** user has Caro extension installed and Caro binary available
   - **When** user opens Caro chat panel and types "find all TypeScript files modified today"
   - **Then** Caro displays a generated `find` command with explanation and safety level

2. **Command Execution**
   - **Given** Caro has generated a safe command
   - **When** user clicks the "Execute" button
   - **Then** the command runs in VS Code's integrated terminal

3. **Context Menu Integration**
   - **Given** user has selected a JSON parsing code block
   - **When** user right-clicks and selects "Ask Caro About Selection"
   - **Then** Caro suggests using `jq` as a shell alternative

4. **Proactive Dockerfile Analysis**
   - **Given** user has enabled proactive analysis
   - **When** user opens a Dockerfile with `RUN curl | bash`
   - **Then** Caro displays a warning about security risk with suggested fix

5. **Safety Blocking**
   - **Given** user asks for a command that matches high-risk patterns
   - **When** Caro generates `rm -rf /`
   - **Then** command is blocked with explanation and safer alternatives

6. **Offline Operation**
   - **Given** user has no internet connection
   - **When** user requests a command
   - **Then** Caro uses local model and generates command without network

### Edge Cases
- What happens when Caro binary is not found?
- How does system handle very long file content for analysis?
- What error messages appear when the user's shell is unsupported?
- How are simultaneous requests handled?
- What happens when proactive analysis finds 100+ issues in a file?

---

## Requirements *(mandatory)*

### Functional Requirements

**Chat Panel Interface**
- **FR-001**: System MUST provide a chat panel accessible from VS Code Activity Bar
- **FR-002**: System MUST accept natural language input from users
- **FR-003**: System MUST display generated commands with syntax highlighting
- **FR-004**: System MUST show command explanations in human-readable format
- **FR-005**: System MUST display safety level indicator (Safe/Moderate/High/Critical)
- **FR-006**: System MUST provide "Execute" button to run command in terminal
- **FR-007**: System MUST provide "Copy" button to copy command to clipboard
- **FR-008**: System MUST show alternative commands when available

**Context Menu Integration**
- **FR-009**: System MUST add "Caro" submenu to editor context menu
- **FR-010**: System MUST provide "Ask Caro About Selection" when text is selected
- **FR-011**: System MUST provide "Generate Shell Command" for any context
- **FR-012**: System MUST provide "Check Script Safety" for script files
- **FR-013**: System MUST include selected text/file context in requests

**Proactive Analysis**
- **FR-014**: System MUST analyze relevant files when proactive mode enabled
- **FR-015**: System MUST filter files by type (Dockerfile, *.sh, package.json, etc.)
- **FR-016**: System MUST display suggestions as VS Code diagnostics
- **FR-017**: System MUST provide Quick Fix actions for suggestions
- **FR-018**: System MUST categorize suggestions (tool, security, posix, performance)
- **FR-019**: Proactive analysis MUST be opt-in via settings

**Safety Validation**
- **FR-020**: System MUST validate all generated commands for safety
- **FR-021**: System MUST block critical-risk commands with explanation
- **FR-022**: System MUST warn about high-risk commands before execution
- **FR-023**: System MUST suggest safer alternatives when available
- **FR-024**: System MUST log blocked commands for troubleshooting

**Terminal Integration**
- **FR-025**: System MUST execute commands in VS Code integrated terminal
- **FR-026**: System MUST detect active shell type (bash, zsh, fish)
- **FR-027**: System MUST preserve terminal context (working directory)

**Architecture**
- **FR-028**: VS Code extension MUST communicate with Caro Rust core via IPC
- **FR-029**: All inference logic MUST run in Caro core, not extension
- **FR-030**: System MUST support running `caro` directly in terminal alongside extension
- **FR-031**: Extension and CLI MUST share configuration

**Configuration**
- **FR-032**: Users MUST be able to configure safety level preference
- **FR-033**: Users MUST be able to enable/disable proactive analysis
- **FR-034**: Users MUST be able to specify custom Caro binary path
- **FR-035**: Configuration MUST be accessible via VS Code settings

### Non-Functional Requirements

**Performance**
- **NFR-001**: Extension activation MUST complete within 100ms
- **NFR-002**: Command generation MUST complete within 2 seconds
- **NFR-003**: Proactive analysis MUST not block editor UI
- **NFR-004**: Extension MUST use < 100MB RAM baseline

**Reliability**
- **NFR-005**: Extension MUST gracefully handle Caro core crashes
- **NFR-006**: Extension MUST reconnect to Caro core automatically
- **NFR-007**: Extension MUST provide clear error messages on failures

**Security**
- **NFR-008**: Extension MUST NOT send data to external servers without consent
- **NFR-009**: Extension MUST use local model by default
- **NFR-010**: Extension MUST respect VS Code workspace trust settings

**Usability**
- **NFR-011**: Extension MUST follow VS Code UX guidelines
- **NFR-012**: Extension MUST support keyboard navigation
- **NFR-013**: Extension MUST provide meaningful loading states

---

## Key Entities

### User-Facing
- **Chat Panel**: Primary interaction surface for command generation
- **Command Result**: Generated command with metadata (explanation, safety, alternatives)
- **Proactive Suggestion**: Background analysis finding with fix action
- **Safety Assessment**: Risk evaluation of a command

### Internal
- **Caro Client**: Extension component managing IPC to Caro core
- **Request Context**: Editor state passed to Caro (selection, file, cwd)
- **Analysis Target**: File being analyzed for proactive suggestions

---

## Constraints

1. **Non-Duplicative**: All logic in Rust core; extension is thin UI layer
2. **Local-First**: Default to local inference without network dependency
3. **Opt-In Proactive**: Background analysis requires explicit user consent
4. **VS Code API Limits**: Work within VS Code extension sandbox constraints
5. **Cross-Platform**: Support macOS, Linux, Windows via Caro core abstraction

---

## Dependencies

### External
- Caro Rust core binary (`caro --server` mode)
- VS Code 1.85.0 or later
- Node.js runtime (bundled with VS Code)

### Internal
- Safety validation module (existing)
- Command generation backends (existing)
- Context detection (existing)
- Proactive analyzer (new - to be implemented)

---

## Open Questions

1. Should chat history persist across VS Code sessions?
2. Should proactive analysis run on file open, save, or both?
3. How should we handle very large workspaces (1000+ script files)?
4. Should we integrate with VS Code's native terminal shell integration?
5. Should we register as a Chat Participant (@caro) for Copilot integration?

---

## Review Checklist

- [ ] All acceptance scenarios have clear Given/When/Then
- [ ] Non-functional requirements have measurable criteria
- [ ] Security and privacy implications addressed
- [ ] Dependencies identified and available
- [ ] Open questions documented for stakeholder input

---

*Specification Version: 1.0*
*Last Updated: 2025-12-22*
