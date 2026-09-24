//! Typed decision primitives — "decisions, not text".
//!
//! Borrowed from TypeSafe AI's System One framing (Jev): most steps in an
//! automation pipeline are not free-text generation but bounded decisions
//! that deserve a *probability*, not a paragraph. Jev exposes three
//! primitives; caro adopts the same vocabulary for its gates:
//!
//! | Primitive | Question shape | caro gate |
//! |---|---|---|
//! | [`Noul`] | yes / no with `p(yes)` | "needs clarification?", "platform fix needed?" |
//! | [`Choice`] | one of N labels with a distribution | risk level, intent category |
//! | [`Score`] | a value on a declared scale with confidence | candidate ranking |
//!
//! Two invariants, unchanged from `crate::safety::blend_smart_decision`:
//!
//! 1. A decision is **advisory above the deterministic safety floor**. It can
//!    never relax a static `Critical` match.
//! 2. A decision that fails to parse or whose confidence sits below a gate's
//!    floor is `None` — the caller falls back to its default path.
//!
//! Parsing is deliberately strict about *types* (an unknown label is a
//! parse failure, not a guess) and lenient about *packaging* (surrounding
//! prose is tolerated). That is the local, no-training equivalent of Jev's
//! "never makes type errors" guarantee. See
//! `docs/adr/ADR-017-typed-decisions-and-calibrated-confidence.md`.

use std::str::FromStr;

/// A yes/no decision with a calibrated probability of "yes".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Noul {
    /// Probability of "yes", clamped to `0.0..=1.0`.
    pub p_yes: f64,
}

impl Noul {
    /// Build a Noul, clamping into range (NaN → 0.0, the fail-safe "no").
    pub fn new(p_yes: f64) -> Self {
        let p = if p_yes.is_nan() { 0.0 } else { p_yes };
        Self {
            p_yes: p.clamp(0.0, 1.0),
        }
    }

    /// Whether "yes" is the more likely answer.
    pub fn is_yes(&self) -> bool {
        self.p_yes >= 0.5
    }

    /// Confidence in the majority answer: `max(p, 1 − p)`.
    pub fn confidence(&self) -> f64 {
        self.p_yes.max(1.0 - self.p_yes)
    }
}

/// A choice among a bounded set of labels, as a normalised distribution.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice<T> {
    /// `(label, probability)` pairs; probabilities sum to 1.0 (or all 0.0
    /// when the input had no mass, in which case [`Choice::argmax`] is `None`).
    pub dist: Vec<(T, f64)>,
}

impl<T: Clone + PartialEq> Choice<T> {
    /// Build a distribution from raw non-negative weights, normalising to 1.
    /// Negative or NaN weights are treated as 0.
    pub fn from_weights(pairs: Vec<(T, f64)>) -> Self {
        let cleaned: Vec<(T, f64)> = pairs
            .into_iter()
            .map(|(t, w)| (t, if w.is_nan() || w < 0.0 { 0.0 } else { w }))
            .collect();
        let total: f64 = cleaned.iter().map(|(_, w)| w).sum();
        let dist = if total > 0.0 {
            cleaned.into_iter().map(|(t, w)| (t, w / total)).collect()
        } else {
            cleaned
        };
        Self { dist }
    }

    /// A degenerate distribution: `label` with `confidence`, the remaining
    /// mass spread evenly over the other `allowed` labels. This is the
    /// "discrete" answer mode of the System One adapter.
    ///
    /// Note: [`Choice::argmax`] equals `label` only while `confidence` is at
    /// least the share left for each other label; a low-confidence discrete
    /// answer is a statement that *another* label is more likely. Callers
    /// that want the reported label regardless should read it from their
    /// own JSON (as the risk judge does) or gate on a confidence floor.
    pub fn discrete(label: T, confidence: f64, allowed: &[T]) -> Self {
        let c = if confidence.is_nan() {
            0.0
        } else {
            confidence.clamp(0.0, 1.0)
        };
        let others: Vec<&T> = allowed.iter().filter(|a| **a != label).collect();
        let mut pairs = vec![(label, c)];
        if !others.is_empty() {
            let share = (1.0 - c) / others.len() as f64;
            pairs.extend(others.into_iter().map(|o| (o.clone(), share)));
        }
        Self::from_weights(pairs)
    }

