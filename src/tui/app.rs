/// Main TUI Application
///
/// The `TuiApp` struct manages the entire TUI lifecycle:
/// - Event loop (keyboard input, terminal resize)
/// - State management (AppState updates)
/// - Component rendering
/// - Integration with backend for command generation
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::time::Duration;

use crate::backends::CommandGenerator;
use crate::config::ConfigManager;
use crate::safety::{SafetyConfig, SafetyValidator};
use crate::tui::{
    components::{Component, HelpFooterComponent, ReplComponent, StatusBarComponent},
    state::{AppEvent, AppMode, AppState},
    utils::{restore_terminal, setup_terminal, TerminalType},
};

/// Main TUI Application
///
/// # Example
///
/// ```rust
/// use cmdai::tui::TuiApp;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut app = TuiApp::new().await?;
///     app.run().await?;
///     Ok(())
/// }
/// ```
pub struct TuiApp {
    /// Application state
    state: AppState,

    /// Terminal instance
    terminal: TerminalType,

    /// Backend for command generation
    backend: Box<dyn CommandGenerator>,

    /// Safety validator for command validation
    validator: SafetyValidator,
}

impl TuiApp {
    /// Create a new TUI application
    pub async fn new() -> Result<Self> {
        // Load configuration
        let config_manager = ConfigManager::new()?;
        let user_config = config_manager.load()?;

        // Setup terminal
        let terminal = setup_terminal()?;

        // Create backend for command generation
        let backend = Self::create_backend(&user_config).await?;

        // Create safety validator
        let safety_config = match user_config.safety_level {
            crate::models::SafetyLevel::Strict => SafetyConfig::strict(),
            crate::models::SafetyLevel::Moderate => SafetyConfig::moderate(),
            crate::models::SafetyLevel::Permissive => SafetyConfig::permissive(),
        };
        let validator = SafetyValidator::new(safety_config)?;

        // Create application state
        let mut state = AppState::new(user_config);

        // Set backend status (for now, just show "Loading...")
        state.set_backend_status("Loading...".to_string(), false, None);

        Ok(Self {
            state,
            terminal,
            backend,
            validator,
        })
    }

