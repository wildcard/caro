//! Privacy-aware context builder for the AI feature.
//!
//! The single gate where opt-in toggles from [`crate::models::AiConfig`] decide
//! what local machine state is allowed to leave the process. Everything in a
//! `[ai.opening]` / `[ai.capabilities]` block defaults to `false`, so the LLM
//! sees only the OS + shell name unless the user explicitly opts in.

use crate::context::ExecutionContext;
use crate::models::AiConfig;

use super::session::AiSession;

/// Pieces of context the LLM may be allowed to see, depending on user config.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextInputs<'a> {
    /// Optional last-command captured by the shell hook (opt-in).
    pub last_command: Option<&'a str>,
}

/// Build the opening-turn context string, honoring every `[ai]` privacy toggle.
///
/// The output is safe to feed into the existing prompt pipeline as
/// [`crate::models::CommandRequest::context`].
pub fn build_context(
    ai_cfg: &AiConfig,
    exec_ctx: &ExecutionContext,
    session: Option<&AiSession>,
    inputs: ContextInputs<'_>,
) -> String {
    // OS + shell are always-on — they're the bare minimum any shell-command
    // generator needs to avoid emitting wrong-platform flags.
    let mut out = String::new();
    out.push_str(&format!("OS: {}\n", exec_ctx.os));
    out.push_str(&format!("Shell: {}\n", exec_ctx.shell));

    if ai_cfg.opening.send_cwd {
        out.push_str(&format!("CWD: {}\n", exec_ctx.cwd.display()));
    }

    if ai_cfg.opening.send_last_command {
        let last = inputs
            .last_command
            .or_else(|| session.and_then(|s| s.last_command()));
        if let Some(cmd) = last {
            out.push_str(&format!("LastCommand: {}\n", cmd));
        }
    }

    if let Some(sess) = session {
        let hist = sess.render_history(6);
        if !hist.is_empty() {
            out.push_str(&hist);
        }
    }

    out
}

/// When this returns `true`, the caller should warn the user that remote
/// transmission may occur with the selected backend and the configured opt-ins.
pub fn may_leak_context_offhost(ai_cfg: &AiConfig, backend_name: &str) -> bool {
    let anything_optin = ai_cfg.opening.send_cwd
        || ai_cfg.opening.send_last_command
        || ai_cfg.capabilities.enable_history_search;
    let remote = matches!(backend_name, "ollama" | "vllm" | "exo" | "claude");
    anything_optin && remote && ai_cfg.endpoint.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AiCapabilities, AiOpening};

    fn ctx() -> ExecutionContext {
        ExecutionContext {
            os: "linux".into(),
            arch: "x86_64".into(),
            os_version: "6.5.0".into(),
            distribution: Some("Ubuntu 22.04".into()),
            cwd: std::path::PathBuf::from("/home/alice/proj"),
            shell: "bash".into(),
            user: "alice".into(),
            available_commands: vec!["ls".into(), "grep".into()],
        }
    }

    #[test]
    fn defaults_emit_only_os_and_shell() {
        let cfg = AiConfig::default();
        let out = build_context(&cfg, &ctx(), None, ContextInputs::default());
        assert!(out.contains("OS: linux"));
        assert!(out.contains("Shell: bash"));
        assert!(!out.contains("CWD"));
        assert!(!out.contains("LastCommand"));
    }

    #[test]
    fn send_cwd_opt_in_includes_cwd() {
        let cfg = AiConfig {
            opening: AiOpening {
                send_cwd: true,
                send_last_command: false,
            },
            ..AiConfig::default()
        };
        let out = build_context(&cfg, &ctx(), None, ContextInputs::default());
        assert!(out.contains("CWD: /home/alice/proj"));
    }

    #[test]
    fn send_last_command_opt_in_uses_input() {
        let cfg = AiConfig {
            opening: AiOpening {
                send_cwd: false,
                send_last_command: true,
            },
            ..AiConfig::default()
        };
        let out = build_context(
            &cfg,
            &ctx(),
            None,
            ContextInputs {
                last_command: Some("git status"),
            },
        );
        assert!(out.contains("LastCommand: git status"));
    }

    #[test]
    fn send_last_command_falls_back_to_session_last() {
        use crate::ai::session::{AiSession, Turn};
        let mut s = AiSession::new(1, "bash");
        s.push(Turn::user("list"));
        s.push(Turn::assistant("ls", ""));
        let cfg = AiConfig {
            opening: AiOpening {
                send_cwd: false,
                send_last_command: true,
            },
            ..AiConfig::default()
        };
        let out = build_context(&cfg, &ctx(), Some(&s), ContextInputs::default());
        assert!(out.contains("LastCommand: ls"));
    }

    #[test]
    fn leak_detection_only_fires_with_optin_and_remote() {
        let base = AiConfig::default();
        assert!(!may_leak_context_offhost(&base, "ollama"));

        let cfg_remote_no_optin = AiConfig {
            endpoint: Some("https://x".into()),
            ..AiConfig::default()
        };
        assert!(!may_leak_context_offhost(&cfg_remote_no_optin, "ollama"));

        let cfg_optin_local = AiConfig {
            opening: AiOpening {
                send_cwd: true,
                ..AiOpening::default()
            },
            ..AiConfig::default()
        };
        assert!(!may_leak_context_offhost(&cfg_optin_local, "embedded"));

        let cfg_optin_remote = AiConfig {
            endpoint: Some("https://x".into()),
            opening: AiOpening {
                send_cwd: true,
                ..AiOpening::default()
            },
            capabilities: AiCapabilities {
                enable_history_search: false,
            },
            ..AiConfig::default()
        };
        assert!(may_leak_context_offhost(&cfg_optin_remote, "ollama"));
    }
}
