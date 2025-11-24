# TUI Testing Strategy for cmdai

**Version:** 1.0
**Date:** 2025-11-19
**Status:** Active

## Executive Summary

This document defines the comprehensive testing strategy for the cmdai Ratatui TUI implementation. The TUI is a safety-critical interface for command generation and validation, requiring thorough testing across unit, integration, and end-to-end levels.

**Current Status:**
- 37 unit tests across 10 modules
- 0 dedicated integration tests for TUI workflows
- ~60-70% estimated coverage of TUI code
- Missing: Integration tests, property tests, error scenarios, async behavior testing

**Priority:** Fill critical gaps in integration testing and error handling before Phase 2 features.

---

## Section 1: Current Coverage Analysis

### 1.1 What's Well-Tested

**State Management (Strong Coverage - ~85%)**
- `AppState` event handling: 9 comprehensive tests
- `ReplState` input operations: 9 tests covering all buffer mutations
- Event conversion and routing: 3 tests
- Mode management: 2 tests

**Component Rendering (Moderate Coverage - ~50%)**
- `StatusBarComponent`: 4 tests (props, colors)
- `HelpFooterComponent`: 5 tests (shortcuts, mode-specific)
- `ReplComponent`: 2 tests (creation, event handling)
- Base `Component` trait: 2 tests

**Strengths:**
- Pure state logic is well-tested and reliable
- Event-to-state transformation has good coverage
- Individual component props and colors are validated
- Key event conversion is tested

### 1.2 What's Missing (Critical Gaps)

**Integration Testing (0% Coverage)**
- Full user workflows (input → generation → validation → display)
- Event flow across multiple components
- State transitions with side effects
- Component coordination and updates

**Error Handling (0% Coverage)**
- Backend failure scenarios
- Network timeout handling
- Invalid state transitions
- Malformed event handling
- Panic recovery in event loop

**Async Operations (0% Coverage)**
- Side effect execution (`SideEffect::GenerateCommand`, `SideEffect::ValidateCommand`)
- Concurrent event handling
- Timeout and cancellation
- Backend response handling

**Terminal Rendering (0% Coverage)**
- Actual widget rendering output
- Layout calculations
- Cursor positioning
- Color and style application
- Text wrapping and overflow

**Edge Cases (10% Coverage)**
- Very long input (>1000 chars)
- Rapid key input (stress testing)
- Terminal resize during operations
- Empty/null states
- Unicode and special characters
- Concurrent state updates

### 1.3 Risk Assessment

| Area | Risk Level | Current Coverage | Impact if Broken | Priority |
|------|------------|-----------------|------------------|----------|
| Command Generation Flow | **CRITICAL** | 0% | System unusable | P0 |
| Validation Flow | **CRITICAL** | 0% | Safety bypass | P0 |
| Error Handling | **HIGH** | 0% | Poor UX, crashes | P0 |
| Side Effect Execution | **HIGH** | 0% | Silent failures | P0 |
| State Management | **MEDIUM** | 85% | Logic errors | P1 |
| Component Rendering | **MEDIUM** | 50% | Visual bugs | P2 |
| Input Handling | **LOW** | 90% | Minor UX issues | P3 |

**Critical Risk Areas:**
1. **No integration tests** - We don't test the system working as a whole
2. **No error testing** - Unknown behavior during failures
3. **No async testing** - Side effects (core functionality) untested
4. **No rendering validation** - Visual output unverified

---

## Section 2: Testing Pyramid

### 2.1 Recommended Test Distribution

```
        ╱╲
       ╱E2E╲         10% - Full TUI workflows (5-8 tests)
      ╱──────╲        - Complete user scenarios
     ╱ Integ. ╲       20% - Multi-component (15-20 tests)
    ╱──────────╲      - Event flows, side effects
   ╱   Unit     ╲     70% - Component/state level (50-60 tests)
  ╱──────────────╲    - Current: 37 tests (need +13-23)
 ╱────────────────╲
```

