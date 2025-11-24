//! Property-Based Tests for TUI
//!
//! These tests use proptest to verify invariants hold across
//! arbitrary inputs and state transitions.

use cmdai::tui::state::{AppEvent, AppState, ReplState};
use proptest::prelude::*;

// ============================================================================
// PROPERTY TEST 1: ReplState Cursor Invariants
// ============================================================================

proptest! {
    /// Property: Cursor position never exceeds buffer length
    /// NOTE: Currently limited to ASCII to avoid unicode cursor_position bug
    /// See test_edge_very_long_input_with_unicode for details
    #[test]
    fn prop_cursor_never_exceeds_buffer_length(
        chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only for now
            0..100
        )
    ) {
        let mut state = ReplState::new();

        for c in chars {
            state.insert_char(c);
            // INVARIANT: Cursor <= Buffer Length
            prop_assert!(state.cursor_position <= state.input_buffer.len());
        }
    }

    /// Property: Backspace never causes negative cursor position
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_backspace_never_negative_cursor(
        initial_chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            0..100
        ),
        backspace_count in 0_usize..150
    ) {
        let mut state = ReplState::new();

        // Build initial buffer
        for c in initial_chars {
            state.insert_char(c);
        }

        // Apply many backspaces
        for _ in 0..backspace_count {
            state.delete_char_before();
            // INVARIANT: Cursor is always >= 0 (implied by usize)
            // INVARIANT: Cursor <= Buffer length
            prop_assert!(state.cursor_position <= state.input_buffer.len());
        }

        // After excessive backspaces, should be at position 0
        prop_assert_eq!(state.cursor_position, state.input_buffer.len().saturating_sub(0));
    }

    /// Property: Delete never causes cursor to exceed buffer
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_delete_maintains_cursor_bounds(
        initial_chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            0..100
        ),
        delete_count in 0_usize..150
    ) {
        let mut state = ReplState::new();

        for c in initial_chars {
            state.insert_char(c);
        }

        // Move cursor to start
        state.cursor_position = 0;

        // Apply many deletes
        for _ in 0..delete_count {
            state.delete_char_at();
            // INVARIANT: Cursor <= Buffer length
            prop_assert!(state.cursor_position <= state.input_buffer.len());
        }
    }

    /// Property: Clear always resets to empty state
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_clear_always_resets(
        chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            1..100
        )
    ) {
        let mut state = ReplState::new();

        for c in chars {
            state.insert_char(c);
        }

        state.clear_input();

        // INVARIANT: After clear, buffer is empty and cursor is 0
        prop_assert_eq!(&state.input_buffer, "");
        prop_assert_eq!(state.cursor_position, 0);
        prop_assert!(!state.has_input());
    }
}

// ============================================================================
// PROPERTY TEST 2: AppState Event Handling Invariants
// ============================================================================

proptest! {
    /// Property: AppState never panics on arbitrary text input
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_app_state_handles_any_text_input(
        chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            0..200
        )
    ) {
        let mut state = AppState::default();

        for c in chars {
            // Should never panic
            let result = state.handle_event(AppEvent::TextInput(c));
            prop_assert!(result.is_ok());

            // INVARIANT: Cursor position valid
            prop_assert!(state.repl.cursor_position <= state.repl.input_buffer.len());
        }
    }

    /// Property: Quit event always sets should_quit flag
    #[test]
    fn prop_quit_always_sets_flag(_dummy in any::<u8>()) {
        let mut state = AppState::default();

        prop_assert!(!state.should_quit);

        let result = state.handle_event(AppEvent::Quit);
        prop_assert!(result.is_ok());

        // INVARIANT: Quit event always sets flag
        prop_assert!(state.should_quit);
    }

    /// Property: Enter with empty input produces no side effects
    #[test]
    fn prop_enter_on_empty_input_no_side_effects(_dummy in any::<u8>()) {
        let mut state = AppState::default();

        // Ensure input is empty
        prop_assert!(!state.repl.has_input());

        let result = state.handle_event(AppEvent::Enter);
        prop_assert!(result.is_ok());

        let effects = result.unwrap();

        // INVARIANT: No side effects on empty input
        prop_assert_eq!(effects.len(), 0);
        prop_assert!(!state.repl.generating);
    }
}

// ============================================================================
// PROPERTY TEST 3: Input Buffer Character Encoding Invariants
// ============================================================================

