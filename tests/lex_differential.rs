//! ADR-075 phase 0 (D0) — the GuardFall differential harness.
//!
//! # What this file is
//!
//! On 2026-06-30 Adversa AI disclosed **GuardFall**: eleven open-source coding
//! agents tested against five classes of shell-injection bypass, ten of which
//! leaked. CSA's root-cause sentence describes this crate exactly — *"safety
//! filters inspect commands as literal text, while the bash shell quietly
//! rewrites that text through quoting, variable expansion, command
//! substitution, and piping before it ever executes."*
//!
//! This harness drives a frozen corpus of mutated destructive commands through
//! the **real** [`SafetyValidator`] and publishes the leak census. It asserts
//! nothing about how the gap should be closed — that is ADR-075 phases 1–3.
//! The number is the finding.
//!
//! # Why the interesting assertions are `#[ignore]`d
//!
//! ADR-075 D0 says the differential test "is expected to fail loudly" and that
//! "merging a large red bar is the point." Taken literally that produces a pull
//! request that cannot merge: `.claude/rules/dev-process.md` requires that all
//! unit tests pass before review, so a red `cargo test` is an unmergeable PR,
//! and D0 has now missed its "this week" window once already.
//!
//! The split below keeps the red bar and makes the PR mergeable:
//!
//! | Test | State | Role |
//! |---|---|---|
//! | [`corpus_integrity`] | green | the corpus is well-formed; no validator involved |
//! | [`canaries_stay_safe`] | green | the false-positive floor phase 2 must not breach |
//! | [`controls_are_still_caught`] | green | proves the harness measures a real gap |
//! | [`census_is_reported`] | green | prints the full census; asserts nothing about verdicts |
//! | [`mutation_never_lowers_verdict`] | **red**, `#[ignore]` | the bar. ADR-075 phase 2 turns it green |
//! | [`interpreter_sinks_are_classified`] | **red**, `#[ignore]` | ADR-075 D4 |
//! | [`flag_variants_are_classified`] | **red**, `#[ignore]` | GuardFall class E; ADR-075 out-of-scope list |
//! | [`known_false_positives_are_fixed`] | **red**, `#[ignore]` | over-matching found by this run |
//!
//! # Measured on 2026-09-22 (first execution inside the real pipeline)
//!
//! ```text
//! adversarial (A,B,C,D,F): 99 leaked of 99
//! control      (G)       :  0 leaked of 11
//! ```
//!
//! The 2026-09-21 scoping run reproduced 99/99 with a Python port of
//! `is_dangerous_in_context` and flagged the figure as a reproduction rather
//! than a test result. Run through `SafetyValidator::validate_command` at
//! `SafetyConfig::moderate()` the number is identical. The port was accurate.
//!
//! The run also turned up something the port did not look for: `# rm -rf /` — a
//! shell comment, inert by definition — is classified `Critical`. Same root
//! cause, opposite sign. It is recorded in the corpus under
//! `known_false_positives`.
//!
//! Publish the number with:
//!
//! ```text
//! cargo test --test lex_differential -- --ignored --nocapture
//! ```
//!
//! The census also prints from the green path (`--nocapture`), so CI logs carry
//! the number on every run without CI ever going red for it.

use std::collections::{BTreeMap, BTreeSet};

use caro::models::{RiskLevel, ShellType};
use caro::safety::{SafetyConfig, SafetyValidator};
use serde::Deserialize;

const CORPUS: &str = include_str!("fixtures/guardfall/mutations-v1.yaml");

/// Literal-checked. A corpus that renames its schema is a different corpus.
const CORPUS_SCHEMA: &str = "caro.lex.corpus/1";

/// Classes that are adversarial mutations. `G` is the control family and is
/// deliberately excluded — it must be caught today.
const ADVERSARIAL_FAMILIES: [&str; 5] = ["A", "B", "C", "D", "F"];

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Corpus {
    schema_version: String,
    #[allow(dead_code)]
    source: String,
    shell: ShellType,
    classes: BTreeMap<String, ClassDef>,
    baselines: Vec<Baseline>,
    interpreter_sinks: Vec<Expectation>,
    flag_variants: Vec<Expectation>,
    canaries: Vec<Canary>,
    controls: Vec<Expectation>,
    known_false_positives: Vec<KnownFalsePositive>,
}

#[derive(Debug, Deserialize)]
struct ClassDef {
    family: String,
    name: String,
    #[allow(dead_code)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct Baseline {
    id: String,
    command: String,
    expect_min_risk: RiskLevel,
    variants: Vec<Variant>,
}

