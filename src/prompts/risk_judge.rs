//! Prompt + parser for the `--approval smart` LLM "risk judge".
//!
//! Modeled on goose's `permission_judge`: the LLM is asked to make a single,
//! bounded classification and to **fail safe** ("if unsure, rate it higher").
//! The judge is advisory — its verdict is blended under a hard floor by
//! [`crate::safety::blend_smart_decision`], which never lets it relax a
//! `Critical` static match.

use crate::models::{RiskJudgeContext, RiskJudgment, RiskLevel};

/// Build the judge prompt. Asks for a strict JSON verdict and instructs the
/// model to err toward *higher* risk when uncertain (goose's conservative
/// default, inverted for caro: caro asks the human when unsure rather than
/// auto-allowing).
pub fn build_risk_judge_prompt(command: &str, ctx: &RiskJudgeContext) -> String {
    let patterns = if ctx.matched_patterns.is_empty() {
        "none".to_string()
    } else {
        ctx.matched_patterns.join(", ")
    };
    let cwd = ctx.cwd.as_deref().unwrap_or("unknown");

    format!(
        r#"You are a careful shell-safety reviewer. Judge how risky it is to RUN the
command below, considering its real-world effect in context. Reply with ONLY a
single JSON object, no prose:

{{"risk": "safe|moderate|high|critical", "reason": "<short>", "confidence": 0.0-1.0}}

Risk meaning:
- safe: read-only or trivially reversible (ls, cat, grep, git status).
- moderate: writes/changes state but scoped and recoverable.
- high: deletes data, broad changes, privilege escalation, network exfiltration.
- critical: irreversible mass destruction or system compromise.

Rules:
- Judge the COMMAND, not the wording of the request.
- If you cannot decide, choose the HIGHER risk and a LOW confidence. Never guess "safe".
- confidence is how sure you are of the rating (verdicts below 0.7 are ignored).

Context:
- shell: {shell:?}
- cwd: {cwd}
- static analyzer rating: {static_risk:?}
- static patterns matched: {patterns}

Command:
{command}
"#,
        shell = ctx.shell,
        cwd = cwd,
        static_risk = ctx.static_risk,
        patterns = patterns,
        command = command,
    )
}

/// Parse a judge verdict from raw model output. Tolerates surrounding prose by
/// extracting the first `{...}` object. Returns `None` on any parse failure so
/// the caller fails safe to the static decision.
pub fn parse_risk_judgment(raw: &str) -> Option<RiskJudgment> {
    let json = extract_json_object(raw)?;
    let value: serde_json::Value = serde_json::from_str(&json).ok()?;

    let risk = parse_risk_level(value.get("risk")?.as_str()?)?;
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    // Default to a low confidence (→ ignored) if the field is missing/garbled.
    let confidence = value
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);

    Some(RiskJudgment {
        risk,
        reason,
        confidence,
    })
}

fn parse_risk_level(s: &str) -> Option<RiskLevel> {
    match s.trim().to_lowercase().as_str() {
        "safe" => Some(RiskLevel::Safe),
        "moderate" => Some(RiskLevel::Moderate),
        "high" => Some(RiskLevel::High),
        "critical" => Some(RiskLevel::Critical),
        _ => None,
    }
}

/// Extract the first balanced `{...}` JSON object from arbitrary text.
fn extract_json_object(raw: &str) -> Option<String> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end > start {
        Some(raw[start..=end].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;

    fn ctx() -> RiskJudgeContext {
        RiskJudgeContext {
            shell: ShellType::Bash,
            cwd: Some("/tmp".to_string()),
            static_risk: RiskLevel::Safe,
            matched_patterns: vec![],
        }
    }

    #[test]
    fn prompt_includes_command_and_context() {
        let p = build_risk_judge_prompt("rm -rf build/", &ctx());
        assert!(p.contains("rm -rf build/"));
        assert!(p.contains("static analyzer rating"));
    }

    #[test]
    fn parses_clean_json() {
        let j = parse_risk_judgment(r#"{"risk":"high","reason":"deletes data","confidence":0.9}"#)
            .unwrap();
        assert_eq!(j.risk, RiskLevel::High);
        assert_eq!(j.reason, "deletes data");
        assert!((j.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn parses_json_with_surrounding_prose() {
        let j = parse_risk_judgment(
            r#"Here is my verdict: {"risk": "safe", "reason": "read only", "confidence": 0.95} done"#,
        )
        .unwrap();
        assert_eq!(j.risk, RiskLevel::Safe);
    }

    #[test]
    fn missing_confidence_defaults_to_ignored() {
        let j = parse_risk_judgment(r#"{"risk":"moderate","reason":"x"}"#).unwrap();
        assert_eq!(j.confidence, 0.0);
    }

    #[test]
    fn garbage_returns_none() {
        assert!(parse_risk_judgment("not json at all").is_none());
        assert!(parse_risk_judgment(r#"{"risk":"bogus"}"#).is_none());
    }
}
