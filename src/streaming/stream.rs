//! Async streaming support for real-time command generation
//!
//! Provides GenerationStream implementation with cancellation, progress tracking,
//! and partial result handling according to the production backend specification.

use crate::models::CommandRequest;
use futures::Stream;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Real-time command generation stream with cancellation and progress feedback
pub struct GenerationStream {
    pub request_id: String,
    pub stream: Pin<Box<dyn Stream<Item = StreamEvent> + Send>>,
    pub cancellation_token: CancellationToken,
}

/// Events emitted by the generation stream
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Progress update with percentage and description
    Progress { percentage: f64, message: String },
    /// Partial command result with confidence score
    PartialResult { command: String, confidence: f64 },
    /// Generation completed with final result
    Completed { result: GenerationResult },
    /// Error occurred during generation
    Error { error: String, recoverable: bool },
    /// Generation was cancelled by user
    Cancelled,
}

/// Final generation result with comprehensive metadata
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub command: String,
    pub explanation: String,
    pub confidence: f64,
    pub generation_time: Duration,
    pub tokens_generated: usize,
    pub safety_validated: bool,
}

/// Configuration for stream generation behavior
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Maximum time to wait for stream completion
    pub timeout: Duration,
    /// Minimum interval between progress updates
    pub progress_interval: Duration,
    /// Buffer size for partial results
    pub buffer_size: usize,
    /// Enable safety validation on partial results
    pub validate_partial: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            progress_interval: Duration::from_millis(100),
            buffer_size: 64,
            validate_partial: true,
        }
    }
}

/// Progress tracker for streaming generation
#[derive(Debug)]
pub struct ProgressTracker {
    start_time: Instant,
    last_update: Instant,
    current_percentage: f64,
    estimated_total_tokens: usize,
    tokens_generated: usize,
}

impl ProgressTracker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
            current_percentage: 0.0,
            estimated_total_tokens: 50, // Default estimate
            tokens_generated: 0,
        }
    }

    /// Update progress based on tokens generated
    pub fn update_tokens(&mut self, tokens: usize) {
        self.tokens_generated = tokens;
        self.current_percentage =
            (tokens as f64 / self.estimated_total_tokens as f64 * 100.0).min(95.0);
        self.last_update = Instant::now();
    }

    /// Update estimated total tokens
    pub fn update_estimate(&mut self, estimate: usize) {
        self.estimated_total_tokens = estimate.max(self.tokens_generated + 10);
    }

    /// Get current progress percentage
    pub fn percentage(&self) -> f64 {
        self.current_percentage
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if progress update is due
    pub fn should_update(&self, interval: Duration) -> bool {
        self.last_update.elapsed() >= interval
    }
}

/// Handler for partial results during streaming
#[derive(Debug)]
pub struct PartialResultHandler {
    buffer: Arc<Mutex<PartialBuffer>>,
    config: StreamConfig,
}

#[derive(Debug)]
struct PartialBuffer {
    content: String,
    chunks: VecDeque<String>,
    confidence_scores: Vec<f64>,
    last_coherent_command: Option<String>,
}

impl PartialResultHandler {
    pub fn new(config: StreamConfig) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(PartialBuffer {
                content: String::new(),
                chunks: VecDeque::new(),
                confidence_scores: Vec::new(),
                last_coherent_command: None,
            })),
            config,
        }
    }

    /// Add a new chunk to the buffer
    pub fn add_chunk(&self, chunk: String) -> Option<PartialResult> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.content.push_str(&chunk);
        buffer.chunks.push_back(chunk);

        // Trim buffer if it gets too large
        if buffer.chunks.len() > self.config.buffer_size {
            if let Some(old_chunk) = buffer.chunks.pop_front() {
                buffer.content = buffer
                    .content
                    .strip_prefix(&old_chunk)
                    .unwrap_or(&buffer.content)
                    .to_string();
            }
        }

        // Try to extract a coherent partial command
        self.extract_partial_command(&mut buffer)
    }

    /// Extract a coherent partial command from the buffer
    fn extract_partial_command(&self, buffer: &mut PartialBuffer) -> Option<PartialResult> {
        let content = &buffer.content;

        // Look for common command patterns
        let commands = [
            "ls", "find", "grep", "cat", "mv", "cp", "rm", "mkdir", "cd", "pwd",
        ];

        for cmd in &commands {
            if content.trim_start().starts_with(cmd) {
                // Found a command start - try to extract a reasonable partial
                let partial_cmd = self.extract_reasonable_prefix(content);
                if partial_cmd != buffer.last_coherent_command.as_deref().unwrap_or("") {
                    buffer.last_coherent_command = Some(partial_cmd.clone());
                    return Some(PartialResult {
                        command: partial_cmd,
                        confidence: self.calculate_confidence(content),
                        is_coherent: true,
                    });
                }
            }
        }

        None
    }

    /// Extract a reasonable prefix that looks like a valid partial command
    fn extract_reasonable_prefix(&self, content: &str) -> String {
        let trimmed = content.trim();

        // Find the last complete word or quoted string
        let mut result = String::new();
        let mut chars = trimmed.chars().peekable();
        let mut in_quotes = false;
        let mut quote_char = '"';

        while let Some(ch) = chars.next() {
            if (ch == '"' || ch == '\'') && !in_quotes {
                in_quotes = true;
                quote_char = ch;
                result.push(ch);
            } else if ch == quote_char && in_quotes {
                in_quotes = false;
                result.push(ch);
            } else if ch.is_whitespace() && !in_quotes {
                // Check if next character starts a new word
                if chars.peek().map_or(false, |c| !c.is_whitespace()) {
                    result.push(ch);
                } else {
                    break; // Stop at incomplete word
                }
            } else {
                result.push(ch);
            }
        }

        result.trim().to_string()
    }

    /// Calculate confidence score for partial content
    fn calculate_confidence(&self, content: &str) -> f64 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0.0;
        }

        let mut score: f64 = 0.5; // Base score

        // Higher confidence if it starts with a known command
        let known_commands = [
            "ls", "find", "grep", "cat", "mv", "cp", "rm", "mkdir", "cd", "pwd", "echo", "sort",
            "uniq", "wc",
        ];
        for cmd in &known_commands {
            if trimmed.starts_with(cmd) {
                score += 0.3;
                break;
            }
        }

        // Higher confidence for complete-looking arguments
        if trimmed.contains(' ') && !trimmed.ends_with(' ') {
            score += 0.1;
        }

        // Lower confidence for incomplete quoted strings
        let quote_count = trimmed.chars().filter(|&c| c == '"' || c == '\'').count();
        if quote_count % 2 != 0 {
            score -= 0.2;
        }

        score.clamp(0.0f64, 1.0f64)
    }

    /// Get the final result from the buffer
    pub fn finalize(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        buffer.content.trim().to_string()
    }
}

