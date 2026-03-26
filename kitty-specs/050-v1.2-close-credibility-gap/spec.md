# Feature Specification: v1.2.0 Close the Credibility Gap

**Feature Branch**: `v1.2-close-credibility-gap`
**Created**: 2026-03-26
**Status**: In Progress
**Input**: Issues #790, #791 — 40+ features advertised on caro.sh that don't work

## User Scenarios & Testing

### User Story 1 — Flags That Don't Error (Priority: P0)

A developer reads the caro.sh FAQ and tries the documented flags. They expect every flag shown in the documentation to be accepted by the CLI without errors.

**Why this priority**: Users hit immediate hard errors. First impression = last impression. If `--quiet` produces `unexpected argument`, the user uninstalls.

**Acceptance Scenarios**:

1. **Given** the user runs `caro --quiet "list files"`, **When** the command completes, **Then** timing output is suppressed and no error is shown
2. **Given** the user runs `caro -e "echo hello"`, **When** the command completes, **Then** "hello" is printed (equivalent to `--execute`)
3. **Given** the user runs `caro --no-telemetry "list files"`, **When** the command completes, **Then** no telemetry events are recorded and no error is shown
4. **Given** the user runs `caro --backend-info`, **When** the command completes, **Then** all available backends are listed with their availability status
5. **Given** the user runs `caro --explain "find large files"`, **When** the command completes, **Then** a detailed breakdown of the generated command is shown

---

### User Story 2 — Telemetry Subcommands Work (Priority: P0)

A privacy-conscious user reads the telemetry documentation on caro.sh/telemetry and tries the documented subcommands. They expect `caro telemetry show` and `caro telemetry export` to work as described.

**Why this priority**: These subcommands are central to the privacy story. Falling through to command generation is worse than showing an error — it means the CLI is silently ignoring the user's intent.

**Acceptance Scenarios**:

1. **Given** telemetry data exists locally, **When** the user runs `caro telemetry show`, **Then** a human-readable summary of sessions, commands, and backend usage is displayed
2. **Given** telemetry data exists locally, **When** the user runs `caro telemetry export -o data.json`, **Then** a valid JSON file is written with all telemetry data
3. **Given** no telemetry data exists, **When** the user runs `caro telemetry show`, **Then** a "no data" message is shown (not a command generation)

---

### User Story 3 — Config Keys Work (Priority: P0)

A user reads the documentation and tries to configure caro using `caro config set <key> <value>`. They expect all documented keys to be accepted.

**Why this priority**: The skill docs and website show 14+ config keys. Only 4 work. Every other key produces "Unknown config key" — a clear trust violation.

**Acceptance Scenarios**:

1. **Given** the user runs `caro config set telemetry.air_gapped true`, **When** the command completes, **Then** the key is accepted and persisted without error
2. **Given** the user runs `caro config set safety.level strict`, **When** the command completes, **Then** the key is accepted and the safety level changes
3. **Given** the user runs `caro config set backend.primary ollama`, **When** the command completes, **Then** the key is accepted
4. **Given** the user runs `caro config set output.format json`, **When** the command completes, **Then** the key is accepted
5. **Given** the user runs `caro config get telemetry.air_gapped`, **When** the value was previously set, **Then** the correct value is shown
6. **Given** the user sets `CARO_TELEMETRY_ENABLED=false` as an env var, **When** they run caro, **Then** telemetry is disabled for that session

---

### User Story 4 — Safer Alternatives When Blocking (Priority: P1)

A user asks caro to run a dangerous command. When safety validation blocks it, they expect to see a suggestion for what to do instead.

**Why this priority**: The docs and skill say "safer alternatives suggested." The `alternatives` field is always empty. Users who get blocked have no path forward.

**Acceptance Scenarios**:

1. **Given** the user runs `caro "delete everything in root directory"`, **When** safety blocks the command, **Then** a CRITICAL warning is shown with a safer alternative command
2. **Given** the user runs `caro --output json "delete all log files"`, **When** the command is generated, **Then** the `alternatives` field in JSON output contains at least one safer suggestion
3. **Given** the user runs a command that triggers a HIGH risk level, **When** the command is flagged, **Then** a preview command is suggested before the destructive action

---

### User Story 5 — Command Generation Quality (Priority: P1)

