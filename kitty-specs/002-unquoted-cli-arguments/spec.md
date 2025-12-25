# Feature Specification: Unquoted CLI Arguments

**Feature Branch**: `002-unquoted-cli-arguments`
**Created**: 2025-12-25
**Status**: Draft
**Input**: User description: "Allow users to type natural language prompts without quotes while maintaining backward compatibility"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Unquoted Prompts (Priority: P1)

Users can type natural language prompts directly without wrapping them in quotes, making the CLI feel more natural and reducing friction.

**Why this priority**: This is the core feature that provides the primary value - reducing cognitive load and typing overhead for the most common use case.

**Independent Test**: Can be fully tested by running `caro list files` and verifying it generates a command, delivering immediate value even if other features aren't implemented.

**Acceptance Scenarios**:

1. **Given** a user wants to list files, **When** they type `caro list files`, **Then** the system treats "list files" as the prompt and generates an appropriate command
2. **Given** a user types a multi-word prompt, **When** they run `caro find large files in current directory`, **Then** the system treats the entire phrase as a single prompt
3. **Given** a user includes flags before the prompt, **When** they run `caro --verbose list all files`, **Then** `--verbose` is treated as a flag and "list all files" becomes the prompt

---

### User Story 2 - Backward Compatibility with Quoted Prompts (Priority: P1)

Existing users who use quoted prompts continue to work without any changes to their workflow.

**Why this priority**: Breaking existing usage would cause immediate pain for current users. This must work from day one to prevent regressions.

**Independent Test**: Can be tested by running existing quoted commands like `caro "list files"` and verifying behavior is unchanged.

**Acceptance Scenarios**:

1. **Given** a user has existing scripts with quoted prompts, **When** they run `caro "list files"`, **Then** the command works exactly as before
2. **Given** a quoted prompt with special characters, **When** user runs `caro "find *.txt files"`, **Then** special characters in quotes are preserved
3. **Given** mixed usage, **When** a user sometimes uses quotes and sometimes doesn't, **Then** both work correctly without conflict

---

### User Story 3 - Non-Interactive Mode with -p/--prompt Flag (Priority: P2)

Power users and automation scripts can use the `-p` or `--prompt` flag to provide prompts explicitly and receive output without interactive confirmation.

**Why this priority**: Critical for scripting and automation use cases, but secondary to basic interactive usage.

**Independent Test**: Can be tested by running `caro -p "list files"` and verifying it outputs the command without interactive prompts.

**Acceptance Scenarios**:

1. **Given** a user wants to script caro usage, **When** they run `caro -p "list files"`, **Then** the generated command is output to stdout without interactive confirmation
2. **Given** a user provides both `-p` and trailing arguments, **When** they run `caro -p "list files" extra text`, **Then** only the `-p` argument value is used as the prompt
3. **Given** automation requires JSON output, **When** user runs `caro -p "list files" --output json`, **Then** the command is output in JSON format without interaction

---

### User Story 4 - Stdin Input Support (Priority: P2)

Users can pipe prompts into caro from other commands or scripts, enabling pipeline composition.

**Why this priority**: Enhances composability with other Unix tools, but less common than direct command-line usage.

**Independent Test**: Can be tested by running `echo "list files" | caro` and verifying it reads from stdin.

**Acceptance Scenarios**:

1. **Given** a user pipes a prompt via stdin, **When** they run `echo "list files" | caro`, **Then** caro reads the prompt from stdin and generates a command
2. **Given** stdin input is provided, **When** user also provides command-line arguments, **Then** stdin takes precedence (following Unix conventions)
3. **Given** empty stdin, **When** user runs `echo "" | caro`, **Then** the system shows help message (same as empty command-line input)

---

### User Story 5 - Shell Operator Handling (Priority: P3)

Shell operators (pipes, redirects, etc.) in the command line are correctly distinguished from prompt text, following BSD/GNU best practices.

**Why this priority**: Important for advanced users but less common than basic usage. Most users won't encounter this edge case.

**Independent Test**: Can be tested by running `caro list files > /tmp/output.txt` and verifying the redirect applies to caro's output, not the prompt.

**Acceptance Scenarios**:

