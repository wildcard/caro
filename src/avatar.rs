//! ASCII Avatar Module for Caro CLI
//!
//! Provides an animated ASCII avatar that reacts to user interaction.
//! Inspired by luminosity-based ASCII shader techniques where character
//! density represents brightness levels.
//!
//! # Character Density Scale (dark to light)
//! `@%#*+=-:. ` - from densest (darkest) to sparsest (lightest)

use colored::Colorize;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Character palette ordered by visual density (dark to light)
/// Used for luminosity-based shading in ASCII art
pub const DENSITY_CHARS: &[char] = &['@', '%', '#', '*', '+', '=', '-', ':', '.', ' '];

/// Avatar states representing different interaction phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarState {
    /// Waiting for user input - neutral, attentive expression
    Idle,
    /// User is typing - eyes following/interested
    Listening,
    /// Processing/thinking - animated thinking pose
    Thinking,
    /// Command generated successfully - happy expression
    Success,
    /// Warning or caution needed - concerned expression
    Warning,
    /// Error occurred - worried expression
    Error,
    /// Goodbye/exit state
    Goodbye,
}

impl AvatarState {
    /// Get the primary color for this state
    pub fn color(&self) -> colored::Color {
        match self {
            AvatarState::Idle => colored::Color::Cyan,
            AvatarState::Listening => colored::Color::Blue,
            AvatarState::Thinking => colored::Color::Yellow,
            AvatarState::Success => colored::Color::Green,
            AvatarState::Warning => colored::Color::Yellow,
            AvatarState::Error => colored::Color::Red,
            AvatarState::Goodbye => colored::Color::Magenta,
        }
    }
}

/// ASCII avatar that can be rendered in terminal
pub struct Avatar {
    /// Current state of the avatar
    state: AvatarState,
    /// Animation frame counter
    frame: usize,
    /// Whether to use colors
    use_color: bool,
    /// Whether to show compact version
    compact: bool,
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Avatar {
    /// Create a new avatar in idle state
    pub fn new() -> Self {
        Self {
            state: AvatarState::Idle,
            frame: 0,
            use_color: true,
            compact: false,
        }
    }

    /// Create a compact avatar (smaller, for inline display)
    pub fn compact() -> Self {
        Self {
            state: AvatarState::Idle,
            frame: 0,
            use_color: true,
            compact: true,
        }
    }

    /// Set avatar state
    pub fn set_state(&mut self, state: AvatarState) {
        self.state = state;
        self.frame = 0;
    }

    /// Advance animation frame
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Enable or disable colors
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    /// Get the ASCII art frames for the current state
    fn get_frames(&self) -> Vec<&'static str> {
        if self.compact {
            return self.get_compact_frames();
        }

        match self.state {
            AvatarState::Idle => vec![IDLE_FRAME_1, IDLE_FRAME_2],
            AvatarState::Listening => vec![LISTENING_FRAME_1, LISTENING_FRAME_2],
            AvatarState::Thinking => vec![
                THINKING_FRAME_1,
                THINKING_FRAME_2,
                THINKING_FRAME_3,
                THINKING_FRAME_4,
            ],
            AvatarState::Success => vec![SUCCESS_FRAME],
            AvatarState::Warning => vec![WARNING_FRAME],
            AvatarState::Error => vec![ERROR_FRAME],
            AvatarState::Goodbye => vec![GOODBYE_FRAME],
        }
    }

