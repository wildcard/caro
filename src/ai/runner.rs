//! Once-mode AI invocation: resume/new session, generate a command, validate
//! safety, persist the turn.
//!
//! The interactive REPL is out of scope for the MVP. `run_once` is the primitive
//! both scripted callers (`caro ai --once "prompt"`) and the future REPL will
//! use, so putting the command generation + safety + persistence logic here
//! keeps it testable without a TTY.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};

use crate::backends::CommandGenerator;
use crate::context::ExecutionContext;
use crate::models::{AiConfig, CommandRequest, RiskLevel, SafetyLevel, ShellType};
use crate::safety::{SafetyConfig, SafetyValidator};

use super::privacy::{build_context, ContextInputs};
use super::session::Turn;
use super::store::SessionStore;

/// How the caller wants a session reconciled before this turn.
#[derive(Debug, Clone, Copy, Default)]
pub enum SessionMode {
    /// Resume the most-recent-within-TTL session, otherwise create a new one.
    #[default]
    ResumeOrNew,
    /// Always create a new session.
    New,
    /// Resume if recent, otherwise fail (useful for strict scripting).
    ResumeStrict,
}

/// Inputs for one AI turn.
pub struct AiInvocation<'a> {
    pub prompt: &'a str,
    pub ai_cfg: &'a AiConfig,
    pub backend: Arc<dyn CommandGenerator>,
    pub backend_name: String,
    pub exec_ctx: ExecutionContext,
    pub validator: Arc<SafetyValidator>,
    pub safety_level: SafetyLevel,
    pub shell: ShellType,
    pub store_path: PathBuf,
    pub session_mode: SessionMode,
    /// Optional last command from shell hook ($HISTCMD, fc, etc.). Only forwarded
    /// when `ai_cfg.opening.send_last_command` is true.
    pub last_command_hint: Option<String>,
}

/// Result of a single AI turn.
#[derive(Debug, Clone)]
pub struct AiOutcome {
    pub session_id: u64,
    pub command: String,
    pub explanation: String,
    pub confidence: f64,
    pub risk: RiskLevel,
    pub warnings: Vec<String>,
    pub allowed: bool,
    /// True when the new session was freshly created (vs resumed).
    pub resumed: bool,
    /// True when the configured backend + opt-in context combination would
    /// send machine-specific data off-host.
    pub warns_offhost: bool,
}

/// Execute a single AI turn: pick or create a session, generate, validate, persist.
pub async fn run_once(inv: AiInvocation<'_>) -> Result<AiOutcome> {
    let mut store = SessionStore::open(inv.store_path.clone())
        .with_context(|| format!("opening AI session store {}", inv.store_path.display()))?;

    let (mut session, resumed) = match inv.session_mode {
        SessionMode::New => (store.create(inv.exec_ctx.shell.clone())?, false),
        SessionMode::ResumeStrict => match store.resume_recent(inv.ai_cfg.session_continue_minutes)
        {
            Some(s) => (s.clone(), true),
            None => {
                return Err(anyhow!(
                    "no recent AI session to resume (TTL {} minutes)",
                    inv.ai_cfg.session_continue_minutes
                ))
            }
        },
        SessionMode::ResumeOrNew => {
            if let Some(s) = store.resume_recent(inv.ai_cfg.session_continue_minutes) {
                (s.clone(), true)
            } else {
                (store.create(inv.exec_ctx.shell.clone())?, false)
            }
        }
    };

    let ctx_str = build_context(
        inv.ai_cfg,
        &inv.exec_ctx,
        Some(&session),
        ContextInputs {
            last_command: if inv.ai_cfg.opening.send_last_command {
                inv.last_command_hint.as_deref()
            } else {
                None
            },
        },
    );

    session.push(Turn::user(inv.prompt));

    let request = CommandRequest::new(inv.prompt, inv.shell)
        .with_safety(inv.safety_level)
        .with_context(ctx_str)
        .with_backend(inv.backend_name.clone());

    let generated = inv
        .backend
        .generate_command(&request)
        .await
        .map_err(|e| anyhow!("backend error: {}", e))?;

    let v = inv
        .validator
        .validate_command(&generated.command, inv.shell)
        .await
        .context("safety validation failed")?;

    let mut assistant = Turn::assistant(&generated.command, &generated.explanation);
    assistant.confidence = Some(generated.confidence_score);
    assistant.risk = Some(format!("{:?}", v.risk_level));
    session.push(assistant);

    store.upsert(&session)?;

    let warns_offhost = super::privacy::may_leak_context_offhost(inv.ai_cfg, &inv.backend_name);

    Ok(AiOutcome {
        session_id: session.id,
        command: generated.command,
        explanation: generated.explanation,
        confidence: generated.confidence_score,
        risk: v.risk_level,
        warnings: v.warnings,
        allowed: v.allowed,
        resumed,
        warns_offhost,
    })
}

