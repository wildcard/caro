//! The CaroML interpreter — turn a parsed `.caro` Task into a `.caro.lock`.
//!
//! Orchestrates per-step generation through the configured backend, runs the
//! validator chain in a refinement loop, and assembles the final Lock with
//! per-platform variants, validations, and a history-trail entry.
//!
//! # v0.1 scope
//!
//! - `generate_lock_for_platform(task, platform, backend, validators, cfg)`
//!   produces a fresh Lock for one platform.
//! - `generate_lock_for_all_platforms(...)` is a thin loop over each
//!   declared platform.
//! - System snapshot is taken once at the start of the loop (no
//!   inter-step state evolution; that's `--interleave` mode in v0.2).
//! - Validators run in parallel for each iteration; the loop terminates when
//!   all pass, when a `must_pass` validator fails, or when `max_iterations`
//!   is exhausted.

use crate::backends::{CommandGenerator, GeneratorError};
use crate::caroml::ast::Task;
use crate::caroml::lock::{
    HistoryEntry, Lock, Meta, Step as LockStep, TaskMeta, ValidationEntry, Variant, SCHEMA_VERSION,
};
use crate::caroml::regen_evaluator::intent_hash as compute_intent_hash;
use crate::caroml::validators::{
    fatal_angle, run_all, should_repair, ValidationOutcome, Validator, ValidatorContext, Verdict,
};
use crate::caroml::variants::{generation_id, sibling_consistency_hint};
use crate::models::{CommandRequest, RiskLevel, SafetyLevel, ShellType};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

/// Configuration for one generation run.
#[derive(Debug, Clone)]
pub struct GenerateConfig {
    /// Maximum number of repair iterations per step. Default 4.
    pub max_iterations: u32,
    /// Caro version stamped into the lock's `[meta]`.
    pub caro_version: String,
    /// Trigger string written into the `[[history]]` lineage entry.
    pub trigger: String,
    /// Path to the source `.caro` file (relative to project root, ideally).
    pub intent_path: String,
}