A user asks caro common questions like "check disk space" or "delete all log files." They expect to receive correct, useful commands — not `ls -la` or `echo 'Unable to generate command'`.

**Why this priority**: This is the core value proposition. If caro can't generate basic shell commands, it's useless regardless of safety or UX.

**Acceptance Scenarios**:

1. **Given** the user runs `caro "delete all log files"`, **When** the command completes, **Then** a `find`-based command is generated (not `echo 'Unable to generate command'`)
2. **Given** the user runs `caro "check disk space"`, **When** the command completes, **Then** `df -h` is generated (not `ls -la`)
3. **Given** the user runs `caro "show top processes by memory"`, **When** the command completes, **Then** a `ps`-based command is generated
4. **Given** the user runs `caro "find files larger than 100MB"`, **When** the command completes, **Then** `find . -type f -size +100M` is generated
5. **Given** the user runs `caro --shell powershell "list all files"`, **When** the command completes, **Then** PowerShell syntax (`Get-ChildItem` or `dir`) is generated

---

### User Story 6 — Visual Safety Indicators (Priority: P1)

A user runs caro in an interactive terminal. They expect to see colored safety levels with emoji indicators matching what the documentation shows.

**Why this priority**: The skill docs show 🟢🟡🟠🔴 but actual output is plain text. Color-coded output makes safety levels immediately scannable.

**Acceptance Scenarios**:

1. **Given** the user runs `caro "list files"` in an interactive terminal, **When** the output is displayed, **Then** the safety level appears with green color and ✅ emoji
2. **Given** the user runs `caro "list files" | cat` (piped), **When** the output is displayed, **Then** plain text without ANSI codes is shown
3. **Given** the user runs `caro --output json "list files"`, **When** the output is displayed, **Then** JSON output includes a `confidence_score` field with a numeric value

---

### User Story 7 — Documentation Matches Reality (Priority: P2)

A Claude Code user reads the caro-shell-helper skill documentation and follows the examples. Every example, config key, and UX pattern described should work as documented.

**Why this priority**: The skill is the primary way Claude Code users interact with caro. Broken skill docs mean every Claude Code user hits failures.

**Acceptance Scenarios**:

1. **Given** the user reads the skill's TOML config example, **When** they apply those config keys, **Then** every key is accepted without error
2. **Given** the user reads about interactive confirmation, **When** they run caro in a TTY, **Then** the `Execute this command? (y/N)` prompt appears
3. **Given** the user reads the skill's installation checker script, **When** they run it, **Then** it uses the correct crate name (`caro` not `Caro`)
4. **Given** the user reads about keyboard shortcuts during confirmation, **When** those shortcuts are referenced, **Then** they reflect actual CLI behavior

---

## Requirements

### Functional

- **FR-001**: CLI accepts `--quiet`, `-e`, `--no-telemetry`, `--backend-info`, `--explain` flags
- **FR-002**: `caro telemetry show` displays local telemetry stats
- **FR-003**: `caro telemetry export -o <file>` writes JSON telemetry data
- **FR-004**: Config system accepts all 14 documented TOML keys
- **FR-005**: Env var overrides work (`CARO_TELEMETRY_ENABLED`, `CARO_BACKEND`, `CARO_SAFETY`)
- **FR-006**: Safety validation populates `alternatives` field with safer suggestions
- **FR-007**: Static matcher handles 20+ common query categories correctly
- **FR-008**: Embedded model uses improved prompt with fallback chain
- **FR-009**: `--shell powershell` generates Windows/PowerShell syntax
- **FR-010**: ANSI color-coded output in TTY, plain text when piped
- **FR-011**: JSON output includes `confidence_score` field
- **FR-012**: Skill documentation references only working features

### Non-Functional

- **NFR-001**: Zero regression — all existing tests pass
- **NFR-002**: Config backward compatibility — old 4 keys still work
- **NFR-003**: No network requirement for telemetry show/export
- **NFR-004**: Color output respects TTY detection (no ANSI in pipes)

---

## Out of Scope

- #681 (merge conflicts) — assigned to Claude Code
- New features not already documented
- v1.3.0 features (TUI, suggested queries, Starship, etc.)
- Performance improvements beyond prompt engineering
- MCP server implementation
- Knowledge/profile features (behind feature gates)
