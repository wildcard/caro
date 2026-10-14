//! Process-level contract for the explicit `--backend vllm` route.
//!
//! These tests run the real `caro` binary against a loopback wiremock server.
//! They prove that the actual routes (`caro test --backend vllm` and
//! `caro --backend vllm --force-llm`) honor the configured endpoint, model and
//! limits, make a real HTTP request for every case (static-matchable prompts
//! included), report distinct errors, and write JSON failures as one parseable
//! object on stdout. They do NOT prove model inference: the "model" is a mock
//! and the reply is canned. A stand-alone llama.cpp run proves only that server.
//! Integration evidence needs both together (see the PR).
//!
//! Download safety: the spawned process gets a private HOME/XDG tree and its
//! HTTP(S) proxy points at a loopback listener owned by the test. Loopback
//! traffic bypasses the proxy (`NO_PROXY`). Any attempt to reach the model
//! downloader would connect to that listener, and the test counts connections:
//! zero is required. A cache snapshot alone is not trusted. The in-process
//! constructor sentinel lives in `tests/vllm_explicit_factory.rs`.
#![cfg(feature = "remote-backends")]

use serde_json::json;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ALIAS: &str = "caro-qwen2.5-coder-0.5b-q4_k_m";
const NONCE_CMD: &str = "ls -d nonce-7f3a9c-dir";
/// A prompt the static matcher answers with `ls -la` when it is not bypassed.
const STATIC_PROMPT: &str = "list all files";

struct Out {
    success: bool,
    stdout: String,
    stderr: String,
}

struct Home {
    dir: PathBuf,
    proxy: TcpListener,
}

impl Home {
    /// A fresh directory this test creates and owns (no pre-delete).
    fn new(tag: &str, vllm_url: &str, config_model: Option<&str>) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("caro-vllm-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir(&dir).expect("fresh dir");
        let cfg = dir.join(".config").join("caro");
        std::fs::create_dir_all(&cfg).unwrap();
        let model_line = config_model
            .map(|m| format!("model_name = \"{m}\"\n"))
            .unwrap_or_default();
        std::fs::write(
            cfg.join("config.toml"),
            format!(
                "{model_line}[backends]\nvllm_url = \"{vllm_url}\"\nvllm_max_tokens = 64\nvllm_timeout_secs = 5\nvllm_logprobs = false\n"
            ),
        )
        .unwrap();
        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        proxy.set_nonblocking(true).unwrap();
        Home { dir, proxy }
    }

    fn path(&self) -> &Path {
        &self.dir
    }

    /// Connections the process made to the (never-answering) proxy: each one is
    /// an attempt to leave loopback, for example to download a model.
    fn proxy_connections(&self) -> usize {
        let mut n = 0;
        while self.proxy.accept().is_ok() {
            n += 1;
        }
        n
    }