    /// The most probable label and its probability. Ties resolve to the
    /// first label in `dist` order; [`parse_choice_json`] orders `dist` by
    /// the caller's `allowed` slice in both answer modes, so list labels in
    /// fail-safe order (e.g. highest risk first) when that matters.
    pub fn argmax(&self) -> Option<(&T, f64)> {
        let mut best: Option<(&T, f64)> = None;
        for (t, p) in &self.dist {
            match best {
                Some((_, bp)) if *p <= bp => {}
                _ if *p > 0.0 => best = Some((t, *p)),
                _ => {}
            }
        }
        best
    }

    /// Probability of the argmax label; `0.0` for an empty distribution.
    pub fn confidence(&self) -> f64 {
        self.argmax().map(|(_, p)| p).unwrap_or(0.0)
    }
}

/// A rating on a declared scale, with a separate confidence in that rating.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score {
    /// The rated value (on whatever scale the caller declared).
    pub value: f64,
    /// Confidence in `value`, clamped to `0.0..=1.0`.
    pub confidence: f64,
}

impl Score {
    /// Build a Score, clamping confidence (NaN → 0.0).
    pub fn new(value: f64, confidence: f64) -> Self {
        let c = if confidence.is_nan() {
            0.0
        } else {
            confidence.clamp(0.0, 1.0)
        };
        Self {
            value,
            confidence: c,
        }
    }
}

/// A clarification decision: should we ask the user a question instead of
/// answering? (`Noul`), and if so which one.
///
/// Replaces two ad-hoc conventions (#1462): the embedded prompt's
/// `QUESTION: <text>` prefix and the remote prompts'
/// `echo 'Please clarify your request'` command. Both are mapped onto this
/// type by [`clarification_from_raw`] so every backend surfaces the same
/// [`crate::backends::GeneratorError::NeedsClarification`].
#[derive(Debug, Clone, PartialEq)]
pub struct Clarification {
    /// Probability that a clarifying question is needed.
    pub needed: Noul,
    /// The question to ask, if the model supplied one.
    pub question: Option<String>,
}

/// Gate floor: below this `p(yes)` the pipeline proceeds to generation
/// (fail toward answering — the safety layer still gates execution).
pub const CLARIFICATION_MIN_CONFIDENCE: f64 = 0.7;

/// Confidence assigned to the legacy `QUESTION:` prefix, which is an
/// unambiguous but unquantified signal from the model.
const LEGACY_QUESTION_PREFIX_P: f64 = 0.9;

impl Clarification {
    /// Whether the gate should fire: needed, and above the floor.
    pub fn should_ask(&self) -> bool {
        self.needed.is_yes() && self.needed.p_yes >= CLARIFICATION_MIN_CONFIDENCE
    }
}

/// Detect a clarification request in raw model output.
///
/// Accepts, in order:
/// 1. the legacy `QUESTION: <text>` prefix (p = 0.9);
/// 2. JSON with `"needs_clarification": true` and optional `"p"` (default
///    0.9) and `"question"`;
/// 3. the legacy `echo 'Please clarify your request'` command (p = 0.9, no
///    question) — a backend that still emits it gets a typed decision
///    instead of a runnable command.
///
/// Returns `None` when the output is an ordinary answer.
pub fn clarification_from_raw(raw: &str) -> Option<Clarification> {
    let trimmed = raw.trim();
    if let Some(q) = trimmed.strip_prefix("QUESTION:") {
        return Some(Clarification {
            needed: Noul::new(LEGACY_QUESTION_PREFIX_P),
            question: Some(q.trim().to_string()).filter(|q| !q.is_empty()),
        });
    }
    if let Some(json) = extract_json_object(trimmed) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
            if value
                .get("needs_clarification")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let p = value
                    .get("p")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(LEGACY_QUESTION_PREFIX_P);
                let question = value
                    .get("question")
                    .and_then(|v| v.as_str())
                    .map(|q| q.trim().to_string())
                    .filter(|q| !q.is_empty());
                return Some(Clarification {
                    needed: Noul::new(p),
                    question,
                });
            }
            if let Some(cmd) = value.get("cmd").and_then(|v| v.as_str()) {
                if is_legacy_clarify_command(cmd) {
                    return Some(Clarification {
                        needed: Noul::new(LEGACY_QUESTION_PREFIX_P),
                        question: None,
                    });
                }
            }
        }
    }
    None
}

