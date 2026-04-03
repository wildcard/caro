//! Integration tests for the Paperclip AI agent integration.
//!
//! Uses wiremock to simulate the Paperclip orchestration API and verifies
//! the full heartbeat cycle: task fetch → command generation → result reporting.

#![cfg(feature = "paperclip")]

use async_trait::async_trait;
use caro::backends::{BackendInfo, CommandGenerator, GeneratorError};
use caro::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel};
use caro::paperclip::{PaperclipClient, PaperclipConfig, PaperclipRunner};
use caro::safety::{SafetyConfig, SafetyValidator};
use caro::SafetyLevel;
use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A simple mock backend that always returns "ls -la" for any query.
struct MockBackend;

#[async_trait]
impl CommandGenerator for MockBackend {
    async fn generate_command(
        &self,
        _request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        Ok(GeneratedCommand {
            command: "ls -la".to_string(),
            explanation: "List files in long format".to_string(),
            safety_level: RiskLevel::Safe,
            estimated_impact: "Read-only operation".to_string(),
            alternatives: vec![],
            backend_used: "mock".to_string(),
            generation_time_ms: 1,
            confidence_score: 0.95,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Mock,
            model_name: "mock-backend".to_string(),
            supports_streaming: false,
            max_tokens: 256,
            typical_latency_ms: 1,
            memory_usage_mb: 0,
            version: "test".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

fn test_config(api_url: &str) -> PaperclipConfig {
    PaperclipConfig {
        agent_id: "test-agent-001".to_string(),
        api_key: "test-key-abc".to_string(),
        api_url: api_url.to_string(),
        run_id: "run-123".to_string(),
    }
}

#[tokio::test]
async fn health_check_returns_true_when_server_is_up() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&server)
        .await;

    let client = PaperclipClient::new(test_config(&server.uri())).unwrap();
    assert!(client.health_check().await.unwrap());
}

#[tokio::test]
async fn health_check_returns_false_when_server_is_down() {
    // Use a port that nothing is listening on
    let client = PaperclipClient::new(test_config("http://127.0.0.1:1")).unwrap();
    assert!(!client.health_check().await.unwrap());
}

#[tokio::test]
async fn get_tasks_returns_empty_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/agents/test-agent-001/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let client = PaperclipClient::new(test_config(&server.uri())).unwrap();
    let tasks = client.get_tasks().await.unwrap();
    assert!(tasks.is_empty());
}

#[tokio::test]
async fn get_tasks_returns_tasks() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/agents/test-agent-001/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "task-1",
                "description": "list all files in the current directory",
                "priority": "high",
                "metadata": {}
            },
            {
                "id": "task-2",
                "description": "show disk usage",
                "metadata": {}
            }
        ])))
        .mount(&server)
        .await;

    let client = PaperclipClient::new(test_config(&server.uri())).unwrap();
    let tasks = client.get_tasks().await.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "task-1");
    assert_eq!(
        tasks[0].description,
        "list all files in the current directory"
    );
    assert_eq!(tasks[1].id, "task-2");
}

#[tokio::test]
async fn full_heartbeat_cycle_completes_tasks() {
    let server = MockServer::start().await;

    // Mock budget check - within budget
    Mock::given(method("GET"))
        .and(path("/api/agents/test-agent-001/budget"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "remaining": 50.0,
            "limit": 100.0,
            "currency": "USD",
            "exceeded": false
        })))
        .mount(&server)
        .await;

    // Mock task fetch - one safe task
    Mock::given(method("GET"))
        .and(path("/api/agents/test-agent-001/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "task-1",
                "description": "list files",
                "metadata": {}
            }
        ])))
        .mount(&server)
        .await;

    // Mock status updates (progress + completion)
    Mock::given(method("POST"))
        .and(path_regex(r"/api/agents/test-agent-001/tasks/.+/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(2) // One progress, one completion
        .mount(&server)
        .await;

    let client = PaperclipClient::new(test_config(&server.uri())).unwrap();
    let backend: Arc<dyn CommandGenerator> = Arc::new(MockBackend);
    let safety_config = SafetyConfig::from_level(SafetyLevel::Moderate);
    let safety = SafetyValidator::new(safety_config).unwrap();

    let runner = PaperclipRunner::new(
        client,
        backend,
        safety,
        caro::ShellType::Bash,
        SafetyLevel::Moderate,
    );

    let summary = runner.run_heartbeat().await.unwrap();
    assert_eq!(summary.tasks_received, 1);
    assert_eq!(summary.tasks_completed, 1);
    assert_eq!(summary.tasks_failed, 0);
    assert_eq!(summary.tasks_blocked_safety, 0);
}

#[tokio::test]
async fn heartbeat_skips_when_budget_exceeded() {
    let server = MockServer::start().await;

    // Mock budget check - exceeded
    Mock::given(method("GET"))
        .and(path("/api/agents/test-agent-001/budget"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "remaining": 0.0,
            "limit": 100.0,
            "currency": "USD",
            "exceeded": true
        })))
        .mount(&server)
        .await;

    let client = PaperclipClient::new(test_config(&server.uri())).unwrap();
    let backend: Arc<dyn CommandGenerator> = Arc::new(MockBackend);
    let safety_config = SafetyConfig::from_level(SafetyLevel::Moderate);
    let safety = SafetyValidator::new(safety_config).unwrap();

    let runner = PaperclipRunner::new(
        client,
        backend,
        safety,
        caro::ShellType::Bash,
        SafetyLevel::Moderate,
    );

    let summary = runner.run_heartbeat().await.unwrap();
    assert_eq!(summary.tasks_received, 0);
    assert_eq!(summary.tasks_completed, 0);
}

#[tokio::test]
async fn check_budget_returns_permissive_default_on_failure() {
    let server = MockServer::start().await;

    // Mock budget endpoint returning 500
    Mock::given(method("GET"))
        .and(path("/api/agents/test-agent-001/budget"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = PaperclipClient::new(test_config(&server.uri())).unwrap();
    let budget = client.check_budget().await.unwrap();
    // Should return permissive default (not error out)
    assert!(!budget.exceeded);
}