    /// Run the TUI application
    ///
    /// This is the main event loop. It:
    /// 1. Renders the current state
    /// 2. Polls for events (keyboard, resize, etc.)
    /// 3. Updates state based on events
    /// 4. Handles side effects (async operations)
    /// 5. Repeats until quit
    pub async fn run(mut self) -> Result<()> {
        // Setup panic handler to restore terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            original_hook(panic);
        }));

        // Initialize backend status
        self.detect_backend().await;

        // Main event loop
        while !self.state.should_quit {
            // Render current state
            self.render()?;

            // Poll for events with timeout
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        let app_event: AppEvent = key.into();
                        self.handle_event(app_event).await?;
                    }
                    Event::Resize(w, h) => {
                        self.handle_event(AppEvent::Resize(w, h)).await?;
                    }
                    _ => {}
                }
            }
        }

        // Cleanup
        restore_terminal(&mut self.terminal)?;

        Ok(())
    }

    /// Handle an application event
    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        // Update state and get side effects
        let side_effects = self.state.handle_event(event)?;

        // Process side effects
        for effect in side_effects {
            self.handle_side_effect(effect).await?;
        }

        Ok(())
    }

    /// Handle a side effect (async operations)
    async fn handle_side_effect(
        &mut self,
        effect: crate::tui::state::SideEffect,
    ) -> Result<()> {
        use crate::models::CommandRequest;
        use crate::tui::state::events::{GeneratedCommandEvent, ValidationResultEvent};

        match effect {
            crate::tui::state::SideEffect::GenerateCommand {
                input,
                shell,
                safety,
            } => {
                // Create command request
                let request = CommandRequest {
                    input: input.clone(),
                    context: None,
                    shell,
                    safety_level: safety,
                    backend_preference: None,
                };

                // Call backend to generate command
                match self.backend.generate_command(&request).await {
                    Ok(generated) => {
                        // Convert to TUI event type
                        let event = AppEvent::CommandGenerated(GeneratedCommandEvent {
                            command: generated.command,
                            explanation: generated.explanation,
                            risk_level: generated.safety_level.into(),
                        });

                        // Update state with success
                        self.state.handle_event(event)?;
                    }
                    Err(e) => {
                        // Update state with error
                        let error_msg = format!("Generation failed: {}", e);
                        self.state
                            .handle_event(AppEvent::GenerationFailed(error_msg))?;
                    }
                }
            }

            crate::tui::state::SideEffect::ValidateCommand { command, shell } => {
                // Call validator to check command
                match self.validator.validate_command(&command, shell).await {
                    Ok(result) => {
                        // Convert to TUI event type
                        let event = AppEvent::ValidationComplete(ValidationResultEvent {
                            risk_level: result.risk_level.into(),
                            warnings: result.warnings,
                            suggestions: vec![], // TODO: Add suggestions if needed
                            matched_patterns: result.matched_patterns,
                        });

                        // Update state with validation result
                        self.state.handle_event(event)?;
                    }
                    Err(e) => {
                        // Handle validation error - show as error message
                        self.state
                            .show_error(format!("Validation failed: {}", e));
                    }
                }
            }

            crate::tui::state::SideEffect::ExecuteCommand(_command) => {
                // TODO: Implement command execution
                // For now, just log that it's not implemented
                tracing::warn!("Command execution not yet implemented in TUI");
            }

            crate::tui::state::SideEffect::ShowError(error) => {
                // Show error message in state
                self.state.show_error(error);
            }
        }

        Ok(())
    }

    /// Create appropriate backend based on user configuration
    async fn create_backend(
        _user_config: &crate::models::UserConfiguration,
    ) -> Result<Box<dyn CommandGenerator>> {
        // For test builds, use mock backend
        #[cfg(any(test, debug_assertions))]
        {
            Ok(Box::new(MockCommandGenerator::new()))
        }

        // Production backend selection
        #[cfg(not(any(test, debug_assertions)))]
        {
            use crate::backends::embedded::{EmbeddedModelBackend, ModelVariant};

            // Try remote backends first
            #[cfg(feature = "remote-backends")]
            {
                use crate::backends::remote::{OllamaBackend, VllmBackend};
                use reqwest::Url;
                use std::sync::Arc;

                // Create embedded backend for fallback
                let embedded_for_fallback = EmbeddedModelBackend::with_variant_and_path(
                    ModelVariant::detect(),
                    std::env::temp_dir().join("cmdai_model.gguf"),
                )
                .map_err(|e| anyhow::anyhow!("Failed to create embedded backend: {}", e))?;
                let embedded_arc: Arc<dyn CommandGenerator> = Arc::new(embedded_for_fallback);

                if let Ok(ollama_url) = Url::parse("http://localhost:11434") {
                    if let Ok(ollama_backend) =
                        OllamaBackend::new(ollama_url, "codellama:7b".to_string())
                            .map(|b| b.with_embedded_fallback(embedded_arc.clone()))
                    {
                        if ollama_backend.is_available().await {
                            tracing::info!("Using Ollama backend with embedded fallback");
                            return Ok(Box::new(ollama_backend));
                        }
                    }
                }

                if let Ok(vllm_url) = Url::parse("http://localhost:8000") {
                    if let Ok(vllm_backend) =
                        VllmBackend::new(vllm_url, "codellama/CodeLlama-7b-hf".to_string())
                            .map(|b| b.with_embedded_fallback(embedded_arc.clone()))
                    {
                        if vllm_backend.is_available().await {
                            tracing::info!("Using vLLM backend with embedded fallback");
                            return Ok(Box::new(vllm_backend));
                        }
                    }
                }
            }

            // Fall back to embedded backend only (no remote backends available)
            tracing::info!("Using embedded backend only");
            let embedded_backend = EmbeddedModelBackend::with_variant_and_path(
                ModelVariant::detect(),
                std::env::temp_dir().join("cmdai_model.gguf"),
            )
            .map_err(|e| anyhow::anyhow!("Failed to create embedded backend: {}", e))?;
            Ok(Box::new(embedded_backend))
        }
    }

    /// Detect available backend
    async fn detect_backend(&mut self) {
        // Get backend info from the already-initialized backend
        let backend_info = self.backend.backend_info();

        self.state.set_backend_status(
            backend_info.model_name.clone(),
            self.backend.is_available().await,
            Some(backend_info.version.clone()),
        );
    }

    /// Render the current state to the terminal
    fn render(&mut self) -> Result<()> {
        // Clone the state we need for rendering (to avoid borrow conflicts)
        let state = self.state.clone();

        self.terminal.draw(|frame| {
            Self::render_frame(frame, &state);
        })?;
        Ok(())
    }

    /// Render a single frame (static method to avoid borrow conflicts)
    fn render_frame(frame: &mut Frame, state: &AppState) {
        let size = frame.area();

        // Main layout: Status bar (1 line), Content (flex), Help footer (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status bar
                Constraint::Min(10),   // Main content
                Constraint::Length(1), // Help footer
            ])
            .split(size);

        // Render status bar
        let status_bar = StatusBarComponent::from_state(state);
        status_bar.render(frame, chunks[0]);

        // Render main content based on mode
        match state.current_mode {
            AppMode::Repl => {
                let repl = ReplComponent::from_state(state);
                // Note: We need to pass repl_state to the render function
                // For now, ReplComponent will use default state
                repl.render(frame, chunks[1]);
            }
            AppMode::History => {
                // TODO: Implement history mode
                Self::render_placeholder(frame, chunks[1], "History Mode (Coming Soon)");
            }
            AppMode::Config => {
                // TODO: Implement config mode
                Self::render_placeholder(frame, chunks[1], "Config Mode (Coming Soon)");
            }
            AppMode::Help => {
                // TODO: Implement help mode
                Self::render_placeholder(frame, chunks[1], "Help Mode (Coming Soon)");
            }
        }

        // Render help footer
        let help_footer = HelpFooterComponent::for_mode(state.current_mode);
        help_footer.render(frame, chunks[2]);
    }

    /// Render a placeholder for unimplemented modes (static method)
    fn render_placeholder(frame: &mut Frame, area: ratatui::layout::Rect, text: &str) {
        use ratatui::{
            style::{Color, Style},
            text::Text,
            widgets::{Block, Borders, Paragraph},
        };

        let paragraph = Paragraph::new(Text::from(text))
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("TODO"));

        frame.render_widget(paragraph, area);
    }
}

