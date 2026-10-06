// Regression guard for #1217: `--output json` reported `"executed": true`
// under `--dry-run`, so consumers could not tell "generated" from "ran".
// `executed` still means "passed safety checks" (pinned by other contract
// tests); `dry_run` says the command was never run.

use caro::cli::{CliApp, IntoCliArgs};

struct Args {
    dry_run: bool,
    execute: bool,
}

impl IntoCliArgs for Args {
    fn prompt(&self) -> Option<String> {
        Some("list files in current directory".to_string())
    }
    fn shell(&self) -> Option<String> {
        None
    }
    fn backend(&self) -> Option<String> {
        None
    }
    fn model_name(&self) -> Option<String> {
        None
    }
    fn safety(&self) -> Option<String> {
        None
    }
    fn output(&self) -> Option<String> {
        Some("json".to_string())
    }
    fn confirm(&self) -> bool {
        false
    }
    fn verbose(&self) -> bool {
        false
    }
    fn config_file(&self) -> Option<String> {
        None
    }
    fn execute(&self) -> bool {
        self.execute
    }
    fn dry_run(&self) -> bool {
        self.dry_run
    }
    fn interactive(&self) -> bool {
        false
    }
    fn force_llm(&self) -> bool {
        false
    }
    fn explain(&self) -> bool {
        false
    }
}

async fn run(dry_run: bool, execute: bool) -> serde_json::Value {
    let cli = CliApp::new().await.expect("CliApp");
    let result = cli
        .run_with_args(Args { dry_run, execute })
        .await
        .expect("generation succeeds");
    serde_json::to_value(&result).expect("CliResult serializes")
}

#[tokio::test]
async fn dry_run_json_says_dry_run_and_never_runs() {
    // --execute together with --dry-run: dry run must still win.
    let json = run(true, true).await;
    assert_eq!(json["dry_run"], true, "JSON must flag the dry run: {json}");
    assert!(
        json["exit_code"].is_null() && json["execution_error"].is_null(),
        "a dry run must not run the command: {json}"
    );
    // The command was eligible to run, so only the dry run kept it from running.
    assert_eq!(
        json["executed"], true,
        "safe command must pass checks: {json}"
    );
}

#[tokio::test]
async fn normal_json_reports_dry_run_false() {
    let json = run(false, false).await;
    assert_eq!(json["dry_run"], false, "{json}");
}