    /// Get compact single-line avatar frames
    fn get_compact_frames(&self) -> Vec<&'static str> {
        match self.state {
            AvatarState::Idle => vec!["(o_o)", "(o_o)"],
            AvatarState::Listening => vec!["(O_O)", "(o_O)"],
            AvatarState::Thinking => vec!["(o_o)", "(-_o)", "(o_-)", "(-_-)"],
            AvatarState::Success => vec!["(^_^)"],
            AvatarState::Warning => vec!["(o_O)"],
            AvatarState::Error => vec!["(x_x)"],
            AvatarState::Goodbye => vec!["(^_~)"],
        }
    }

    /// Render the current frame
    pub fn render(&self) -> String {
        let frames = self.get_frames();
        let frame_idx = self.frame % frames.len();
        let art = frames[frame_idx];

        if self.use_color {
            art.color(self.state.color()).to_string()
        } else {
            art.to_string()
        }
    }

    /// Render with a message alongside
    pub fn render_with_message(&self, message: &str) -> String {
        if self.compact {
            let avatar = self.render();
            return format!("{} {}", avatar, message);
        }

        let frames = self.get_frames();
        let frame_idx = self.frame % frames.len();
        let art_lines: Vec<&str> = frames[frame_idx].lines().collect();

        // Find the middle line to place the message
        let mid = art_lines.len() / 2;

        let mut result = String::new();
        for (i, line) in art_lines.iter().enumerate() {
            let colored_line = if self.use_color {
                line.color(self.state.color()).to_string()
            } else {
                line.to_string()
            };

            if i == mid {
                result.push_str(&format!("{}  {}\n", colored_line, message));
            } else {
                result.push_str(&format!("{}\n", colored_line));
            }
        }
        result
    }

    /// Print avatar to stdout
    pub fn print(&self) {
        print!("{}", self.render());
    }

    /// Print avatar to stderr (for wrapper mode)
    pub fn eprint(&self) {
        eprint!("{}", self.render());
    }
}

// ============================================================================
// ASCII Art Frames
// ============================================================================
// Using luminosity-based character density:
// Dense chars (@%#) for dark/solid areas
// Medium chars (*+=) for mid-tones
// Sparse chars (-:.) for light areas
// Space for background

/// Idle state - neutral, attentive (frame 1)
const IDLE_FRAME_1: &str = r#"
    .---.
   /     \
  | o   o |
  |   >   |
  |  ___  |
   \_____/
"#;

/// Idle state - subtle blink (frame 2)
const IDLE_FRAME_2: &str = r#"
    .---.
   /     \
  | -   - |
  |   >   |
  |  ___  |
   \_____/
"#;

/// Listening state - attentive, ears perked (frame 1)
const LISTENING_FRAME_1: &str = r#"
    .---.
   /     \
  | O   O |
  |   >   |
  |  ___  |
   \_____/
"#;

/// Listening state - looking at input (frame 2)
const LISTENING_FRAME_2: &str = r#"
    .---.
   /     \
  | o   O |
  |   >   |
  |  ___  |
   \_____/
"#;

/// Thinking state - processing (frame 1)
const THINKING_FRAME_1: &str = r#"
    .---.  .
   /     \
  | o   o |
  |   <   |
  |  ~~~  |
   \_____/
"#;

/// Thinking state - processing (frame 2)
const THINKING_FRAME_2: &str = r#"
    .---. ..
   /     \
  | -   o |
  |   <   |
  |  ~~~  |
   \_____/
"#;

/// Thinking state - processing (frame 3)
const THINKING_FRAME_3: &str = r#"
    .---.  o
   /     \
  | o   - |
  |   <   |
  |  ~~~  |
   \_____/
"#;

/// Thinking state - processing (frame 4)
const THINKING_FRAME_4: &str = r#"
    .---. *
   /     \
  | -   - |
  |   <   |
  |  ~~~  |
   \_____/
"#;

/// Success state - happy
const SUCCESS_FRAME: &str = r#"
    .---.
   /     \
  | ^   ^ |
  |   >   |
  |  \_/  |
   \_____/
"#;

/// Warning state - concerned
const WARNING_FRAME: &str = r#"
    .---.
   /     \
  | o   O |
  |   >   |
  |  ---  |
   \_____/
"#;

/// Error state - worried
const ERROR_FRAME: &str = r#"
    .---.
   /     \
  | x   x |
  |   >   |
  |  ___  |
   \_____/
"#;

/// Goodbye state - winking
const GOODBYE_FRAME: &str = r#"
    .---.
   /     \
  | ^   ~ |
  |   >   |
  |  \_/  |
   \_____/
"#;

// ============================================================================
// Animated Avatar Display
// ============================================================================

/// Animated avatar that runs in a background thread
pub struct AnimatedAvatar {
    running: Arc<AtomicBool>,
    state: Arc<AtomicUsize>,
    handle: Option<thread::JoinHandle<()>>,
}

impl AnimatedAvatar {
    /// Start an animated avatar display
    ///
    /// Returns a handle that can be used to change state or stop the animation.
    pub fn start(initial_state: AvatarState) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let state = Arc::new(AtomicUsize::new(initial_state as usize));

        let running_clone = Arc::clone(&running);
        let state_clone = Arc::clone(&state);