// Import for panic handler
use crossterm::{
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use std::io;

/// Mock command generator for testing
///
/// SECURITY: This mock is restricted to debug builds via #[cfg(any(test, debug_assertions))].
/// It can generate dangerous commands for testing the safety validator,
/// which would be a security risk if used in production.
/// Production (release) builds will not include this code.
#[cfg(any(test, debug_assertions))]
struct MockCommandGenerator;

#[cfg(any(test, debug_assertions))]
impl MockCommandGenerator {
    fn new() -> Self {
        Self
    }
}

#[cfg(any(test, debug_assertions))]
#[async_trait::async_trait]
impl CommandGenerator for MockCommandGenerator {
    async fn generate_command(
        &self,
        request: &crate::models::CommandRequest,
    ) -> Result<crate::models::GeneratedCommand, crate::backends::GeneratorError> {
        use std::time::Duration;

        // Simulate generation time
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Analyze the input to determine appropriate command
        let command = if request.input.contains("list") && request.input.contains("files") {
            match request.shell {
                crate::models::ShellType::PowerShell => "Get-ChildItem".to_string(),
                crate::models::ShellType::Cmd => "dir".to_string(),
                _ => "ls -la".to_string(),
            }
        } else if request.input.contains("directory") || request.input.contains("pwd") {
            "pwd".to_string()
        } else if request.input.contains("delete") && request.input.contains("system") {
            // Very dangerous command for testing
            "rm -rf /".to_string()
        } else if request.input.contains("delete") || request.input.contains("remove") {
            "rm -rf /tmp/*".to_string() // Potentially dangerous
        } else {
            format!("echo '{}'", request.input)
        };

        Ok(crate::models::GeneratedCommand {
            command,
            explanation: format!("Command for: {}", request.input),
            safety_level: crate::models::RiskLevel::Safe,
            estimated_impact: Default::default(),
            alternatives: vec!["Alternative command".to_string()],
            backend_used: "mock".to_string(),
            generation_time_ms: 50,
            confidence_score: 0.95,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> crate::backends::BackendInfo {
        crate::backends::BackendInfo {
            backend_type: crate::models::BackendType::Ollama,
            model_name: "mock-model".to_string(),
            supports_streaming: false,
            max_tokens: 1000,
            typical_latency_ms: 50,
            memory_usage_mb: 100,
            version: "1.0.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), crate::backends::GeneratorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserConfiguration;

    #[test]
    fn test_app_state_initialization() {
        let config = UserConfiguration::default();
        let state = AppState::new(config);

        assert_eq!(state.current_mode, AppMode::Repl);
        assert!(!state.should_quit);
    }

    // Note: Full integration tests require a terminal, so they're limited here
}
