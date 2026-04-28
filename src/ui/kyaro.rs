//! Kyaro the Shiba Inu — the brand mascot, embedded as ASCII art.
//!
//! Each function returns a representative single frame of the corresponding
//! animation state. The source files live in `assets/kyaro/` and are part of
//! the design-system handoff bundle. Only one frame per state is wired today;
//! follow-up work tracks frame cycling for the full animation library.
//!
//! Source frames are drawn on a 100-column × 55-row canvas with the actual
//! sprite occupying ~25-29 rows of ~42-60 columns. [`compact`] strips the
//! padding so the sprite renders inline at decision time without dominating
//! the terminal.

const IDLE_FRAME: &str = include_str!("../../assets/kyaro/001-idle/idle_ASCII/Idle_1.txt");
const SHOCKED_FRAME: &str =
    include_str!("../../assets/kyaro/008-shocked/shocked_ASCII/Shocked1.txt");
const HAPPY_BOUNCE_FRAME: &str = include_str!(
    "../../assets/kyaro/006-happy bounce/happy bounce_ASCII/HappyBounce1.txt"
);
const SLEEPING_FRAME: &str =
    include_str!("../../assets/kyaro/003-sleeping/sleeping_ASCII/Sleeping01.txt");

/// Idle Shiba — used as the default companion frame next to safe commands.
pub fn idle() -> &'static str {
    IDLE_FRAME
}

/// Shocked Shiba — used when the safety validator flags a command as
/// HIGH or CRITICAL risk.
pub fn shocked() -> &'static str {
    SHOCKED_FRAME
}

/// Happy-bounce Shiba — used after a successful confirmed run.
pub fn happy_bounce() -> &'static str {
    HAPPY_BOUNCE_FRAME
}

/// Sleeping Shiba — for setup-wizard idle states or `caro --version` flair.
pub fn sleeping() -> &'static str {
    SLEEPING_FRAME
}

/// Trim a Kyaro ASCII frame for inline terminal display.
///
/// - Strips leading and trailing fully-blank rows (the source canvas pads to
///   55 rows so each frame can be cycled in lock-step).
/// - Right-trims whitespace from each remaining row so terminals do not paint
///   trailing spaces over background colours.
/// - Preserves the absolute column positions of the sprite so consecutive
///   frames in an animation stay aligned.
///
/// The result is a freshly-allocated `String` with `\n`-separated lines.
pub fn compact(frame: &str) -> String {
    let lines: Vec<&str> = frame.split('\n').collect();
    let first = lines
        .iter()
        .position(|l| l.chars().any(|c| !c.is_whitespace()))
        .unwrap_or(0);
    let last = lines
        .iter()
        .rposition(|l| l.chars().any(|c| !c.is_whitespace()))
        .unwrap_or(lines.len().saturating_sub(1));

    let mut out = String::with_capacity(frame.len());
    for line in &lines[first..=last] {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_frame_loaded() {
        assert!(!idle().is_empty(), "idle ASCII frame should be non-empty");
    }

    #[test]
    fn compact_idle_drops_padding() {
        let raw = idle();
        let compacted = compact(raw);
        let raw_lines = raw.split('\n').count();
        let compacted_lines = compacted.lines().count();
        assert!(compacted_lines < raw_lines);
        assert!(
            compacted_lines <= 35,
            "compacted idle should fit in roughly 35 rows: got {compacted_lines}"
        );
    }

    #[test]
    fn compact_preserves_sprite_pixels() {
        // The mascot's main body uses '%' and '@' fill characters; ensure both
        // survive the compaction.
        let compacted = compact(idle());
        assert!(compacted.contains('%'));
        assert!(compacted.contains('@'));
    }

    #[test]
    fn all_states_load_non_empty() {
        // Sanity-check that every include_str! path resolved successfully.
        // A precise byte-comparison is too brittle — neighboring frames in
        // an animation can be near-identical and still legitimate.
        assert!(!idle().is_empty());
        assert!(!shocked().is_empty());
        assert!(!happy_bounce().is_empty());
        assert!(!sleeping().is_empty());
    }
}
