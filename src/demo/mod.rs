// Demo module - Enhanced demonstration mode for showcasing capabilities

pub mod agent;
pub mod prompts;
pub mod recorder;

use crate::backends::CommandGenerator;
use crate::cli::{CliError, CliResult};
use crate::models::{CommandRequest, GeneratedCommand};
use async_trait::async_trait;
use std::path::PathBuf;

/// Demo mode orchestrator
pub struct DemoMode {
    agent: agent::DemoAgent,
    recorder: Option<recorder::AsciiCinemaRecorder>,
}

impl DemoMode {
    /// Create a new demo mode instance
    pub fn new(output_file: Option<PathBuf>) -> Result<Self, CliError> {
        let agent = agent::DemoAgent::new();
        let recorder = if let Some(path) = output_file {
            Some(recorder::AsciiCinemaRecorder::new(path)?)
        } else {
            None
        };

        Ok(Self { agent, recorder })
    }

    /// Process a request in demo mode
    pub async fn process_request(
        &mut self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, CliError> {
        // Record the input if recording is enabled
        if let Some(recorder) = &mut self.recorder {
            recorder.record_input(&request.input)?;
        }

        // Generate command using demo agent
        let result = self
            .agent
            .generate_command(request)
            .await
            .map_err(|e| CliError::GenerationFailed {
                details: e.to_string(),
            })?;

        // Record the output if recording is enabled
        if let Some(recorder) = &mut self.recorder {
            recorder.record_output(&result)?;
        }

        Ok(result)
    }

    /// Save the recording if enabled
    pub fn save_recording(&mut self) -> Result<(), CliError> {
        if let Some(recorder) = &mut self.recorder {
            recorder.save()?;
        }
        Ok(())
    }
}

/// Trait for demo-specific result formatting
#[async_trait]
pub trait DemoResultFormatter {
    /// Format result with enhanced demo information
    async fn format_demo_result(&self, result: &mut CliResult) -> Result<(), CliError>;
}

// Re-export key types
pub use agent::DemoAgent;
pub use recorder::AsciiCinemaRecorder;
