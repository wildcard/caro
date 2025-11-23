//! Error Handling and Edge Case Tests for TUI
//!
//! These tests verify the TUI handles error scenarios and edge cases gracefully.

use cmdai::tui::state::{AppEvent, AppState, ReplState};
use cmdai::tui::state::events::{GeneratedCommandEvent, RiskLevel, ValidationResultEvent};
use anyhow::Result;

// ============================================================================
// ERROR TEST 1: Empty Input Handling
// ============================================================================

#[test]
fn test_error_empty_input_enter() -> Result<()> {
    let mut state = AppState::default();

    // Press Enter without any input
    let effects = state.handle_event(AppEvent::Enter)?;

    // Should not trigger generation
    assert_eq!(effects.len(), 0);
    assert!(!state.repl.generating);
    assert!(state.repl.generated_command.is_none());

    Ok(())
}

#[test]
fn test_error_empty_input_validation() -> Result<()> {
    let mut state = AppState::default();

    // Try to validate without a generated command
    let effects = state.handle_event(AppEvent::ValidateCommand)?;

    // Should not trigger validation
    assert_eq!(effects.len(), 0);
    assert!(!state.repl.validating);

    Ok(())
}

// ============================================================================
// ERROR TEST 2: Generation Failures
// ============================================================================

#[test]
fn test_error_generation_timeout() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('t');
    state.repl.insert_char('e');
    state.repl.insert_char('s');
    state.repl.insert_char('t');

    // Trigger generation
    state.handle_event(AppEvent::Enter)?;
    assert!(state.repl.generating);

    // Simulate timeout error
    state.handle_event(AppEvent::GenerationFailed(
        "Request timeout after 30 seconds".to_string(),
    ))?;

    assert!(!state.repl.generating);
    assert!(state.repl.generation_error.is_some());
    assert_eq!(
        state.repl.generation_error.as_ref().unwrap(),
        "Request timeout after 30 seconds"
    );
    assert!(state.repl.generated_command.is_none());

    Ok(())
}

#[test]
fn test_error_generation_network_failure() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('l');
    state.repl.insert_char('s');

    state.handle_event(AppEvent::Enter)?;

    // Simulate network error
    state.handle_event(AppEvent::GenerationFailed(
        "Network error: Connection refused".to_string(),
    ))?;

    assert!(!state.repl.generating);
    assert!(state.repl.generation_error.is_some());
    assert!(state.repl.generated_command.is_none());

    // User should be able to retry
    state.handle_event(AppEvent::Enter)?;
    assert!(state.repl.generating);
    assert!(state.repl.generation_error.is_none()); // Error should be cleared

    Ok(())
}

#[test]
fn test_error_generation_invalid_response() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('t');
    state.repl.insert_char('e');
    state.repl.insert_char('s');
    state.repl.insert_char('t');

    state.handle_event(AppEvent::Enter)?;

    // Simulate parsing error
    state.handle_event(AppEvent::GenerationFailed(
        "Invalid JSON response from backend".to_string(),
    ))?;

    assert!(!state.repl.generating);
    assert!(state.repl.generation_error.is_some());

    Ok(())
}

// ============================================================================
// ERROR TEST 3: State Consistency During Errors
// ============================================================================

#[test]
fn test_error_state_consistency_after_generation_failure() -> Result<()> {
    let mut state = AppState::default();

    // Set up initial state
    state.repl.insert_char('l');
    state.repl.insert_char('s');
    let original_input = state.repl.input_buffer.clone();

    state.handle_event(AppEvent::Enter)?;

    // Fail generation
    state.handle_event(AppEvent::GenerationFailed("Backend error".to_string()))?;

    // Input should be preserved
    assert_eq!(state.repl.input_buffer, original_input);
    assert_eq!(state.repl.cursor_position, 2);

    // State should be consistent
    assert!(!state.repl.generating);
    assert!(!state.repl.validating);
    assert!(state.repl.generated_command.is_none());
    assert!(state.repl.validation_result.is_none());
    assert!(state.repl.generation_error.is_some());

    Ok(())
}

#[test]
fn test_error_clearing_input_clears_errors() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('t');
    state.repl.insert_char('e');
    state.repl.insert_char('s');
    state.repl.insert_char('t');

    state.handle_event(AppEvent::Enter)?;
    state.handle_event(AppEvent::GenerationFailed("Error".to_string()))?;

    assert!(state.repl.generation_error.is_some());

    // Clear input
    state.handle_event(AppEvent::ClearInput)?;

    // Error should be cleared
    assert!(state.repl.generation_error.is_none());
    assert!(state.repl.generated_command.is_none());
    assert!(state.repl.validation_result.is_none());

    Ok(())
}