/// The pre-#1462 remote-backend convention, kept only so old model outputs
/// map onto a typed decision rather than a runnable `echo`.
fn is_legacy_clarify_command(cmd: &str) -> bool {
    let c = cmd.trim().to_lowercase();
    c.starts_with("echo") && c.contains("please clarify")
}

/// Extract the first *balanced* `{...}` JSON object from arbitrary model
/// output. Lenient about packaging (prose before/after is ignored) but
/// brace-aware: a response containing several objects yields the first
/// complete one instead of a span from the first `{` to the last `}`.
/// String literals are skipped so braces inside quoted text do not count.
pub fn extract_json_object(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let start = raw.find('{')?;
    let mut depth = 0usize;
    let mut in_str = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_str {
            match b {
                b'\\' if !escaped => escaped = true,
                b'"' if !escaped => in_str = false,
                _ => escaped = false,
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(raw[start..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Tolerance for a probabilities map summing to 1.0.
const PROBABILITY_SUM_TOLERANCE: f64 = 0.05;

/// Parse a [`Choice`] from model output.
///
/// Accepts exactly one of the two answer modes of the System One adapter:
///
/// - discrete: `{"<key>": "<label>", "confidence": c}` — `key` is the
///   caller's field name (e.g. `"risk"`); `c` must be a finite number in
///   `0.0..=1.0` (missing or malformed → `None`, never a guess);
/// - probabilities: `{"probabilities": {"<label>": p, ...}}` — must name
///   **every** allowed label, every `p` finite and non-negative, and the
///   total within [`PROBABILITY_SUM_TOLERANCE`] of 1.0. Labels are ordered
///   by `allowed` so [`Choice::argmax`] ties resolve in the caller's
///   fail-safe order in both modes.
///
/// Returns `None` on any type error: unparsable JSON, both modes present,
/// a label outside `allowed`, a non-string label, a missing/extra label in
/// probabilities mode, or a probability that is not a finite number. This
/// output relaxes safety decisions, so malformed input never becomes a
/// confident verdict.
pub fn parse_choice_json<T>(raw: &str, key: &str, allowed: &[T]) -> Option<Choice<T>>
where
    T: FromStr + Clone + PartialEq,
{
    let json = extract_json_object(raw)?;
    let value: serde_json::Value = serde_json::from_str(&json).ok()?;

    let has_label = value.get(key).is_some();
    let probs = value.get("probabilities");
    if has_label && probs.is_some() {
        return None; // mixed answer modes are contradictory, not a verdict
    }

    if let Some(probs) = probs {
        let map = probs.as_object()?;
        if map.len() != allowed.len() {
            return None;
        }
        let mut pairs: Vec<(T, f64)> = Vec::with_capacity(allowed.len());
        let mut total = 0.0_f64;
        for want in allowed {
            let (_, p) = map
                .iter()
                .find(|(label, _)| parse_label::<T>(label, allowed).as_ref() == Some(want))?;
            let p = p.as_f64()?;
            if !p.is_finite() || !(0.0..=1.0).contains(&p) {
                return None;
            }
            total += p;
            pairs.push((want.clone(), p));
        }
        if (total - 1.0).abs() > PROBABILITY_SUM_TOLERANCE {
            return None;
        }
        let choice = Choice::from_weights(pairs);
        return choice.argmax().is_some().then_some(choice);
    }

    let label = value.get(key)?.as_str()?;
    let t = parse_label(label, allowed)?;
    let confidence = value.get("confidence")?.as_f64()?;
    if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
        return None;
    }
    Some(Choice::discrete(t, confidence, allowed))
}

/// Parse a label case-insensitively and require it to be in `allowed`.
fn parse_label<T>(s: &str, allowed: &[T]) -> Option<T>
where
    T: FromStr + Clone + PartialEq,
{
    let t = T::from_str(s.trim().to_lowercase().as_str()).ok()?;
    allowed.contains(&t).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Lvl {
        Low,
        High,
    }
    impl FromStr for Lvl {
        type Err = ();
        fn from_str(s: &str) -> Result<Self, ()> {
            match s {
                "low" => Ok(Lvl::Low),
                "high" => Ok(Lvl::High),
                _ => Err(()),
            }
        }
    }
    const ALL: [Lvl; 2] = [Lvl::High, Lvl::Low];

    #[test]
    fn noul_clamps_and_reports() {
        assert_eq!(Noul::new(1.7).p_yes, 1.0);
        assert_eq!(Noul::new(-0.2).p_yes, 0.0);
        assert_eq!(Noul::new(f64::NAN).p_yes, 0.0);
        let n = Noul::new(0.3);
        assert!(!n.is_yes());
        assert!((n.confidence() - 0.7).abs() < 1e-12);
    }

    #[test]
    fn choice_normalises_weights() {
        let c = Choice::from_weights(vec![(Lvl::High, 3.0), (Lvl::Low, 1.0)]);
        assert!((c.dist[0].1 - 0.75).abs() < 1e-12);
        assert_eq!(c.argmax(), Some((&Lvl::High, 0.75)));
        assert!((c.confidence() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn choice_zero_mass_has_no_argmax() {
        let c = Choice::from_weights(vec![(Lvl::High, 0.0), (Lvl::Low, -1.0)]);
        assert_eq!(c.argmax(), None);
        assert_eq!(c.confidence(), 0.0);
    }

    #[test]
    fn choice_ties_resolve_to_first_label() {
        let c = Choice::from_weights(vec![(Lvl::High, 1.0), (Lvl::Low, 1.0)]);
        assert_eq!(c.argmax().unwrap().0, &Lvl::High);
    }

    #[test]
    fn discrete_spreads_remaining_mass() {
        let c = Choice::discrete(Lvl::Low, 0.8, &ALL);
        assert_eq!(c.argmax(), Some((&Lvl::Low, 0.8)));
        let high = c.dist.iter().find(|(t, _)| *t == Lvl::High).unwrap().1;
        assert!((high - 0.2).abs() < 1e-12);
    }

    #[test]
    fn score_clamps_confidence_only() {
        let s = Score::new(42.0, 9.0);
        assert_eq!(s.value, 42.0);
        assert_eq!(s.confidence, 1.0);
    }

    #[test]
    fn parses_discrete_mode_with_prose() {
        let c = parse_choice_json::<Lvl>(
            r#"verdict: {"risk": "HIGH", "confidence": 0.9} ok"#,
            "risk",
            &ALL,
        )
        .unwrap();
        assert_eq!(c.argmax(), Some((&Lvl::High, 0.9)));
    }

    #[test]
    fn discrete_mode_missing_or_invalid_confidence_is_none() {
        assert!(parse_choice_json::<Lvl>(r#"{"risk": "low"}"#, "risk", &ALL).is_none());
        assert!(
            parse_choice_json::<Lvl>(r#"{"risk": "low", "confidence": "x"}"#, "risk", &ALL)
                .is_none()
        );
        assert!(
            parse_choice_json::<Lvl>(r#"{"risk": "low", "confidence": 1.7}"#, "risk", &ALL)
                .is_none()
        );
        assert!(
            parse_choice_json::<Lvl>(r#"{"risk": "low", "confidence": -0.1}"#, "risk", &ALL)
                .is_none()
        );
    }

    #[test]
    fn mixed_answer_modes_are_rejected() {
        assert!(parse_choice_json::<Lvl>(
            r#"{"risk": "low", "confidence": 0.9, "probabilities": {"low": 0.1, "high": 0.9}}"#,
            "risk",
            &ALL
        )
        .is_none());
    }

    #[test]
    fn partial_or_unnormalised_probabilities_are_rejected() {
        // Omitted label must not be renormalised into a confident verdict.
        assert!(
            parse_choice_json::<Lvl>(r#"{"probabilities": {"low": 0.6}}"#, "risk", &ALL).is_none()
        );
        // Does not sum to one.
        assert!(parse_choice_json::<Lvl>(
            r#"{"probabilities": {"low": 0.6, "high": 0.6}}"#,
            "risk",
            &ALL
        )
        .is_none());
        // Negative / non-finite.
        assert!(parse_choice_json::<Lvl>(
            r#"{"probabilities": {"low": -0.2, "high": 1.2}}"#,
            "risk",
            &ALL
        )
        .is_none());
        // Within tolerance is fine.
        let c = parse_choice_json::<Lvl>(
            r#"{"probabilities": {"low": 0.3, "high": 0.72}}"#,
            "risk",
            &ALL,
        )
        .unwrap();
        assert_eq!(c.argmax().unwrap().0, &Lvl::High);
    }

    #[test]
    fn probabilities_ties_resolve_in_allowed_order() {
        // ALL = [High, Low]; lexical order would pick "high" anyway, so use a
        // reversed allowed slice to prove ordering follows the caller.
        let rev = [Lvl::Low, Lvl::High];
        let c = parse_choice_json::<Lvl>(
            r#"{"probabilities": {"high": 0.5, "low": 0.5}}"#,
            "risk",
            &rev,
        )
        .unwrap();
        assert_eq!(c.argmax().unwrap().0, &Lvl::Low);
    }

    #[test]
    fn extracts_first_balanced_object() {
        assert_eq!(
            extract_json_object(r#"a {"x": {"y": 1}} b {"z": 2}"#).as_deref(),
            Some(r#"{"x": {"y": 1}}"#)
        );
        assert_eq!(
            extract_json_object(r#"{"s": "br}ace"} tail"#).as_deref(),
            Some(r#"{"s": "br}ace"}"#)
        );
        assert!(extract_json_object("{unterminated").is_none());
    }

    #[test]
    fn parses_probabilities_mode() {
        let c = parse_choice_json::<Lvl>(
            r#"{"probabilities": {"low": 0.25, "high": 0.75}}"#,
            "risk",
            &ALL,
        )
        .unwrap();
        assert_eq!(c.argmax(), Some((&Lvl::High, 0.75)));
    }

    #[test]
    fn clarification_question_prefix() {
        let c = clarification_from_raw("QUESTION: Delete which directory?").unwrap();
        assert!(c.should_ask());
        assert_eq!(c.question.as_deref(), Some("Delete which directory?"));
    }

    #[test]
    fn clarification_json_mode_with_floor() {
        let c = clarification_from_raw(
            r#"{"needs_clarification": true, "p": 0.95, "question": "Which port?"}"#,
        )
        .unwrap();
        assert!(c.should_ask());
        assert_eq!(c.question.as_deref(), Some("Which port?"));

        let low = clarification_from_raw(r#"{"needs_clarification": true, "p": 0.4}"#).unwrap();
        assert!(!low.should_ask());
        assert!(clarification_from_raw(r#"{"needs_clarification": false, "cmd": "ls"}"#).is_none());
    }

    #[test]
    fn clarification_legacy_echo_is_typed() {
        let c = clarification_from_raw(r#"{"cmd": "echo 'Please clarify your request'"}"#).unwrap();
        assert!(c.should_ask());
        assert_eq!(c.question, None);
        assert!(clarification_from_raw(r#"{"cmd": "echo hello"}"#).is_none());
    }

    #[test]
    fn type_errors_return_none() {
        assert!(parse_choice_json::<Lvl>("not json", "risk", &ALL).is_none());
        assert!(parse_choice_json::<Lvl>(r#"{"risk": "bogus"}"#, "risk", &ALL).is_none());
        assert!(parse_choice_json::<Lvl>(r#"{"risk": 3}"#, "risk", &ALL).is_none());
        assert!(
            parse_choice_json::<Lvl>(r#"{"probabilities": {"low": "x"}}"#, "risk", &ALL).is_none()
        );
        assert!(
            parse_choice_json::<Lvl>(r#"{"probabilities": {"nope": 1}}"#, "risk", &ALL).is_none()
        );
        assert!(parse_choice_json::<Lvl>(
            r#"{"probabilities": {"low": 0, "high": 0}}"#,
            "risk",
            &ALL
        )
        .is_none());
    }
}
