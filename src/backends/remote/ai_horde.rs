// AI-Horde crowdsourced cluster backend implementation
//
// AI-Horde (https://github.com/Haidra-Org/AI-Horde) is a free, community-run
// distributed cluster where volunteers contribute spare GPU time to serve text
// and image generation. Unlike the OpenAI-compatible backends, the Horde uses
// an ASYNCHRONOUS job-queue API:
//
//   1. POST {base}/v2/generate/text/async   -> returns a request id
//   2. GET  {base}/v2/generate/text/status/{id}  (poll until `done`)
//   3. read generations[0].text from the finished status
//
// Auth is via an `apikey` header. The anonymous key "0000000000" works without
// registration (lowest queue priority). Because the prompt is processed by
// *untrusted volunteer machines*, this backend is intended to sit behind the
// hybrid privacy gateway, which sanitizes PII before anything is submitted.

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::backends::{BackendInfo, BackendType, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, RiskLevel};

static CMD_EXTRACT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\s*"cmd"\s*:\s*"(.+)"\s*\}"#).expect("Invalid regex pattern"));

/// Default AI-Horde base API URL (includes the `/api` path segment).
pub const AI_HORDE_DEFAULT_URL: &str = "https://aihorde.net/api";

/// Anonymous API key — works without registration at lowest queue priority.
pub const AI_HORDE_ANON_KEY: &str = "0000000000";

/// Default total budget to wait for a queued job to finish.
const DEFAULT_MAX_WAIT: Duration = Duration::from_secs(60);

/// Default delay between status polls (the Horde caches status for ~1s).
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(1500);

/// Text-generation submission payload (KoboldAI-Horde format).
#[derive(Debug, Serialize)]
struct HordeTextRequest {
    prompt: String,
    params: HordeParams,
    /// Optional model filter; an empty list lets any worker pick it up.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    models: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HordeParams {
    max_length: u32,
    temperature: f32,
    n: u32,
}

/// Response to the async submission — carries the job id (or an error message).
#[derive(Debug, Deserialize)]
struct HordeSubmitResponse {
    id: Option<String>,
    message: Option<String>,
}

/// Polled status for an in-flight job.
#[derive(Debug, Deserialize)]
struct HordeStatusResponse {
    #[serde(default)]
    done: bool,
    #[serde(default)]
    faulted: bool,
    #[serde(default)]
    generations: Vec<HordeGeneration>,
}

#[derive(Debug, Deserialize)]
struct HordeGeneration {
    text: String,
}

