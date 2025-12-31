# WP02: Tips Engine Core

**Work Package**: WP02
**Status**: planned
**Priority**: high
**Estimated Effort**: 2-3 days
**Depends On**: WP01

## Objective

Build the core tips engine that generates contextual suggestions based on executed commands and local aliases.

## Tasks

### T2.1: Tips Engine Orchestrator
- [ ] Create `src/tips/engine.rs`
- [ ] Implement `TipsEngine` struct with shell environment
- [ ] Add `suggest(&self, command: &str) -> Option<Tip>` method
- [ ] Implement tip frequency/rate limiting logic
- [ ] Track shown tips in current session

### T2.2: Alias Suggester
- [ ] Create `src/tips/suggestions/alias_suggester.rs`
- [ ] Match command against known aliases (exact match)
- [ ] Match command prefix against alias expansions
- [ ] Calculate keystroke savings
- [ ] Generate `AliasSuggestion` struct

### T2.3: Tip Display Formatting
- [ ] Create `src/tips/suggestions/display.rs`
- [ ] Format "Did you know?" message
- [ ] Show alias and expansion
- [ ] Display keystroke savings
- [ ] Colorize output (hint style)
- [ ] Support inline and box styles

### T2.4: Configuration Integration
- [ ] Add `[tips]` section to config schema
- [ ] Implement `TipsConfig` struct
- [ ] Add enabled/disabled toggle
- [ ] Add frequency setting (always/sometimes/rarely/never)
- [ ] Add category filters
- [ ] Add max_per_session limit

### T2.5: Session State Management
- [ ] Create session state tracking
- [ ] Store shown tips in session
- [ ] Implement cooldown logic (don't repeat same tip)
- [ ] Persist session state to file

### T2.6: Integration Tests
- [ ] Test tip suggestion for common commands
- [ ] Test rate limiting behavior
- [ ] Test configuration options
- [ ] Test display formatting

## Acceptance Criteria

- [ ] Running `git status` suggests `gst` alias when available
- [ ] Tips respect frequency settings
- [ ] Same tip not shown twice in session
- [ ] Display is clean and non-intrusive
- [ ] Configuration works as documented
- [ ] Integration tests passing

## Technical Notes

**Suggestion Flow**:
```rust
pub async fn suggest(&self, command: &str) -> Result<Option<Tip>> {
    // Check if tips enabled
    if !self.config.enabled {
        return Ok(None);
    }

    // Check rate limiting
    if !self.should_suggest() {
        return Ok(None);
    }

    // Try alias suggestion
    if let Some(tip) = self.alias_suggester.suggest(command) {
        if !self.was_recently_shown(&tip.id) {
            return Ok(Some(tip));
        }
    }

    Ok(None)
}
```

**Display Format**:
```
> Did you know?
> You have `gst` aliased to `git status` in ~/.zshrc
> Use it to save 7 keystrokes!
```

## Dependencies

- WP01 (Shell Intelligence)
- `colored` crate for terminal colors

## Files to Create

```
src/tips/
├── engine.rs
├── config.rs
└── suggestions/
    ├── mod.rs
    ├── alias_suggester.rs
    └── display.rs
```
