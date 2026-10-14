//! Process-level contract for the explicit `--backend vllm` route.
//!
//! These tests run the real `caro` binary against a loopback wiremock server.
//! They prove that the actual routes (`caro test --backend vllm` and
//! `caro --backend vllm --force-llm`) honor the configured endpoint, model and
//! limits, make a real HTTP request, never construct the embedded backend, and
//! report distinct errors. They do NOT prove model inference: the "model" is a
//! mock and the reply is canned. A stand-alone llama.cpp run proves only that
//! server. Integration evidence needs both together (see the PR).
//!
//! The spawned process gets a private HOME/XDG tree and a dead HTTP(S) proxy,
//! so even a bug that reached the model downloader could not fetch anything.
#![cfg(feature = "remote-backends")]

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ALIAS: &str = "qwen-test-alias";
const NONCE_CMD: &str = "ls -d nonce-7f3a9c-dir";

struct Home(PathBuf);

impl Home {
    /// A fresh directory this test creates and owns (no pre-delete).
    fn new(tag: &str, vllm_url: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("caro-vllm-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir(&dir).expect("fresh dir");
        let cfg = dir.join(".config").join("caro");
        std::fs::create_dir_all(&cfg).unwrap();
        std::fs::write(
            cfg.join("config.toml"),
            format!(
                "[backends]\nvllm_url = \"{vllm_url}\"\nvllm_max_tokens = 64\nvllm_timeout_secs = 5\nvllm_logprobs = false\n"
            ),
        )
        .unwrap();
        Home(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// True if anything under the private tree looks like the embedded model cache.
    fn embedded_cache_exists(&self) -> bool {
        fn walk(p: &Path) -> bool {
            std::fs::read_dir(p)
                .into_iter()
                .flatten()
                .flatten()
                .any(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name == "models"
                        || name.ends_with(".gguf")
                        || name == "huggingface"
                        || walk(&e.path())
                })
        }
        walk(&self.0.join(".cache")) || walk(&self.0.join(".local"))
    }

    fn caro(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_caro"))
            .args(args)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", &self.0)
            .env("XDG_CONFIG_HOME", self.0.join(".config"))
            .env("XDG_CACHE_HOME", self.0.join(".cache"))
            .env("XDG_DATA_HOME", self.0.join(".local/share"))
            .env("NO_PROXY", "127.0.0.1,localhost")
            .env("HTTP_PROXY", "http://127.0.0.1:9")
            .env("HTTPS_PROXY", "http://127.0.0.1:9")
            .env("CARO_TELEMETRY", "off")
            .output()
            .expect("run caro")
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn suite(home: &Home) -> PathBuf {
    let p = home.path().join("suite.yaml");
    std::fs::write(
        &p,
        format!(
            "metadata:\n  name: vllm-route\n  description: route check\ntest_cases:\n  - input: \"xyzzy nonce query\"\n    expected_outputs:\n      - \"{}\"\n    category: file_management\n",
            NONCE_CMD
        ),
    )
    .unwrap();
    p
}

async fn serve(models: &[&str], chat: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": models.iter().map(|m| json!({"id": m})).collect::<Vec<_>>()
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(chat)
        .mount(&server)
        .await;
    server
}

fn ok_reply() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "model": ALIAS,
        "choices": [{"message": {"role": "assistant",
            "content": format!("{{\"cmd\": \"{}\"}}", NONCE_CMD)}}],
        "usage": {"prompt_tokens": 31, "completion_tokens": 9, "total_tokens": 40}
    }))
}

async fn posts(server: &MockServer) -> Vec<serde_json::Value> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.method.as_str() == "POST")
        .map(|r| serde_json::from_slice(&r.body).unwrap())
        .collect()
}

