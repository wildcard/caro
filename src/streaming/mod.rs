//! Streaming Command Generation Support
//!
//! This module provides real-time streaming capabilities for command generation,
//! allowing users to see partial results as they're generated rather than waiting
//! for complete responses.
//!
//! # Features
//!
//! - **Progressive Generation**: Stream partial command suggestions in real-time
//! - **Safety Integration**: Apply safety validation to streaming content
//! - **Cancellation Support**: Allow users to interrupt long-running generations
//! - **Buffer Management**: Handle partial responses and maintain generation state
//! - **Multi-Backend Support**: Work with both local and remote backends
//!
//! # Architecture
//!
//! The streaming system operates through async streams that yield partial results:
//!
//! ```text
//! User Input → StreamingGenerator → Safety Validation → UI Display
//!                     ↓                    ↓              ↑
//!              Partial Chunks     Risk Assessment    Live Updates
//! ```
//!
//! # Example
//!
//! ```no_run
//! use cmdai::streaming::{StreamingGenerator, StreamingConfig, StreamChunk};
//! use cmdai::models::{CommandRequest, ShellType};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = StreamingConfig::default();
//! let generator = StreamingGenerator::new(config).await?;
//! 
//! let request = CommandRequest::new("find large files", ShellType::Bash);
//! let mut stream = generator.generate_streaming(&request).await?;
//!
//! while let Some(chunk) = stream.next().await {
//!     match chunk? {
//!         StreamChunk::Partial { content, confidence } => {
//!             print!("{}", content); // Show partial command as it generates
//!         }
//!         StreamChunk::Complete { final_command } => {
//!             println!("\nFinal: {}", final_command.command);
//!             break;
//!         }
//!         StreamChunk::Error { error, partial } => {
//!             eprintln!("Error: {}", error);
//!             break;
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod stream;

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};
use crate::safety::SafetyValidator;

/// Configuration for streaming command generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum time to wait between stream chunks
    pub chunk_timeout_ms: u64,
    /// Minimum chunk size before yielding partial results
    pub min_chunk_size: usize,
    /// Maximum buffer size for accumulating chunks
    pub max_buffer_size: usize,
    /// Enable real-time safety validation of partial content
    pub enable_streaming_safety: bool,
    /// Yield intermediate results even if unsafe
    pub yield_unsafe_partial: bool,
    /// Debounce time for rapid updates (UI smoothing)
    pub debounce_ms: u64,
    /// Maximum streaming duration before timeout
    pub max_streaming_duration_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_timeout_ms: 100,
            min_chunk_size: 5,
            max_buffer_size: 4096,
            enable_streaming_safety: true,
            yield_unsafe_partial: false,
            debounce_ms: 50,
            max_streaming_duration_ms: 30000, // 30 seconds
        }
    }
}

impl StreamingConfig {
    /// Configuration optimized for interactive UI use
    pub fn interactive() -> Self {
        Self {
            chunk_timeout_ms: 50,
            min_chunk_size: 3,
            debounce_ms: 25,
            max_streaming_duration_ms: 15000, // Faster timeout for UI
            ..Self::default()
        }
    }

    /// Configuration for batch processing with larger chunks
    pub fn batch() -> Self {
        Self {
            chunk_timeout_ms: 500,
            min_chunk_size: 20,
            max_buffer_size: 8192,
            debounce_ms: 100,
            max_streaming_duration_ms: 60000, // Longer timeout for batch
            ..Self::default()
        }
    }
}

/// Types of streaming chunks yielded during generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamChunk {
    /// Partial command content with confidence score
    Partial {
        content: String,
        confidence: f32,
        is_safe: bool,
        accumulated_length: usize,
    },
    /// Complete generated command with full validation
    Complete {
        final_command: GeneratedCommand,
        generation_stats: StreamingStats,
    },
    /// Error occurred during generation
    Error {
        error: String,
        partial_content: Option<String>,
        recovery_suggestion: Option<String>,
    },
    /// Generation was cancelled by user
    Cancelled {
        partial_content: String,
        reason: String,
    },
    /// Safety warning for partial content
    SafetyWarning {
        warning: String,
        risk_level: RiskLevel,
        affected_content: String,
    },
}