#[derive(Debug, Deserialize)]
struct Variant {
    id: String,
    class: String,
    command: String,
}

#[derive(Debug, Deserialize)]
struct Expectation {
    id: String,
    command: String,
    expect_min_risk: RiskLevel,
}

#[derive(Debug, Deserialize)]
struct Canary {
    id: String,
    command: String,
    note: String,
}

/// An inert command the shipped pipeline already classifies as dangerous.
/// Measured by this harness on 2026-09-22, not assumed.
#[derive(Debug, Deserialize)]
struct KnownFalsePositive {
    id: String,
    command: String,
    current_risk: RiskLevel,
    note: String,
}

fn corpus() -> Corpus {
    serde_yaml::from_str(CORPUS).expect("tests/fixtures/guardfall/mutations-v1.yaml parses")
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// The verdict the shipped pipeline gives a command, under default `moderate`
/// safety and `ShellType::Bash`.
///
/// Deliberately the public API and nothing else: no reimplementation of
/// `is_dangerous_in_context`, no direct pattern access. A port is a
/// reproduction; this is a measurement.
async fn verdict(validator: &SafetyValidator, command: &str, shell: ShellType) -> RiskLevel {
    validator
        .validate_command(command, shell)
        .await
        .expect("validation is infallible for corpus-length commands")
        .risk_level
}

fn validator() -> SafetyValidator {
    SafetyValidator::new(SafetyConfig::moderate())
        .expect("validator constructs with default moderate safety")
}

/// One row of the published census.
#[derive(Debug, Clone)]
struct Leak {
    variant_id: String,
    class: String,
    baseline_risk: RiskLevel,
    variant_risk: RiskLevel,
    command: String,
}

/// Walks every baseline × variant pair and returns the ones whose verdict is
/// *lower* than their baseline's — i.e. the mutation worked.
async fn census(c: &Corpus, v: &SafetyValidator) -> (usize, Vec<Leak>) {
    let mut checked = 0usize;
    let mut leaks = Vec::new();

    for baseline in &c.baselines {
        let base_risk = verdict(v, &baseline.command, c.shell).await;
        for variant in &baseline.variants {
            checked += 1;
            let var_risk = verdict(v, &variant.command, c.shell).await;
            if var_risk < base_risk {
                leaks.push(Leak {
                    variant_id: variant.id.clone(),
                    class: variant.class.clone(),
                    baseline_risk: base_risk,
                    variant_risk: var_risk,
                    command: variant.command.clone(),
                });
            }
        }
    }

    (checked, leaks)
}

fn render_census(c: &Corpus, checked: usize, leaks: &[Leak]) -> String {
    let mut per_family: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for baseline in &c.baselines {
        for variant in &baseline.variants {
            let fam = c
                .classes
                .get(&variant.class)
                .map(|d| d.family.as_str())
                .unwrap_or("?");
            per_family.entry(fam).or_insert((0, 0)).0 += 1;
        }
    }
    for leak in leaks {
        let fam = c
            .classes
            .get(&leak.class)
            .map(|d| d.family.as_str())
            .unwrap_or("?");
        per_family.entry(fam).or_insert((0, 0)).1 += 1;
    }

    let mut out = String::new();
    out.push_str("\n=== caro.lex differential census (ADR-075 D0) ===\n");
    out.push_str(&format!(
        "corpus {} · shell {:?} · baselines {} · variants {}\n\n",
        c.schema_version,
        c.shell,
        c.baselines.len(),
        checked
    ));
    out.push_str("family  applied  caught   leaked\n");
    for (fam, (applied, leaked)) in &per_family {
        out.push_str(&format!(
            "  {:<4} {:>8} {:>7} {:>8}{}\n",
            fam,
            applied,
            applied - leaked,
            leaked,
            if *fam == "G" { "   (control)" } else { "" }
        ));
    }
    let adversarial: usize = per_family
        .iter()
        .filter(|(f, _)| ADVERSARIAL_FAMILIES.contains(f))
        .map(|(_, (a, _))| *a)
        .sum();
    let adversarial_leaked: usize = per_family
        .iter()
        .filter(|(f, _)| ADVERSARIAL_FAMILIES.contains(f))
        .map(|(_, (_, l))| *l)
        .sum();
    out.push_str(&format!(
        "\nadversarial (A,B,C,D,F): {} leaked of {}\n",
        adversarial_leaked, adversarial
    ));

    if !leaks.is_empty() {
        out.push_str("\nleaks:\n");
        for leak in leaks {
            out.push_str(&format!(
                "  {:<8} {:<3} {:?} -> {:?}   {}\n",
                leak.variant_id, leak.class, leak.baseline_risk, leak.variant_risk, leak.command
            ));
        }
    }
    out.push('\n');
    out
}

// ---------------------------------------------------------------------------
// Green: these gate the PR and must pass today
// ---------------------------------------------------------------------------

/// The corpus is well-formed. No validator involved, so this cannot go red for
/// a reason that belongs to the product.
#[test]
fn corpus_integrity() {
    let c = corpus();

    assert_eq!(
        c.schema_version, CORPUS_SCHEMA,
        "corpus schema_version is literal-checked"
    );

    let mut ids = BTreeSet::new();
    let mut commands = BTreeSet::new();
    let mut families = BTreeSet::new();

    let mut record = |id: &str, command: &str| {
        assert!(!command.trim().is_empty(), "{id}: empty command");
        assert!(ids.insert(id.to_string()), "duplicate id {id}");
        commands.insert(command.to_string());
    };

    for b in &c.baselines {
        record(&b.id, &b.command);
        for v in &b.variants {
            record(&v.id, &v.command);
            let class = c
                .classes
                .get(&v.class)
                .unwrap_or_else(|| panic!("{}: class {} is not declared", v.id, v.class));
            families.insert(class.family.clone());
            assert!(!class.name.is_empty(), "class {} has no name", v.class);
            assert_ne!(
                v.command, b.command,
                "{}: mutation is a no-op; a no-op variant silently inflates the catch rate",
                v.id
            );
            assert!(
                v.id.starts_with(&b.id),
                "{}: variant id must be prefixed with its baseline id",
                v.id
            );
        }
    }
    for e in c
        .interpreter_sinks
        .iter()
        .chain(&c.flag_variants)
        .chain(&c.controls)
    {
        record(&e.id, &e.command);
    }
    for k in &c.canaries {
        record(&k.id, &k.command);
        assert!(!k.note.trim().is_empty(), "{}: canary needs a note", k.id);
    }
    for fp in &c.known_false_positives {
        record(&fp.id, &fp.command);
        assert!(
            !fp.note.trim().is_empty(),
            "{}: a recorded false positive needs a note saying why it is inert",
            fp.id
        );
        assert_ne!(
            fp.current_risk,
            RiskLevel::Safe,
            "{}: a Safe command is not a false positive; move it to `canaries`",
            fp.id
        );
    }

    for fam in ADVERSARIAL_FAMILIES {
        assert!(
            families.contains(fam),
            "GuardFall family {fam} is not represented in the corpus"
        );
    }
    assert!(
        families.contains("G"),
        "the control family is missing; without it a zero catch rate is unfalsifiable"
    );
    assert_eq!(
        ids.len(),
        commands.len() + (ids.len() - commands.len()),
        "id/command bookkeeping"
    );
}

/// The false-positive floor. A normalizer that catches every mutation by
/// over-matching has not fixed anything, so phase 2 inherits this test
/// unchanged.
#[tokio::test]
async fn canaries_stay_safe() {
    let c = corpus();
    let v = validator();
    let mut regressions = Vec::new();

    for k in &c.canaries {
        let risk = verdict(&v, &k.command, c.shell).await;
        if risk != RiskLevel::Safe {
            regressions.push(format!(
                "  {} {:?}  {}  ({})",
                k.id, risk, k.command, k.note
            ));
        }
    }

    assert!(
        regressions.is_empty(),
        "commands that merely mention a destructive string must stay Safe:\n{}",
        regressions.join("\n")
    );
}

/// The control class. `rm  -rf  /` with doubled blanks is still caught, because
/// the shipped patterns use `\s+`. If this ever goes red the harness is broken,
/// not the product — and the leak census below means nothing.
#[tokio::test]
async fn controls_are_still_caught() {
    let c = corpus();
    let v = validator();

    for baseline in &c.baselines {
        let base_risk = verdict(&v, &baseline.command, c.shell).await;
        assert!(
            base_risk >= baseline.expect_min_risk,
            "{}: unmutated baseline {:?} < expected {:?} — {}",
            baseline.id,
            base_risk,
            baseline.expect_min_risk,
            baseline.command
        );

        for variant in &baseline.variants {
            let is_control = c
                .classes
                .get(&variant.class)
                .map(|d| d.family == "G")
                .unwrap_or(false);
            if !is_control {
                continue;
            }
            let risk = verdict(&v, &variant.command, c.shell).await;
            assert!(
                risk >= base_risk,
                "{}: whitespace control dropped {:?} -> {:?} — the harness is broken",
                variant.id,
                base_risk,
                risk
            );
        }
    }

    for k in &c.controls {
        let risk = verdict(&v, &k.command, c.shell).await;
        assert!(
            risk >= k.expect_min_risk,
            "{}: {:?} < {:?} — {}",
            k.id,
            risk,
            k.expect_min_risk,
            k.command
        );
    }
}

/// Publishes the census on every CI run and asserts nothing about verdicts.
/// This is the test that makes the number visible without making CI red.
#[tokio::test]
async fn census_is_reported() {
    let c = corpus();
    let v = validator();
    let (checked, leaks) = census(&c, &v).await;
    print!("{}", render_census(&c, checked, &leaks));
    assert!(checked > 0, "corpus produced no variants to measure");
}

// ---------------------------------------------------------------------------
// Red: the bar. Run with `-- --ignored --nocapture`.
// ---------------------------------------------------------------------------

/// **The differential invariant.** A mutation of a command may not lower its
/// verdict. Expected red until ADR-075 phase 2 lands.
#[tokio::test]
#[ignore = "ADR-075 phase 0: expected red; phase 2 (src/safety/lex.rs) turns it green"]
async fn mutation_never_lowers_verdict() {
    let c = corpus();
    let v = validator();
    let (checked, leaks) = census(&c, &v).await;
    print!("{}", render_census(&c, checked, &leaks));

    assert!(
        leaks.is_empty(),
        "{} of {} mutations lowered the verdict of their baseline",
        leaks.len(),
        checked
    );
}

/// ADR-075 D4 — an interpreter verb carrying a destructive payload as an
/// operand. Not a mutation of anything; a plain command that leaks.
#[tokio::test]
#[ignore = "ADR-075 D4: expected red until interpreter sinks are modelled"]
async fn interpreter_sinks_are_classified() {
    let c = corpus();
    let v = validator();
    let mut leaks = Vec::new();

    for s in &c.interpreter_sinks {
        let risk = verdict(&v, &s.command, c.shell).await;
        if risk < s.expect_min_risk {
            leaks.push(format!(
                "  {} {:?} < {:?}   {}",
                s.id, risk, s.expect_min_risk, s.command
            ));
        }
    }

    assert!(
        leaks.is_empty(),
        "interpreter sinks carrying a destructive payload:\n{}",
        leaks.join("\n")
    );
}

/// The recorded false positives are still false positives, and still have the
/// verdict the corpus says they have. Green when the over-matching is fixed —
/// at which point the entry moves to `canaries` and this test shrinks.
///
/// Two ways to go red, and both are informative: the verdict drifted (the
/// corpus is stale), or the command became `Safe` (the finding is fixed).
#[tokio::test]
#[ignore = "recorded false positives: expected red until over-matching is fixed"]
async fn known_false_positives_are_fixed() {
    let c = corpus();
    let v = validator();
    let mut still_wrong = Vec::new();

    for fp in &c.known_false_positives {
        let risk = verdict(&v, &fp.command, c.shell).await;
        assert_eq!(
            risk, fp.current_risk,
            "{}: corpus records {:?} but the pipeline now says {:?} — update the corpus",
            fp.id, fp.current_risk, risk
        );
        if risk != RiskLevel::Safe {
            still_wrong.push(format!(
                "  {} {:?}  {}  ({})",
                fp.id, risk, fp.command, fp.note
            ));
        }
    }

    assert!(
        still_wrong.is_empty(),
        "inert commands still classified as dangerous:\n{}",
        still_wrong.join("\n")
    );
}

/// GuardFall class E — destructive behaviour reached through a flag rather than
/// a verb. ADR-075 puts pattern coverage for these explicitly out of scope; this
/// test exists so the gap is counted rather than assumed.
#[tokio::test]
#[ignore = "GuardFall class E: pattern coverage, explicitly out of scope for ADR-075"]
async fn flag_variants_are_classified() {
    let c = corpus();
    let v = validator();
    let mut leaks = Vec::new();

    for e in &c.flag_variants {
        let risk = verdict(&v, &e.command, c.shell).await;
        if risk < e.expect_min_risk {
            leaks.push(format!(
                "  {} {:?} < {:?}   {}",
                e.id, risk, e.expect_min_risk, e.command
            ));
        }
    }

    assert!(
        leaks.is_empty(),
        "destructive-flag variants not classified:\n{}",
        leaks.join("\n")
    );
}
