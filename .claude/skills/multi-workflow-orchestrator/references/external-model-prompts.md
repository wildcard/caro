# External Model Prompt Templates

Expert prompts tuned per model for consistent, parseable output.

## Shared Output Schema

All external model prompts request this output format for reliable parsing:

```markdown
## Summary
<2-3 sentence summary of findings>

## Key Findings
- Finding 1 (with evidence or reasoning)
- Finding 2
- Finding 3

## Recommendations
- Specific, actionable recommendation 1
- Recommendation 2

## Caveats
- Limitations, risks, or edge cases to consider
```

---

## Codex CLI Prompts

### Code Analysis
```
RESEARCH TASK: Analyze the following Rust code for {FOCUS_AREA}.

CONTEXT:
Project: Caro - Rust CLI converting natural language to safe POSIX shell commands
Module: {MODULE_NAME}
Purpose: {MODULE_PURPOSE}

CODE:
```rust
{CODE_CONTENT}
```

Focus your analysis on:
1. Correctness and edge cases
2. Rust idiom adherence (clippy-level)
3. Error handling completeness
4. Performance characteristics
5. Safety implications for command execution

OUTPUT FORMAT: [standard schema above]
```

### Implementation Strategy
```
RESEARCH TASK: Recommend an implementation strategy for {FEATURE_DESCRIPTION}.

CONTEXT:
Project: Caro - Rust CLI (edition 2021, MSRV 1.83)
Architecture: Backend trait system with multiple inference backends (MLX, CPU, Ollama, vLLM, Claude API)
Safety: 52+ regex patterns validating generated commands
Current structure:
{RELEVANT_FILE_TREE}

Key constraints:
- Must maintain zero false positives in safety validation
- POSIX compliance required for generated commands
- Performance target: <1s for command generation

Design the approach considering:
1. Where this fits in the existing architecture
2. Required trait modifications or new traits
3. Testing strategy (TDD preferred)
4. Migration path if modifying existing code

OUTPUT FORMAT: [standard schema above]
```

### Debugging Analysis
```
RESEARCH TASK: Diagnose the root cause of {BUG_DESCRIPTION}.

CONTEXT:
Error message: {ERROR_TEXT}
Relevant code:
```rust
{CODE_CONTENT}
```

Test output:
{TEST_OUTPUT}

Analyze:
1. Most likely root cause
2. Why this bug occurs (mechanism)
3. Recommended fix with code
4. How to prevent similar issues

OUTPUT FORMAT: [standard schema above]
```

---

## Gemini CLI Prompts

### Broad Research
```
RESEARCH TASK: Research {TOPIC} for a Rust CLI tool context.

CONTEXT:
Project: Caro - Rust CLI that converts natural language to safe POSIX shell commands
Language: Rust (edition 2021)
Target platforms: macOS (Apple Silicon + Intel), Linux (x86_64 + ARM64)

Research areas:
1. Current ecosystem state (crates, libraries, tools)
2. Common patterns and best practices
3. Trade-offs between approaches
4. Production readiness and maintenance status of options
5. Community adoption and documentation quality

OUTPUT FORMAT: [standard schema above]
```

### Code Review
```
REVIEW TASK: Review the following Rust code for quality and correctness.

CONTEXT:
Project: Caro - Rust CLI for safe shell command generation
This code is part of: {MODULE_DESCRIPTION}
Review focus: {REVIEW_FOCUS}

CODE:
```rust
{CODE_CONTENT}
```

Review for:
1. Logic errors and edge cases
2. Security vulnerabilities (OWASP top 10 where applicable)
3. Rust best practices (ownership, lifetimes, error handling)
4. Code clarity and maintainability
5. Missing test coverage areas

Format issues as:
### Critical
- [CRIT-N] Description (line ~N) → Suggestion

### Important
- [IMP-N] Description (line ~N) → Suggestion

### Minor
- [MIN-N] Description (line ~N) → Suggestion

### Positive
- What's done well
```

### Documentation Analysis
```
RESEARCH TASK: Analyze documentation and usage patterns for {CRATE_OR_TOOL}.

CONTEXT:
We're evaluating {CRATE_OR_TOOL} for use in Caro (Rust CLI).
Usage: {INTENDED_USE}

Analyze:
1. API stability and versioning policy
2. Documentation completeness
3. Example quality and relevance to our use case
4. Known issues or gotchas from GitHub issues/discussions
5. Maintenance activity (recent commits, response time)
6. Alternative options worth considering

OUTPUT FORMAT: [standard schema above]
```

---

## Prompt Construction Guidelines

### Context Inclusion Rules
1. **Always include**: Project name, language, purpose
2. **Include when relevant**: File tree, module description, constraints
3. **Include for code tasks**: Actual code content (via stdin, never file paths)
4. **Never include**: API keys, secrets, credentials, internal URLs
5. **Size limit**: Cap context at ~4000 tokens per model invocation

### Scoping Rules
1. One focused question per invocation (not the full user request)
2. Match question complexity to model strength:
   - Codex: deep, focused code questions
   - Gemini: broad, comparative research questions
3. Include enough context for the model to give a useful answer
4. Request structured output for reliable parsing

### Security Rules
1. Strip any credentials or secrets from code before sending
2. Use generic descriptions for internal systems
3. Never send file paths -- only file contents
4. Never ask models to execute commands or write files
