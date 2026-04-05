# Refactor: Extract shared `parse_command_response` from remote backends

**Labels:** tech-debt, refactor

## Summary

All 4 remote backends duplicate the same 4-stage JSON parsing logic in their `parse_command_response` methods. This should be extracted into a shared utility function.

## Duplicated Code Locations

- `src/backends/remote/ollama.rs:105` — `fn parse_command_response`
- `src/backends/remote/vllm.rs:135` — `fn parse_command_response`
- `src/backends/remote/exo.rs:231` — `fn parse_command_response`
- `src/backends/remote/claude.rs:176` — `fn parse_command_response`

Each implementation follows the same 4-stage fallback pattern:
1. Direct `serde_json::from_str` parsing
2. Extract JSON substring from surrounding text
3. Line-by-line `cmd:` pattern matching
4. Regex fallback via `CMD_EXTRACT_REGEX` for malformed JSON

## Impact

- **5 backends** now duplicate this logic (with quant.cpp addition)
- Each new remote backend must copy-paste ~50 lines of parsing code
- Bug fixes to parsing must be applied to all backends independently
- Each backend defines its own `CMD_EXTRACT_REGEX` static

## Proposed Solution

Extract to a shared function in `src/backends/remote/mod.rs` or a new `src/backends/remote/parsing.rs`:

```rust
/// Parse a command response from an LLM with 4-stage fallback
pub fn parse_command_response(response: &str) -> Result<String, GeneratorError> {
    // ... shared implementation
}
```

## Acceptance Criteria

- [ ] Single `parse_command_response` function in shared module
- [ ] Single `CMD_EXTRACT_REGEX` definition
- [ ] All remote backends use the shared function
- [ ] No behavior change (existing tests still pass)
- [ ] Reduce ~250 lines of duplicated code to ~50