    /// True if anything under the private tree looks like a model cache.
    fn embedded_cache_exists(&self) -> bool {
        fn walk(p: &Path) -> bool {
            std::fs::read_dir(p)
                .into_iter()
                .flatten()
                .flatten()
                .any(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name == "models"
                        || name == "huggingface"
                        || name.ends_with(".gguf")
                        || walk(&e.path())
                })
        }
        walk(&self.dir)
    }

    /// Run `caro` with a private environment, killing only this child on timeout.
    fn caro(&self, args: &[&str], extra_env: &[(&str, &str)]) -> Out {
        let proxy = format!("http://{}", self.proxy.local_addr().unwrap());
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_caro"));
        cmd.args(args)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", &self.dir)
            .env("XDG_CONFIG_HOME", self.dir.join(".config"))
            .env("XDG_CACHE_HOME", self.dir.join(".cache"))
            .env("XDG_DATA_HOME", self.dir.join(".local/share"))
            .env("NO_PROXY", "127.0.0.1,localhost")
            .env("HTTP_PROXY", &proxy)
            .env("HTTPS_PROXY", &proxy)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("spawn caro");
        let mut so = child.stdout.take().unwrap();
        let mut se = child.stderr.take().unwrap();
        let t_out = std::thread::spawn(move || {
            let mut b = String::new();
            let _ = so.read_to_string(&mut b);
            b
        });
        let t_err = std::thread::spawn(move || {
            let mut b = String::new();
            let _ = se.read_to_string(&mut b);
            b
        });
        let start = Instant::now();
        let status = loop {
            if let Some(st) = child.try_wait().unwrap() {
                break Some(st);
            }
            if start.elapsed() > Duration::from_secs(90) {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        let out = Out {
            success: status.is_some_and(|s| s.success()),
            stdout: t_out.join().unwrap(),
            stderr: t_err.join().unwrap(),
        };
        assert!(
            status.is_some(),
            "caro timed out and was killed\n{}\n{}",
            out.stdout,
            out.stderr
        );
        out
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Two cases: one nobody but the mock can answer, and one the static matcher
/// would answer (`ls -la`) if it were not bypassed. Both expect the mock's reply.
fn suite(home: &Home) -> PathBuf {
    let p = home.path().join("suite.yaml");
    let case = |input: &str| {
        format!("  - input: \"{input}\"\n    expected_outputs:\n      - \"{NONCE_CMD}\"\n    category: file_management\n")
    };
    std::fs::write(
        &p,
        format!(
            "metadata:\n  name: vllm-route\n  description: route check\ntest_cases:\n{}{}",
            case("xyzzy nonce query"),
            case(STATIC_PROMPT)
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

fn reply_for(model: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "model": model,
        "choices": [{"message": {"role": "assistant", "content": format!("{{\"cmd\": \"{NONCE_CMD}\"}}")}}],
        "usage": {"prompt_tokens": 31, "completion_tokens": 9, "total_tokens": 40}
    }))
}

fn ok_reply() -> ResponseTemplate {
    reply_for(ALIAS)
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

async fn run(
    home: &Arc<Home>,
    args: Vec<&'static str>,
    owned: Vec<String>,
    env: Vec<(&'static str, String)>,
) -> Out {
    let home = home.clone();
    tokio::task::spawn_blocking(move || {
        let mut all: Vec<&str> = args.clone();
        all.extend(owned.iter().map(String::as_str));
        let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
        home.caro(&all, &env)
    })
    .await
    .unwrap()
}

fn eval_args(
    model: Option<&str>,
    suite: &Path,
    json_out: bool,
) -> (Vec<&'static str>, Vec<String>) {
    let mut owned = vec![];
    if let Some(m) = model {
        owned.push("--model-name".to_string());
        owned.push(m.to_string());
    }
    owned.push("--suite".to_string());
    owned.push(suite.to_string_lossy().to_string());
    if json_out {
        owned.push("--output".to_string());
        owned.push("json".to_string());
    }
    (vec!["test", "--backend", "vllm"], owned)
}

fn cli_args(
    prompt: &'static str,
    model: Option<&str>,
    extra: &[&'static str],
) -> (Vec<&'static str>, Vec<String>) {
    let mut a = vec!["--backend", "vllm", "--force-llm", "--dry-run"];
    a.extend_from_slice(extra);
    a.push("-p");
    a.push(prompt);
    let owned = model
        .map(|m| vec!["--model-name".to_string(), m.to_string()])
        .unwrap_or_default();
    (a, owned)
}

// -------------------------------------------------------------------------
// Evaluation route: `caro test --backend vllm`
// -------------------------------------------------------------------------

#[tokio::test]
async fn eval_route_sends_a_real_request_for_every_case_including_static_matchable_ones() {
    let server = serve(&["some-other-alias", ALIAS, "third"], ok_reply()).await;
    let home = Arc::new(Home::new("eval", &server.uri(), None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), false);
    let out = run(&home, a, o, vec![]).await;
    // Both cases pass only if the mock's nonce came back for both, including
    // the prompt the static matcher would have answered with `ls -la`.
    assert!(out.success, "{}\n{}", out.stdout, out.stderr);
    let sent = posts(&server).await;
    assert_eq!(sent.len(), 2, "one real generation request per case");
    for body in &sent {
        assert_eq!(body["model"], ALIAS);
        assert_eq!(body["max_tokens"], 64);
        assert!(body.get("logprobs").is_none());
    }
    let prompts: String = sent.iter().map(|b| b.to_string()).collect();
    assert!(
        prompts.contains(STATIC_PROMPT),
        "the static-matchable prompt reached the server"
    );
    assert!(!out.stdout.contains("ls -la"), "no static answer was used");
    assert_eq!(home.proxy_connections(), 0, "no attempt to leave loopback");
    assert!(!home.embedded_cache_exists());
}

#[tokio::test]
async fn eval_route_with_the_endpoint_down_fails_every_case_and_never_answers_statically() {
    let home = Arc::new(Home::new("down", "http://127.0.0.1:9", None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), false);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    assert!(
        out.stderr.contains("127.0.0.1:9")
            && out.stderr.contains(ALIAS)
            && out.stderr.contains("unreachable"),
        "{}",
        out.stderr
    );
    assert_eq!(
        out.stderr.matches("could not be generated").count(),
        2,
        "both cases fail, none skipped: {}",
        out.stderr
    );
    assert!(!out.stdout.contains("ls -la"), "{}", out.stdout);
    assert_eq!(home.proxy_connections(), 0);
    assert!(!home.embedded_cache_exists());
}

#[tokio::test]
async fn eval_route_fails_when_the_selected_alias_is_absent_from_several() {
    let server = serve(&["alias-a", "alias-b"], ok_reply()).await;
    let home = Arc::new(Home::new("missing", &server.uri(), None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), false);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    assert!(
        out.stderr.contains("is not served")
            && out.stderr.contains("alias-a")
            && out.stderr.contains("alias-b"),
        "{}",
        out.stderr
    );
    assert!(
        posts(&server).await.is_empty(),
        "no generation request when the alias is missing"
    );
    assert_eq!(home.proxy_connections(), 0);
}

#[tokio::test]
async fn eval_route_fails_when_the_reply_names_another_model() {
    let server = serve(&[ALIAS], reply_for("a-different-model")).await;
    let home = Arc::new(Home::new("mismatch", &server.uri(), None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), false);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    assert!(
        out.stderr.contains("a-different-model") && out.stderr.contains("does not match"),
        "{}",
        out.stderr
    );
}

#[tokio::test]
async fn eval_route_fails_when_the_reply_does_not_name_a_model() {
    let reply = ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"role": "assistant", "content": format!("{{\"cmd\": \"{NONCE_CMD}\"}}")}}],
        "usage": {"prompt_tokens": 31, "completion_tokens": 9, "total_tokens": 40}
    }));
    let server = serve(&[ALIAS], reply).await;
    let home = Arc::new(Home::new("no-model", &server.uri(), None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), false);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    assert!(
        out.stderr.contains("does not name the model") && out.stderr.contains(ALIAS),
        "{}",
        out.stderr
    );
}