### 2.2 Current vs. Target

| Test Level | Current Count | Target Count | Gap | Status |
|------------|--------------|--------------|-----|--------|
| Unit | 37 | 50-60 | +13-23 | 🟡 Moderate |
| Integration | 0 | 15-20 | +15-20 | 🔴 Critical |
| E2E | 0 | 5-8 | +5-8 | 🔴 Critical |
| **Total** | **37** | **70-88** | **+33-51** | 🟡 |

### 2.3 Test Type Breakdown

**Unit Tests (70%) - Component/Module Level**
- State reducers (pure functions)
- Event conversions
- Component prop handling
- Individual widget logic
- Helper functions
- Input validation

**Integration Tests (20%) - Multi-Component**
- Event flow across components
- State updates triggering re-renders
- Side effect execution
- Backend integration
- Error propagation
- Async operation coordination

**E2E Tests (10%) - Full Workflows**
- Complete REPL session
- Command generation → validation → execution
- Mode switching scenarios
- Error recovery workflows
- Configuration changes
- Multi-step user interactions

---

## Section 3: Test Types and Approaches

### 3.1 Unit Tests

**Current Approach:** ✅ Good
- Pure function testing
- Component creation and props
- State mutation verification

**Recommended Additions:**
- Boundary value testing (empty, max length, special chars)
- Negative test cases (invalid inputs)
- State transition edge cases
- Component lifecycle testing

**Example:**
```rust
#[test]
fn test_repl_state_max_input_length() {
    let mut state = ReplState::new();
    let long_input = "a".repeat(10000);

    for c in long_input.chars() {
        state.insert_char(c);
    }

    assert_eq!(state.input_buffer.len(), 10000);
    assert!(state.cursor_position <= 10000);
}
```

### 3.2 Integration Tests

**Current Approach:** ❌ Missing entirely

**Required Tests:**
1. **Event Flow Tests** - Verify events propagate correctly through the system
2. **State Transition Tests** - Multiple state changes in sequence
3. **Component Interaction Tests** - How components update based on state
4. **Side Effect Tests** - Async operations complete and update state correctly
5. **Error Flow Tests** - Errors propagate and are handled gracefully

**Location:** `/home/user/cmdai/tests/tui_integration_tests.rs`

### 3.3 Terminal Rendering Tests

**Challenge:** Ratatui rendering is hard to test without a real terminal.

**Approaches:**

**A. Backend Capture Testing (Recommended)**
```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_status_bar_renders_correctly() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        let component = StatusBarComponent::new(/* props */);
        component.render(f, f.size());
    }).unwrap();

    let buffer = terminal.backend().buffer();
    // Verify buffer contains expected text/colors
}
```

**B. Snapshot Testing**
- Capture terminal output as snapshots
- Compare against golden files
- Tool: `insta` crate

**C. Property-Based Rendering**
- Verify rendering doesn't panic
- Verify layout constraints are respected
- Use `proptest` for random props

### 3.4 Async/Side Effect Testing

**Current Approach:** ❌ Side effect handling is stubbed (TODO in app.rs)

**Required Tests:**
```rust
#[tokio::test]
async fn test_command_generation_side_effect() {
    let mut app = create_test_app();

    // Trigger command generation
    app.handle_event(AppEvent::GenerateCommand).await.unwrap();

    // Wait for side effect to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify state was updated
    assert!(app.state.repl.generated_command.is_some());
}
```

**Approach:**
- Use `tokio::test` for async tests
- Mock backend for deterministic responses
- Test timeout scenarios
- Test concurrent operations

### 3.5 Error Handling Tests

**Current Approach:** ❌ No error scenario testing

**Critical Scenarios to Test:**
1. Backend unavailable during startup
2. Backend disconnects mid-operation
3. Invalid JSON responses
4. Timeout during generation
5. Validation errors
6. Malformed user input
7. Terminal resize during render
8. Out-of-memory scenarios (very large inputs)