/// Statistics about a streaming generation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStats {
    pub total_chunks: u32,
    pub total_duration_ms: u64,
    pub average_chunk_size: f32,
    pub safety_checks_performed: u32,
    pub backend_used: String,
    pub final_length: usize,
    pub was_cancelled: bool,
}

/// Cancellation token for interrupting streaming generation
#[derive(Debug, Clone)]
pub struct CancellationToken {
    sender: mpsc::UnboundedSender<()>,
}

impl CancellationToken {
    /// Cancel the streaming operation
    pub fn cancel(&self) -> Result<(), StreamingError> {
        self.sender.send(()).map_err(|_| StreamingError::AlreadyCancelled)?;
        Ok(())
    }
}

/// Buffer for accumulating partial streaming content
#[derive(Debug)]
struct StreamBuffer {
    content: String,
    chunks: VecDeque<String>,
    last_update: Instant,
    total_chunks: u32,
    safety_checks: u32,
}

impl StreamBuffer {
    fn new() -> Self {
        Self {
            content: String::new(),
            chunks: VecDeque::new(),
            last_update: Instant::now(),
            total_chunks: 0,
            safety_checks: 0,
        }
    }

    fn add_chunk(&mut self, chunk: String) {
        self.content.push_str(&chunk);
        self.chunks.push_back(chunk);
        self.last_update = Instant::now();
        self.total_chunks += 1;
    }

    fn should_yield(&self, config: &StreamingConfig) -> bool {
        self.content.len() >= config.min_chunk_size &&
        self.last_update.elapsed() >= Duration::from_millis(config.debounce_ms)
    }

    fn get_stats(&self, duration: Duration, backend: String) -> StreamingStats {
        StreamingStats {
            total_chunks: self.total_chunks,
            total_duration_ms: duration.as_millis() as u64,
            average_chunk_size: if self.total_chunks > 0 {
                self.content.len() as f32 / self.total_chunks as f32
            } else {
                0.0
            },
            safety_checks_performed: self.safety_checks,
            backend_used: backend,
            final_length: self.content.len(),
            was_cancelled: false,
        }
    }
}

/// Error types for streaming operations
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum StreamingError {
    #[error("Streaming not supported by backend: {backend}")]
    NotSupported { backend: String },

    #[error("Streaming timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    #[error("Buffer overflow: {size} bytes exceeds limit")]
    BufferOverflow { size: usize },

    #[error("Generation cancelled by user")]
    Cancelled,

    #[error("Already cancelled")]
    AlreadyCancelled,

    #[error("Safety validation failed: {reason}")]
    SafetyViolation { reason: String },

    #[error("Backend error during streaming: {details}")]
    BackendError { details: String },

    #[error("Invalid streaming configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Internal streaming error: {message}")]
    Internal { message: String },
}

/// Trait for backends that support streaming command generation
#[async_trait]
pub trait StreamingCommandGenerator: CommandGenerator {
    /// Generate a command using streaming, yielding partial results
    async fn generate_streaming(
        &self,
        request: &CommandRequest,
        config: &StreamingConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, GeneratorError>> + Send + 'static>>, GeneratorError>;

    /// Check if this backend supports streaming
    fn supports_streaming(&self) -> bool {
        true
    }

    /// Get optimal chunk size for this backend
    fn optimal_chunk_size(&self) -> usize {
        10
    }
}

/// Main streaming generator that wraps existing CommandGenerator backends
pub struct StreamingGenerator {
    backend: Arc<dyn CommandGenerator>,
    safety_validator: Option<SafetyValidator>,
    config: StreamingConfig,
}

impl StreamingGenerator {
    /// Create a new streaming generator
    pub async fn new(
        backend: Arc<dyn CommandGenerator>,
        config: StreamingConfig,
    ) -> Result<Self, StreamingError> {
        // Validate configuration
        if config.chunk_timeout_ms == 0 || config.max_buffer_size == 0 {
            return Err(StreamingError::InvalidConfig {
                message: "Timeout and buffer size must be positive".to_string(),
            });
        }

        let safety_validator = if config.enable_streaming_safety {
            Some(SafetyValidator::new(crate::safety::SafetyConfig::moderate())
                .map_err(|e| StreamingError::Internal {
                    message: format!("Failed to create safety validator: {}", e),
                })?)
        } else {
            None
        };

        Ok(Self {
            backend,
            safety_validator,
            config,
        })
    }