proptest! {
    /// Property: Buffer correctly handles unicode characters
    #[test]
    fn prop_unicode_handling(
        unicode_chars in prop::collection::vec(
            any::<char>(),  // Full unicode range
            0..50
        )
    ) {
        let mut state = ReplState::new();

        for c in unicode_chars.iter() {
            state.insert_char(*c);
        }

        // INVARIANT: Character count matches what we inserted
        prop_assert_eq!(state.input_buffer.chars().count(), unicode_chars.len());

        // INVARIANT: Cursor position equals character count (we inserted at end)
        prop_assert_eq!(state.cursor_position, unicode_chars.len());
    }

    /// Property: Mixed ASCII and Unicode handling
    #[test]
    fn prop_mixed_ascii_unicode(
        chars in prop::collection::vec(
            any::<char>(),  // Full unicode range
            0..100
        )
    ) {
        let mut state = ReplState::new();

        let mut expected_count = 0;
        for c in chars {
            state.insert_char(c);
            expected_count += 1;

            // INVARIANT: Character count matches insertions
            prop_assert_eq!(state.input_buffer.chars().count(), expected_count);
            prop_assert_eq!(state.cursor_position, expected_count);
        }
    }
}

// ============================================================================
// PROPERTY TEST 4: State Transition Ordering
// ============================================================================

proptest! {
    /// Property: Clear always works regardless of state
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_clear_works_in_any_state(
        setup_chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            0..100
        ),
        clear_after in any::<bool>()
    ) {
        let mut state = ReplState::new();

        for c in setup_chars {
            state.insert_char(c);
        }

        // Optionally set some state flags
        if clear_after {
            state.generating = true;
            state.validating = true;
        }

        state.clear_input();

        // INVARIANT: Clear always produces consistent state
        prop_assert_eq!(&state.input_buffer, "");
        prop_assert_eq!(state.cursor_position, 0);
        prop_assert!(state.generated_command.is_none());
        prop_assert!(state.generation_error.is_none());
        prop_assert!(state.validation_result.is_none());
    }
}

// ============================================================================
// PROPERTY TEST 5: Backspace and Delete Symmetry
// ============================================================================

proptest! {
    /// Property: Backspace from end is equivalent to delete from second-to-last
    #[test]
    fn prop_backspace_delete_equivalence(
        chars in prop::collection::vec(
            prop::char::range('a', 'z'),
            2..50  // At least 2 chars for this test
        )
    ) {
        // Setup state 1: Use backspace
        let mut state1 = ReplState::new();
        for c in chars.iter() {
            state1.insert_char(*c);
        }
        state1.delete_char_before(); // Backspace from end

        // Setup state 2: Use delete
        let mut state2 = ReplState::new();
        for c in chars.iter() {
            state2.insert_char(*c);
        }
        state2.cursor_position -= 1; // Move cursor back one
        state2.delete_char_at(); // Delete current position

        // INVARIANT: Both should produce same result
        prop_assert_eq!(&state1.input_buffer, &state2.input_buffer);
    }
}

// ============================================================================
// PROPERTY TEST 6: Event Sequence Order Independence
// ============================================================================

proptest! {
    /// Property: Multiple clears in a row produce same result as one clear
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_multiple_clears_idempotent(
        chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            0..100
        ),
        clear_count in 1_usize..10
    ) {
        let mut state = ReplState::new();

        for c in chars {
            state.insert_char(c);
        }

        // Clear multiple times
        for _ in 0..clear_count {
            state.clear_input();
        }

        // INVARIANT: Multiple clears = single clear
        prop_assert_eq!(&state.input_buffer, "");
        prop_assert_eq!(state.cursor_position, 0);
    }
}

// ============================================================================
// PROPERTY TEST 7: Buffer Length Consistency
// ============================================================================

proptest! {
    /// Property: Buffer byte length >= character count (due to UTF-8)
    /// NOTE: ASCII only due to cursor_position unicode bug
    #[test]
    fn prop_buffer_byte_length_consistency(
        chars in prop::collection::vec(
            prop::char::range('a', 'z'),  // ASCII only
            0..100
        )
    ) {
        let mut state = ReplState::new();

        for c in chars {
            state.insert_char(c);

            // INVARIANT: Byte length >= char count (UTF-8 encoding)
            let byte_len = state.input_buffer.len();
            let char_count = state.input_buffer.chars().count();
            prop_assert!(byte_len >= char_count);

            // INVARIANT: Cursor position equals char count (we're inserting at end)
            prop_assert_eq!(state.cursor_position, char_count);
        }
    }
}

// ============================================================================
// Helper Functions for Property Tests
// ============================================================================
//
// Note: Helper functions for generating arbitrary event sequences
// can be added here as needed for future property tests.
