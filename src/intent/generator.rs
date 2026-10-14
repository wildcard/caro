//! `SelectFilesGenerator`: the deterministic `--backend select-files` route.
//!
//! Selection (`new`) runs the GNU identity check once. Generation is pure:
//! one clock read, no process, no file system, no network, no model.

use std::time::Instant;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};

use super::check::{verify, Pattern, VerifiedIntent};
use super::clock::{Clock, SystemClock};
use super::compile;
use super::facts::{parse, Outcome};
use super::find_identity::{self, FindReceipt};
use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{
    BackendType, CommandRequest, ConfidenceSource, GeneratedCommand, RiskLevel, ShellType,
};

/// Provenance string: this route uses no model.
pub const BACKEND_USED: &str = "intent-deterministic";

pub struct SelectFilesGenerator {
    receipt: FindReceipt,
    clock: Box<dyn Clock>,
}

/// Output of the pure planning step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub command: String,
    pub argv: Vec<String>,
    pub explanation: String,
}

impl SelectFilesGenerator {
    /// Select the backend: verifies GNU `find` once. Refuses on failure.
    pub fn new() -> Result<Self, GeneratorError> {
        let receipt = find_identity::verify().map_err(|e| GeneratorError::BackendUnavailable {
            reason: format!("select-files needs GNU findutils: {e}"),
        })?;
        Ok(Self::with_parts(receipt, Box::new(SystemClock)))
    }

    /// Build from an existing receipt and clock (tests).
    pub fn with_parts(receipt: FindReceipt, clock: Box<dyn Clock>) -> Self {
        Self { receipt, clock }
    }

    pub fn receipt(&self) -> &FindReceipt {
        &self.receipt
    }

    /// Pure: request bytes + `t0` -> command and explanation, or an error.
    pub fn plan(&self, input: &str, t0: i64) -> Result<Plan, GeneratorError> {
        let facts = match parse(input) {
            Outcome::Accept(f) => f,
            Outcome::Clarify(q) => {
                return Err(GeneratorError::NeedsClarification {
                    question: Some(q),
                    p: 1.0,
                })
            }
            Outcome::Decline(d) => return Err(invalid(d.to_string())),
        };
        let intent = verify(&facts, input, t0).map_err(|d| invalid(d.to_string()))?;
        let argv = compile::argv(&intent, &self.receipt.path);
        compile::check_closed(&argv, &self.receipt.path).map_err(|e| GeneratorError::Internal {
            message: format!("compiler produced an unexpected command: {e}"),
        })?;
        let command = compile::render(&argv);
        let explanation = explain(&intent, &self.receipt, t0);
        Ok(Plan {
            command,
            argv,
            explanation,
        })
    }
}

fn invalid(message: String) -> GeneratorError {
    GeneratorError::InvalidRequest { message }
}

fn utc(secs: i64) -> String {
    Utc.timestamp_opt(secs, 0)
        .single()
        .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| format!("@{secs}"))
}

fn explain(i: &VerifiedIntent, r: &FindReceipt, t0: i64) -> String {
    let mut l = vec![
        format!(
            "{} (find -print0), machine-readable",
            super::OUTPUT_FRAMING_LABEL
        ),
        "family: regular files, searched recursively (symlinks are not followed or listed)"
            .to_string(),
    ];
    if i.root_source() == i.root_emit() {
        l.push(format!("root: {}", i.root_emit()));
    } else {
        l.push(format!(
            "root: {} (written as {} so find reads it as a path)",
            i.root_source(),
            i.root_emit()
        ));
    }
    match i.pattern() {
        None => l.push("filter: none".into()),
        Some(Pattern::Name(n)) => l.push(format!("filter: exact name {n} (case-sensitive)")),
        Some(Pattern::ExtInsensitive(e)) => {
            l.push(format!("filter: extension .{e} (case-insensitive)"))
        }
        Some(Pattern::ExtSensitive(e)) => {
            l.push(format!("filter: extension .{e} (case-sensitive)"))
        }
    }
    match i.window() {
        None => l.push("window: none".into()),
        Some(w) => {
            l.push(format!(
                "window: modified after {} and not after {} ({} {}{} elapsed)",
                utc(w.cutoff),
                utc(w.t0),
                w.count,
                w.unit.name(),
                if w.count == 1 { "" } else { "s" }
            ));
            l.push(
                "window frozen at generation: files changed after that instant are not listed, \
                 and running the command later does not move the window"
                    .into(),
            );
        }
    }
    l.push(format!("generated at: {}", utc(t0)));
    l.push(format!(
        "find identity: {} at {} (checked when the backend was selected)",
        r.banner, r.path
    ));
    l.push("no model used; approval=smart has no model judge for this backend".into());
    l.join("\n")
}