/// Partial result extracted from the stream
#[derive(Debug, Clone)]
pub struct PartialResult {
    pub command: String,
    pub confidence: f64,
    pub is_coherent: bool,
}

impl GenerationStream {
    /// Create a new generation stream with the given configuration
    pub fn new(request: CommandRequest, config: StreamConfig) -> Self {
        let request_id = Uuid::new_v4().to_string();
        let cancellation_token = CancellationToken::new();

        let stream = Self::create_stream(request, config, cancellation_token.clone());

        Self {
            request_id,
            stream: Box::pin(stream),
            cancellation_token,
        }
    }

    /// Create the actual async stream
    fn create_stream(
        request: CommandRequest,
        config: StreamConfig,
        cancellation_token: CancellationToken,
    ) -> impl Stream<Item = StreamEvent> {
        async_stream::stream! {
            let mut progress_tracker = ProgressTracker::new();
            let partial_handler = PartialResultHandler::new(config.clone());
            let start_time = Instant::now();

            // Initial progress
            yield StreamEvent::Progress {
                percentage: 0.0,
                message: "Initializing generation...".to_string(),
            };

            // Simulate streaming generation with realistic behavior
            let estimated_length = request.input.len() * 2; // Rough estimate
            progress_tracker.update_estimate(estimated_length);

            let mut generated_tokens = 0;
            let mut current_command = String::new();

            // Simulate token-by-token generation
            let target_command = Self::generate_target_command(&request);
            let tokens: Vec<String> = Self::tokenize_command(&target_command);

            for (i, token) in tokens.iter().enumerate() {
                // Check for cancellation
                if cancellation_token.is_cancelled() {
                    yield StreamEvent::Cancelled;
                    return;
                }

                // Check for timeout
                if start_time.elapsed() > config.timeout {
                    yield StreamEvent::Error {
                        error: "Generation timeout".to_string(),
                        recoverable: false,
                    };
                    return;
                }

                // Add token and update progress
                current_command.push_str(token);
                generated_tokens += 1;
                progress_tracker.update_tokens(generated_tokens);

                // Emit progress updates
                if progress_tracker.should_update(config.progress_interval) {
                    let percentage = (i as f64 / tokens.len() as f64) * 100.0;
                    yield StreamEvent::Progress {
                        percentage,
                        message: format!("Generating command... ({}/{})", i + 1, tokens.len()),
                    };
                }

                // Try to emit partial results
                if let Some(partial) = partial_handler.add_chunk(token.clone()) {
                    if partial.is_coherent && partial.confidence > 0.6 {
                        yield StreamEvent::PartialResult {
                            command: partial.command,
                            confidence: partial.confidence,
                        };
                    }
                }

                // Simulate realistic generation delay
                tokio::time::sleep(Duration::from_millis(20 + (i % 100) as u64)).await;
            }

            // Final progress update
            yield StreamEvent::Progress {
                percentage: 100.0,
                message: "Finalizing command...".to_string(),
            };

            // Generate final result
            let final_command = partial_handler.finalize();
            let result = GenerationResult {
                command: final_command,
                explanation: Self::generate_explanation(&request),
                confidence: 0.95,
                generation_time: start_time.elapsed(),
                tokens_generated: generated_tokens,
                safety_validated: config.validate_partial,
            };

            yield StreamEvent::Completed { result };
        }
    }

