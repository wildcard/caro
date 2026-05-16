//! Caro brand palette as RGB triples.
//!
//! These are the same values shipped in
//! `caro-design-system/project/colors_and_type.css` so the CLI and the
//! marketing site speak the same colour vocabulary. RGB triples (rather than
//! hex literals) match `colored::Colorize::truecolor()`'s signature.
//!
//! The `colored` crate auto-honors `NO_COLOR` and non-TTY stdout, so callers
//! do not need a separate "stripped" code path — `.truecolor()` becomes a
//! no-op in those environments.

/// (R, G, B) triple consumed by `colored::Colorize::truecolor`.
pub type Rgb = (u8, u8, u8);

// ---------- Primary brand greys (paper-and-ink) ----------
pub const CARO_GREY_950: Rgb = (0x1a, 0x1a, 0x1a);
pub const CARO_GREY_900: Rgb = (0x2b, 0x2b, 0x2b);
pub const CARO_GREY_800: Rgb = (0x3a, 0x3a, 0x3a);
pub const CARO_GREY_700: Rgb = (0x4f, 0x4f, 0x4f); // "retro console grey"
pub const CARO_GREY_500: Rgb = (0x7a, 0x7a, 0x7a);
pub const CARO_GREY_400: Rgb = (0xa0, 0xa0, 0xa0);
pub const CARO_GREY_300: Rgb = (0xc9, 0xc7, 0xc1);

// ---------- Beige / paper ----------
pub const CARO_BEIGE_50: Rgb = (0xfa, 0xf8, 0xec);
pub const CARO_BEIGE_100: Rgb = (0xf4, 0xf1, 0xdf); // "retro console beige"
pub const CARO_BEIGE_200: Rgb = (0xe9, 0xe4, 0xc8);

// ---------- Signal red (replaces deprecated orange gradient) ----------
pub const CARO_RED_500: Rgb = (0xef, 0x33, 0x33);
pub const CARO_RED_600: Rgb = (0xe6, 0x36, 0x36);
pub const CARO_RED_700: Rgb = (0xc0, 0x20, 0x20);

// ---------- Highlighter yellow (terminal prompt + selections) ----------
pub const CARO_YELLOW_400: Rgb = (0xfc, 0xfc, 0x62);
pub const CARO_YELLOW_500: Rgb = (0xf0, 0xed, 0x3b);

// ---------- Risk-status colours (mirrors the website's CSS tokens) ----------
pub const STATUS_SAFE: Rgb = (0x7c, 0xd3, 0x89); // soft green
pub const STATUS_MODERATE: Rgb = CARO_YELLOW_400;
pub const STATUS_HIGH: Rgb = (0xff, 0x6b, 0x6b); // alarm red, slightly washed
pub const STATUS_CRITICAL: Rgb = (0xff, 0x38, 0x38); // brand-red sibling, urgent

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_signal_red_is_canonical() {
        assert_eq!(CARO_RED_500, (239, 51, 51));
    }

    #[test]
    fn paper_beige_is_canonical() {
        assert_eq!(CARO_BEIGE_100, (244, 241, 223));
    }

    #[test]
    fn moderate_aliases_yellow() {
        assert_eq!(STATUS_MODERATE, CARO_YELLOW_400);
    }
}