// ============================================================================
// EDGE CASE TEST 4: Very Long Input
// ============================================================================

#[test]
fn test_edge_very_long_input() -> Result<()> {
    let mut state = ReplState::new();

    // Insert 1000 characters
    for i in 0..1000 {
        state.insert_char(if i % 2 == 0 { 'a' } else { 'b' });
    }

    assert_eq!(state.input_buffer.len(), 1000);
    assert_eq!(state.cursor_position, 1000);

    // Should still work
    state.delete_char_before();
    assert_eq!(state.input_buffer.len(), 999);
    assert_eq!(state.cursor_position, 999);

    Ok(())
}

#[test]
fn test_edge_very_long_input_with_unicode() {
    // Test that multi-byte unicode characters are handled correctly
    let mut state = ReplState::new();

    // Insert emoji (multi-byte characters)
    state.insert_char('🚀');
    state.insert_char('🚀');
    state.insert_char('你');
    state.insert_char('好');

    // Should have 4 characters
    assert_eq!(state.input_buffer.chars().count(), 4);
    assert_eq!(state.cursor_position, 4);

    // Backspace should work
    state.delete_char_before();
    assert_eq!(state.input_buffer.chars().count(), 3);
    assert_eq!(state.cursor_position, 3);
}

// ============================================================================
// EDGE CASE TEST 5: Special Characters
// ============================================================================

#[test]
fn test_edge_special_characters() -> Result<()> {
    let mut state = ReplState::new();

    let special_chars = vec![
        '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', ']', '{', '}',
        '\\', '|', ';', ':', '\'', '"', ',', '.', '<', '>', '/', '?', '~', '`',
    ];

    for &c in &special_chars {
        state.insert_char(c);
    }

    assert_eq!(state.input_buffer.chars().count(), special_chars.len());
    assert_eq!(state.cursor_position, special_chars.len());

    Ok(())
}

#[test]
fn test_edge_newline_and_tab_characters() -> Result<()> {
    let mut state = ReplState::new();

    // Insert newline (though UI might filter this)
    state.insert_char('\n');
    state.insert_char('\t');

    // Should be stored
    assert_eq!(state.input_buffer.len(), 2);

    Ok(())
}

#[test]
fn test_edge_null_character() -> Result<()> {
    let mut state = ReplState::new();

    // Insert null character
    state.insert_char('\0');

    // Should be stored (though might be filtered by UI)
    assert_eq!(state.input_buffer.len(), 1);

    Ok(())
}

// ============================================================================
// EDGE CASE TEST 6: Cursor Position Edge Cases
// ============================================================================

#[test]
fn test_edge_backspace_at_start() -> Result<()> {
    let mut state = ReplState::new();

    state.insert_char('a');
    state.cursor_position = 0; // Move cursor to start

    // Backspace at start should do nothing
    state.delete_char_before();

    assert_eq!(state.input_buffer, "a");
    assert_eq!(state.cursor_position, 0);

    Ok(())
}

#[test]
fn test_edge_delete_at_end() -> Result<()> {
    let mut state = ReplState::new();

    state.insert_char('a');
    // Cursor is at end (position 1)

    // Delete at end should do nothing
    state.delete_char_at();

    assert_eq!(state.input_buffer, "a");
    assert_eq!(state.cursor_position, 1);

    Ok(())
}

#[test]
fn test_edge_multiple_backspaces_beyond_start() -> Result<()> {
    let mut state = ReplState::new();

    state.insert_char('a');
    state.insert_char('b');

    // Backspace many times
    for _ in 0..10 {
        state.delete_char_before();
    }

    // Should be empty
    assert_eq!(state.input_buffer, "");
    assert_eq!(state.cursor_position, 0);

    Ok(())
}

// ============================================================================
// EDGE CASE TEST 7: Concurrent State Changes
// ============================================================================

#[test]
fn test_edge_generation_during_validation() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('t');
    state.repl.insert_char('e');
    state.repl.insert_char('s');
    state.repl.insert_char('t');

    // Start generation
    state.handle_event(AppEvent::Enter)?;
    assert!(state.repl.generating);

    // While generating, try to start validation
    // (This shouldn't happen in practice, but test for safety)
    let effects = state.handle_event(AppEvent::ValidateCommand)?;

    // Should not start validation without a command
    assert_eq!(effects.len(), 0);

    Ok(())
}