    /// Generate a target command for the given request (simplified)
    fn generate_target_command(request: &CommandRequest) -> String {
        let input_lower = request.input.to_lowercase();

        if input_lower.contains("list") && input_lower.contains("files") {
            "ls -la".to_string()
        } else if input_lower.contains("find") {
            "find . -name \"*.txt\" -type f".to_string()
        } else if input_lower.contains("search") || input_lower.contains("grep") {
            "grep -r \"pattern\" .".to_string()
        } else if input_lower.contains("directory") && input_lower.contains("create") {
            "mkdir -p new_directory".to_string()
        } else if input_lower.contains("copy") || input_lower.contains("cp") {
            "cp source.txt destination.txt".to_string()
        } else if input_lower.contains("move") || input_lower.contains("mv") {
            "mv oldname.txt newname.txt".to_string()
        } else if input_lower.contains("disk") && input_lower.contains("usage") {
            "du -h --max-depth=1".to_string()
        } else if input_lower.contains("process") {
            "ps aux | grep process_name".to_string()
        } else {
            format!("echo 'Generated command for: {}'", request.input)
        }
    }

    /// Generate explanation for the command
    fn generate_explanation(request: &CommandRequest) -> String {
        format!(
            "This command was generated based on your request: '{}'",
            request.input
        )
    }

    /// Tokenize command into realistic chunks
    fn tokenize_command(command: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in command.chars() {
            if ch.is_whitespace() {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                tokens.push(ch.to_string());
            } else {
                current_token.push(ch);
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// Cancel the generation stream
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    /// Check if the stream has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get the request ID for this stream
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl Stream for GenerationStream {
    type Item = StreamEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SafetyLevel, ShellType};
    use futures::StreamExt;

    #[tokio::test]
    async fn test_generation_stream_creation() {
        let request = CommandRequest::new("list files", ShellType::Bash);
        let config = StreamConfig::default();
        let stream = GenerationStream::new(request, config);

        assert!(!stream.request_id.is_empty());
        assert!(!stream.is_cancelled());
    }

    #[tokio::test]
    async fn test_stream_events() {
        let request = CommandRequest::new("list files", ShellType::Bash);
        let config = StreamConfig {
            timeout: Duration::from_secs(5),
            progress_interval: Duration::from_millis(50),
            ..Default::default()
        };
        let mut stream = GenerationStream::new(request, config);

        let mut events = Vec::new();
        let mut completed = false;

        while let Some(event) = stream.next().await {
            match &event {
                StreamEvent::Completed { .. } => {
                    completed = true;
                    events.push(event);
                    break;
                }
                StreamEvent::Error { .. } | StreamEvent::Cancelled => {
                    events.push(event);
                    break;
                }
                _ => {
                    events.push(event);
                }
            }

            // Prevent infinite loops in tests
            if events.len() > 100 {
                break;
            }
        }

        assert!(completed, "Stream should complete successfully");

        // Should have at least progress and completion events
        let has_progress = events
            .iter()
            .any(|e| matches!(e, StreamEvent::Progress { .. }));
        let has_completion = events
            .iter()
            .any(|e| matches!(e, StreamEvent::Completed { .. }));

        assert!(has_progress, "Should have progress events");
        assert!(has_completion, "Should have completion event");
    }

    #[tokio::test]
    async fn test_cancellation() {
        let request = CommandRequest::new("long running command", ShellType::Bash);
        let config = StreamConfig::default();
        let mut stream = GenerationStream::new(request, config);

        // Cancel immediately
        stream.cancel();

        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event);
            if events.len() > 10 {
                break; // Safety limit
            }
        }

        let was_cancelled = events.iter().any(|e| matches!(e, StreamEvent::Cancelled));
        assert!(was_cancelled, "Stream should emit Cancelled event");
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new();

        assert_eq!(tracker.percentage(), 0.0);

        tracker.update_tokens(25);
        assert!(tracker.percentage() > 0.0);
        assert!(tracker.percentage() < 100.0);

        tracker.update_tokens(50);
        assert!(tracker.percentage() >= 95.0); // Should cap at 95% until complete
    }

    #[test]
    fn test_partial_result_handler() {
        let config = StreamConfig::default();
        let handler = PartialResultHandler::new(config);

        // Test adding chunks that form a command
        handler.add_chunk("ls".to_string());
        let result = handler.add_chunk(" -la".to_string());

        if let Some(partial) = result {
            assert!(partial.command.starts_with("ls"));
            assert!(partial.confidence > 0.0);
        }

        let final_result = handler.finalize();
        assert_eq!(final_result, "ls -la");
    }

    #[test]
    fn test_command_tokenization() {
        let tokens = GenerationStream::tokenize_command("ls -la file.txt");
        assert!(tokens.len() > 3);
        assert!(tokens.contains(&"ls".to_string()));
        assert!(tokens.contains(&"-la".to_string()));
        assert!(tokens.contains(&"file.txt".to_string()));
    }
}