        let handle = thread::spawn(move || {
            let mut avatar = Avatar::new();
            let mut last_state = initial_state;

            while running_clone.load(Ordering::Relaxed) {
                // Get current state
                let state_val = state_clone.load(Ordering::Relaxed);
                let current_state = match state_val {
                    0 => AvatarState::Idle,
                    1 => AvatarState::Listening,
                    2 => AvatarState::Thinking,
                    3 => AvatarState::Success,
                    4 => AvatarState::Warning,
                    5 => AvatarState::Error,
                    6 => AvatarState::Goodbye,
                    _ => AvatarState::Idle,
                };

                if current_state != last_state {
                    avatar.set_state(current_state);
                    last_state = current_state;
                }

                // Clear previous frame and render new one
                // Move cursor up and clear lines
                let frame = avatar.render();
                let line_count = frame.lines().count();

                // Move up and clear
                for _ in 0..line_count {
                    eprint!("\x1B[A\x1B[2K");
                }

                // Print new frame
                eprint!("{}", frame);
                let _ = io::stderr().flush();

                avatar.tick();
                thread::sleep(Duration::from_millis(500));
            }
        });

        Self {
            running,
            state,
            handle: Some(handle),
        }
    }

    /// Change the avatar state
    pub fn set_state(&self, state: AvatarState) {
        self.state.store(state as usize, Ordering::Relaxed);
    }

    /// Stop the animation
    pub fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AnimatedAvatar {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        // Note: We can't join here because we don't have ownership of the handle
    }
}

// ============================================================================
// Display Helpers
// ============================================================================

/// Display a static avatar with a state-specific message
pub fn display_avatar(state: AvatarState, message: Option<&str>) {
    let mut avatar = Avatar::new();
    avatar.set_state(state);

    if let Some(msg) = message {
        eprintln!("{}", avatar.render_with_message(msg));
    } else {
        eprintln!("{}", avatar.render());
    }
}

/// Display a compact inline avatar
pub fn display_compact_avatar(state: AvatarState, message: &str) {
    let mut avatar = Avatar::compact();
    avatar.set_state(state);
    eprintln!("{}", avatar.render_with_message(message));
}

/// Show a quick reaction avatar inline (no newlines)
pub fn show_reaction(state: AvatarState) -> String {
    let mut avatar = Avatar::compact();
    avatar.set_state(state);
    avatar.render()
}

// ============================================================================
// 3D Character Avatar Integration
// ============================================================================

use crate::ascii3d::{render_character_frame, AnimatedCharacter, CharacterExpression};

/// Display a 3D ASCII character with the given state
pub fn display_3d_avatar(state: AvatarState, width: usize, height: usize) {
    let mut character = AnimatedCharacter::new();

    // Map avatar state to character expression
    let expression = match state {
        AvatarState::Idle => CharacterExpression::Neutral,
        AvatarState::Listening => CharacterExpression::Surprised,
        AvatarState::Thinking => CharacterExpression::Thinking,
        AvatarState::Success => CharacterExpression::Happy,
        AvatarState::Warning => CharacterExpression::Thinking,
        AvatarState::Error => CharacterExpression::Sad,
        AvatarState::Goodbye => CharacterExpression::Happy,
    };
    character.set_expression(expression);

    // Set rotation based on state
    let angle = match state {
        AvatarState::Thinking => 0.3, // Looking to the side
        AvatarState::Listening => -0.2,
        _ => 0.0, // Forward facing
    };
    character.set_rotation(-0.1, angle);

    let rendered = character.render(width, height);

    // Apply color based on state
    if std::io::stderr().is_terminal() {
        use colored::Colorize;
        let colored_output = match state {
            AvatarState::Idle => rendered.cyan(),
            AvatarState::Listening => rendered.blue(),
            AvatarState::Thinking => rendered.yellow(),
            AvatarState::Success => rendered.green(),
            AvatarState::Warning => rendered.yellow(),
            AvatarState::Error => rendered.red(),
            AvatarState::Goodbye => rendered.magenta(),
        };
        eprint!("{}", colored_output);
    } else {
        eprint!("{}", rendered);
    }
}

/// Display a spinning 3D character animation frame
pub fn display_3d_frame(frame: usize, width: usize, height: usize) {
    let rendered = render_character_frame(width, height, frame);
    eprint!("{}", rendered);
}

