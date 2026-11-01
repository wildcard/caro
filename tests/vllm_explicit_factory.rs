//! In-process contract for the explicit vLLM selection: the same function the
//! CLI calls (`caro::cli::explicit_vllm_backend`) builds a fail-closed backend
//! without ever running an embedded-backend constructor. The sentinel counts
//! constructor calls in this process; no test here builds an embedded backend,
//! so any nonzero count is a defect. Mock replies are not inference.
#![cfg(feature = "remote-backends")]

use caro::backends::embedded::embedded_constructions;
use caro::models::{CommandRequest, SafetyLevel, ShellType, UserConfiguration};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ALIAS: &str = "caro-qwen2.5-coder-0.5b-q4_k_m";

fn req() -> CommandRequest {
    CommandRequest {
        input: "list files".into(),
        shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        context: None,
        backend_preference: None,
    }
}

fn config(url: &str, backend: &str) -> UserConfiguration {
    UserConfiguration {
        default_model: Some(backend.to_string()),
        model_name: Some(ALIAS.to_string()),
        backends: caro::models::BackendsConfig {
            vllm_url: Some(url.to_string()),
            vllm_max_tokens: Some(64),
            vllm_logprobs: Some(false),
            ..Default::default()
        },
        ..Default::default()
    }
}

async fn server() -> MockServer {
    let s = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": ALIAS}]})))
        .mount(&s)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": ALIAS,
            "choices": [{"message": {"role": "assistant", "content": "{\"cmd\": \"ls -la\"}"}}],
            "usage": {"prompt_tokens": 20, "completion_tokens": 5, "total_tokens": 25}
        })))
        .mount(&s)
        .await;
    s
}

#[tokio::test]
async fn explicit_vllm_never_constructs_the_embedded_backend() {
    let before = embedded_constructions();
    let s = server().await;

    // Mixed letter case still selects the explicit route.
    let backend = caro::cli::explicit_vllm_backend(&config(&s.uri(), "VLLM"))
        .expect("explicit vllm selected")
        .expect("backend builds");
    let out = backend
        .generate_command(&req())
        .await
        .expect("generation succeeds");
    assert_eq!(out.command, "ls -la");

    // Unreachable endpoint: an error that names endpoint and model, not a switch.
    let down = caro::cli::explicit_vllm_backend(&config("http://127.0.0.1:9", "vllm"))
        .unwrap()
        .unwrap();
    let err = down.generate_command(&req()).await.unwrap_err().to_string();
    assert!(err.contains("127.0.0.1:9") && err.contains(ALIAS), "{err}");

    // Model not served: same.
    let other = {
        let mut c = config(&s.uri(), "vllm");
        c.model_name = Some("not-served".into());
        caro::cli::explicit_vllm_backend(&c).unwrap().unwrap()
    };
    let err = other
        .generate_command(&req())
        .await
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("not-served") && err.contains("is not served"),
        "{err}"
    );

    assert_eq!(
        embedded_constructions(),
        before,
        "no embedded constructor ran"
    );
    assert_eq!(
        embedded_constructions(),
        0,
        "and none ran earlier in this process"
    );
}

#[test]
fn other_backends_are_not_taken_by_the_explicit_route() {
    for b in ["embedded", "ollama", "mesh", "auto"] {
        assert!(
            caro::cli::explicit_vllm_backend(&config("http://127.0.0.1:1", b)).is_none(),
            "{b}"
        );
    }
    let mut none = config("http://127.0.0.1:1", "vllm");
    none.default_model = None;
    assert!(caro::cli::explicit_vllm_backend(&none).is_none());
}

#[test]
fn model_precedence_is_flag_then_env_then_config() {
    use caro::cli::resolve_model_name as r;
    let s = |v: &str| Some(v.to_string());
    assert_eq!(r(s("flag"), s("env"), s("cfg")), s("flag"));
    assert_eq!(r(None, s("env"), s("cfg")), s("env"));
    assert_eq!(r(None, None, s("cfg")), s("cfg"));
    assert_eq!(r(None, None, None), None);
}