**Example:**
```rust
#[tokio::test]
async fn test_backend_failure_during_generation() {
    let mut app = create_test_app_with_failing_backend();

    app.state.repl.insert_char('l');
    app.state.repl.insert_char('s');

    app.handle_event(AppEvent::GenerateCommand).await.unwrap();

    // Should gracefully handle failure
    assert!(app.state.repl.generation_error.is_some());
    assert!(!app.state.should_quit); // Should not crash
}
```

### 3.6 Property-Based Tests

**Current Approach:** ❌ Not implemented

**Use Cases:**
- State invariant verification
- Input handling robustness
- Event ordering independence
- Rendering stability

**Example:**
```rust
proptest! {
    #[test]
    fn test_repl_state_invariants(
        inputs in prop::collection::vec(any::<char>(), 0..100)
    ) {
        let mut state = ReplState::new();

        for c in inputs {
            state.insert_char(c);
        }

        // Invariants that should always hold
        assert!(state.cursor_position <= state.input_buffer.len());
        assert_eq!(state.input_buffer.chars().count(), state.input_buffer.len());
    }
}
```

---

## Section 4: Critical Test Scenarios

### 4.1 Must-Have Test Scenarios (P0)

**These scenarios MUST be tested before Phase 2:**

#### Scenario 1: Happy Path - Command Generation
```
User Action → Expected State/Output

1. User types "list all files"
   → input_buffer = "list all files"
   → cursor_position = 14

2. User presses Enter
   → generating = true
   → SideEffect::GenerateCommand emitted

3. Backend responds with "ls -la"
   → generated_command = Some(...)
   → generating = false
   → SideEffect::ValidateCommand emitted

4. Validation completes (Safe)
   → validation_result = Some(Safe)
   → validating = false
   → UI shows green checkmark

**Critical Assertions:**
- State transitions are atomic
- Side effects are executed in order
- UI reflects current state at each step
```

#### Scenario 2: Error Handling - Backend Failure
```
User Action → Expected Behavior

1. User types "list files"
2. User presses Enter
3. Backend returns error (unavailable/timeout)
   → generation_error = Some("Backend unavailable")
   → generating = false
   → Error message displayed in UI
   → App remains responsive (no crash)
   → User can retry or clear input

**Critical Assertions:**
- Errors don't propagate to crash the app
- User sees helpful error message
- State is consistent (no partial updates)
- Can recover and retry
```

#### Scenario 3: Validation - Dangerous Command
```
User Action → Expected Behavior

1. User types "delete everything"
2. Backend generates "rm -rf /"
3. Validation detects critical risk
   → validation_result.risk_level = Critical
   → UI shows red warning icon
   → Warnings displayed: ["This will delete your entire filesystem"]
   → Command not auto-executed

**Critical Assertions:**
- Dangerous commands are flagged
- User sees clear warnings
- Execution requires explicit confirmation
- Safety level is respected
```

#### Scenario 4: State Management - Rapid Input
```
User Action → Expected Behavior

1. User types quickly: "l" "s" " " "-" "l" "a"
2. Each keypress handled correctly
3. No race conditions
4. Final state: input_buffer = "ls -la"

**Critical Assertions:**
- All keypresses registered
- Cursor position accurate
- No dropped events
- No state corruption
```

#### Scenario 5: Input Editing - Cursor Movement
```
User Action → Expected State

1. User types "list files"
   → buffer = "list files", cursor = 10
2. User moves cursor to position 5 (after "list")
3. User presses Backspace
   → buffer = "lis files", cursor = 4
4. User types "t"
   → buffer = "list files", cursor = 5

**Critical Assertions:**
- Cursor movement accurate
- Character insertion at cursor
- Deletion at correct position
```

### 4.2 Important Scenarios (P1)

