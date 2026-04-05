# ADR-015: Agent Event System

**Status**: Proposed

**Date**: 2026-04-05

**Authors**: @wildcard

**Target**: Community

## Context

Caro's agent loop (`src/agent/mod.rs`) currently mixes generation logic with output formatting and safety handling. There is no structured way for the CLI or other consumers to observe what the agent loop is doing at each step. This creates several problems:

- **Tight coupling**: The agent loop is bound to CLI output patterns
- **Poor observability**: No structured logging of generation phases
- **Testing friction**: Integration tests must mock CLI output to verify behavior
- **Limited extensibility**: Adding new UIs (TUI, web, IDE plugin) requires modifying agent internals

Research into [SafeRL-Lab/nano-claude-code](https://github.com/SafeRL-Lab/nano-claude-code) revealed a clean pattern: the agent loop yields typed events (`TextChunk`, `ToolStart`, `ToolEnd`, `PermissionRequest`, `TurnDone`) via a generator, and the REPL/UI layer consumes them independently. This pattern maps naturally to Rust enums and callbacks.

Additionally, nano-claude-code's permission-as-event pattern (where the agent loop yields `PermissionRequest` instead of directly prompting the user) provides a cleaner model for integrating safety validation into the agent loop rather than handling it at the CLI layer.

## Decision

Introduce a typed `AgentEvent` enum and callback-based event system for the agent loop. Safety validation will move from the CLI layer into the agent loop, with permission decisions yielded as events.

### Core Design

1. **`AgentEvent` enum** in `src/agent/events.rs` covering all generation phases
2. **Callback pattern**: `AgentLoop` accepts `Option<Box<dyn Fn(AgentEvent) + Send + Sync>>`
3. **Permission events**: Safety validation moves into agent loop; risky commands yield `PermissionRequired` events
4. **CLI consumer**: `src/cli/mod.rs` wires the callback for terminal output and user prompts

## Rationale

- **Rust-native**: Enums are Rust's natural discriminated union — zero-cost abstraction, exhaustive matching, no runtime overhead when callback is `None`
- **Proven pattern**: nano-claude-code demonstrates this works well for AI agent loops
- **Incremental adoption**: Callback is optional; existing code works unchanged until wired
- **Safety improvement**: Moving validation into the agent loop ensures all code paths go through safety checks, not just the CLI path
- **Future-proof**: Event stream can be consumed by TUI, web UI, IDE plugins, or test harnesses

## Consequences

### Benefits

- Clean separation between agent core and UI layer
- Structured observability for debugging and logging
- Testable event sequences without CLI mocking
- Safety validation guaranteed for all consumers, not just CLI
- Foundation for future streaming UI and multi-frontend support

### Trade-offs

- Callback ergonomics in async Rust require care (closures capturing state)
- Slight API surface increase (new `events.rs` module)
- Permission handling becomes async (callback must signal back to agent loop for PermissionRequired)

### Risks

- Callback-based permission requires bidirectional communication → Mitigation: Use `oneshot` channel or `Arc<Mutex<>>` for permission response
- Performance overhead of event emission → Mitigation: Callback is optional; `None` check is branch-predicted away

## Alternatives Considered

### Alternative 1: Channel-based events (mpsc)
- Description: Use `tokio::sync::mpsc` channel instead of callback
- Pros: Natural async pattern, backpressure support
- Cons: Requires consumer task, more complex setup for simple CLI use case

### Alternative 2: Trait-based observer
- Description: Define `trait AgentObserver` with methods per event type
- Pros: Compile-time checked, IDE-friendly
- Cons: More boilerplate, harder to extend with new events (breaking change)

### Alternative 3: No event system (status quo)
- Description: Keep current inline approach
- Pros: No changes needed
- Cons: Perpetuates coupling, blocks multi-frontend support, safety validation stays in CLI

## Implementation Notes

- **Phase 1** (v1.3.0): AgentEvent enum + callback + safety-in-loop
- **Files**: New `src/agent/events.rs`, modify `src/agent/mod.rs` and `src/cli/mod.rs`
- **Testing**: Unit tests asserting event sequence; integration tests for PermissionRequired flow
- **Migration**: Existing behavior preserved when no callback is provided

## Success Metrics

- Zero performance regression when callback is `None`
- All existing tests pass without modification
- Event sequence is deterministic and testable
- Safety validation works identically from agent loop (zero false positives maintained)

## References

- [SafeRL-Lab/nano-claude-code](https://github.com/SafeRL-Lab/nano-claude-code) — Agent loop event yielding pattern
- [#839](https://github.com/wildcard/caro/issues/839) — Add typed AgentEvent system
- [#840](https://github.com/wildcard/caro/issues/840) — Move SafetyValidator into agent loop
- [#841](https://github.com/wildcard/caro/issues/841) — Subprocess timeouts

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-04-05 | @wildcard | Initial draft based on nano-claude-code research |
