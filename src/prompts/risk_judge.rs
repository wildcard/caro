//! Prompt + parser for the `--approval smart` LLM "risk judge".
//!
//! Modeled on goose's `permission_judge`: the LLM is asked to make a single,
//! bounded classification and to **fail safe** ("if unsure, rate it higher").
//! The judge is advisory — its verdict is blended under a hard floor by
//! [`crate::safety::blend_smart_decision`], which never lets it relax a
//! `Critical` static match.

use crate::decision::{extract_json_object, parse_choice_json, Choice};
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
/// extracting the first balanced `{...}` object. Returns `None` on any parse
/// failure so the caller fails safe to the static decision.
///
/// The verdict is a typed [`Choice`] over [`RiskLevel`] (see
/// [`crate::decision`]): an unknown label, a missing/malformed confidence, a
/// partial or unnormalised probabilities map, or a response mixing both
/// answer modes is a type error and yields `None` — never a guess.
pub fn parse_risk_judgment(raw: &str) -> Option<RiskJudgment> {
    const LEVELS: [RiskLevel; 4] = [
        RiskLevel::Critical,
        RiskLevel::High,
        RiskLevel::Moderate,
        RiskLevel::Safe,
    ];
    let choice: Choice<RiskLevel> = parse_choice_json(raw, "risk", &LEVELS)?;

    let json = extract_json_object(raw)?;
    let value: serde_json::Value = serde_json::from_str(&json).ok()?;
    let (risk, confidence) = match value.get("risk").and_then(|v| v.as_str()) {
        // Discrete mode: the judge's own label and confidence are the verdict
        // (parse_choice_json already validated both).
        Some(label) => (
            label.parse().ok()?,
            value.get("confidence")?.as_f64()?.clamp(0.0, 1.0),
        ),
        // Probabilities mode: argmax of a complete, normalised distribution.
        None => {
            let (risk, p) = choice.argmax()?;
            (*risk, p)
        }
    };
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    Some(RiskJudgment {
        risk,
        reason,
        confidence,
    })
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
    fn missing_confidence_is_rejected() {
        // A verdict without a confidence cannot clear the floor either way;
        // rejecting it keeps the static decision (fail-safe) and never lets
        // a zero-mass label masquerade as a distribution.
        assert!(parse_risk_judgment(r#"{"risk":"moderate","reason":"x"}"#).is_none());
    }

    #[test]
    fn partial_probabilities_cannot_relax_safety() {
        // Omitting labels must not renormalise into a confident "safe".
        assert!(parse_risk_judgment(r#"{"probabilities": {"safe": 0.6}}"#).is_none());
        assert!(parse_risk_judgment(
            r#"{"risk":"safe","confidence":0.9,"probabilities":{"safe":1,"moderate":0,"high":0,"critical":0}}"#
        )
        .is_none());
    }

    #[test]
    fn parses_probabilities_mode_via_argmax() {
        let j = parse_risk_judgment(
            r#"{"probabilities": {"safe": 0.1, "moderate": 0.2, "high": 0.7, "critical": 0.0}, "reason": "rm"}"#,
        )
        .unwrap();
        assert_eq!(j.risk, RiskLevel::High);
        assert!((j.confidence - 0.7).abs() < 1e-9);
        assert_eq!(j.reason, "rm");
    }

    #[test]
    fn garbage_returns_none() {
        assert!(parse_risk_judgment("not json at all").is_none());
        assert!(parse_risk_judgment(r#"{"risk":"bogus"}"#).is_none());
    }
}
