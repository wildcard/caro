# TUI Showcase Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for the cmdai TUI Component Showcase system. Our approach follows the testing pyramid principle with a focus on maintainability, speed, and confidence.

## Testing Goals

1. **Reliability**: Ensure components render correctly without panics
2. **Maintainability**: Make tests easy to write and update
3. **Speed**: Keep the entire test suite under 10 seconds
4. **Coverage**: Achieve >70% code coverage on core framework
5. **Confidence**: Catch regressions before they reach production

## Testing Pyramid

```
        /\
       /  \      E2E (10%)
      /    \     Integration Tests
     /------\    Browser state, navigation flow
    /        \
   /          \  Integration (20%)
  /------------\ Framework tests, registry
 /              \
/________________\ Unit (70%)
  Component tests, metadata, stories
```

### Layer 1: Unit Tests (70% of tests)

**What we test:**
- Component metadata (name, description, category, version)
- Story count and correctness
- Story names and descriptions
- Render functions don't panic
- Edge cases and error conditions

**Example:**
```rust
#[test]
fn test_simple_text_metadata() {
    let component = SimpleTextComponent;
    let metadata = component.metadata();
    assert_eq!(metadata.name, "SimpleText");
    assert_eq!(metadata.category, "Display");
}
```

**Coverage targets:**
- All components: metadata validation
- All components: story enumeration
- All components: render function smoke tests

### Layer 2: Framework Tests (20% of tests)

**What we test:**
- ShowcaseRegistry: add, get, count
- Component registration workflow
- Metadata retrieval and validation
- Story enumeration across components
- State management and lifecycle hooks

**Example:**
```rust
#[test]
fn test_registry_component_registration() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(SimpleTextComponent));
    assert_eq!(registry.len(), 1);
}
```

**Coverage targets:**
- Registry operations: 100%
- Component trait defaults: 100%
- Metadata builders: 100%

### Layer 3: Integration Tests (10% of tests)

**What we test:**
- Browser state transitions
- Navigation flow (ComponentList → StoryList → StoryView)
- Keyboard input handling
- Component initialization/cleanup
- Complete user workflows

**Example:**
```rust
#[test]
fn test_navigation_component_to_story() {
    let mut app = create_test_app();
    assert_eq!(app.view_state, ViewState::ComponentList);

    app.handle_key(KeyCode::Enter);
    assert_eq!(app.view_state, ViewState::StoryList);
}
```

**Coverage targets:**
- State transitions: 100%
- Navigation paths: All major paths
- Error handling: Critical paths

## Test Organization

```
cmdai/
├── src/
│   └── tui/
│       ├── showcase.rs              # Framework code
│       │   └── #[cfg(test)] mod tests  # Framework unit tests
│       └── components/
│           ├── simple_text.rs       # Component code
│           │   └── #[cfg(test)] mod tests  # Component unit tests
│           └── ...
└── tests/
    └── tui/
        ├── showcase_registry_tests.rs    # Framework integration tests
        ├── browser_integration_tests.rs  # Browser integration tests
        └── test_helpers.rs               # Shared test utilities
```

## Test Utilities and Fixtures

### Common Test Helpers

**Location**: `/home/user/cmdai/tests/tui/test_helpers.rs`

```rust
/// Create a minimal test backend for rendering tests
pub fn create_test_backend() -> TestBackend { ... }

/// Create a test frame for component rendering
pub fn create_test_frame() -> Frame { ... }

/// Create a test area with specified dimensions
pub fn test_area(width: u16, height: u16) -> Rect { ... }

/// Assert a component has expected metadata
pub fn assert_metadata(component: &dyn ShowcaseComponent, expected: ComponentMetadata) { ... }

/// Assert a render function doesn't panic
pub fn assert_renders_without_panic(render_fn: &impl Fn(&mut Frame, Rect)) { ... }
```

### Mock Components

For testing the framework, we provide minimal mock components:

```rust
pub struct MockComponent {
    pub name: String,
    pub story_count: usize,
}

impl ShowcaseComponent for MockComponent { ... }
```

## Test Naming Conventions

We follow a consistent naming pattern for clarity:

```rust
// Pattern: test_<what>_<condition>_<expected_result>
#[test]
fn test_metadata_returns_correct_name() { }

#[test]
fn test_stories_count_matches_expected() { }

#[test]
fn test_render_with_small_area_does_not_panic() { }

#[test]
fn test_navigation_from_component_list_to_story_list() { }
```

## Testing Patterns

### 1. Metadata Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let component = SimpleTextComponent;
        let metadata = component.metadata();

        assert_eq!(metadata.name, "SimpleText");
        assert_eq!(metadata.description, "Basic text display with various styling options");
        assert_eq!(metadata.category, "Display");
        assert_eq!(metadata.version, "1.0.0");
    }
}
```

### 2. Story Enumeration

```rust
#[test]
fn test_stories_count() {
    let component = SimpleTextComponent;
    let stories = component.stories();

    assert_eq!(stories.len(), 3, "SimpleText should have exactly 3 stories");
}