#[tokio::test]
async fn eval_json_success_is_one_parseable_object_with_no_summary_text() {
    let server = serve(&[ALIAS], ok_reply()).await;
    let home = Arc::new(Home::new("evaljson", &server.uri(), None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), true);
    let out = run(&home, a, o, vec![]).await;
    assert!(out.success, "{}\n{}", out.stdout, out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("stdout is exactly one JSON value");
    assert_eq!(v["backend"], "vllm");
    assert_eq!(v["passed"], 2);
    assert!(
        !out.stdout.contains("Evaluation Results") && !out.stdout.contains("Running evaluation")
    );
}

#[tokio::test]
async fn eval_json_failure_is_one_object_on_stdout_with_a_nonzero_status() {
    let home = Arc::new(Home::new("evaljsondown", "http://127.0.0.1:9", None));
    let (a, o) = eval_args(Some(ALIAS), &suite(&home), true);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("stdout is exactly one JSON value");
    assert_eq!(v["failed"], 2);
    let errors = v["individual_results"].to_string();
    assert!(
        errors.contains("127.0.0.1:9") && errors.contains(ALIAS),
        "{errors}"
    );
    assert!(
        !out.stdout.contains("Evaluation Results"),
        "no human summary on stdout"
    );
    assert!(
        out.stderr.contains("could not be generated"),
        "diagnostics on stderr: {}",
        out.stderr
    );
}

#[tokio::test]
async fn eval_json_setup_failure_is_one_error_object() {
    let home = Arc::new(Home::new("evaljsonsetup", "http://127.0.0.1:9", None));
    let missing = home.path().join("no-such-suite.yaml");
    let (a, o) = eval_args(Some(ALIAS), &missing, true);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).expect("one JSON object");
    assert_eq!(v["executed"], false);
    assert_eq!(v["error"]["kind"], "EvaluationFailed");
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("no-such-suite"));
}

// -------------------------------------------------------------------------
// CLI route: `caro --backend vllm --force-llm`
// -------------------------------------------------------------------------

#[tokio::test]
async fn cli_route_sends_one_real_request_even_for_a_static_matchable_prompt() {
    let server = serve(&[ALIAS], ok_reply()).await;
    let home = Arc::new(Home::new("cli", &server.uri(), None));
    let (a, o) = cli_args(STATIC_PROMPT, Some(ALIAS), &[]);
    let out = run(&home, a, o, vec![]).await;
    assert!(out.success, "{}\n{}", out.stdout, out.stderr);
    assert!(
        out.stdout.contains("nonce-7f3a9c"),
        "the mock's command, not the static `ls -la`: {}",
        out.stdout
    );
    assert!(!out.stdout.contains("ls -la"));
    let sent = posts(&server).await;
    assert_eq!(sent.len(), 1, "one real request; static matching bypassed");
    assert_eq!(sent[0]["model"], ALIAS);
    assert_eq!(home.proxy_connections(), 0);
    assert!(!home.embedded_cache_exists());
}