    /// Generate a command with streaming support
    pub async fn generate_streaming(
        &self,
        request: &CommandRequest,
    ) -> Result<(Pin<Box<dyn Stream<Item = Result<StreamChunk, StreamingError>> + Send>>, CancellationToken), StreamingError> {
        let _start_time = Instant::now();
        
        // Create cancellation channel
        let (cancel_tx, mut cancel_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken { sender: cancel_tx };

        // Check if backend supports streaming
        let streaming_backend = self.backend.clone();
        let backend_info = streaming_backend.backend_info();
        
        // For now, simulate streaming with non-streaming backends
        let request_clone = request.clone();
        let config_clone = self.config.clone();
        let safety_validator = self.safety_validator.as_ref().map(|_| {
            SafetyValidator::new(crate::safety::SafetyConfig::moderate()).unwrap()
        });

        let stream = async_stream::stream! {
            let mut buffer = StreamBuffer::new();
            let generation_start = Instant::now();

            // Try to detect if backend has streaming support via trait casting
            // For now, we'll simulate streaming by generating in chunks
            let result = tokio::select! {
                res = streaming_backend.generate_command(&request_clone) => res,
                _ = cancel_rx.recv() => {
                    yield Ok(StreamChunk::Cancelled {
                        partial_content: buffer.content.clone(),
                        reason: "User cancelled".to_string(),
                    });
                    return;
                }
                _ = sleep(Duration::from_millis(config_clone.max_streaming_duration_ms)) => {
                    yield Err(StreamingError::Timeout {
                        duration_ms: config_clone.max_streaming_duration_ms
                    });
                    return;
                }
            };

            match result {
                Ok(final_command) => {
                    let command_text = &final_command.command;
                    
                    // Simulate streaming by yielding the command in chunks
                    let chars: Vec<char> = command_text.chars().collect();
                    let chunk_size = config_clone.min_chunk_size.max(1);
                    
                    // Yield partial chunks
                    let mut i = 0;
                    while i < chars.len() {
                        // Check for cancellation
                        if cancel_rx.try_recv().is_ok() {
                            yield Ok(StreamChunk::Cancelled {
                                partial_content: buffer.content.clone(),
                                reason: "User cancelled during streaming".to_string(),
                            });
                            return;
                        }

                        let end = std::cmp::min(i + chunk_size, chars.len());
                        let chunk: String = chars[i..end].iter().collect();
                        buffer.add_chunk(chunk.clone());
                        
                        // Perform safety check if enabled
                        let mut is_safe = true;
                        if let Some(ref validator) = safety_validator {
                            if config_clone.enable_streaming_safety {
                                match validator.validate_command(&buffer.content, request_clone.shell).await {
                                    Ok(validation) => {
                                        buffer.safety_checks += 1;
                                        if !validation.allowed {
                                            is_safe = false;
                                            if !config_clone.yield_unsafe_partial {
                                                yield Ok(StreamChunk::SafetyWarning {
                                                    warning: validation.explanation,
                                                    risk_level: validation.risk_level,
                                                    affected_content: buffer.content.clone(),
                                                });
                                                continue;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        yield Ok(StreamChunk::SafetyWarning {
                                            warning: format!("Safety validation error: {}", e),
                                            risk_level: RiskLevel::Moderate,
                                            affected_content: buffer.content.clone(),
                                        });
                                    }
                                }
                            }
                        }

                        // Yield partial result if buffer conditions are met
                        if buffer.should_yield(&config_clone) || i + chunk_size >= chars.len() {
                            let confidence = if i + chunk_size >= chars.len() { 1.0 } else {
                                0.5 + (i as f32 / chars.len() as f32) * 0.5
                            };

                            yield Ok(StreamChunk::Partial {
                                content: buffer.content.clone(),
                                confidence,
                                is_safe,
                                accumulated_length: buffer.content.len(),
                            });
                        }

                        i += chunk_size;
                        
                        // Simulate realistic streaming delay
                        tokio::time::sleep(Duration::from_millis(config_clone.chunk_timeout_ms)).await;
                    }

                    // Yield final complete result
                    let generation_duration = generation_start.elapsed();
                    let stats = buffer.get_stats(generation_duration, backend_info.model_name.clone());

                    yield Ok(StreamChunk::Complete {
                        final_command,
                        generation_stats: stats,
                    });
                }
                Err(error) => {
                    yield Ok(StreamChunk::Error {
                        error: error.to_string(),
                        partial_content: if buffer.content.is_empty() { None } else { Some(buffer.content.clone()) },
                        recovery_suggestion: Some("Try simplifying your request or check backend availability".to_string()),
                    });
                }
            }
        };

        Ok((Box::pin(stream), cancellation_token))
    }

    /// Get current configuration
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: StreamingConfig) -> Result<(), StreamingError> {
        // Validate new configuration
        if config.chunk_timeout_ms == 0 || config.max_buffer_size == 0 {
            return Err(StreamingError::InvalidConfig {
                message: "Timeout and buffer size must be positive".to_string(),
            });
        }

        self.config = config;
        Ok(())
    }

    /// Check if streaming is available for current backend
    pub fn is_streaming_available(&self) -> bool {
        // For now, we support streaming simulation for all backends
        true
    }
}

/// Wrapper to add streaming capabilities to existing CommandGenerator
pub struct StreamingWrapper<T: CommandGenerator> {
    inner: T,
}

impl<T: CommandGenerator> StreamingWrapper<T> {
    pub fn new(generator: T) -> Self {
        Self { inner: generator }
    }
}

#[async_trait]
impl<T: CommandGenerator> CommandGenerator for StreamingWrapper<T> {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        self.inner.generate_command(request).await
    }

    async fn is_available(&self) -> bool {
        self.inner.is_available().await
    }

    fn backend_info(&self) -> BackendInfo {
        let mut info = self.inner.backend_info();
        info.supports_streaming = true; // Mark as streaming-capable
        info
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        self.inner.shutdown().await
    }
}

#[async_trait]
impl<T: CommandGenerator> StreamingCommandGenerator for StreamingWrapper<T> {
    async fn generate_streaming(
        &self,
        request: &CommandRequest,
        config: &StreamingConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, GeneratorError>> + Send + 'static>>, GeneratorError> {
        // For backends that don't natively support streaming, we simulate it
        let result = self.inner.generate_command(request).await?;
        let command = result.command;
        let chunk_size = config.min_chunk_size.max(1);
        let chunk_timeout = config.chunk_timeout_ms;
        
        let stream = async_stream::stream! {
            let chars: Vec<char> = command.chars().collect();
            let mut i = 0;
            
            while i < chars.len() {
                let end = std::cmp::min(i + chunk_size, chars.len());
                let chunk: String = chars[i..end].iter().collect();
                yield Ok(chunk);
                i += chunk_size;
                
                // Add realistic delay between chunks
                tokio::time::sleep(Duration::from_millis(chunk_timeout)).await;
            }
        };
        
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, ShellType};
    use futures::StreamExt;
    use std::time::Duration;

