# Data Model (Discovery Draft)

**Feature**: Unquoted CLI Arguments
**Created**: 2025-12-25
**Status**: Phase 0 Research

This document captures the entities and relationships identified during research for implementing unquoted CLI argument support in caro.

## Entities

### Entity: CLIArguments
- **Description**: Represents the parsed command-line input provided by the user when invoking caro
- **Attributes**:
  - `raw_args` (Vec<String>) – Unprocessed command-line arguments as received from shell
  - `flags` (HashMap<String, Value>) – Parsed flags and options (e.g., --verbose, -p)
  - `trailing_args` (Vec<String>) – Words after all flags, potentially forming the prompt
  - `stdin_content` (Option<String>) – Content piped via stdin, if any
  - `prompt_flag` (Option<String>) – Value of -p/--prompt flag if provided
- **Identifiers**: N/A (transient structure created per invocation)
- **Lifecycle Notes**: Created during CLI initialization, used once, discarded after prompt extraction

### Entity: PromptSource
- **Description**: Enumeration representing where the final prompt text originated
- **Attributes**:
  - `source_type` (enum: Flag | Stdin | TrailingArgs) – Which input method provided the prompt
  - `priority` (u8) – Numeric priority for conflict resolution (Flag=1, Stdin=2, TrailingArgs=3)
  - `is_interactive` (bool) – Whether this source requires interactive confirmation
- **Identifiers**: N/A (enum value)
- **Lifecycle Notes**: Determined once during prompt resolution, informs confirmation workflow

### Entity: ParsedPrompt
- **Description**: The final, validated natural language prompt ready for LLM inference
- **Attributes**:
  - `text` (String) – The prompt text after all processing (whitespace normalization, operator removal)
  - `source` (PromptSource) – Where this prompt came from
  - `is_valid` (bool) – Whether prompt passes validation (non-empty, not whitespace-only)
  - `original_length` (usize) – Character count before processing (for metrics)
  - `processed_length` (usize) – Character count after processing
- **Identifiers**: N/A (output of parsing stage)
- **Lifecycle Notes**: Created after argument parsing, passed to inference backend

### Entity: ShellOperator
- **Description**: Detected shell metacharacter that terminates prompt parsing
- **Attributes**:
  - `operator_char` (String) – The operator sequence (">", "|", "<", ">>", "2>", "&", ";")
  - `position` (usize) – Index in `trailing_args` where operator was found
  - `is_posix` (bool) – Whether this operator follows POSIX shell specification
  - `terminates_prompt` (bool) – Whether this operator should end prompt parsing
- **Identifiers**: N/A (detected during parsing)
- **Lifecycle Notes**: Detected during trailing argument scan, used to determine prompt boundaries

### Entity: ValidationResult
- **Description**: Outcome of prompt validation checks
- **Attributes**:
  - `is_valid` (bool) – Overall validation status
  - `validation_type` (enum: Empty | WhitespaceOnly | HasContent) – Specific validation outcome
  - `should_show_help` (bool) – Whether to display help instead of processing
  - `error_message` (Option<String>) – User-facing error if validation fails
- **Identifiers**: N/A (output of validation stage)
- **Lifecycle Notes**: Created during prompt validation, determines control flow (help vs inference)

## Relationships

| Source | Relation | Target | Cardinality | Notes |
|--------|----------|--------|-------------|-------|
| CLIArguments | produces | ParsedPrompt | 1:1 | Each CLI invocation produces exactly one prompt (or validation error) |
| CLIArguments | detects | ShellOperator | 1:N | May detect zero or more shell operators in trailing args |
| ParsedPrompt | has | PromptSource | 1:1 | Every prompt must have exactly one source type |
| PromptSource | determines | ValidationResult | 1:1 | Source type affects validation (e.g., stdin can be empty) |
| ShellOperator | modifies | CLIArguments | N:1 | Detected operators affect which args become the prompt |

## Data Flow

```
User Input (Shell)
    ↓
CLIArguments (raw parsing)
    ↓
ShellOperator Detection (scan for >, |, etc.)
    ↓
PromptSource Resolution (flag > stdin > trailing args)
    ↓
ParsedPrompt (join, normalize whitespace)
    ↓
ValidationResult (check empty, whitespace-only)
    ↓
[Help Message] OR [LLM Inference]
```

## Validation & Governance

- **Data quality requirements**:
  - Empty prompt input must show help, not error
  - Whitespace normalization: multiple spaces → single space
  - Shell operators must follow POSIX/BSD/GNU conventions
  - Prompt text preserves special characters (*, ?, ~) literally

- **Compliance considerations**:
  - N/A (no PII, no data retention - transient processing only)

- **Source of truth**:
  - clap library for flag parsing (`trailing_var_arg = true`, `num_args = 0..`)
  - POSIX Shell Command Language spec for shell operator definitions
  - Unix tool conventions (cat, grep) for stdin prioritization logic

## Edge Cases Handled by Data Model

1. **Multiple Input Sources**: PromptSource priority ensures deterministic behavior when user provides -p flag AND stdin AND trailing args
2. **Empty Input**: ValidationResult distinguishes between "no input" (show help) vs "invalid input" (error)
3. **Shell Operators Mid-Prompt**: ShellOperator.position tracks where operators appear, allowing prompt truncation
4. **Backward Compatibility**: CLIArguments.trailing_args handles both quoted strings and unquoted words
5. **Whitespace Normalization**: ParsedPrompt tracks original vs processed length for metrics/logging

## Implementation Notes

- **Minimal State**: All entities are stack-allocated, no heap persistence required
- **Error Propagation**: Use Result<ParsedPrompt, ValidationError> throughout parsing pipeline
- **Testing Strategy**: Each entity should have unit tests, plus integration tests for full pipeline
- **Performance**: Argument parsing must complete in <10ms (startup overhead budget)

---

> This model represents Phase 0 understanding. Implementation may refine attribute types, add helper methods, or introduce intermediate structures as needed.
