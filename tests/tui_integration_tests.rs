//! Integration Tests for TUI
//!
//! These tests verify multi-component workflows and event flows
//! across the entire TUI system.

use cmdai::tui::state::{AppEvent, AppMode, AppState};
use cmdai::tui::state::events::{GeneratedCommandEvent, RiskLevel, ValidationResultEvent};
use cmdai::models::{SafetyLevel, ShellType, UserConfiguration};
use anyhow::Result;

/// Helper to create a test AppState
fn create_test_state() -> AppState {
    let config = UserConfiguration {
        default_shell: Some(ShellType::Bash),
        safety_level: SafetyLevel::Moderate,
        ..Default::default()
    };
    AppState::new(config)
}

// ============================================================================
// INTEGRATION TEST 1: Full Command Generation Flow (Happy Path)
// ============================================================================

#[test]
fn test_integration_command_generation_happy_path() -> Result<()> {
    let mut state = create_test_state();

    // STEP 1: User types input
    state.handle_event(AppEvent::TextInput('l'))?;
    state.handle_event(AppEvent::TextInput('i'))?;
    state.handle_event(AppEvent::TextInput('s'))?;
    state.handle_event(AppEvent::TextInput('t'))?;
    state.handle_event(AppEvent::TextInput(' '))?;
    state.handle_event(AppEvent::TextInput('a'))?;
    state.handle_event(AppEvent::TextInput('l'))?;
    state.handle_event(AppEvent::TextInput('l'))?;

    assert_eq!(state.repl.input_buffer, "list all");
    assert_eq!(state.repl.cursor_position, 8);
    assert!(!state.repl.generating);

    // STEP 2: User presses Enter to trigger generation
    let effects = state.handle_event(AppEvent::Enter)?;

    assert_eq!(effects.len(), 1);
    assert!(state.repl.generating);

    // STEP 3: Simulate backend response with generated command
    let generated = GeneratedCommandEvent {
        command: "ls -la".to_string(),
        explanation: "List all files in long format including hidden files".to_string(),
        risk_level: RiskLevel::Safe,
    };

    let effects = state.handle_event(AppEvent::CommandGenerated(generated.clone()))?;

    // Should trigger validation
    assert_eq!(effects.len(), 1);
    assert!(!state.repl.generating);
    assert!(state.repl.generated_command.is_some());
    assert_eq!(state.repl.generated_command.as_ref().unwrap().command, "ls -la");

    // STEP 4: Simulate validation completion
    let validation = ValidationResultEvent {
        risk_level: RiskLevel::Safe,
        warnings: vec![],
        suggestions: vec![],
        matched_patterns: vec![],
    };

    let effects = state.handle_event(AppEvent::ValidationComplete(validation))?;

    assert_eq!(effects.len(), 0);
    assert!(!state.repl.validating);
    assert!(state.repl.validation_result.is_some());
    assert_eq!(state.repl.validation_result.as_ref().unwrap().risk_level, RiskLevel::Safe);

    // FINAL STATE: Ready for user to execute or modify
    assert_eq!(state.repl.input_buffer, "list all");
    assert!(state.repl.generated_command.is_some());
    assert!(state.repl.validation_result.is_some());
    assert!(state.repl.generation_error.is_none());
    assert!(!state.should_quit);

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 2: Error Handling - Backend Failure
// ============================================================================

#[test]
fn test_integration_backend_failure_graceful_handling() -> Result<()> {
    let mut state = create_test_state();

    // User types input
    state.handle_event(AppEvent::TextInput('l'))?;
    state.handle_event(AppEvent::TextInput('s'))?;

    // User triggers generation
    let effects = state.handle_event(AppEvent::Enter)?;
    assert_eq!(effects.len(), 1);
    assert!(state.repl.generating);

    // Simulate backend failure
    let error_msg = "Backend unavailable: Connection timeout after 5s";
    let effects = state.handle_event(AppEvent::GenerationFailed(error_msg.to_string()))?;

    // Should handle gracefully
    assert_eq!(effects.len(), 0);
    assert!(!state.repl.generating);
    assert!(state.repl.generation_error.is_some());
    assert_eq!(state.repl.generation_error.as_ref().unwrap(), error_msg);
    assert!(state.repl.generated_command.is_none());

    // App should still be responsive
    assert!(!state.should_quit);

    // User can clear and retry
    state.handle_event(AppEvent::ClearInput)?;
    assert_eq!(state.repl.input_buffer, "");
    assert!(state.repl.generation_error.is_none());

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 3: Dangerous Command Validation
// ============================================================================

#[test]
fn test_integration_dangerous_command_validation() -> Result<()> {
    let mut state = create_test_state();

    // User types potentially dangerous query
    for c in "delete all files".chars() {
        state.handle_event(AppEvent::TextInput(c))?;
    }

    // Trigger generation
    state.handle_event(AppEvent::Enter)?;

    // Backend generates dangerous command
    let generated = GeneratedCommandEvent {
        command: "rm -rf /".to_string(),
        explanation: "Recursively delete all files starting from root".to_string(),
        risk_level: RiskLevel::Critical,
    };

    state.handle_event(AppEvent::CommandGenerated(generated))?;

    // Validation flags it as critical
    let validation = ValidationResultEvent {
        risk_level: RiskLevel::Critical,
        warnings: vec![
            "This command will delete your entire filesystem!".to_string(),
            "This is an irreversible operation".to_string(),
        ],
        suggestions: vec![
            "Specify a directory instead of /".to_string(),
            "Use trash instead of permanent deletion".to_string(),
        ],
        matched_patterns: vec!["rm -rf /".to_string()],
    };

    state.handle_event(AppEvent::ValidationComplete(validation.clone()))?;

    // Verify critical risk is flagged
    assert_eq!(
        state.repl.validation_result.as_ref().unwrap().risk_level,
        RiskLevel::Critical
    );
    assert_eq!(
        state.repl.validation_result.as_ref().unwrap().warnings.len(),
        2
    );
    assert_eq!(
        state.repl.validation_result.as_ref().unwrap().suggestions.len(),
        2
    );

    // User should see clear warnings before execution
    assert!(state.repl.generated_command.is_some());
    assert!(state.repl.validation_result.is_some());

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 4: Event Flow - State Transitions
// ============================================================================

#[test]
fn test_integration_state_transitions_are_atomic() -> Result<()> {
    let mut state = create_test_state();

    // Initial state
    assert_eq!(state.current_mode, AppMode::Repl);
    assert!(!state.repl.generating);
    assert!(!state.repl.validating);

    // State transition 1: Typing
    state.handle_event(AppEvent::TextInput('t'))?;
    state.handle_event(AppEvent::TextInput('e'))?;
    state.handle_event(AppEvent::TextInput('s'))?;
    state.handle_event(AppEvent::TextInput('t'))?;

    // Verify atomic state after typing
    assert_eq!(state.repl.input_buffer, "test");
    assert_eq!(state.repl.cursor_position, 4);
    assert!(state.repl.generated_command.is_none());
    assert!(state.repl.generation_error.is_none());

    // State transition 2: Generation start
    state.handle_event(AppEvent::Enter)?;

    assert!(state.repl.generating);
    assert!(!state.repl.validating);
    assert!(state.repl.generated_command.is_none());

    // State transition 3: Generation complete
    let generated = GeneratedCommandEvent {
        command: "echo test".to_string(),
        explanation: "Print 'test' to console".to_string(),
        risk_level: RiskLevel::Safe,
    };
    state.handle_event(AppEvent::CommandGenerated(generated))?;

    assert!(!state.repl.generating);
    assert!(state.repl.generated_command.is_some());
    // Validation should be triggered but not yet complete
    assert!(!state.repl.validating);

    // State transition 4: Validation complete
    let validation = ValidationResultEvent {
        risk_level: RiskLevel::Safe,
        warnings: vec![],
        suggestions: vec![],
        matched_patterns: vec![],
    };
    state.handle_event(AppEvent::ValidationComplete(validation))?;

    assert!(!state.repl.validating);
    assert!(state.repl.validation_result.is_some());

    // Final state is consistent
    assert_eq!(state.repl.input_buffer, "test");
    assert!(state.repl.generated_command.is_some());
    assert!(state.repl.validation_result.is_some());
    assert!(state.repl.generation_error.is_none());
    assert!(!state.repl.generating);
    assert!(!state.repl.validating);

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 5: Input Editing Workflow
// ============================================================================

#[test]
fn test_integration_input_editing_preserves_state_consistency() -> Result<()> {
    let mut state = create_test_state();

    // Type some input
    for c in "hello world".chars() {
        state.handle_event(AppEvent::TextInput(c))?;
    }

    assert_eq!(state.repl.input_buffer, "hello world");
    assert_eq!(state.repl.cursor_position, 11);

    // Delete some characters
    state.handle_event(AppEvent::Backspace)?;
    state.handle_event(AppEvent::Backspace)?;
    state.handle_event(AppEvent::Backspace)?;
    state.handle_event(AppEvent::Backspace)?;
    state.handle_event(AppEvent::Backspace)?;
    state.handle_event(AppEvent::Backspace)?;

    assert_eq!(state.repl.input_buffer, "hello");
    assert_eq!(state.repl.cursor_position, 5);

    // Clear all input
    state.handle_event(AppEvent::ClearInput)?;

    assert_eq!(state.repl.input_buffer, "");
    assert_eq!(state.repl.cursor_position, 0);

    // Type new input
    for c in "new command".chars() {
        state.handle_event(AppEvent::TextInput(c))?;
    }

    assert_eq!(state.repl.input_buffer, "new command");
    assert_eq!(state.repl.cursor_position, 11);

    // State consistency checks
    assert!(state.repl.cursor_position <= state.repl.input_buffer.len());
    assert!(!state.should_quit);
    assert_eq!(state.current_mode, AppMode::Repl);

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 6: Mode Switching (Future Feature)
// ============================================================================

#[test]
fn test_integration_mode_switching_preserves_state() -> Result<()> {
    let mut state = create_test_state();

    // Type some input in REPL mode
    for c in "test input".chars() {
        state.handle_event(AppEvent::TextInput(c))?;
    }

    assert_eq!(state.current_mode, AppMode::Repl);
    assert_eq!(state.repl.input_buffer, "test input");

    // Switch to History mode (future feature)
    state.handle_event(AppEvent::SwitchMode(AppMode::History))?;

    assert_eq!(state.current_mode, AppMode::History);
    // Input should still be preserved
    assert_eq!(state.repl.input_buffer, "test input");

    // Switch back to REPL
    state.handle_event(AppEvent::SwitchMode(AppMode::Repl))?;

    assert_eq!(state.current_mode, AppMode::Repl);
    assert_eq!(state.repl.input_buffer, "test input");

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 7: Quit Handling
// ============================================================================

#[test]
fn test_integration_quit_event_sets_flag() -> Result<()> {
    let mut state = create_test_state();

    assert!(!state.should_quit);

    // User presses Ctrl+C
    state.handle_event(AppEvent::Quit)?;

    assert!(state.should_quit);

    Ok(())
}

// ============================================================================
// INTEGRATION TEST 8: Rapid Input Sequence
// ============================================================================

#[test]
fn test_integration_rapid_input_sequence_no_dropped_events() -> Result<()> {
    let mut state = create_test_state();

    let expected_input = "find all files modified today";

    // Simulate rapid typing
    for c in expected_input.chars() {
        state.handle_event(AppEvent::TextInput(c))?;
    }

    // Verify no characters were dropped
    assert_eq!(state.repl.input_buffer, expected_input);
    assert_eq!(state.repl.cursor_position, expected_input.len());

    // Verify state is consistent
    assert!(state.repl.cursor_position <= state.repl.input_buffer.len());

    Ok(())
}