#### Scenario 6: Mode Switching
```
1. User in REPL mode with partial input
2. User presses Ctrl+H (switch to History mode)
3. History mode displays
4. User presses Esc (back to REPL)
5. Previous input is preserved

**Assertions:**
- Mode switches correctly
- State preserved across modes
- UI updates reflect mode
```

#### Scenario 7: Terminal Resize
```
1. User has command displayed
2. Terminal window resized (80x24 → 120x40)
3. Layout recalculates
4. Content still visible
5. No crashes or corrupted display

**Assertions:**
- Resize event handled
- Layout adapts gracefully
- No content lost
```

#### Scenario 8: Empty Input Handling
```
1. User presses Enter with empty input
2. No generation triggered
3. UI shows placeholder/hint
4. App remains responsive

**Assertions:**
- No unnecessary backend calls
- Clear UX feedback
- No errors logged
```

### 4.3 Edge Cases (P2)

#### Scenario 9: Very Long Input
```
1. User types 1000+ character input
2. UI handles correctly (scrolling/wrapping)
3. Generation succeeds or fails gracefully
4. No memory issues or crashes
```

#### Scenario 10: Unicode and Special Characters
```
1. User types: "find files with emoji 🔍"
2. Input buffer stores correctly
3. Cursor position accurate (accounting for multi-byte chars)
4. Backend handles unicode
```

#### Scenario 11: Concurrent Operations
```
1. User triggers command generation
2. While generating, user presses Ctrl+C to quit
3. Cleanup happens gracefully
4. No hanging async operations
```

#### Scenario 12: Configuration Changes
```
1. User changes safety level from Moderate → Strict
2. Active generation respects new setting
3. Validation becomes more restrictive
4. UI reflects new configuration
```

### 4.4 Performance Scenarios (P3)

#### Scenario 13: Rendering Performance
```
1. Measure frame render time
2. Target: <16ms (60 FPS) for interactive UX
3. Verify no blocking operations in render loop
```

#### Scenario 14: Memory Usage
```
1. Monitor memory during long session
2. Verify no memory leaks
3. Test with large history (1000+ commands)
```

#### Scenario 15: Input Latency
```
1. Measure keystroke → screen update latency
2. Target: <50ms for responsive feel
3. Test under load (backend busy)
```

---

## Section 5: Testing Challenges and Solutions

### 5.1 Challenge: Terminal Rendering Testing

**Problem:**
- Ratatui renders to terminal, hard to capture/verify
- No headless testing mode by default
- Visual output is difficult to assert programmatically

**Solutions:**

**Option 1: TestBackend (Recommended)**
```rust
use ratatui::backend::TestBackend;

#[test]
fn test_component_renders_expected_content() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = create_test_state();

    terminal.draw(|f| {
        TuiApp::render_frame(f, &state);
    }).unwrap();

    let buffer = terminal.backend().buffer();

    // Assert specific positions contain expected text
    assert_eq!(buffer.get(0, 0).symbol(), "⚙");
    assert!(buffer_contains_text(buffer, "Ollama"));
}
```

**Option 2: Snapshot Testing**
- Use `insta` crate for snapshot testing
- Capture buffer as string, compare to golden file
- Good for regression detection

**Option 3: Property-Based Rendering**
- Don't test exact output, test properties:
  - Rendering doesn't panic
  - Layout constraints are satisfied
  - Text fits within bounds

### 5.2 Challenge: Async Event Handling

**Problem:**
- Side effects are async
- Need to coordinate timing in tests
- Race conditions possible

**Solutions:**

**Option 1: Tokio Test Runtime**
```rust
#[tokio::test]
async fn test_async_operation() {
    let mut app = create_test_app();

    app.handle_event(AppEvent::GenerateCommand).await.unwrap();

    // Use timeout to avoid hanging tests
    tokio::time::timeout(
        Duration::from_secs(1),
        wait_for_state_update(&app)
    ).await.unwrap();

    assert!(app.state.repl.generated_command.is_some());
}
```