1. **Given** a user wants to redirect output, **When** they run `caro list files > output.txt`, **Then** "list files" is the prompt and `> output.txt` redirects caro's generated command
2. **Given** a user wants to pipe caro output, **When** they run `caro find files | grep txt`, **Then** "find files" is the prompt and `| grep txt` pipes caro's output
3. **Given** shell operators in various positions, **When** user runs commands with `>`, `|`, `<`, `>>`, `2>`, etc., **Then** these operators are treated as shell constructs, not prompt text

---

### Edge Cases

- **Empty input**: Running `caro` with no arguments shows help message (not an error)
- **Whitespace-only input**: Running `caro    ` (only spaces) shows help message
- **Mixed flags and prompt**: `caro --dry-run --shell zsh list files` correctly parses flags before treating remaining text as prompt
- **Special characters in unquoted prompts**: Characters like `*` or `?` are treated literally in the prompt, not as shell glob patterns
- **Very long prompts**: System handles prompts with hundreds of words without buffer overflow or truncation
- **Conflicting input methods**: When both stdin and command-line prompt are provided, stdin takes precedence per Unix conventions
- **Quote characters within unquoted text**: `caro find John's files` handles apostrophes correctly
- **Tab characters and newlines**: Whitespace normalization handles tabs and embedded newlines in prompts

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept unquoted text after all flags as the prompt (e.g., `caro list files`)
- **FR-002**: System MUST maintain backward compatibility with quoted prompts (e.g., `caro "list files"`)
- **FR-003**: System MUST parse command-line arguments with flags before the prompt (e.g., `caro --verbose list files`)
- **FR-004**: System MUST show help message when invoked with no arguments or whitespace-only input
- **FR-005**: System MUST support `-p` or `--prompt` flag for explicit prompt specification
- **FR-006**: System MUST support reading prompts from stdin when available
- **FR-007**: System MUST treat shell operators (`>`, `|`, `<`, `>>`, `2>`, `&`, `;`) as terminating the prompt, not part of it
- **FR-008**: System MUST prioritize input sources in this order: (1) `-p`/`--prompt` flag, (2) stdin, (3) trailing arguments
- **FR-009**: System MUST join multiple trailing argument words with spaces to form a single prompt
- **FR-010**: System MUST preserve special characters within the prompt text (e.g., `*`, `?`, `~`)
- **FR-011**: All existing tests from the codebase MUST pass on Linux, macOS, and Windows
- **FR-012**: System MUST fix test failures identified in PR #68 during implementation

### Key Entities

- **Prompt Source**: Represents where the prompt text originates (flag, stdin, or trailing args)
- **Argument Parser**: Distinguishes between flags/options and prompt text in command-line input
- **Shell Operator Detector**: Identifies shell metacharacters that terminate prompt parsing

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can type unquoted prompts and receive correct command generation 100% of the time for simple cases (2-5 word prompts)
- **SC-002**: Backward compatibility maintained - all existing quoted prompt usage continues to work without modification
- **SC-003**: All platform-specific tests (Linux, macOS, Windows) pass without failures
- **SC-004**: Help message displays correctly when caro is invoked with no arguments or whitespace-only
- **SC-005**: Non-interactive mode with `-p` flag outputs commands without requiring user confirmation
- **SC-006**: Stdin input processing works correctly for piped commands
- **SC-007**: Shell operators are correctly distinguished from prompt text in 100% of test cases covering `>`, `|`, `<`, `>>`, `2>`, `&`, `;`

## Assumptions

1. **Shell Operator Detection**: We follow BSD/GNU conventions for recognizing shell metacharacters as defined in POSIX shell specification
2. **Input Prioritization**: The order (flag > stdin > args) follows standard Unix tool conventions (e.g., how `cat`, `grep` handle multiple input sources)
3. **Whitespace Normalization**: Multiple consecutive spaces in prompts are collapsed to single spaces (standard text processing behavior)
4. **PR #68 Investigation**: Test failures will be diagnosed during implementation; root causes may include argument parsing issues, test environment differences, or edge case handling

## Out of Scope

- Auto-completion for unquoted prompts (future enhancement)
- Syntax highlighting or validation of prompts before submission
- Support for multi-line prompts via unquoted arguments (stdin or `-p` flag should be used for complex multi-line input)
- Custom shell operator configuration (we use POSIX standard)
