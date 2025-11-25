//! Slash command system for interactive cmdai sessions
//!
//! Provides in-session command capabilities like /test, /help, /config
//! that allow users to access advanced features without exiting the main application.

pub mod handler;
pub mod parser;
pub mod session;

pub use handler::{SlashCommandHandler, CommandResponse, CommandContext};
pub use parser::{SlashCommandParser, SlashCommand, ParsedCommand};
pub use session::{SessionManager, SessionState, SessionMode};