impl GenerateConfig {
    pub fn for_intent(intent_path: impl Into<String>) -> Self {
        Self {
            max_iterations: 4,
            caro_version: env!("CARGO_PKG_VERSION").to_string(),
            trigger: "intent_hash_mismatch".into(),
            intent_path: intent_path.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("backend error: {0}")]
    Backend(#[from] GeneratorError),
    #[error("safety must-pass validator failed on step {step_line}: {note}")]
    SafetyBlocked { step_line: usize, note: String },
}

/// Generate a complete Lock for one platform.
pub async fn generate_lock_for_platform(
    task: &Task,
    platform: &str,
    backend: &dyn CommandGenerator,
    validators: &[Box<dyn Validator>],
    cfg: &GenerateConfig,
) -> Result<Lock, GenerateError> {
    let mut lock = Lock {
        schema_version: SCHEMA_VERSION,
        meta: Meta {
            caro_version: cfg.caro_version.clone(),
            intent_path: cfg.intent_path.clone(),
            intent_hash: compute_intent_hash(task),
            supported_platforms: vec![platform.to_string()],
            last_full_regen: Some(Utc::now()),
        },
        task: build_task_meta(task),
        steps: Vec::with_capacity(task.steps.len()),
        history: Vec::new(),
    };

    let now = Utc::now();
    let gen_id = generation_id(now, platform, 0);
    lock.history.push(HistoryEntry {
        generation_id: gen_id.clone(),
        trigger: cfg.trigger.clone(),
        caro_version: cfg.caro_version.clone(),
        model: "(probed-from-backend)".into(),
        backend: "(probed-from-backend)".into(),
        platform: platform.to_string(),
        generated_at: now,
        intent_hash: lock.meta.intent_hash.clone(),
        cve_feed_rev: None,
        notes: Some(format!("Initial generation for platform `{}`.", platform)),
    });

    // We need the backend's identity for the History entry; probe it once.
    let backend_info = backend.backend_info();
    let backend_label = format!("{:?}", backend_info.backend_type).to_lowercase();
    if let Some(h) = lock.history.last_mut() {
        h.model = backend_info.model_name.clone();
        h.backend = backend_label.clone();
    }

    // Iterate steps in order; each gets one variant for this platform.
    for (idx, step) in task.steps.iter().enumerate() {
        let prior_pairs: Vec<(&str, &str)> = lock
            .steps
            .iter()
            .filter_map(|ls| {
                ls.active_variant(platform)
                    .map(|v| (ls.intent.as_str(), v.command.as_str()))
            })
            .collect();

        let outcome = generate_step(
            task,
            step,
            idx,
            platform,
            backend,
            validators,
            cfg,
            &prior_pairs,
            &lock,
            &gen_id,
            &backend_info.model_name,
            &backend_label,
        )
        .await?;

        lock.steps.push(outcome);
    }

    Ok(lock)
}

/// Generate Locks for every declared platform.
///
/// If the task has no `ON <platform>` pragmas, falls back to the single
/// `current_platform`. Returns one Lock per platform with the union of
/// `supported_platforms` set in each `[meta]` block.
pub async fn generate_lock_for_all_platforms(
    task: &Task,
    current_platform: &str,
    backend: &dyn CommandGenerator,
    validators: &[Box<dyn Validator>],
    cfg: &GenerateConfig,
) -> Result<Lock, GenerateError> {
    let platforms = declared_platforms_or(task, current_platform);

    let mut combined: Option<Lock> = None;
    for platform in &platforms {
        let single = generate_lock_for_platform(task, platform, backend, validators, cfg).await?;
        combined = Some(merge_lock(combined, single, platform));
    }
    let mut final_lock = combined.expect("at least one platform must be processed");
    final_lock.meta.supported_platforms = platforms;
    Ok(final_lock)
}

// ---------------------------------------------------------------------------
// Per-step generation
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
async fn generate_step(
    task: &Task,
    step: &crate::caroml::ast::Step,
    step_idx: usize,
    platform: &str,
    backend: &dyn CommandGenerator,
    validators: &[Box<dyn Validator>],
    cfg: &GenerateConfig,
    prior_pairs: &[(&str, &str)],
    lock_so_far: &Lock,
    gen_id: &str,
    model: &str,
    backend_name: &str,
) -> Result<LockStep, GenerateError> {
    let mut iter = 0;
    let mut prior_failures: Vec<ValidationOutcome> = Vec::new();
    let sudo_declared = task.needs.iter().any(|n| n == "sudo");

    let (final_command, outcomes, iterations) = loop {
        let context = build_step_context(
            task,
            step,
            step_idx,
            platform,
            prior_pairs,
            lock_so_far,
            &prior_failures,
        );
        let request = CommandRequest::new(step.intent.clone(), shell_for(platform))
            .with_safety(SafetyLevel::Moderate)
            .with_context(context);

        let generated = backend.generate_command(&request).await?;

        let validator_ctx = ValidatorContext {
            command: &generated.command,
            intent: &step.intent,
            task_title: &task.title,
            platform,
            sudo_declared,
            capability_profile: None,
        };
        let outcomes = run_all(validators, &validator_ctx).await;

        // Must-pass failures (e.g. safety) ALWAYS short-circuit, even on the
        // last allowed iteration — otherwise a max-iter exhaustion would
        // silently land the bad command in the lock with risk_level=high.
        if let Some(angle) = fatal_angle(validators, &outcomes) {
            if angle == "safety" {
                return Err(GenerateError::SafetyBlocked {
                    step_line: step.line,
                    note: outcomes
                        .iter()
                        .find(|o| o.angle == angle)
                        .and_then(|o| o.note.clone())
                        .unwrap_or_else(|| "blocked by safety validator".to_string()),
                });
            }
        }

        if !should_repair(&outcomes) || iter >= cfg.max_iterations.saturating_sub(1) {
            break (generated, outcomes, iter + 1);
        }

        prior_failures = outcomes
            .into_iter()
            .filter(|o| o.result != Verdict::Pass)
            .collect();
        iter += 1;
    };

    let variant = build_variant(
        platform,
        gen_id,
        &final_command.command,
        &final_command.explanation,
        final_command.confidence_score as f32,
        iterations,
        outcomes,
        model,
        backend_name,
    );

    Ok(LockStep {
        line: step.line,
        intent: step.intent.clone(),
        intent_hash: hash_step_intent(&step.intent),
        notes: step.notes.clone(),
        variants: vec![variant],
    })
}

#[allow(clippy::too_many_arguments)]
fn build_variant(
    platform: &str,
    generation_id: &str,
    command: &str,
    reasoning: &str,
    confidence: f32,
    iterations: u32,
    outcomes: Vec<ValidationOutcome>,
    model: &str,
    backend: &str,
) -> Variant {
    // Risk level — derived from validator outcomes.
    let risk = if outcomes
        .iter()
        .any(|o| o.angle == "safety" && o.result == Verdict::Fail)
    {
        "high"
    } else if outcomes.iter().any(|o| o.result != Verdict::Pass) {
        "moderate"
    } else {
        "safe"
    };

    let validations = outcomes
        .iter()
        .map(|o| ValidationEntry {
            angle: o.angle.clone(),
            result: match o.result {
                Verdict::Pass => "pass",
                Verdict::Warn => "warn",
                Verdict::Fail => "fail",
            }
            .to_string(),
            note: o.note.clone(),
        })
        .collect();

    let warnings: Vec<String> = outcomes
        .iter()
        .filter(|o| o.result != Verdict::Pass)
        .filter_map(|o| o.note.clone())
        .collect();

    Variant {
        platform: platform.to_string(),
        active: true,
        generation_id: generation_id.to_string(),
        command: command.to_string(),
        reasoning: reasoning.to_string(),
        exports: vec![],
        imports: vec![],
        risk_level: risk.to_string(),
        matched_patterns: vec![],
        warnings,
        confidence,
        iterations,
        validations,
        generated_at: Utc::now(),
        model: model.to_string(),
        backend: backend.to_string(),
        tool_versions: BTreeMap::new(),
        track_record: Default::default(),
        retired_at: None,
        runbook_hash: String::new(),
    }
}

// ---------------------------------------------------------------------------
// Prompt context
// ---------------------------------------------------------------------------

fn build_step_context(
    task: &Task,
    step: &crate::caroml::ast::Step,
    step_idx: usize,
    platform: &str,
    prior_pairs: &[(&str, &str)],
    lock_so_far: &Lock,
    prior_failures: &[ValidationOutcome],
) -> String {
    let mut s = String::new();
    s.push_str("== TASK ==\n");
    s.push_str(&format!("TITLE: {}\n", task.title));
    if let Some(why) = &task.why {
        s.push_str(&format!("WHY:   {}\n", why));
    }
    if !task.needs.is_empty() {
        s.push_str(&format!("NEED:  {}\n", task.needs.join(", ")));
    }
    for p in &task.platform_pragmas {
        s.push_str(&format!("ON     {}", p.platform));
        if !p.prefer.is_empty() {
            s.push_str(&format!(" PREFER {}", p.prefer.join(", ")));
        }
        if !p.avoid.is_empty() {
            s.push_str(&format!(" AVOID {}", p.avoid.join(", ")));
        }
        s.push('\n');
    }

    if !task.params.is_empty() {
        s.push_str("\n== PARAMS ==\n");
        for p in &task.params {
            s.push_str(&format!("{} = {}\n", p.name, p.value));
        }
    }

    s.push_str(&format!("\n== PLATFORM ==\nTarget: {}\n", platform));

    if !prior_pairs.is_empty() {
        s.push_str("\n== PRIOR STEPS ==\n");
        for (i, (intent, command)) in prior_pairs.iter().enumerate() {
            s.push_str(&format!(
                "{}. (intent: {})\n   command: {}\n",
                i + 1,
                intent,
                command
            ));
        }
    }

    if !step.notes.is_empty() {
        s.push_str("\n== AUTHOR NOTES (for this step) ==\n");
        for note in &step.notes {
            s.push_str(&format!("- {}\n", note));
        }
    }

    if let Some(hint) = sibling_consistency_hint(lock_so_far, step_idx, platform) {
        s.push_str("\n== SIBLING-PLATFORM HINT ==\n");
        s.push_str(&hint);
        s.push('\n');
    }

    if !prior_failures.is_empty() {
        s.push_str("\n== REPAIR HINTS (prior iteration) ==\n");
        for failure in prior_failures {
            if let Some(hint) = &failure.repair_hint {
                s.push_str(&format!("- [{}] {}\n", failure.angle, hint));
            } else if let Some(note) = &failure.note {
                s.push_str(&format!("- [{}] {}\n", failure.angle, note));
            }
        }
    }

    s
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_task_meta(task: &Task) -> TaskMeta {
    let mut prefers_by_platform = BTreeMap::new();
    let mut avoids_by_platform = BTreeMap::new();
    for p in &task.platform_pragmas {
        if !p.prefer.is_empty() {
            prefers_by_platform.insert(p.platform.clone(), p.prefer.clone());
        }
        if !p.avoid.is_empty() {
            avoids_by_platform.insert(p.platform.clone(), p.avoid.clone());
        }
    }
    let mut params = BTreeMap::new();
    for p in &task.params {
        params.insert(p.name.clone(), p.value.clone());
    }
    TaskMeta {
        title: task.title.clone(),
        why: task.why.clone(),
        needs: task.needs.clone(),
        prefers_by_platform,
        avoids_by_platform,
        params,
    }
}

fn declared_platforms_or(task: &Task, fallback: &str) -> Vec<String> {
    let declared: Vec<String> = task
        .platform_pragmas
        .iter()
        .map(|p| p.platform.clone())
        .collect();
    if declared.is_empty() {
        vec![fallback.to_string()]
    } else {
        declared
    }
}

fn shell_for(platform: &str) -> ShellType {
    match platform {
        "windows" => ShellType::PowerShell,
        _ => ShellType::Bash,
    }
}

fn hash_step_intent(intent: &str) -> String {
    let mut h = Sha256::new();
    h.update(intent.as_bytes());
    let hex = h
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    format!("sha256:{}", hex)
}

/// Merge a per-platform lock into a multi-platform combined lock by appending
/// each step's variants. The first call (combined = None) returns `single` as-is.
fn merge_lock(combined: Option<Lock>, single: Lock, _platform: &str) -> Lock {
    let mut base = match combined {
        Some(b) => b,
        None => return single,
    };
    // Steps must align by line number.
    for (i, step) in single.steps.into_iter().enumerate() {
        if let Some(existing) = base.steps.get_mut(i) {
            existing.variants.extend(step.variants);
        } else {
            base.steps.push(step);
        }
    }
    base.history.extend(single.history);
    base
}

// Avoid clippy::risk warning on RiskLevel re-export
#[allow(dead_code)]
fn _risk_level_used(_: RiskLevel) {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
    use crate::caroml::ast::{Step, Task};
    use crate::caroml::validators::default_chain;
    use crate::models::{BackendType, GeneratedCommand};
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// Test double that yields pre-set commands keyed by call index.
    pub struct ScriptedBackend {
        pub responses: Mutex<Vec<Result<GeneratedCommand, GeneratorError>>>,
    }

    impl ScriptedBackend {
        pub fn new(responses: Vec<GeneratedCommand>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().map(Ok).collect()),
            }
        }
    }

    #[async_trait]
    impl CommandGenerator for ScriptedBackend {
        async fn generate_command(
            &self,
            _request: &CommandRequest,
        ) -> Result<GeneratedCommand, GeneratorError> {
            let mut q = self.responses.lock().unwrap();
            if q.is_empty() {
                return Err(GeneratorError::GenerationFailed {
                    details: "scripted backend exhausted".into(),
                });
            }
            q.remove(0)
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Mock,
                model_name: "scripted-test".into(),
                supports_streaming: false,
                max_tokens: 1024,
                typical_latency_ms: 1,
                memory_usage_mb: 0,
                version: "0.0.1".into(),
            }
        }

        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    fn gen_cmd(command: &str) -> GeneratedCommand {
        GeneratedCommand {
            command: command.into(),
            explanation: "test".into(),
            safety_level: RiskLevel::Safe,
            estimated_impact: "test".into(),
            alternatives: vec![],
            backend_used: "scripted".into(),
            generation_time_ms: 1,
            confidence_score: 0.9,
        }
    }

    fn one_step_task() -> Task {
        Task {
            source_path: None,
            title: "demo".into(),
            why: None,
            needs: vec![],
            platform_pragmas: vec![],
            params: vec![],
            steps: vec![Step {
                line: 2,
                intent: "list /tmp".into(),
                raw_intent: "list /tmp".into(),
                notes: vec![],
            }],
        }
    }

    #[tokio::test]
    async fn single_step_single_platform_generates_lock() {
        let task = one_step_task();
        let backend = ScriptedBackend::new(vec![gen_cmd("ls /tmp")]);
        let validators = default_chain();
        let cfg = GenerateConfig::for_intent("tasks/demo.caro");

        let lock = generate_lock_for_platform(&task, "macos", &backend, &validators, &cfg)
            .await
            .unwrap();

        assert_eq!(lock.schema_version, SCHEMA_VERSION);
        assert_eq!(lock.meta.intent_path, "tasks/demo.caro");
        assert_eq!(lock.steps.len(), 1);
        let v = &lock.steps[0].variants[0];
        assert_eq!(v.command, "ls /tmp");
        assert!(v.active);
        assert_eq!(v.platform, "macos");
        assert_eq!(v.iterations, 1);
        assert_eq!(lock.history.len(), 1);
    }

    #[tokio::test]
    async fn validation_loop_repairs_on_safety_warn_then_pass() {
        // First response triggers the secrets warn (basic-auth URL),
        // second response is clean. Loop should converge in 2 iterations.
        let task = one_step_task();
        let backend = ScriptedBackend::new(vec![
            gen_cmd("curl https://admin:secret@example.com"),
            gen_cmd("curl -u admin:$PASS https://example.com"),
        ]);
        let validators = default_chain();
        let cfg = GenerateConfig::for_intent("tasks/demo.caro");

        let lock = generate_lock_for_platform(&task, "linux", &backend, &validators, &cfg)
            .await
            .unwrap();
        let v = &lock.steps[0].variants[0];
        assert_eq!(v.iterations, 2, "expected 2 iterations of repair");
        assert!(v.command.contains("$PASS"));
    }

    #[tokio::test]
    async fn safety_must_pass_failure_returns_error() {
        // `rm -rf /` is hard-blocked by the safety validator.
        let task = one_step_task();
        let backend = ScriptedBackend::new(vec![
            gen_cmd("rm -rf /"),
            gen_cmd("rm -rf /"),
            gen_cmd("rm -rf /"),
            gen_cmd("rm -rf /"),
            gen_cmd("rm -rf /"),
        ]);
        let validators = default_chain();
        let cfg = GenerateConfig::for_intent("tasks/demo.caro");

        let result = generate_lock_for_platform(&task, "linux", &backend, &validators, &cfg).await;
        match result {
            Err(GenerateError::SafetyBlocked { step_line: 2, .. }) => {}
            other => panic!("expected SafetyBlocked, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn multi_platform_yields_one_variant_per_platform_per_step() {
        let mut task = one_step_task();
        task.platform_pragmas = vec![
            crate::caroml::ast::PlatformPragma {
                platform: "macos".into(),
                prefer: vec![],
                avoid: vec![],
            },
            crate::caroml::ast::PlatformPragma {
                platform: "linux".into(),
                prefer: vec![],
                avoid: vec![],
            },
        ];
        let backend = ScriptedBackend::new(vec![
            gen_cmd("ls -G /tmp"),      // macos
            gen_cmd("ls --color /tmp"), // linux
        ]);
        let validators = default_chain();
        let cfg = GenerateConfig::for_intent("tasks/demo.caro");

        let lock = generate_lock_for_all_platforms(&task, "macos", &backend, &validators, &cfg)
            .await
            .unwrap();

        assert_eq!(lock.steps.len(), 1);
        assert_eq!(lock.steps[0].variants.len(), 2);
        let platforms: Vec<&str> = lock.steps[0]
            .variants
            .iter()
            .map(|v| v.platform.as_str())
            .collect();
        assert!(platforms.contains(&"macos"));
        assert!(platforms.contains(&"linux"));
        assert_eq!(
            lock.meta.supported_platforms,
            vec!["macos".to_string(), "linux".to_string()]
        );
    }

    #[tokio::test]
    async fn intent_hash_is_set_in_meta() {
        let task = one_step_task();
        let backend = ScriptedBackend::new(vec![gen_cmd("ls")]);
        let validators = default_chain();
        let cfg = GenerateConfig::for_intent("tasks/demo.caro");
        let lock = generate_lock_for_platform(&task, "macos", &backend, &validators, &cfg)
            .await
            .unwrap();
        assert!(lock.meta.intent_hash.starts_with("sha256:"));
        assert_eq!(lock.meta.intent_hash, compute_intent_hash(&task));
    }
}