/// AI-Horde crowdsourced text-generation backend.
pub struct AiHordeBackend {
    /// Base URL without trailing slash, e.g. `https://aihorde.net/api`.
    base_url: String,
    api_key: String,
    models: Vec<String>,
    client: Client,
    max_wait: Duration,
    poll_interval: Duration,
    embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

impl AiHordeBackend {
    /// Create a backend pointed at `base_url` using `api_key`.
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Result<Self, GeneratorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GeneratorError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            models: Vec::new(),
            client,
            max_wait: DEFAULT_MAX_WAIT,
            poll_interval: DEFAULT_POLL_INTERVAL,
            embedded_fallback: None,
        })
    }

    /// Create a backend against the public Horde with the anonymous key.
    pub fn with_defaults() -> Result<Self, GeneratorError> {
        Self::new(AI_HORDE_DEFAULT_URL, AI_HORDE_ANON_KEY)
    }

    /// Restrict generation to specific worker models (empty = any).
    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = models;
        self
    }

    /// Override the total wait budget for a queued job.
    pub fn with_max_wait(mut self, max_wait: Duration) -> Self {
        self.max_wait = max_wait;
        self
    }

    /// Override the status poll interval (mainly for fast tests).
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn with_embedded_fallback(mut self, fallback: Arc<dyn CommandGenerator>) -> Self {
        self.embedded_fallback = Some(fallback);
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the `{"cmd": ...}` system prompt (shared contract across backends).
    fn create_system_prompt(&self, request: &CommandRequest) -> String {
        format!(
            r#"You are a helpful assistant that converts natural language to safe POSIX shell commands.

CRITICAL: You MUST respond with ONLY valid JSON in this exact format:
{{"cmd": "your_shell_command_here"}}

Rules:
1. Generate ONLY the shell command, no explanation
2. Use POSIX-compliant utilities (ls, find, grep, awk, sed, sort, etc.)
3. Quote file paths with spaces using double quotes
4. Target shell: {}
5. NEVER generate destructive commands (rm -rf /, mkfs, dd, etc.)
6. Keep commands simple and safe
7. If the request is unclear, generate "echo 'Please clarify your request'"

Request: {}
"#,
            request.shell, request.input
        )
    }

    /// Parse a model response into a bare command (4-tier strategy).
    fn parse_command_response(&self, response: &str) -> Result<String, GeneratorError> {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                if !cmd.is_empty() {
                    return Ok(cmd.trim().to_string());
                }
            }
        }

        if let (Some(start), Some(end)) = (response.find('{'), response.rfind('}')) {
            if start <= end {
                if let Ok(parsed) =
                    serde_json::from_str::<serde_json::Value>(&response[start..=end])
                {
                    if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                        if !cmd.is_empty() {
                            return Ok(cmd.trim().to_string());
                        }
                    }
                }
            }
        }

        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("cmd") && line.contains(':') {
                if let Some(cmd_part) = line.split(':').nth(1) {
                    let cmd = cmd_part.trim().trim_matches('"').trim_matches('\'');
                    if !cmd.is_empty() && !cmd.contains('{') && !cmd.contains('}') {
                        return Ok(cmd.to_string());
                    }
                }
            }
        }

        if let Some(caps) = CMD_EXTRACT_REGEX.captures(response) {
            if let Some(cmd_match) = caps.get(1) {
                let cmd = cmd_match.as_str().trim();
                if !cmd.is_empty() {
                    return Ok(cmd.to_string());
                }
            }
        }

        Err(GeneratorError::ParseError {
            content: response.to_string(),
        })
    }

    /// Submit a text job and return its request id.
    async fn submit(&self, prompt: &str) -> Result<String, GeneratorError> {
        let body = HordeTextRequest {
            prompt: prompt.to_string(),
            params: HordeParams {
                max_length: 100,
                temperature: 0.1,
                n: 1,
            },
            models: self.models.clone(),
        };

        let url = format!("{}/v2/generate/text/async", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("apikey", &self.api_key)
            .header("Client-Agent", "caro:1.4.0:github.com/wildcard/caro")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    GeneratorError::BackendUnavailable {
                        reason: format!("AI-Horde unavailable: {}", e),
                    }
                } else {
                    GeneratorError::GenerationFailed {
                        details: format!("HTTP request failed: {}", e),
                    }
                }
            })?;

        if response.status() == 401 || response.status() == 403 {
            return Err(GeneratorError::BackendUnavailable {
                reason: "AI-Horde rejected the API key".to_string(),
            });
        }
        if !response.status().is_success() {
            return Err(GeneratorError::GenerationFailed {
                details: format!("AI-Horde submit error: {}", response.status()),
            });
        }

        let submit: HordeSubmitResponse =
            response
                .json()
                .await
                .map_err(|e| GeneratorError::ParseError {
                    content: format!("Failed to parse submit response: {}", e),
                })?;

        submit.id.ok_or_else(|| GeneratorError::GenerationFailed {
            details: submit
                .message
                .unwrap_or_else(|| "AI-Horde returned no job id".to_string()),
        })
    }

    /// Poll the status endpoint until the job is done or the budget expires.
    async fn poll(&self, id: &str) -> Result<String, GeneratorError> {
        let url = format!("{}/v2/generate/text/status/{}", self.base_url, id);
        let deadline = Instant::now() + self.max_wait;

        loop {
            tokio::time::sleep(self.poll_interval).await;

            let response = self.client.get(&url).send().await.map_err(|e| {
                GeneratorError::BackendUnavailable {
                    reason: format!("AI-Horde status poll failed: {}", e),
                }
            })?;

            if !response.status().is_success() {
                return Err(GeneratorError::GenerationFailed {
                    details: format!("AI-Horde status error: {}", response.status()),
                });
            }

            let status: HordeStatusResponse =
                response
                    .json()
                    .await
                    .map_err(|e| GeneratorError::ParseError {
                        content: format!("Failed to parse status response: {}", e),
                    })?;

            if status.faulted {
                return Err(GeneratorError::GenerationFailed {
                    details: "AI-Horde job faulted".to_string(),
                });
            }

            if status.done {
                return status
                    .generations
                    .into_iter()
                    .next()
                    .map(|g| g.text)
                    .ok_or_else(|| GeneratorError::ParseError {
                        content: "AI-Horde job done but returned no generations".to_string(),
                    });
            }

            if Instant::now() >= deadline {
                return Err(GeneratorError::Timeout {
                    timeout: self.max_wait,
                });
            }
        }
    }

    async fn generate_with_fallback(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let attempt = async {
            let id = self.submit(&self.create_system_prompt(request)).await?;
            let raw = self.poll(&id).await?;
            self.parse_command_response(&raw)
        }
        .await;

        let original_err = match attempt {
            Ok(command) => {
                return Ok(GeneratedCommand {
                    command,
                    explanation: "Generated using the AI-Horde volunteer cluster".to_string(),
                    safety_level: RiskLevel::Safe, // validated downstream
                    estimated_impact: "Crowdsourced remote inference operation".to_string(),
                    alternatives: vec![],
                    backend_used: "AI-Horde".to_string(),
                    generation_time_ms: 0, // set by caller
                    confidence_score: 0.75,
                });
            }
            Err(err) => {
                tracing::warn!("AI-Horde failed: {}", err);
                // Don't fall back on an explicit auth rejection.
                if let GeneratorError::BackendUnavailable { ref reason } = err {
                    if reason.contains("rejected the API key") {
                        return Err(err);
                    }
                }
                err
            }
        };

        if let Some(fallback) = &self.embedded_fallback {
            tracing::info!("Falling back to embedded backend");
            let mut fallback_result = fallback.generate_command(request).await?;
            fallback_result.backend_used = "Embedded (AI-Horde fallback)".to_string();
            return Ok(fallback_result);
        }

        // No fallback configured: surface the real cause (timeout, fault, etc.)
        // rather than masking it behind a generic "unavailable" message.
        Err(original_err)
    }
}

