/// Theme and Styling Module
///
/// Defines colors, styles, and visual elements for the TUI.

use ratatui::style::Color;

/// Application color theme
#[derive(Debug, Clone)]
pub struct Theme {
    // Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,

    // UI colors
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub muted: Color,

    // Status colors
    pub info: Color,
    pub loading: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Black,
            foreground: Color::White,
            border: Color::DarkGray,
            muted: Color::Gray,
            info: Color::Blue,
            loading: Color::Cyan,
        }
    }
}

impl Theme {
    /// Get the default theme
    pub fn default_theme() -> Self {
        Self::default()
    }
}