#[async_trait]
impl CommandGenerator for SelectFilesGenerator {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        if matches!(request.shell, ShellType::PowerShell | ShellType::Cmd) {
            return Err(invalid(
                "UnsupportedShell: select-files emits POSIX shell syntax".into(),
            ));
        }
        let start = Instant::now();
        // The single clock read of this generation call.
        let t0 = self
            .clock
            .now_unix_secs()
            .map_err(|e| GeneratorError::Internal {
                message: format!("clock: {e}"),
            })?;
        let plan = self.plan(&request.input, t0)?;
        Ok(GeneratedCommand {
            command: plan.command,
            explanation: plan.explanation,
            safety_level: RiskLevel::Safe,
            estimated_impact: "Read-only listing; changes nothing.".to_string(),
            alternatives: vec![],
            backend_used: BACKEND_USED.to_string(),
            generation_time_ms: start.elapsed().as_millis() as u64,
            confidence_score: 0.0,
            confidence_source: ConfidenceSource::Unknown,
        })
    }

    // classify_risk keeps the trait default (None) and supports_risk_judge
    // stays false: this route never calls a model.

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::SelectFiles,
            model_name: "none (deterministic)".to_string(),
            supports_streaming: false,
            max_tokens: 0,
            typical_latency_ms: 1,
            memory_usage_mb: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::clock::FixedClock;
    use std::sync::Arc;

    const T0: i64 = 1_760_000_000;

    fn receipt() -> FindReceipt {
        FindReceipt {
            path: "/usr/bin/find".into(),
            banner: "find (GNU findutils) 4.9.0".into(),
            version: "4.9.0".into(),
        }
    }

    fn generator() -> (SelectFilesGenerator, Arc<FixedClock>) {
        let c = Arc::new(FixedClock::new(T0));
        (
            SelectFilesGenerator::with_parts(receipt(), Box::new(c.clone())),
            c,
        )
    }

    fn req(s: &str) -> CommandRequest {
        CommandRequest::new(s, ShellType::Bash)
    }

    #[tokio::test]
    async fn one_call_reads_the_clock_once_and_uses_no_model() {
        let (g, c) = generator();
        let out = g
            .generate_command(&req("find python files modified in the last 7 days"))
            .await
            .unwrap();
        assert_eq!(c.reads(), 1);
        assert_eq!(out.backend_used, "intent-deterministic");
        assert_eq!(out.confidence_source, ConfidenceSource::Unknown);
        assert!(out.command.ends_with("-print0"));
        assert!(out.explanation.starts_with("output-framing: nul-separated"));
        assert!(out.explanation.contains("window frozen at generation"));
        assert!(out.explanation.contains("find (GNU findutils) 4.9.0"));
        assert!(!g.supports_risk_judge());
        assert_eq!(g.backend_info().backend_type, BackendType::SelectFiles);
    }

    #[tokio::test]
    async fn clarify_and_decline_are_typed_and_read_clock_once() {
        let (g, c) = generator();
        let e = g
            .generate_command(&req("files modified today"))
            .await
            .unwrap_err();
        assert!(matches!(e, GeneratorError::NeedsClarification { .. }));
        let e = g.generate_command(&req("please")).await.unwrap_err();
        assert!(matches!(e, GeneratorError::InvalidRequest { .. }));
        assert_eq!(c.reads(), 2);
    }

    #[tokio::test]
    async fn non_posix_shells_are_refused() {
        let (g, _) = generator();
        let r = CommandRequest::new("list all python files", ShellType::PowerShell);
        assert!(g.generate_command(&r).await.is_err());
    }

    #[tokio::test]
    async fn classify_risk_is_unavailable_without_network_or_model() {
        let (g, _) = generator();
        let ctx = crate::models::RiskJudgeContext {
            shell: ShellType::Bash,
            cwd: None,
            static_risk: RiskLevel::Safe,
            matched_patterns: vec![],
        };
        assert!(g.classify_risk("find .", &ctx).await.is_none());
    }

    #[test]
    fn explanation_shows_both_roots_and_utc_window() {
        let (g, _) = generator();
        let p = g
            .plan("find files in \"-x\" modified in the last 2 hours", T0)
            .unwrap();
        assert!(p.explanation.contains("root: -x (written as ./-x"));
        assert!(
            p.explanation.contains("2025-10-09 08:53:20 UTC"),
            "{}",
            p.explanation
        );
        assert!(p.explanation.contains("2 hours elapsed"));
    }
}
