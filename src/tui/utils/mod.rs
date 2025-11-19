/// Terminal Utilities
///
/// Helper functions for terminal setup, cleanup, and management.
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

/// Setup the terminal for TUI mode
///
/// This:
/// - Enables raw mode (no line buffering)
/// - Enters alternate screen (preserves existing terminal content)
/// - Creates a Ratatui terminal instance
pub fn setup_terminal() -> Result<TerminalType> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to normal mode
///
/// This:
/// - Leaves alternate screen (restores previous content)
/// - Disables raw mode
pub fn restore_terminal(terminal: &mut TerminalType) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Terminal setup/restore tests require a real terminal environment.
    // Integration tests should be added when running in a CI environment with
    // proper terminal emulation.
}
