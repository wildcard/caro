//! Candidate sources — adapters that produce a [`Candidate`] from existing
//! caro primitives.
//!
//! Phase 2 wires `BackendSource` around any [`crate::backends::CommandGenerator`]
//! impl (static matcher, embedded backend, ollama, vllm, openrouter). The
//! generated command's `confidence_score` flows into `llm_confidence`, and
//! `generation_time_ms` into `latency_ms`, so the scorer has real signal
//! before any hydrator runs.
//!
//! A `KnowledgeRetrievalSource` (turning a top-1 k-NN hit into a candidate)
//! will land in a follow-up commit once we finalize how to fetch the
//! knowledge index handle from the [`crate::agent::AgentLoop`].

use async_trait::async_trait;
use std::sync::Arc;

use super::{Candidate, CandidateSource, PipelineError};
use crate::backends::CommandGenerator;
use crate::models::{CommandRequest, ShellType};

/// Adapter that turns any [`CommandGenerator`] into a [`CandidateSource`].
///
/// `temperature` is stored purely as a label for `source` (e.g.
/// `"embedded@T=0.4"`) — the actual sampling temperature is set on the
/// underlying backend before it's wrapped. Phase 2 keeps that wiring at the
/// call site; a temperature override per-call would require a breaking
/// change to the `CommandGenerator` trait.
pub struct BackendSource {
    backend: Arc<dyn CommandGenerator>,
    label: String,
    shell: ShellType,
}

impl BackendSource {
    pub fn new(backend: Arc<dyn CommandGenerator>, label: impl Into<String>) -> Self {
        Self {
            backend,
            label: label.into(),
            shell: ShellType::Bash,
        }
    }

    pub fn with_shell(mut self, shell: ShellType) -> Self {
        self.shell = shell;
        self
    }
}

#[async_trait]
impl CandidateSource for BackendSource {
    async fn produce(&self, prompt: &str) -> Result<Candidate, PipelineError> {
        let request = CommandRequest::new(prompt, self.shell);
        let generated =
            self.backend
                .generate_command(&request)
                .await
                .map_err(|e| PipelineError::SourceFailed {
                    name: self.label.clone(),
                    message: e.to_string(),
                })?;

        let mut candidate = Candidate::new(generated.command, self.label.clone());
        candidate.features.llm_confidence = generated.confidence_score as f32;
        candidate.features.latency_ms = generated.generation_time_ms;
        Ok(candidate)
    }

    fn name(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{BackendInfo, GeneratorError};
    use crate::models::{BackendType, GeneratedCommand, RiskLevel};

    struct FakeBackend {
        confidence: f64,
        latency_ms: u64,
        cmd: String,
    }

    #[async_trait]
    impl CommandGenerator for FakeBackend {
        async fn generate_command(
            &self,
            _r: &CommandRequest,
        ) -> Result<GeneratedCommand, GeneratorError> {
            Ok(GeneratedCommand {
                command: self.cmd.clone(),
                explanation: "test".into(),
                safety_level: RiskLevel::Safe,
                estimated_impact: "none".into(),
                alternatives: vec![],
                backend_used: "fake".into(),
                generation_time_ms: self.latency_ms,
                confidence_score: self.confidence,
            })
        }
        async fn is_available(&self) -> bool {
            true
        }
        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Embedded,
                model_name: "fake".into(),
                supports_streaming: false,
                max_tokens: 0,
                typical_latency_ms: 0,
                memory_usage_mb: 0,
                version: "0".into(),
            }
        }
        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn backend_source_propagates_confidence_and_latency() {
        let backend = Arc::new(FakeBackend {
            confidence: 0.73,
            latency_ms: 250,
            cmd: "ls -la".into(),
        });
        let source = BackendSource::new(backend, "test-backend");
        let c = source.produce("list files").await.unwrap();
        assert_eq!(c.command, "ls -la");
        assert_eq!(c.source, "test-backend");
        assert!((c.features.llm_confidence - 0.73).abs() < 1e-5);
        assert_eq!(c.features.latency_ms, 250);
    }

    #[tokio::test]
    async fn backend_source_maps_errors() {
        struct BrokenBackend;
        #[async_trait]
        impl CommandGenerator for BrokenBackend {
            async fn generate_command(
                &self,
                _r: &CommandRequest,
            ) -> Result<GeneratedCommand, GeneratorError> {
                Err(GeneratorError::BackendUnavailable {
                    reason: "down".into(),
                })
            }
            async fn is_available(&self) -> bool {
                false
            }
            fn backend_info(&self) -> BackendInfo {
                BackendInfo {
                    backend_type: BackendType::Embedded,
                    model_name: "broken".into(),
                    supports_streaming: false,
                    max_tokens: 0,
                    typical_latency_ms: 0,
                    memory_usage_mb: 0,
                    version: "0".into(),
                }
            }
            async fn shutdown(&self) -> Result<(), GeneratorError> {
                Ok(())
            }
        }
        let source = BackendSource::new(Arc::new(BrokenBackend), "broken");
        let err = source.produce("anything").await.unwrap_err();
        assert!(matches!(err, PipelineError::SourceFailed { .. }));
    }
}