#[test]
fn test_story_names() {
    let component = SimpleTextComponent;
    let stories = component.stories();

    let names: Vec<_> = stories.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Default", "Styled", "MultiLine"]);
}
```

### 3. Render Smoke Tests

```rust
#[test]
fn test_render_does_not_panic() {
    let component = SimpleTextComponent;
    let stories = component.stories();

    // Create a test backend
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Test each story renders without panicking
    for story in stories {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            }).unwrap();
        }));

        assert!(result.is_ok(), "Story '{}' panicked during render", story.name);
    }
}
```

### 4. State Transition Testing

```rust
#[test]
fn test_state_transitions() {
    let mut app = create_test_app();

    // Initial state
    assert_eq!(app.view_state, ViewState::ComponentList);

    // Transition to story list
    app.handle_key(KeyCode::Enter);
    assert_eq!(app.view_state, ViewState::StoryList);

    // Transition to story view
    app.handle_key(KeyCode::Enter);
    assert_eq!(app.view_state, ViewState::StoryView);

    // Back to story list
    app.handle_key(KeyCode::Backspace);
    assert_eq!(app.view_state, ViewState::StoryList);
}
```

## Property-Based Testing

For complex validation, we use `proptest`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_registry_handles_any_number_of_components(count in 0..100usize) {
        let mut registry = ShowcaseRegistry::new();

        for i in 0..count {
            registry.register(Box::new(MockComponent::new(&format!("Component{}", i))));
        }

        assert_eq!(registry.len(), count);
    }
}
```

## Running Tests

### All Tests
```bash
cargo test
```

### TUI Tests Only
```bash
cargo test --test showcase_registry_tests
cargo test --test browser_integration_tests
```

### Component Tests Only
```bash
cargo test --lib tui::components::simple_text
```

### With Coverage
```bash
cargo tarpaulin --lib --tests --out Html
```

### Watch Mode (Development)
```bash
cargo watch -x test
```

## CI/CD Integration

Tests are integrated into the GitHub Actions workflow at `/home/user/cmdai/.github/workflows/tui-visual-testing.yml`:

1. **Test Execution Step** (runs before build):
   ```yaml
   - name: Run TUI component tests
     run: |
       echo "🧪 Running TUI component test suite..."
       cargo test --lib tui
       cargo test --test showcase_registry_tests
       cargo test --test browser_integration_tests
       echo "✓ All tests passed!"
   ```

2. **Failure Handling**:
   - Build fails if any test fails
   - Test output shown in logs
   - Coverage reports uploaded as artifacts

3. **Performance Monitoring**:
   - Track test execution time
   - Alert if suite exceeds 10s threshold

## Coverage Goals

| Component | Target | Current |
|-----------|--------|---------|
| showcase.rs (framework) | 90% | - |
| ShowcaseRegistry | 100% | - |
| ComponentMetadata | 100% | - |
| ShowcaseStory | 80% | - |
| Individual components | 70% | - |
| Browser app logic | 80% | - |

## Testing Anti-Patterns to Avoid

1. **Don't test implementation details**
   - ❌ Testing private helper functions
   - ✅ Testing public API behavior

2. **Don't duplicate test logic**
   - ❌ Copy-pasting test code
   - ✅ Use test helpers and fixtures

3. **Don't test ratatui internals**
   - ❌ Testing ratatui widget rendering details
   - ✅ Testing our component abstraction

4. **Don't create flaky tests**
   - ❌ Tests that depend on timing or randomness
   - ✅ Deterministic, repeatable tests

## Future Enhancements

### Visual Regression Testing (Phase 2)

Use snapshot testing to detect visual changes:

```rust
#[test]
fn test_visual_snapshot() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        let component = SimpleTextComponent;
        let stories = component.stories();
        (stories[0].render)(frame, frame.size());
    }).unwrap();

    let snapshot = terminal.backend().buffer().clone();
    insta::assert_snapshot!(format!("{:?}", snapshot));
}
```

**Dependencies needed:**
- `insta` for snapshot testing
- Custom buffer formatter for human-readable snapshots

### Performance Testing (Phase 3)

Benchmark component rendering performance:

```rust
#[bench]
fn bench_component_render(b: &mut Bencher) {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let component = SimpleTextComponent;
    let story = &component.stories()[0];

    b.iter(|| {
        terminal.draw(|frame| {
            (story.render)(frame, frame.size());
        }).unwrap();
    });
}
```

## Maintenance Guidelines

1. **Add tests for new components**
   - Every new component must include metadata tests
   - Every new component must include story enumeration tests
   - At least smoke tests for render functions

2. **Update tests when APIs change**
   - Keep test utilities in sync with framework changes
   - Update documentation when patterns change

3. **Review test failures**
   - Don't ignore failing tests
   - Don't disable tests without understanding why they fail
   - Fix or remove flaky tests immediately

4. **Monitor test suite performance**
   - Keep total execution time under 10s
   - Optimize slow tests
   - Use parallel execution where possible

## Questions?

For questions about testing strategy or implementing tests:
1. Review this document
2. Check existing test examples in `/home/user/cmdai/tests/tui/`
3. Review the Testing Guide at `/home/user/cmdai/docs/TESTING_GUIDE.md`
4. Ask in GitHub Discussions

## Summary

This testing strategy ensures the TUI showcase system remains reliable and maintainable as it grows. By following the testing pyramid, using consistent patterns, and maintaining good coverage, we can confidently iterate on components and add new features.

**Key Takeaways:**
- 70% unit tests, 20% framework tests, 10% integration tests
- All components must have metadata and story tests
- Tests should be fast, deterministic, and maintainable
- Use test helpers to avoid duplication
- Integrate tests into CI/CD pipeline
- Aim for >70% coverage on core framework