async fn run(home: &std::sync::Arc<Home>, args: Vec<String>) -> Output {
    let home = home.clone();
    tokio::task::spawn_blocking(move || {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        home.caro(&refs)
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn eval_route_makes_one_real_request_with_configured_model_and_limits() {
    let server = serve(&[ALIAS], ok_reply()).await;
    let home = std::sync::Arc::new(Home::new("eval", &server.uri()));
    let suite = suite(&home);
    let out = run(
        &home,
        vec![
            "test".into(),
            "--backend".into(),
            "vllm".into(),
            "--model-name".into(),
            ALIAS.into(),
            "--suite".into(),
            suite.to_string_lossy().into(),
        ],
    )
    .await;
    // The case passes only if the canned nonce command came back over HTTP:
    // the static matcher cannot produce it.
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let sent = posts(&server).await;
    assert_eq!(sent.len(), 1, "exactly one real generation request");
    assert_eq!(sent[0]["model"], ALIAS);
    assert_eq!(sent[0]["max_tokens"], 64);
    assert!(sent[0].get("logprobs").is_none());
    assert!(
        !home.embedded_cache_exists(),
        "embedded backend must not be built"
    );
}

#[tokio::test]
async fn eval_route_reports_unreachable_endpoint_and_model_without_embedded() {
    let home = std::sync::Arc::new(Home::new("down", "http://127.0.0.1:9"));
    let suite = suite(&home);
    let out = run(
        &home,
        vec![
            "test".into(),
            "--backend".into(),
            "vllm".into(),
            "--model-name".into(),
            ALIAS.into(),
            "--suite".into(),
            suite.to_string_lossy().into(),
        ],
    )
    .await;
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("127.0.0.1:9") && stderr.contains(ALIAS) && stderr.contains("unreachable"),
        "{stderr}"
    );
    assert!(!home.embedded_cache_exists());
}

#[tokio::test]
async fn eval_route_fails_when_selected_model_is_not_served() {
    let server = serve(&["some-other-model"], ok_reply()).await;
    let home = std::sync::Arc::new(Home::new("missing", &server.uri()));
    let suite = suite(&home);
    let out = run(
        &home,
        vec![
            "test".into(),
            "--backend".into(),
            "vllm".into(),
            "--model-name".into(),
            ALIAS.into(),
            "--suite".into(),
            suite.to_string_lossy().into(),
        ],
    )
    .await;
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("is not served") && stderr.contains("some-other-model"),
        "{stderr}"
    );
    assert!(
        posts(&server).await.is_empty(),
        "no generation request when the model is missing"
    );
}

#[tokio::test]
async fn cli_route_forces_the_backend_and_returns_its_command() {
    let server = serve(&[ALIAS], ok_reply()).await;
    let home = std::sync::Arc::new(Home::new("cli", &server.uri()));
    let out = run(
        &home,
        vec![
            "--backend".into(),
            "vllm".into(),
            "--model-name".into(),
            ALIAS.into(),
            "--force-llm".into(),
            "--dry-run".into(),
            "-p".into(),
            "xyzzy nonce query".into(),
        ],
    )
    .await;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("nonce-7f3a9c"), "{stdout}");
    let sent = posts(&server).await;
    assert_eq!(sent.len(), 1, "one real request; static matching bypassed");
    assert_eq!(sent[0]["model"], ALIAS);
    assert!(!home.embedded_cache_exists());
}

#[tokio::test]
async fn cli_route_unreachable_is_a_nonzero_error_naming_endpoint_and_model() {
    let home = std::sync::Arc::new(Home::new("clidown", "http://127.0.0.1:9"));
    let out = run(
        &home,
        vec![
            "--backend".into(),
            "vllm".into(),
            "--model-name".into(),
            ALIAS.into(),
            "--force-llm".into(),
            "--dry-run".into(),
            "-p".into(),
            "xyzzy nonce query".into(),
        ],
    )
    .await;
    assert!(!out.status.success());
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains("127.0.0.1:9") && all.contains(ALIAS), "{all}");
    assert!(
        !home.embedded_cache_exists(),
        "no embedded fallback, no model download"
    );
}