fn content_reply(model: Option<&str>, content: &str) -> ResponseTemplate {
    let mut body = json!({
        "choices": [{"message": {"role": "assistant", "content": content}}],
        "usage": {"prompt_tokens": 31, "completion_tokens": 9, "total_tokens": 40}
    });
    if let Some(m) = model {
        body["model"] = json!(m);
    }
    ResponseTemplate::new(200).set_body_json(body)
}

/// Synthetic data, never run: unescaped quotes plus trailing decision fields.
const CORRUPT_CONTENT: &str = r#"{"cmd": "find . -name "*.py" corrupt-marker", "needs_clarification": true, "p": 0.9, "question": "which directory?"}"#;

#[tokio::test]
async fn cli_route_fails_when_the_reply_does_not_name_a_model() {
    let server = serve(
        &[ALIAS],
        content_reply(None, &format!("{{\"cmd\": \"{NONCE_CMD}\"}}")),
    )
    .await;
    let home = Arc::new(Home::new("cli-no-model", &server.uri(), None));
    let (a, o) = cli_args(STATIC_PROMPT, Some(ALIAS), &[]);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success, "{}\n{}", out.stdout, out.stderr);
    let all = format!("{}{}", out.stdout, out.stderr);
    assert!(all.contains("does not name the model"), "{all}");
    assert!(!out.stdout.contains("nonce-7f3a9c"), "{}", out.stdout);
    assert!(!all.contains("unverified"), "{all}");
    assert_eq!(posts(&server).await.len(), 1, "no retry");
}

#[tokio::test]
async fn cli_and_eval_routes_reject_corrupt_structured_content() {
    for (name, eval) in [("cli-corrupt", false), ("eval-corrupt", true)] {
        let server = serve(&[ALIAS], content_reply(Some(ALIAS), CORRUPT_CONTENT)).await;
        let home = Arc::new(Home::new(name, &server.uri(), None));
        let (a, o) = if eval {
            eval_args(Some(ALIAS), &suite(&home), false)
        } else {
            cli_args(STATIC_PROMPT, Some(ALIAS), &[])
        };
        let out = run(&home, a, o, vec![]).await;
        assert!(!out.success, "{name}: {}\n{}", out.stdout, out.stderr);
        // The error may quote the rejected reply; no other line may carry it.
        for line in format!("{}\n{}", out.stdout, out.stderr).lines() {
            assert!(
                !line.contains("corrupt-marker") || line.contains("reply is not a command"),
                "{name}: corrupted text surfaced outside an error: {line}"
            );
        }
        if eval {
            assert!(out.stdout.contains("Overall: 0/2"), "{}", out.stdout);
        }
        assert!(
            out.stderr.contains(ALIAS) || out.stdout.contains(ALIAS),
            "{name}: {}\n{}",
            out.stdout,
            out.stderr
        );
    }
}

#[tokio::test]
async fn cli_route_unreachable_is_a_nonzero_error_naming_endpoint_and_model() {
    let home = Arc::new(Home::new("clidown", "http://127.0.0.1:9", None));
    let (a, o) = cli_args(STATIC_PROMPT, Some(ALIAS), &[]);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    let all = format!("{}{}", out.stdout, out.stderr);
    assert!(all.contains("127.0.0.1:9") && all.contains(ALIAS), "{all}");
    assert!(
        !out.stdout.contains("ls -la"),
        "no static answer when the endpoint is down"
    );
    assert_eq!(home.proxy_connections(), 0, "no fallback download attempt");
    assert!(!home.embedded_cache_exists());
}

#[tokio::test]
async fn cli_json_failure_is_one_object_on_stdout_and_diagnostics_on_stderr() {
    let home = Arc::new(Home::new("clijson", "http://127.0.0.1:9", None));
    let (a, o) = cli_args(STATIC_PROMPT, Some(ALIAS), &["-o", "json"]);
    let out = run(&home, a, o, vec![]).await;
    assert!(!out.success);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("stdout is exactly one JSON object");
    assert_eq!(v["executed"], false);
    assert_eq!(v["dry_run"], true);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("127.0.0.1:9") && msg.contains(ALIAS), "{msg}");
    assert!(
        out.stderr.contains("Error:") && out.stderr.contains(ALIAS),
        "human diagnostics on stderr: {}",
        out.stderr
    );
}