/// Convenience: construct a safety validator matching the active safety level.
pub fn build_validator(level: SafetyLevel) -> Arc<SafetyValidator> {
    let cfg = SafetyConfig::from_level(level);
    let v = SafetyValidator::new(cfg).expect("built-in safety config is valid");
    Arc::new(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{BackendInfo, GeneratorError};
    use crate::models::{BackendType, GeneratedCommand};
    use async_trait::async_trait;
    use tempfile::tempdir;

    struct FakeBackend {
        reply: GeneratedCommand,
    }

    #[async_trait]
    impl CommandGenerator for FakeBackend {
        async fn generate_command(
            &self,
            _request: &CommandRequest,
        ) -> Result<GeneratedCommand, GeneratorError> {
            Ok(self.reply.clone())
        }
        async fn is_available(&self) -> bool {
            true
        }
        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Mock,
                model_name: "fake".into(),
                supports_streaming: false,
                max_tokens: 0,
                typical_latency_ms: 0,
                memory_usage_mb: 0,
                version: "0".into(),
            }
        }
        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    fn fake_exec_ctx() -> ExecutionContext {
        ExecutionContext {
            os: "linux".into(),
            arch: "x86_64".into(),
            os_version: "6.5.0".into(),
            distribution: Some("Ubuntu 22.04".into()),
            cwd: std::path::PathBuf::from("/tmp"),
            shell: "bash".into(),
            user: "test".into(),
            available_commands: vec!["ls".into()],
        }
    }

    fn safe_reply(cmd: &str) -> GeneratedCommand {
        GeneratedCommand {
            command: cmd.into(),
            explanation: "generated".into(),
            safety_level: RiskLevel::Safe,
            estimated_impact: "none".into(),
            alternatives: vec![],
            backend_used: "fake".into(),
            generation_time_ms: 1,
            confidence_score: 0.9,
        }
    }

    #[tokio::test]
    async fn once_creates_session_and_persists_turn() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ai.json");

        let backend: Arc<dyn CommandGenerator> = Arc::new(FakeBackend {
            reply: safe_reply("ls -la"),
        });
        let validator = build_validator(SafetyLevel::Moderate);

        let outcome = run_once(AiInvocation {
            prompt: "list files",
            ai_cfg: &AiConfig::default(),
            backend,
            backend_name: "embedded".into(),
            exec_ctx: fake_exec_ctx(),
            validator,
            safety_level: SafetyLevel::Moderate,
            shell: ShellType::Bash,
            store_path: path.clone(),
            session_mode: SessionMode::ResumeOrNew,
            last_command_hint: None,
        })
        .await
        .unwrap();

        assert_eq!(outcome.command, "ls -la");
        assert!(outcome.allowed);
        assert!(!outcome.resumed);

        // The session round-trips through disk.
        let store = SessionStore::open(path).unwrap();
        let sess = store.get(outcome.session_id).unwrap();
        assert_eq!(sess.turns.len(), 2);
        assert_eq!(sess.last_command(), Some("ls -la"));
    }

    #[tokio::test]
    async fn once_flags_dangerous_command_from_validator() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ai.json");
        let backend: Arc<dyn CommandGenerator> = Arc::new(FakeBackend {
            reply: safe_reply("rm -rf /"),
        });
        let validator = build_validator(SafetyLevel::Moderate);

        let outcome = run_once(AiInvocation {
            prompt: "wipe everything",
            ai_cfg: &AiConfig::default(),
            backend,
            backend_name: "embedded".into(),
            exec_ctx: fake_exec_ctx(),
            validator,
            safety_level: SafetyLevel::Moderate,
            shell: ShellType::Bash,
            store_path: path,
            session_mode: SessionMode::New,
            last_command_hint: None,
        })
        .await
        .unwrap();

        // Whatever the exact risk-level mapping, the 52-pattern suite MUST
        // refuse to mark this as unconditionally allowed.
        assert!(!outcome.allowed || matches!(outcome.risk, RiskLevel::High | RiskLevel::Critical));
    }
}