#[test]
fn test_edge_quit_during_generation() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('t');
    state.repl.insert_char('e');
    state.repl.insert_char('s');
    state.repl.insert_char('t');

    state.handle_event(AppEvent::Enter)?;
    assert!(state.repl.generating);

    // User presses Ctrl+C during generation
    state.handle_event(AppEvent::Quit)?;

    // Should set quit flag
    assert!(state.should_quit);

    // Generation state is irrelevant (app is quitting)

    Ok(())
}

// ============================================================================
// EDGE CASE TEST 8: Validation Edge Cases
// ============================================================================

#[test]
fn test_edge_validation_with_empty_warnings() -> Result<()> {
    let mut state = AppState::default();

    state.repl.set_generated_command(GeneratedCommandEvent {
        command: "ls".to_string(),
        explanation: "List files".to_string(),
        risk_level: RiskLevel::Safe,
    });

    let validation = ValidationResultEvent {
        risk_level: RiskLevel::Safe,
        warnings: vec![],
        suggestions: vec![],
        matched_patterns: vec![],
    };

    state.handle_event(AppEvent::ValidationComplete(validation))?;

    assert!(state.repl.validation_result.is_some());
    assert_eq!(state.repl.validation_result.as_ref().unwrap().warnings.len(), 0);

    Ok(())
}

#[test]
fn test_edge_validation_with_many_warnings() -> Result<()> {
    let mut state = AppState::default();

    state.repl.set_generated_command(GeneratedCommandEvent {
        command: "rm -rf /".to_string(),
        explanation: "Delete everything".to_string(),
        risk_level: RiskLevel::Critical,
    });

    let validation = ValidationResultEvent {
        risk_level: RiskLevel::Critical,
        warnings: vec![
            "Warning 1".to_string(),
            "Warning 2".to_string(),
            "Warning 3".to_string(),
            "Warning 4".to_string(),
            "Warning 5".to_string(),
        ],
        suggestions: vec![
            "Suggestion 1".to_string(),
            "Suggestion 2".to_string(),
        ],
        matched_patterns: vec!["rm -rf /".to_string()],
    };

    state.handle_event(AppEvent::ValidationComplete(validation.clone()))?;

    assert!(state.repl.validation_result.is_some());
    assert_eq!(
        state.repl.validation_result.as_ref().unwrap().warnings.len(),
        5
    );
    assert_eq!(
        state.repl.validation_result.as_ref().unwrap().suggestions.len(),
        2
    );

    Ok(())
}

// ============================================================================
// EDGE CASE TEST 9: Mode Switching Edge Cases
// ============================================================================

#[test]
fn test_edge_mode_switch_during_generation() -> Result<()> {
    let mut state = AppState::default();

    state.repl.insert_char('t');
    state.repl.insert_char('e');
    state.repl.insert_char('s');
    state.repl.insert_char('t');

    state.handle_event(AppEvent::Enter)?;
    assert!(state.repl.generating);

    // Switch modes during generation
    state.handle_event(AppEvent::SwitchMode(cmdai::tui::state::AppMode::History))?;

    // Mode should change
    assert_eq!(state.current_mode, cmdai::tui::state::AppMode::History);

    // Generation state preserved
    assert!(state.repl.generating);

    Ok(())
}

// ============================================================================
// EDGE CASE TEST 10: Resize Event Handling
// ============================================================================

#[test]
fn test_edge_resize_event() -> Result<()> {
    let mut state = AppState::default();

    // Resize event
    let effects = state.handle_event(AppEvent::Resize(100, 40))?;

    // Should be handled gracefully (no state change needed)
    assert_eq!(effects.len(), 0);
    assert!(!state.should_quit);

    Ok(())
}

#[test]
fn test_edge_resize_to_very_small() -> Result<()> {
    let mut state = AppState::default();

    // Resize to very small terminal
    let effects = state.handle_event(AppEvent::Resize(10, 5))?;

    // Should not crash
    assert_eq!(effects.len(), 0);

    Ok(())
}

#[test]
fn test_edge_resize_to_very_large() -> Result<()> {
    let mut state = AppState::default();

    // Resize to very large terminal
    let effects = state.handle_event(AppEvent::Resize(500, 200))?;

    // Should not crash
    assert_eq!(effects.len(), 0);

    Ok(())
}