**Option 2: Mock Async Backend**
- Create deterministic mock backend
- Control timing precisely
- Test timeout scenarios

**Option 3: Channel-Based Testing**
- Use channels to signal completion
- Await specific events
- More explicit control flow

### 5.3 Challenge: Component Interaction Testing

**Problem:**
- Components update based on global state
- Need to test coordination between components
- Hard to isolate without integration tests

**Solutions:**

**Option 1: Integration Test Suite**
- Create dedicated `tests/tui_integration_tests.rs`
- Test multi-component scenarios
- Use `TuiApp` directly, not individual components

**Option 2: State-Based Testing**
- Test state transitions produce correct component updates
- Verify each component renders correctly for given state
- Use `TestBackend` to verify combined output

**Option 3: Event Replay Testing**
- Record sequence of events
- Replay in tests
- Verify final state and rendered output

### 5.4 Challenge: Error Scenario Coverage

**Problem:**
- Many error paths are not obvious
- Failure modes are diverse (network, parsing, validation)
- Need systematic coverage

**Solutions:**

**Option 1: Failure Injection**
```rust
#[tokio::test]
async fn test_handles_backend_timeout() {
    let backend = MockBackend::new()
        .with_timeout(Duration::from_millis(100));

    let mut app = TuiApp::with_backend(backend);

    app.handle_event(AppEvent::GenerateCommand).await.unwrap();

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert!(app.state.repl.generation_error.is_some());
}
```

**Option 2: Chaos Testing**
- Randomly inject failures
- Verify app remains stable
- Use property-based testing

**Option 3: Error Catalog**
- Document all possible error types
- Create test for each error path
- Track coverage in test matrix

### 5.5 Challenge: Test Maintainability

**Problem:**
- TUI tests can be brittle (exact text matching)
- Hard to maintain as UI evolves
- Risk of tests breaking on minor changes

**Solutions:**

**Option 1: Semantic Assertions**
```rust
// Bad - brittle
assert_eq!(buffer.get(0, 0).symbol(), "⚙ Ollama • bash");

// Good - semantic
assert!(status_bar_shows_backend(buffer, "Ollama"));
assert!(status_bar_shows_shell(buffer, "bash"));
```

**Option 2: Test Helpers**
- Create assertion helpers for common checks
- Abstract away buffer manipulation
- Focus on behavior, not implementation

**Option 3: Layered Testing**
- Unit tests for pure logic (stable)
- Integration tests for workflows (moderate stability)
- Minimal E2E tests for critical paths (may change)

---

## Section 6: Implementation Roadmap

### Phase 1: Critical Gaps (P0) - Week 1-2

**Goal:** Achieve basic integration test coverage and error handling

**Tasks:**
1. ✅ Create `/home/user/cmdai/tests/tui_integration_tests.rs`
2. ✅ Implement Scenario 1: Happy path command generation (integration test)
3. ✅ Implement Scenario 2: Backend failure handling (integration test)
4. ✅ Implement Scenario 3: Dangerous command validation (integration test)
5. ✅ Implement side effect execution in `app.rs` (currently TODO)
6. ✅ Add async operation tests
7. ✅ Set up mock backend for testing

**Acceptance Criteria:**
- 5+ integration tests passing
- Side effects actually execute and update state
- Backend failures don't crash the app
- Test coverage increases to ~70%

**Estimated Effort:** 12-16 hours

### Phase 2: Property Tests and Edge Cases (P1) - Week 3

**Goal:** Robust edge case handling and invariant verification

**Tasks:**
1. ✅ Add `proptest` to dev dependencies
2. ✅ Implement property tests for `ReplState` invariants
3. ✅ Implement property tests for event handling
4. ✅ Add edge case tests (long input, unicode, rapid input)
5. ✅ Add terminal resize handling tests
6. ✅ Add mode switching tests

