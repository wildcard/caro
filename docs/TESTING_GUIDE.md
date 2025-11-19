# TUI Showcase Testing Guide

A practical guide for writing tests for TUI components in the cmdai showcase system.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Writing Component Tests](#writing-component-tests)
3. [Using Test Helpers](#using-test-helpers)
4. [Running Tests](#running-tests)
5. [Test Patterns](#test-patterns)
6. [Common Issues](#common-issues)

## Quick Start

### Running Tests

```bash
# Run all tests
cargo test

# Run only TUI tests
cargo test --lib tui
cargo test --test showcase_registry_tests
cargo test --test browser_integration_tests

# Run tests for a specific component
cargo test --lib tui::components::simple_text

# Run with output
cargo test -- --nocapture

# Run in watch mode (requires cargo-watch)
cargo watch -x test
```

### Test Coverage

```bash
# Generate coverage report (requires cargo-tarpaulin)
cargo tarpaulin --lib --tests --out Html
```

## Writing Component Tests

Every new component should include tests. Here's how to add them:

### Step 1: Add Test Module to Your Component

At the end of your component file (e.g., `/home/user/cmdai/src/tui/components/my_component.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // Tests go here
}
```

### Step 2: Test Metadata

Test that your component's metadata is correct:

```rust
#[test]
fn test_metadata_name() {
    let component = MyComponent;
    assert_eq!(component.metadata().name, "MyComponent");
}

#[test]
fn test_metadata_description() {
    let component = MyComponent;
    assert_eq!(
        component.metadata().description,
        "My component description"
    );
}

#[test]
fn test_metadata_category() {
    let component = MyComponent;
    assert_eq!(component.metadata().category, "Display");
}

#[test]
fn test_metadata_version() {
    let component = MyComponent;
    assert_eq!(component.metadata().version, "1.0.0");
}
```

### Step 3: Test Story Count and Names

```rust
#[test]
fn test_story_count() {
    let component = MyComponent;
    assert_eq!(
        component.stories().len(),
        3,
        "MyComponent should have 3 stories"
    );
}

#[test]
fn test_story_names() {
    let component = MyComponent;
    let stories = component.stories();
    let names: Vec<&str> = stories.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(names, vec!["Default", "WithData", "Error"]);
}

#[test]
fn test_all_stories_have_descriptions() {
    let component = MyComponent;
    let stories = component.stories();

    for story in stories {
        assert!(
            !story.description.is_empty(),
            "Story '{}' should have a description",
            story.name
        );
    }
}
```

### Step 4: Test Rendering

Test that your stories render without panicking:

```rust
#[test]
fn test_all_stories_render_without_panic() {
    let component = MyComponent;
    let stories = component.stories();

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    for story in stories {
        let result = terminal.draw(|frame| {
            (story.render)(frame, frame.size());
        });

        assert!(
            result.is_ok(),
            "Story '{}' should render without errors",
            story.name
        );
    }
}
```

### Step 5: Test Edge Cases

Test rendering with different terminal sizes:

```rust
#[test]
fn test_renders_with_various_sizes() {
    let component = MyComponent;
    let stories = component.stories();

    let sizes = vec![
        (20, 5),    // Very small
        (40, 10),   // Small
        (80, 24),   // Standard
        (120, 40),  // Large
        (200, 60),  // Very large
    ];

    for (width, height) in sizes {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in &stories {
            let result = terminal.draw(|frame| {
                (story.render)(frame, frame.size());
            });

            assert!(
                result.is_ok(),
                "Story '{}' should render at size {}x{}",
                story.name, width, height
            );
        }
    }
}
```

## Using Test Helpers

The test helpers module (`/home/user/cmdai/tests/tui/test_helpers.rs`) provides utilities to make testing easier.

### Available Helpers

#### Creating Test Terminals

```rust
use crate::tui::test_helpers::{create_test_terminal, create_test_terminal_with_size};

// Standard 80x24 terminal
let terminal = create_test_terminal();

// Custom size
let terminal = create_test_terminal_with_size(120, 40);
```

#### Creating Test Areas

```rust
use crate::tui::test_helpers::test_area;

let area = test_area(80, 24);
assert_eq!(area.width, 80);
assert_eq!(area.height, 24);
```

#### Asserting Component Metadata

```rust
use crate::tui::test_helpers::assert_component_metadata;

let component = SimpleTextComponent;
assert_component_metadata(
    &component,
    "SimpleText",
    "Basic text display with various styling options",
    "Display",
    "1.0.0"
);
```

#### Testing All Stories Render

```rust
use crate::tui::test_helpers::assert_all_stories_render;

let component = SimpleTextComponent;
assert_all_stories_render(&component);
```

#### Testing Story Count

```rust
use crate::tui::test_helpers::assert_story_count;

let component = SimpleTextComponent;
assert_story_count(&component, 3);
```

#### Testing Story Names

```rust
use crate::tui::test_helpers::assert_story_names;

let component = SimpleTextComponent;
assert_story_names(&component, &["Default", "Styled", "MultiLine"]);
```

#### Testing Render with Various Sizes

```rust
use crate::tui::test_helpers::test_render_with_various_sizes;

let component = SimpleTextComponent;
let stories = component.stories();

test_render_with_various_sizes(
    |frame, area| (stories[0].render)(frame, area),
    "Default"
);
```

### Mock Components for Testing

When testing the framework itself, use mock components:

```rust
use crate::tui::test_helpers::MockComponent;

let mut registry = ShowcaseRegistry::new();
let component = MockComponent::new("TestComponent")
    .with_stories(5)
    .with_category("Test");

registry.register(Box::new(component));
```

## Running Tests

### Basic Test Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_metadata_name

# Run tests matching a pattern
cargo test metadata

# Run tests in a specific module
cargo test tui::showcase::tests

# Show test output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test

# Run tests serially
cargo test -- --test-threads=1
```

### Watch Mode for Development

```bash
# Install cargo-watch if you haven't already
cargo install cargo-watch

# Run tests on file changes
cargo watch -x test

# Run specific test on changes
cargo watch -x "test test_metadata"

# Clear screen between runs
cargo watch -c -x test
```

### Coverage Reporting

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --lib --tests --out Html

# Open the report
open tarpaulin-report.html  # macOS
xdg-open tarpaulin-report.html  # Linux
```

### CI/CD Testing

Tests run automatically in GitHub Actions on:
- Pull requests touching TUI code
- Pushes to `claude/terminal-ui-storybook-*` branches

View test results in the Actions tab of the GitHub repository.

## Test Patterns

### Pattern 1: Metadata Testing

Always test the four metadata fields:

```rust
#[test]
fn test_metadata() {
    let component = MyComponent;
    let metadata = component.metadata();

    assert_eq!(metadata.name, "MyComponent");
    assert_eq!(metadata.description, "Component description");
    assert_eq!(metadata.category, "Display");
    assert_eq!(metadata.version, "1.0.0");
}
```

### Pattern 2: Story Enumeration

Test story count and names:

```rust
#[test]
fn test_stories() {
    let component = MyComponent;
    let stories = component.stories();

    assert_eq!(stories.len(), 3);

    let names: Vec<_> = stories.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Story1", "Story2", "Story3"]);
}
```

### Pattern 3: Render Smoke Tests

Ensure stories don't panic:

```rust
#[test]
fn test_render_smoke() {
    let component = MyComponent;
    let stories = component.stories();

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    for story in stories {
        terminal.draw(|frame| {
            (story.render)(frame, frame.size());
        }).expect(&format!("Story '{}' should render", story.name));
    }
}
```

### Pattern 4: Size Resilience

Test with various terminal sizes:

```rust
#[test]
fn test_size_resilience() {
    let component = MyComponent;
    let stories = component.stories();

    for (width, height) in [(20, 5), (80, 24), (200, 60)] {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in &stories {
            terminal.draw(|frame| {
                (story.render)(frame, frame.size());
            }).expect(&format!(
                "Story '{}' should render at {}x{}",
                story.name, width, height
            ));
        }
    }
}
```

### Pattern 5: Property-Based Testing

For complex components, use property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_handles_any_terminal_size(
        width in 10u16..300u16,
        height in 5u16..100u16
    ) {
        let component = MyComponent;
        let stories = component.stories();

        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            terminal.draw(|frame| {
                (story.render)(frame, frame.size());
            }).unwrap();
        }
    }
}
```

## Common Issues

### Issue 1: Test Panics with "already borrowed"

**Problem**: Terminal is borrowed multiple times in the same test.

**Solution**: Create a new terminal for each draw call:

```rust
// ❌ Bad
let mut terminal = create_test_terminal();
for story in stories {
    terminal.draw(|frame| { /* ... */ }).unwrap();
}

// ✅ Good
for story in stories {
    let mut terminal = create_test_terminal();
    terminal.draw(|frame| { /* ... */ }).unwrap();
}
```

### Issue 2: Tests Pass Locally but Fail in CI

**Problem**: Tests depend on terminal capabilities not available in CI.

**Solution**: Use `TestBackend` instead of `CrosstermBackend`:

```rust
// ❌ Bad - depends on actual terminal
use ratatui::backend::CrosstermBackend;

// ✅ Good - works in CI
use ratatui::backend::TestBackend;
```

### Issue 3: Test Names Not Descriptive

**Problem**: Hard to understand what failed from test name alone.

**Solution**: Use descriptive test names:

```rust
// ❌ Bad
#[test]
fn test1() { /* ... */ }

// ✅ Good
#[test]
fn test_default_story_renders_without_panic() { /* ... */ }
```

### Issue 4: Tests Are Slow

**Problem**: Tests take too long to run.

**Solutions**:
- Don't test every possible size, pick representative ones
- Use smaller test terminals (e.g., 40x10 instead of 200x60)
- Parallelize with `cargo test --jobs 4`

### Issue 5: Flaky Tests

**Problem**: Tests sometimes pass, sometimes fail.

**Solutions**:
- Remove any timing dependencies
- Don't use random data (or use a fixed seed)
- Ensure tests are independent (no shared state)

## Best Practices

### DO:
- ✅ Test all metadata fields
- ✅ Test story count and names
- ✅ Test rendering doesn't panic
- ✅ Test with various terminal sizes
- ✅ Use descriptive test names
- ✅ Use test helpers to reduce duplication
- ✅ Keep tests focused and simple
- ✅ Test edge cases (empty data, very large data, etc.)

### DON'T:
- ❌ Test ratatui internals
- ❌ Test pixel-perfect rendering (use snapshot tests for that)
- ❌ Create flaky tests with timing dependencies
- ❌ Share state between tests
- ❌ Test implementation details (private methods)
- ❌ Copy-paste test code (use helpers instead)

## Example: Complete Component Test Suite

Here's a complete example for a new component:

```rust
// src/tui/components/my_component.rs

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{Frame, layout::Rect};

pub struct MyComponent;

impl ShowcaseComponent for MyComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new("MyComponent", "A sample component")
            .with_category("Display")
            .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new("Default", "Default state", |frame, area| {
                // Render default state
            }),
            ShowcaseStory::new("WithData", "With sample data", |frame, area| {
                // Render with data
            }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // Metadata tests
    #[test]
    fn test_metadata_name() {
        let component = MyComponent;
        assert_eq!(component.metadata().name, "MyComponent");
    }

    #[test]
    fn test_metadata_description() {
        let component = MyComponent;
        assert_eq!(component.metadata().description, "A sample component");
    }

    #[test]
    fn test_metadata_category() {
        let component = MyComponent;
        assert_eq!(component.metadata().category, "Display");
    }

    #[test]
    fn test_metadata_version() {
        let component = MyComponent;
        assert_eq!(component.metadata().version, "1.0.0");
    }

    // Story tests
    #[test]
    fn test_story_count() {
        let component = MyComponent;
        assert_eq!(component.stories().len(), 2);
    }

    #[test]
    fn test_story_names() {
        let component = MyComponent;
        let stories = component.stories();
        let names: Vec<&str> = stories.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["Default", "WithData"]);
    }

    // Render tests
    #[test]
    fn test_all_stories_render() {
        let component = MyComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                (story.render)(frame, frame.size());
            });

            assert!(
                result.is_ok(),
                "Story '{}' should render without errors",
                story.name
            );
        }
    }

    // Edge case tests
    #[test]
    fn test_renders_with_various_sizes() {
        let component = MyComponent;
        let stories = component.stories();

        let sizes = vec![(20, 5), (80, 24), (120, 40)];

        for (width, height) in sizes {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();

            for story in &stories {
                let result = terminal.draw(|frame| {
                    (story.render)(frame, frame.size());
                });

                assert!(
                    result.is_ok(),
                    "Story '{}' should render at {}x{}",
                    story.name, width, height
                );
            }
        }
    }
}
```

## Next Steps

1. Read the [Testing Strategy](/home/user/cmdai/TESTING_STRATEGY.md) for the overall approach
2. Check out example tests in existing components:
   - `/home/user/cmdai/src/tui/components/simple_text.rs`
   - `/home/user/cmdai/src/tui/components/command_preview.rs`
3. Review test helpers at `/home/user/cmdai/tests/tui/test_helpers.rs`
4. Look at integration tests in `/home/user/cmdai/tests/tui/`

## Getting Help

- Review existing component tests for examples
- Check the TESTING_STRATEGY.md for architectural decisions
- Ask questions in GitHub Discussions
- Open an issue if you find a gap in the testing infrastructure

---

**Remember**: Good tests give us confidence to refactor and add features without breaking existing functionality. Invest time in writing clear, maintainable tests!