    #[derive(Debug)]
    struct MockStreamingBackend;

    #[async_trait]
    impl CommandGenerator for MockStreamingBackend {
        async fn generate_command(&self, _request: &CommandRequest) -> Result<GeneratedCommand, GeneratorError> {
            Ok(GeneratedCommand {
                command: "find /home -type f -size +100M".to_string(),
                explanation: "Find large files in home directory".to_string(),
                safety_level: RiskLevel::Safe,
                estimated_impact: "Read-only file search".to_string(),
                alternatives: vec![],
                backend_used: "mock".to_string(),
                generation_time_ms: 100,
                confidence_score: 0.9,
            })
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Embedded,
                model_name: "mock-streaming".to_string(),
                supports_streaming: true,
                max_tokens: 100,
                typical_latency_ms: 50,
                memory_usage_mb: 50,
                version: "test".to_string(),
            }
        }

        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_streaming_generator_creation() {
        let backend = Arc::new(MockStreamingBackend);
        let config = StreamingConfig::default();
        let generator = StreamingGenerator::new(backend, config).await;
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_streaming_generation() {
        let backend = Arc::new(MockStreamingBackend);
        let config = StreamingConfig::interactive();
        let generator = StreamingGenerator::new(backend, config).await.unwrap();

        let request = CommandRequest::new("find large files", ShellType::Bash);
        let (mut stream, _token) = generator.generate_streaming(&request).await.unwrap();

        let mut chunks_received = 0;
        let mut final_result = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            match chunk {
                StreamChunk::Partial { content, confidence, is_safe, .. } => {
                    assert!(!content.is_empty());
                    assert!(confidence >= 0.0 && confidence <= 1.0);
                    assert!(is_safe); // Mock command should be safe
                    chunks_received += 1;
                }
                StreamChunk::Complete { final_command, generation_stats } => {
                    assert_eq!(final_command.command, "find /home -type f -size +100M");
                    assert!(generation_stats.total_chunks > 0);
                    final_result = Some(final_command);
                    break;
                }
                StreamChunk::Error { .. } => panic!("Unexpected error chunk"),
                StreamChunk::Cancelled { .. } => panic!("Unexpected cancellation"),
                StreamChunk::SafetyWarning { .. } => {
                    // Safety warnings are acceptable but shouldn't occur with mock safe command
                }
            }
        }

        assert!(chunks_received > 0, "Should receive partial chunks");
        assert!(final_result.is_some(), "Should receive final result");
    }