**Acceptance Criteria:**
- 10+ property-based tests
- Edge cases covered (unicode, long input, etc.)
- State invariants verified automatically
- Test coverage ~80%

**Estimated Effort:** 8-12 hours

### Phase 3: Rendering and Visual Tests (P2) - Week 4

**Goal:** Verify visual output and layout

**Tasks:**
1. ✅ Set up `TestBackend` infrastructure
2. ✅ Create rendering assertion helpers
3. ✅ Add component rendering tests
4. ✅ Add layout constraint tests
5. ✅ Optional: Set up snapshot testing with `insta`

**Acceptance Criteria:**
- 8+ rendering tests
- Layout constraints verified
- Component visual output tested
- Test coverage ~85%

**Estimated Effort:** 6-8 hours

### Phase 4: Performance and Stress Tests (P3) - Week 5

**Goal:** Ensure performance and stability under load

**Tasks:**
1. ✅ Add `criterion` benchmarks for rendering
2. ✅ Add `criterion` benchmarks for event handling
3. ✅ Create stress tests (rapid input, long sessions)
4. ✅ Add memory leak detection tests
5. ✅ Profile and optimize hotspots

**Acceptance Criteria:**
- Frame render time <16ms (60 FPS)
- Input latency <50ms
- No memory leaks in long sessions
- Benchmarks in CI

**Estimated Effort:** 6-10 hours

### Phase 5: Continuous Improvement (Ongoing)

**Tasks:**
- Monitor test coverage and add tests for new features
- Refactor brittle tests
- Update test strategy document
- Review failed production scenarios and add regression tests

---

## Section 7: Test Infrastructure Requirements

### 7.1 Dependencies

**Add to `Cargo.toml` `[dev-dependencies]`:**
```toml
# Property-based testing
proptest = "1"

# Snapshot testing (optional)
insta = "1"

# Better assertions
assert_matches = "1.5"
pretty_assertions = "1"

# Async testing helpers (already have tokio-test)
tokio-test = "0.4"

# Mocking
mockall = "0.12"  # For mocking backends
```

### 7.2 Test Organization

```
cmdai/
├── src/
│   └── tui/
│       ├── app.rs (#[cfg(test)] mod tests)
│       ├── state/
│       │   ├── app_state.rs (#[cfg(test)] mod tests)
│       │   └── ...
│       └── components/
│           └── ... (#[cfg(test)] mod tests)
├── tests/
│   ├── tui_integration_tests.rs         # NEW - Integration tests
│   ├── tui_rendering_tests.rs           # NEW - Rendering tests
│   ├── tui_property_tests.rs            # NEW - Property tests
│   └── tui_e2e_tests.rs                 # NEW - E2E scenarios
└── benches/
    └── tui_performance.rs               # NEW - Performance benchmarks
```

### 7.3 CI/CD Integration

**Update `.github/workflows/ci.yml`:**
```yaml
- name: Run TUI tests
  run: |
    cargo test --lib tui
    cargo test --test tui_integration_tests
    cargo test --test tui_property_tests

- name: Run TUI benchmarks (for regression detection)
  run: cargo bench --bench tui_performance --no-run

- name: Check test coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml --lib --tests
    # Upload to codecov or similar
```

### 7.4 Test Helpers and Utilities

**Create `tests/common/tui_helpers.rs`:**
```rust
// Helper functions for TUI tests
pub fn create_test_app() -> TuiApp { ... }
pub fn create_test_state() -> AppState { ... }
pub fn mock_backend() -> MockBackend { ... }
pub fn assert_buffer_contains(buffer: &Buffer, text: &str) { ... }
pub async fn wait_for_state_change<F>(app: &TuiApp, predicate: F)
    where F: Fn(&AppState) -> bool { ... }
```

---

## Section 8: Success Metrics and Quality Gates

### 8.1 Coverage Targets

