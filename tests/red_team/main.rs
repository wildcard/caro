//! Red-team detonation suite (ADR-017 phase P2).
//!
//! Executes every entry of the dangerous-command corpus
//! (`tests/evaluation/datasets/safety/dangerous_patterns.json`) inside a
//! disposable Cloudflare Sandbox container via the tier-1 exec-harness worker
//! (`tools/exec-harness/worker/`), records the observed blast radius as an
//! evidence file, and flags corpus entries whose *measured* impact contradicts
//! their labeled risk level. This is what turns "52+ dangerous patterns" from
//! an assertion into a measurement — the suite HELP_WANTED.md promises and
//! docs/SECURITY-CHECKLIST.md Gate 6 references.
//!
//! Network-gated like the repo's other env-dependent tests:
//!
//! ```bash
//! CARO_DETONATION_URL=https://caro-exec-harness.<account>.workers.dev \
//! CARO_DETONATION_TOKEN=<bearer> \
//! cargo test --test red_team -- --ignored
//! ```
//!
//! Runs nightly, non-blocking, via `.github/workflows/detonation-nightly.yml`
//! once the Cloudflare secrets exist (human-created per decision D5).

use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct CorpusFile {
    test_cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    expected_command: String,
    risk_level: String,
    #[serde(default)]
    category: String,
}

#[derive(Debug, Deserialize)]
struct DetonationResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    exit_code: i64,
    #[serde(default)]
    timed_out: bool,
    #[serde(default)]
    blast: Blast,
}

#[derive(Debug, Default, Deserialize)]
struct Blast {
    #[serde(default)]
    canaries_destroyed: Vec<String>,
    #[serde(default)]
    files_removed: u64,
    #[serde(default)]
    files_modified: u64,
    #[serde(default)]
    files_created: u64,
    #[serde(default)]
    system_intact: bool,
}

impl Blast {
    /// True when the run left any observable trace at all.
    fn observable_impact(&self) -> bool {
        !self.canaries_destroyed.is_empty()
            || self.files_removed > 0
            || self.files_modified > 0
            || self.files_created > 0
            || !self.system_intact
    }
}

const CORPUS: &str = "tests/evaluation/datasets/safety/dangerous_patterns.json";

#[tokio::test]
#[ignore = "requires CARO_DETONATION_URL + CARO_DETONATION_TOKEN (nightly detonation lane)"]
async fn detonate_dangerous_corpus() {
    let url = std::env::var("CARO_DETONATION_URL").expect("CARO_DETONATION_URL not set");
    let token = std::env::var("CARO_DETONATION_TOKEN").expect("CARO_DETONATION_TOKEN not set");

    let corpus: CorpusFile =
        serde_json::from_str(&std::fs::read_to_string(CORPUS).expect("corpus file readable"))
            .expect("corpus parses");
    assert!(!corpus.test_cases.is_empty(), "corpus must not be empty");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .expect("client builds");

    let mut evidence: Vec<serde_json::Value> = Vec::new();
    // A critical-labeled command that completes cleanly with zero observable
    // effects contradicts its label: either the pattern is miscalibrated or
    // the sandbox hides the effect class (e.g. raw device writes). Both are
    // findings a human should review, so they fail the suite.
    let mut contradictions: Vec<String> = Vec::new();
    let mut infra_failures: Vec<String> = Vec::new();

    for case in &corpus.test_cases {
        let response = client
            .post(format!("{}/detonate", url.trim_end_matches('/')))
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "id": case.id,
                "command": case.expected_command,
                "risk_level": case.risk_level,
                "timeout_ms": 15000,
            }))
            .send()
            .await;

        let parsed: Option<DetonationResponse> = match response {
            Ok(r) => r.json().await.ok(),
            Err(e) => {
                infra_failures.push(format!("{}: transport error: {e}", case.id));
                None
            }
        };

        let Some(det) = parsed else {
            infra_failures.push(format!("{}: unparseable response", case.id));
            continue;
        };

        if !det.ok {
            infra_failures.push(format!(
                "{}: harness error: {}",
                case.id,
                det.error.as_deref().unwrap_or("unknown")
            ));
            continue;
        }

        let benign_run = det.exit_code == 0 && !det.timed_out && !det.blast.observable_impact();
        if case.risk_level == "critical" && benign_run {
            contradictions.push(format!(
                "{} ({} / {}): labeled critical but completed exit 0 with zero observable effects",
                case.id, case.category, case.expected_command
            ));
        }

        evidence.push(serde_json::json!({
            "id": case.id,
            "command": case.expected_command,
            "risk_level": case.risk_level,
            "category": case.category,
            "exit_code": det.exit_code,
            "timed_out": det.timed_out,
            "canaries_destroyed": det.blast.canaries_destroyed,
            "files_removed": det.blast.files_removed,
            "files_modified": det.blast.files_modified,
            "files_created": det.blast.files_created,
            "system_intact": det.blast.system_intact,
            "label_contradiction": case.risk_level == "critical" && benign_run,
        }));
    }

    // Evidence artifact: uploaded by the nightly workflow, reviewable by humans.
    let out_dir = std::path::Path::new("tests/red_team/evidence");
    std::fs::create_dir_all(out_dir).expect("evidence dir");
    let out_path = out_dir.join(format!(
        "{}-detonation.json",
        chrono::Utc::now().format("%Y-%m-%d")
    ));
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "corpus": CORPUS,
            "total_cases": corpus.test_cases.len(),
            "detonated": evidence.len(),
            "infra_failures": infra_failures,
            "results": evidence,
        }))
        .expect("evidence serializes"),
    )
    .expect("evidence written");
    eprintln!("detonation evidence: {}", out_path.display());

    // More than 20% infra failures means the lane itself is broken — fail
    // loudly rather than reporting a hollow green.
    let infra_ratio = infra_failures.len() as f64 / corpus.test_cases.len() as f64;
    assert!(
        infra_ratio <= 0.2,
        "detonation lane unhealthy ({} of {} requests failed):\n{}",
        infra_failures.len(),
        corpus.test_cases.len(),
        infra_failures.join("\n")
    );

    assert!(
        contradictions.is_empty(),
        "risk-label contradictions found (pattern miscalibrated or sandbox blind spot):\n{}",
        contradictions.join("\n")
    );
}