// -------------------------------------------------------------------------
// Model precedence on both routes: flag > CARO_MODEL > config model_name
// -------------------------------------------------------------------------

#[tokio::test]
async fn model_precedence_holds_on_the_cli_and_the_eval_route() {
    // The mock serves all three names so only the precedence decides the request.
    let server = serve(
        &["from-config", "from-env", "from-flag"],
        reply_for("from-config"),
    )
    .await;
    let home = Arc::new(Home::new("prec", &server.uri(), Some("from-config")));

    // config only
    let (a, o) = cli_args(STATIC_PROMPT, None, &[]);
    let _ = run(&home, a, o, vec![]).await;
    // env beats config
    let (a, o) = cli_args(STATIC_PROMPT, None, &[]);
    let _ = run(&home, a, o, vec![("CARO_MODEL", "from-env".into())]).await;
    // flag beats env
    let (a, o) = cli_args(STATIC_PROMPT, Some("from-flag"), &[]);
    let _ = run(&home, a, o, vec![("CARO_MODEL", "from-env".into())]).await;
    let cli_models: Vec<String> = posts(&server)
        .await
        .iter()
        .map(|b| b["model"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        cli_models,
        ["from-config", "from-env", "from-flag"],
        "CLI route"
    );

    let before = posts(&server).await.len();
    let one_case = home.path().join("one.yaml");
    std::fs::write(
        &one_case,
        format!("metadata:\n  name: p\n  description: p\ntest_cases:\n  - input: \"xyzzy\"\n    expected_outputs: [\"{NONCE_CMD}\"]\n    category: file_management\n"),
    )
    .unwrap();
    for (flag, env) in [
        (None, None),
        (None, Some("from-env")),
        (Some("from-flag"), Some("from-env")),
    ] {
        let (a, o) = eval_args(flag, &one_case, false);
        let envs = env
            .map(|e| vec![("CARO_MODEL", e.to_string())])
            .unwrap_or_default();
        let _ = run(&home, a, o, envs).await;
    }
    let eval_models: Vec<String> = posts(&server).await[before..]
        .iter()
        .map(|b| b["model"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        eval_models,
        ["from-config", "from-env", "from-flag"],
        "eval route"
    );
    assert_eq!(home.proxy_connections(), 0);
}

// -------------------------------------------------------------------------
// Verbose JSON: log lines must stay off stdout (#916 coordination)
// -------------------------------------------------------------------------

#[tokio::test]
async fn verbose_json_success_keeps_logs_off_stdout() {
    let server = serve(&[ALIAS], ok_reply()).await;
    let home = Arc::new(Home::new("vjsonok", &server.uri(), None));
    let (a, o) = cli_args(STATIC_PROMPT, Some(ALIAS), &["-v", "-o", "json"]);
    let out = run(&home, a, o, vec![("RUST_LOG", "caro=info".into())]).await;
    assert!(out.success, "{}\n{}", out.stdout, out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("stdout is exactly one JSON object");
    assert!(v.is_object());
    assert!(
        v["generated_command"]
            .as_str()
            .unwrap()
            .contains("nonce-7f3a9c"),
        "{v}"
    );
    assert!(
        out.stderr.contains("Using vLLM backend"),
        "the log line exists and went to stderr: {}",
        out.stderr
    );
    assert!(
        !out.stdout.contains("Using vLLM backend"),
        "no log text on stdout"
    );
    assert_eq!(posts(&server).await.len(), 1, "a real mock HTTP receipt");
    assert_eq!(home.proxy_connections(), 0);
    assert!(!home.embedded_cache_exists(), "no embedded path");
}

#[tokio::test]
async fn verbose_json_failure_keeps_logs_off_stdout_and_is_nonzero() {
    let home = Arc::new(Home::new("vjsonfail", "http://127.0.0.1:9", None));
    let (a, o) = cli_args(STATIC_PROMPT, Some(ALIAS), &["-v", "-o", "json"]);
    let out = run(&home, a, o, vec![("RUST_LOG", "caro=info".into())]).await;
    assert!(!out.success);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("stdout is exactly one JSON object");
    assert_eq!(v["executed"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("127.0.0.1:9") && msg.contains(ALIAS), "{msg}");
    assert!(
        out.stderr.contains("Using vLLM backend") && out.stderr.contains("Error:"),
        "{}",
        out.stderr
    );
    assert!(!out.stdout.contains("Using vLLM backend"));
    assert_eq!(home.proxy_connections(), 0);
    assert!(!home.embedded_cache_exists(), "no embedded path");
}
