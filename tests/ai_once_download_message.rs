use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use caro::model_catalog::ModelCatalog;
use caro::prompts::CapabilityProfile;
use serial_test::serial;
use tempfile::tempdir;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct CapabilityCacheGuard(Option<std::ffi::OsString>);

impl CapabilityCacheGuard {
    fn set(path: &std::path::Path) -> Self {
        let previous = std::env::var_os("CARO_CAPABILITY_CACHE");
        std::env::set_var("CARO_CAPABILITY_CACHE", path);
        Self(previous)
    }
}

impl Drop for CapabilityCacheGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(value) => std::env::set_var("CARO_CAPABILITY_CACHE", value),
            None => std::env::remove_var("CARO_CAPABILITY_CACHE"),
        }
    }
}

#[tokio::test]
#[serial]
async fn ai_once_reports_download_before_the_request_finishes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503).set_delay(Duration::from_secs(120)))
        .mount(&server)
        .await;

    let temp_dir = tempdir().expect("create isolated CLI directories");
    let app_data = temp_dir.path().join("appdata");
    let local_app_data = temp_dir.path().join("localappdata");
    let hf_home = temp_dir.path().join("hf-home");
    let model_cache = temp_dir.path().join("model-cache");
    let capability_cache = temp_dir.path().join("capabilities.json");

    {
        let _cache_guard = CapabilityCacheGuard::set(&capability_cache);
        CapabilityProfile::detect_and_cache().await;
    }

    let mut child = ChildGuard(
        Command::new(assert_cmd::cargo::cargo_bin!("caro"))
            .args(["ai", "--once", "list files"])
            .current_dir(temp_dir.path())
            .env("CARO_BACKEND", "embedded")
            .env_remove("CARO_MOCK_BACKEND")
            .env("CARO_MODEL", "smollm-135m-q4")
            .env("CARO_CACHE_DIR", &model_cache)
            .env("CARO_CONFIG_DIR", &app_data)
            .env("CARO_CAPABILITY_CACHE", &capability_cache)
            .env("HF_ENDPOINT", server.uri())
            .env("HF_HOME", &hf_home)
            .env("APPDATA", &app_data)
            .env("LOCALAPPDATA", &local_app_data)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start caro ai --once"),
    );

    let stderr = child.0.stderr.take().expect("capture caro stderr");
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(Ok(line)) = lines.next() {
            if sender.send(line).is_err() {
                break;
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(120);
    let mut preceding_stderr = Vec::new();
    let download_message = loop {
        if let Ok(line) = receiver.try_recv() {
            if line.contains("Downloading") && line.contains("Hugging Face Hub") {
                break line;
            }
            preceding_stderr.push(line);
        }
        assert!(
            Instant::now() < deadline,
            "caro did not report the model download before the timeout; stderr: {preceding_stderr:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };

    assert!(
        download_message.contains("Downloading") && download_message.contains("Hugging Face Hub"),
        "unexpected first stderr line: {download_message}"
    );
    assert!(
        !download_message.contains("Initializing backend"),
        "the notice must describe the download, not backend construction"
    );

    let request_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if server
            .received_requests()
            .await
            .is_some_and(|requests| !requests.is_empty())
        {
            break;
        }
        assert!(
            Instant::now() < request_deadline,
            "the download notice was not followed by a model request"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(
        child.0.try_wait().expect("check child status").is_none(),
        "the delayed model request unexpectedly finished before the notice was checked"
    );

    child.0.kill().expect("stop delayed download");
    child.0.wait().expect("wait for caro process");
    reader.join().expect("join stderr reader");
}

#[test]
#[serial]
fn ai_once_does_not_report_a_download_for_a_cached_model() {
    let temp_dir = tempdir().expect("create isolated CLI directories");
    let app_data = temp_dir.path().join("appdata");
    let local_app_data = temp_dir.path().join("localappdata");
    let model_cache = temp_dir.path().join("model-cache");
    let model = ModelCatalog::by_id("smollm-135m-q4").expect("model in catalog");
    let model_path = model_cache.join(model.filename);
    std::fs::create_dir_all(model_path.parent().expect("model cache parent"))
        .expect("create isolated model cache");
    std::fs::write(&model_path, b"cached model placeholder").expect("write cached model marker");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("caro"))
        .args(["ai", "--once", "list files"])
        .current_dir(temp_dir.path())
        .env("CARO_BACKEND", "embedded")
        .env_remove("CARO_MOCK_BACKEND")
        .env("CARO_MODEL", "smollm-135m-q4")
        .env("CARO_CACHE_DIR", &model_cache)
        .env("CARO_CONFIG_DIR", &app_data)
        .env("APPDATA", app_data)
        .env("LOCALAPPDATA", local_app_data)
        .stdin(Stdio::null())
        .output()
        .expect("run caro ai --once with a cached model");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Downloading") && !stderr.contains("Hugging Face Hub"),
        "cached model unexpectedly produced a download notice: {stderr}"
    );
}