/// Display a 3D avatar with a message alongside
pub fn display_3d_avatar_with_message(state: AvatarState, message: &str, width: usize, height: usize) {
    let mut character = AnimatedCharacter::new();

    let expression = match state {
        AvatarState::Idle => CharacterExpression::Neutral,
        AvatarState::Listening => CharacterExpression::Surprised,
        AvatarState::Thinking => CharacterExpression::Thinking,
        AvatarState::Success => CharacterExpression::Happy,
        AvatarState::Warning => CharacterExpression::Thinking,
        AvatarState::Error => CharacterExpression::Sad,
        AvatarState::Goodbye => CharacterExpression::Happy,
    };
    character.set_expression(expression);
    character.set_rotation(-0.1, 0.0);

    let rendered = character.render(width, height);
    let lines: Vec<&str> = rendered.lines().collect();
    let mid = lines.len() / 2;

    // Apply color based on state
    if std::io::stderr().is_terminal() {
        use colored::Colorize;

        for (i, line) in lines.iter().enumerate() {
            let colored_line = match state {
                AvatarState::Idle => line.cyan(),
                AvatarState::Listening => line.blue(),
                AvatarState::Thinking => line.yellow(),
                AvatarState::Success => line.green(),
                AvatarState::Warning => line.yellow(),
                AvatarState::Error => line.red(),
                AvatarState::Goodbye => line.magenta(),
            };

            if i == mid {
                eprintln!("{}  {}", colored_line, message);
            } else {
                eprintln!("{}", colored_line);
            }
        }
    } else {
        for (i, line) in lines.iter().enumerate() {
            if i == mid {
                eprintln!("{}  {}", line, message);
            } else {
                eprintln!("{}", line);
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_creation() {
        let avatar = Avatar::new();
        assert_eq!(avatar.state, AvatarState::Idle);
        assert_eq!(avatar.frame, 0);
    }

    #[test]
    fn test_avatar_state_change() {
        let mut avatar = Avatar::new();
        avatar.set_state(AvatarState::Thinking);
        assert_eq!(avatar.state, AvatarState::Thinking);
        assert_eq!(avatar.frame, 0); // Frame resets on state change
    }

    #[test]
    fn test_avatar_tick() {
        let mut avatar = Avatar::new();
        avatar.tick();
        assert_eq!(avatar.frame, 1);
        avatar.tick();
        assert_eq!(avatar.frame, 2);
    }

    #[test]
    fn test_avatar_render() {
        let avatar = Avatar::new().with_color(false);
        let rendered = avatar.render();
        assert!(rendered.contains("o   o")); // Eyes in idle state
    }

    #[test]
    fn test_compact_avatar() {
        let avatar = Avatar::compact().with_color(false);
        let rendered = avatar.render();
        assert!(rendered.contains("(o_o)")); // Compact idle face
    }

    #[test]
    fn test_avatar_with_message() {
        let avatar = Avatar::compact().with_color(false);
        let rendered = avatar.render_with_message("Hello!");
        assert!(rendered.contains("(o_o)"));
        assert!(rendered.contains("Hello!"));
    }

    #[test]
    fn test_all_states_render() {
        let states = [
            AvatarState::Idle,
            AvatarState::Listening,
            AvatarState::Thinking,
            AvatarState::Success,
            AvatarState::Warning,
            AvatarState::Error,
            AvatarState::Goodbye,
        ];

        for state in states {
            let mut avatar = Avatar::new().with_color(false);
            avatar.set_state(state);
            let rendered = avatar.render();
            assert!(!rendered.is_empty(), "State {:?} should render", state);
        }
    }

    #[test]
    fn test_density_chars_order() {
        // Verify density chars are in correct order (dense to sparse)
        assert_eq!(DENSITY_CHARS[0], '@'); // Densest
        assert_eq!(DENSITY_CHARS[DENSITY_CHARS.len() - 1], ' '); // Sparsest
    }

    #[test]
    fn test_state_colors() {
        assert_eq!(AvatarState::Success.color(), colored::Color::Green);
        assert_eq!(AvatarState::Error.color(), colored::Color::Red);
        assert_eq!(AvatarState::Warning.color(), colored::Color::Yellow);
    }

    #[test]
    fn test_show_reaction() {
        let reaction = show_reaction(AvatarState::Success);
        // Should contain the happy face, possibly with ANSI codes
        assert!(!reaction.is_empty());
    }
}