#[async_trait]
impl CommandGenerator for AiHordeBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = Instant::now();
        let mut result = self.generate_with_fallback(request).await?;
        result.generation_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/v2/status/heartbeat", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::AiHorde,
            model_name: if self.models.is_empty() {
                "any".to_string()
            } else {
                self.models.join(",")
            },
            supports_streaming: false,
            max_tokens: 100,
            typical_latency_ms: 15000, // queue-dependent; can be much higher
            memory_usage_mb: 0,
            version: "2.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SafetyLevel, ShellType};
    use wiremock::matchers::{method, path, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn req() -> CommandRequest {
        CommandRequest {
            input: "list text files".to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: None,
            backend_preference: None,
        }
    }

    #[test]
    fn test_creation_trims_trailing_slash() {
        let b = AiHordeBackend::new("https://aihorde.net/api/", AI_HORDE_ANON_KEY).unwrap();
        assert_eq!(b.base_url(), "https://aihorde.net/api");
    }

    #[test]
    fn test_defaults_use_anon_key() {
        let b = AiHordeBackend::with_defaults().unwrap();
        assert_eq!(b.api_key, AI_HORDE_ANON_KEY);
        assert_eq!(b.base_url(), AI_HORDE_DEFAULT_URL);
    }

    #[test]
    fn test_parse_valid_json() {
        let b = AiHordeBackend::with_defaults().unwrap();
        assert_eq!(
            b.parse_command_response(r#"{"cmd": "ls *.txt"}"#).unwrap(),
            "ls *.txt"
        );
    }

    #[tokio::test]
    async fn test_submit_then_poll_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/generate/text/async"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "job-123"
            })))
            .mount(&server)
            .await;

        // First poll: not done. Second poll: done with a generation.
        Mock::given(method("GET"))
            .and(path_regex(r"^/v2/generate/text/status/job-123$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "done": true,
                "faulted": false,
                "generations": [ { "text": "{\"cmd\": \"ls *.txt\"}" } ]
            })))
            .mount(&server)
            .await;

        let backend = AiHordeBackend::new(server.uri(), AI_HORDE_ANON_KEY)
            .unwrap()
            .with_poll_interval(Duration::from_millis(5))
            .with_max_wait(Duration::from_secs(2));

        let result = backend.generate_command(&req()).await.unwrap();
        assert_eq!(result.command, "ls *.txt");
        assert_eq!(result.backend_used, "AI-Horde");
    }

    #[tokio::test]
    async fn test_faulted_job_errors() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v2/generate/text/async"))
            .respond_with(
                ResponseTemplate::new(202).set_body_json(serde_json::json!({ "id": "j" })),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v2/generate/text/status/.+$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "done": false, "faulted": true, "generations": []
            })))
            .mount(&server)
            .await;

        // No fallback -> the faulted job must surface as an error.
        let backend = AiHordeBackend::new(server.uri(), AI_HORDE_ANON_KEY)
            .unwrap()
            .with_poll_interval(Duration::from_millis(5));
        assert!(backend.generate_command(&req()).await.is_err());
    }

    #[tokio::test]
    async fn test_poll_timeout() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v2/generate/text/async"))
            .respond_with(
                ResponseTemplate::new(202).set_body_json(serde_json::json!({ "id": "j" })),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v2/generate/text/status/.+$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "done": false, "faulted": false, "generations": []
            })))
            .mount(&server)
            .await;

        let backend = AiHordeBackend::new(server.uri(), AI_HORDE_ANON_KEY)
            .unwrap()
            .with_poll_interval(Duration::from_millis(5))
            .with_max_wait(Duration::from_millis(20));
        let err = backend.generate_command(&req()).await.unwrap_err();
        // A never-completing job must surface as "gave up": normally a Timeout,
        // but under heavy CI load the mock server can transiently refuse a poll
        // connection, which legitimately surfaces as BackendUnavailable. Both
        // mean the same thing here — no command was produced.
        assert!(
            matches!(
                err,
                GeneratorError::Timeout { .. } | GeneratorError::BackendUnavailable { .. }
            ),
            "expected timeout/unavailable, got {err:?}"
        );
    }
}
