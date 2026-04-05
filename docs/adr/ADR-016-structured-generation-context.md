# ADR-016: Structured Generation Context

**Status**: Proposed

**Date**: 2026-04-05

**Authors**: @wildcard

**Target**: Community

## Context

Caro's agent loop builds context for LLM generation by concatenating strings:

```rust
context: Some(format!("{}{}{}\nSYSTEM_PROMPT:\n{}",
    context_str, dir_context_str, knowledge_context_str, system_prompt))
```

This approach has several problems:

- **Type-unsafe**: Context is an opaque string; backends can't introspect or transform it
- **Not composable**: Adding new context types requires modifying string formatting code
- **Backend-agnostic lost**: Remote backends (Ollama, vLLM) could use structured messages (system/user/assistant roles) instead of a flat string, but the current interface forces everything through `Option<String>`

Research into [nano-claude-code](https://github.com/SafeRL-Lab/nano-claude-code) revealed that it uses a provider-neutral message format, converting to Anthropic or OpenAI wire format only at API boundaries. This enables zero-cost backend switching and cleaner multi-turn support.

## Decision

Replace `context: Option<String>` in `CommandRequest` with a typed `GenerationContext` struct. Each backend converts this struct to its own wire format. Additionally, introduce a neutral `Message` type with roles for remote backends.

### Core Design

```rust
pub struct GenerationContext {
    pub execution: ExecutionContext,
    pub directory: DirectoryContext,
    pub capability: CapabilityProfile,
    pub knowledge: Option<Vec<SimilarCommand>>,
    pub repair: Option<RepairContext>,
}

pub enum MessageRole { System, User, Assistant }
pub struct Message { pub role: MessageRole, pub content: String }
```

This is a **breaking change** to the `CommandGenerator` trait.

## Rationale

- **Type safety**: Context composition is compile-time checked
- **Backend flexibility**: Each backend converts structured context to its optimal wire format
- **Testability**: Context can be inspected and asserted in tests without string parsing
- **Foundation for multi-turn**: Message roles are essential for future conversational mode

## Consequences

### Benefits

- Compile-time verified context composition
- Each backend optimizes context for its API format
- Clean foundation for multi-turn conversation in v2.0.0+
- Eliminates string concatenation bugs

### Trade-offs

- Breaking change to `CommandGenerator` trait — all backends must update
- More complex `CommandRequest` struct
- Conversion logic needed in each backend

### Risks

- Migration effort for all backend implementations → Mitigation: Provide `GenerationContext::to_prompt_string()` helper for backends that just need a flat string
- Context size management → Mitigation: `GenerationContext` can implement truncation per-field

## Alternatives Considered

### Alternative 1: Keep string context, add structured fields alongside
- Description: Add `GenerationContext` as optional alongside existing `context: Option<String>`
- Pros: Non-breaking, gradual migration
- Cons: Two ways to pass context, confusing API, maintenance burden

### Alternative 2: Use serde_json::Value
- Description: Pass context as `serde_json::Value` map
- Pros: Flexible, no breaking change to trait
- Cons: Loses compile-time safety, runtime errors for missing fields

## Implementation Notes

- **Target**: v2.0.0 (breaking change, coordinate with other v2.0 changes)
- **Files**: `src/backends/mod.rs` (trait), all backend implementations, `src/agent/mod.rs`
- **Migration**: Provide `to_prompt_string()` to ease backend migration
- **Testing**: All backend tests updated; add context composition tests

## Success Metrics

- All backends accept structured context
- No string concatenation in agent loop for context building
- Context composition is testable via unit tests
- Remote backends use proper message roles

## References

- [SafeRL-Lab/nano-claude-code](https://github.com/SafeRL-Lab/nano-claude-code) — Provider-neutral message format
- [#842](https://github.com/wildcard/caro/issues/842) — Replace string context with structured GenerationContext
- [#843](https://github.com/wildcard/caro/issues/843) — Provider-neutral message roles
- ADR-015 — Agent Event System (companion ADR)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-04-05 | @wildcard | Initial draft based on nano-claude-code research |