    #[tokio::test]
    async fn test_streaming_cancellation() {
        let backend = Arc::new(MockStreamingBackend);
        let mut config = StreamingConfig::default();
        config.chunk_timeout_ms = 200; // Slower chunks for testing cancellation
        let generator = StreamingGenerator::new(backend, config).await.unwrap();

        let request = CommandRequest::new("long command", ShellType::Bash);
        let (mut stream, cancellation_token) = generator.generate_streaming(&request).await.unwrap();

        // Cancel after first chunk
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = cancellation_token.cancel();
        });

        let mut was_cancelled = false;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            if let StreamChunk::Cancelled { .. } = chunk {
                was_cancelled = true;
                break;
            }
        }

        assert!(was_cancelled, "Stream should have been cancelled");
    }

    #[tokio::test]
    async fn test_streaming_config_validation() {
        let backend = Arc::new(MockStreamingBackend);
        
        let invalid_config = StreamingConfig {
            chunk_timeout_ms: 0, // Invalid
            ..StreamingConfig::default()
        };
        
        let result = StreamingGenerator::new(backend, invalid_config).await;
        assert!(result.is_err());
        
        if let Err(StreamingError::InvalidConfig { message }) = result {
            assert!(message.contains("positive"));
        } else {
            panic!("Expected InvalidConfig error");
        }
    }

    #[tokio::test]
    async fn test_streaming_wrapper() {
        let backend = MockStreamingBackend;
        let wrapper = StreamingWrapper::new(backend);
        
        assert!(wrapper.backend_info().supports_streaming);
        assert!(wrapper.is_available().await);
        
        let request = CommandRequest::new("test", ShellType::Bash);
        let result = wrapper.generate_command(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_different_streaming_configs() {
        let backend = Arc::new(MockStreamingBackend);
        
        // Test interactive config
        let interactive_config = StreamingConfig::interactive();
        let generator = StreamingGenerator::new(backend.clone(), interactive_config).await.unwrap();
        assert!(generator.is_streaming_available());
        
        // Test batch config
        let batch_config = StreamingConfig::batch();
        let generator = StreamingGenerator::new(backend, batch_config).await.unwrap();
        assert_eq!(generator.config().min_chunk_size, 20);
    }
}