| Metric | Current | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|--------|---------|---------|---------|---------|---------|
| Line Coverage | ~65% | 70% | 80% | 85% | 90% |
| Branch Coverage | ~50% | 60% | 70% | 75% | 80% |
| Unit Tests | 37 | 45 | 55 | 60 | 65 |
| Integration Tests | 0 | 5 | 12 | 18 | 20 |
| Property Tests | 0 | 0 | 8 | 10 | 12 |
| E2E Tests | 0 | 0 | 0 | 3 | 5 |

### 8.2 Quality Gates (CI Must Pass)

**Pre-Commit:**
- All tests pass (`cargo test`)
- No compiler warnings (`cargo clippy -- -D warnings`)
- Code formatted (`cargo fmt --check`)

**Pre-Merge:**
- All tests pass (unit + integration + property)
- Code coverage ≥ 70% (Phase 1), ≥ 80% (Phase 2+)
- No new unsafe code without review
- Documentation updated

**Pre-Release:**
- All tests pass (including E2E)
- Benchmarks show no performance regression (>10% slower)
- Manual testing checklist completed
- All P0 scenarios tested

### 8.3 Monitoring and Reporting

**Weekly:**
- Review test failures and flaky tests
- Update coverage dashboard
- Review test execution time (keep <5 minutes total)

**Per-PR:**
- Comment with coverage diff
- Flag any decrease in coverage
- Require tests for new features

**Per-Release:**
- Generate test coverage report
- Document known gaps
- Update this strategy document

---

## Section 9: Appendix

### 9.1 Test Naming Conventions

**Unit Tests:**
- `test_<function>_<scenario>_<expected_result>`
- Example: `test_handle_event_text_input_updates_buffer`

**Integration Tests:**
- `test_<workflow>_<scenario>`
- Example: `test_command_generation_happy_path`

**Property Tests:**
- `prop_<invariant>_<condition>`
- Example: `prop_cursor_position_never_exceeds_buffer_length`

**E2E Tests:**
- `e2e_<user_story>`
- Example: `e2e_user_generates_validates_and_executes_command`

### 9.2 Mock Backend Interface

```rust
pub struct MockBackend {
    responses: HashMap<String, CommandGenerationResult>,
    delay: Option<Duration>,
    fail_after: Option<usize>,
}

impl MockBackend {
    pub fn new() -> Self { ... }
    pub fn with_response(mut self, input: &str, output: CommandGenerationResult) -> Self { ... }
    pub fn with_delay(mut self, delay: Duration) -> Self { ... }
    pub fn with_failure_after(mut self, count: usize) -> Self { ... }
}
```

### 9.3 Useful Testing Resources

**Ratatui Testing:**
- https://ratatui.rs/recipes/testing/
- https://github.com/ratatui-org/ratatui/tree/main/tests

**Property-Based Testing:**
- https://proptest-rs.github.io/proptest/
- https://github.com/AltSysrq/proptest

**Async Testing:**
- https://tokio.rs/tokio/topics/testing
- https://docs.rs/tokio-test/

**TUI Testing Examples:**
- https://github.com/orhun/rattop (performance monitoring TUI)
- https://github.com/Rigellute/spotify-tui (Spotify TUI client)

### 9.4 Known Limitations

1. **Terminal-dependent tests** - Some rendering tests require specific terminal capabilities
2. **Timing-sensitive tests** - Async tests may be flaky on slow CI runners
3. **Visual regression** - Hard to detect visual-only bugs without manual testing
4. **Accessibility** - Screen reader compatibility not easily testable

### 9.5 Future Improvements

- **Visual regression testing** - Screenshot comparison for UI changes
- **Accessibility testing** - Automated checks for screen reader support
- **Fuzzing** - AFL/cargo-fuzz for input validation
- **Mutation testing** - cargo-mutants to verify test effectiveness
- **Distributed testing** - Test on multiple terminal emulators (iTerm, Alacritty, etc.)

---

## Document Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | 2025-11-19 | Initial strategy document | QA Expert |

---

**End of Document**
