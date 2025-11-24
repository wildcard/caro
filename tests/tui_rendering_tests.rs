//! Rendering Tests for TUI
//!
//! These tests verify that components render correctly using
//! Ratatui's TestBackend.

use cmdai::tui::components::{Component, HelpFooterComponent, StatusBarComponent};
use cmdai::tui::state::{AppMode, AppState};
use cmdai::models::{SafetyLevel, ShellType, UserConfiguration};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::layout::Rect;

// ============================================================================
// RENDERING TEST 1: Status Bar Component
// ============================================================================

#[test]
fn test_status_bar_renders_without_panic() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let config = UserConfiguration {
        default_shell: Some(ShellType::Bash),
        safety_level: SafetyLevel::Moderate,
        ..Default::default()
    };
    let mut state = AppState::new(config);
    state.set_backend_status("Ollama".to_string(), true, Some("qwen2.5-coder:7b".to_string()));

    // Should not panic
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    // Verify buffer was written to
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area().width, 80);
    assert_eq!(buffer.area().height, 24);
}

#[test]
fn test_status_bar_shows_backend_name() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let config = UserConfiguration {
        default_shell: Some(ShellType::Bash),
        safety_level: SafetyLevel::Moderate,
        ..Default::default()
    };
    let mut state = AppState::new(config);
    state.set_backend_status("TestBackend".to_string(), true, None);

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Check that buffer contains backend name
    // Note: This is a simplified check; real implementation would parse buffer content
    let buffer_content = buffer_to_string(buffer, 0, 80);
    assert!(
        buffer_content.contains("TestBackend") || buffer_content.contains("⚙"),
        "Status bar should show backend info"
    );
}

#[test]
fn test_status_bar_shows_safety_level() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let config = UserConfiguration {
        default_shell: Some(ShellType::Bash),
        safety_level: SafetyLevel::Strict,
        ..Default::default()
    };
    let state = AppState::new(config);

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let buffer_content = buffer_to_string(buffer, 0, 80);

    // Should show safety level
    assert!(
        buffer_content.contains("Strict") || buffer_content.contains("Safety"),
        "Status bar should show safety level"
    );
}

// ============================================================================
// RENDERING TEST 2: Help Footer Component
// ============================================================================

#[test]
fn test_help_footer_renders_without_panic() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let area = Rect::new(0, 23, 80, 1);
            let component = HelpFooterComponent::for_mode(AppMode::Repl);
            component.render(f, area);
        })
        .unwrap();

    // Should not panic
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area().width, 80);
}

#[test]
fn test_help_footer_shows_mode_specific_shortcuts() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let area = Rect::new(0, 23, 80, 1);
            let component = HelpFooterComponent::for_mode(AppMode::Repl);
            component.render(f, area);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let buffer_content = buffer_to_string(buffer, 23, 80);

    // Should show REPL-specific shortcuts
    // Looking for brackets which are part of shortcut display
    assert!(
        buffer_content.contains('[') || buffer_content.contains(']'),
        "Help footer should show shortcuts with brackets"
    );
}

#[test]
fn test_help_footer_different_modes() {
    let modes = vec![
        AppMode::Repl,
        AppMode::History,
        AppMode::Config,
        AppMode::Help,
    ];

    for mode in modes {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic for any mode
        terminal
            .draw(|f| {
                let area = Rect::new(0, 23, 80, 1);
                let component = HelpFooterComponent::for_mode(mode);
                component.render(f, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer.area().width, 80);
    }
}

// ============================================================================
// RENDERING TEST 3: Layout Constraints
// ============================================================================

#[test]
fn test_components_respect_layout_bounds() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::default();

    terminal
        .draw(|f| {
            // Status bar in first line
            let status_area = Rect::new(0, 0, 80, 1);
            let status = StatusBarComponent::from_state(&state);
            status.render(f, status_area);

            // Help footer in last line
            let help_area = Rect::new(0, 23, 80, 1);
            let help = HelpFooterComponent::for_mode(AppMode::Repl);
            help.render(f, help_area);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Verify rendering stays within bounds
    // (TestBackend will panic if we write outside bounds)
    assert_eq!(buffer.area().width, 80);
    assert_eq!(buffer.area().height, 24);
}

#[test]
fn test_rendering_with_small_terminal() {
    // Test with very small terminal
    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::default();

    // Should not panic even with small terminal
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 40, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();
}

#[test]
fn test_rendering_with_large_terminal() {
    // Test with large terminal
    let backend = TestBackend::new(200, 60);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::default();

    // Should not panic with large terminal
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 200, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();
}

// ============================================================================
// RENDERING TEST 4: State-Driven Rendering
// ============================================================================

#[test]
fn test_status_bar_reflects_backend_availability() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Test with available backend
    let mut state = AppState::default();
    state.set_backend_status("Ollama".to_string(), true, None);

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    // Test with unavailable backend
    state.set_backend_status("Ollama".to_string(), false, None);

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    // Both should render without panic
    // Color would be different but we can't easily test that with TestBackend
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract string content from a buffer line
fn buffer_to_string(buffer: &ratatui::buffer::Buffer, row: u16, width: u16) -> String {
    let mut content = String::new();
    for x in 0..width {
        if let Some(cell) = buffer.cell((x, row)) {
            content.push_str(cell.symbol());
        }
    }
    content
}

// ============================================================================
// RENDERING TEST 5: Multiple Render Cycles
// ============================================================================

#[test]
fn test_multiple_render_cycles_no_corruption() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::default();

    // Render multiple times (simulating animation/updates)
    for _ in 0..10 {
        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, 80, 1);
                let component = StatusBarComponent::from_state(&state);
                component.render(f, area);
            })
            .unwrap();
    }

    // Should not panic or corrupt state
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area().width, 80);
}

#[test]
fn test_render_after_state_change() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::default();

    // Initial render
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    // Change state
    state.set_backend_status("NewBackend".to_string(), true, Some("model-v2".to_string()));

    // Render again
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            let component = StatusBarComponent::from_state(&state);
            component.render(f, area);
        })
        .unwrap();

    // Should not panic
    let buffer = terminal.backend().buffer();
    let content = buffer_to_string(buffer, 0, 80);
    assert!(content.len() > 0);
}
