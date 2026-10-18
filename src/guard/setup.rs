//! `caro guard setup`: install, inspect and verify the PreToolUse hook
//! (spec 011 §P2.2).
//!
//! The installer edits the harness's hook file idempotently: other hooks are
//! kept, an existing caro entry is replaced, nothing else changes. `--verify`
//! checks what the installer can see (the entry exists, `caro` resolves, and a
//! synthetic `rm -rf /` payload comes back `deny`) and reports it as
//! "adapter verified". Caro cannot observe the host ingesting the hook, so it
//! never claims "host verified".

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{GuardMode, Harness};

/// Which harness to install the hook into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SetupTarget {
    /// Claude Code: `hooks.PreToolUse` in `settings.json`.
    Claude,
    /// OpenAI Codex CLI: same hook shape in `hooks.json`.
    Codex,
    /// OpenCode: the TypeScript plugin in `~/.config/opencode/plugin/`.
    Opencode,
}

impl SetupTarget {
    fn harness(self) -> Harness {
        match self {
            Self::Claude => Harness::Claude,
            Self::Codex => Harness::Codex,
            Self::Opencode => Harness::Opencode,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Opencode => "opencode",
        }
    }
}

/// Where the hook file lives. User scope is the default (ADR-020): a
/// project-scope hook can be edited by the agent it guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Project,
}

/// Tools the hook receives: the shell tool plus the file tools that the
/// protected-path policy covers (spec 011 §P2.3).
pub const HOOK_MATCHER: &str = "Bash|Read|Write|Edit|MultiEdit|NotebookEdit|Grep|Glob";

/// Marker that identifies caro's own hook entry inside a hooks array.
const CARO_HOOK_PREFIX: &str = "caro guard";

/// The OpenCode plugin, shipped inside the binary so `setup` needs no
/// network and no checkout.
pub const OPENCODE_PLUGIN: &str = include_str!("../../integrations/opencode/caro-guard.ts");

/// The hook file for a target and scope. `None` when the platform has no
/// home directory.
pub fn hook_file(target: SetupTarget, scope: Scope) -> Option<PathBuf> {
    match (target, scope) {
        (SetupTarget::Claude, Scope::User) => {
            dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
        }
        (SetupTarget::Claude, Scope::Project) => {
            Some(PathBuf::from(".claude").join("settings.json"))
        }
        (SetupTarget::Codex, Scope::User) => {
            dirs::home_dir().map(|h| h.join(".codex").join("hooks.json"))
        }
        (SetupTarget::Codex, Scope::Project) => Some(PathBuf::from(".codex").join("hooks.json")),
        (SetupTarget::Opencode, Scope::User) => {
            dirs::config_dir().map(|c| c.join("opencode").join("plugin").join("caro-guard.ts"))
        }
        (SetupTarget::Opencode, Scope::Project) => Some(
            PathBuf::from(".opencode")
                .join("plugin")
                .join("caro-guard.ts"),
        ),
    }
}

/// The shell line the harness runs for each tool call.
pub fn hook_command(target: SetupTarget, mode: GuardMode) -> String {
    let mode = match mode {
        GuardMode::Shadow => "shadow",
        GuardMode::Enforce => "enforce",
    };
    format!(
        "{CARO_HOOK_PREFIX} --harness {} --mode {mode}",
        target.harness_name()
    )
}

impl SetupTarget {
    fn harness_name(self) -> &'static str {
        match self.harness() {
            Harness::Claude => "claude",
            Harness::Codex => "codex",
            Harness::Grok => "grok",
            Harness::Opencode => "opencode",
            Harness::Generic => "generic",
        }
    }
}

/// One PreToolUse entry in the Claude hook format.
pub fn hook_entry(target: SetupTarget, mode: GuardMode) -> Value {
    json!({
        "matcher": HOOK_MATCHER,
        "hooks": [{
            "type": "command",
            "command": hook_command(target, mode),
            "timeout": 10
        }]
    })
}

fn is_caro_entry(entry: &Value) -> bool {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .map(|hooks| {
            hooks.iter().any(|h| {
                h.get("command")
                    .and_then(Value::as_str)
                    .map(|c| c.trim_start().starts_with(CARO_HOOK_PREFIX))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Insert or replace caro's PreToolUse entry in a settings document.
/// Returns `true` when the document changed. Every other key and every other
/// hook entry is left as it was.
pub fn merge_hook(settings: &mut Value, target: SetupTarget, mode: GuardMode) -> bool {
    if !settings.is_object() {
        *settings = json!({});
    }
    let hooks = settings
        .as_object_mut()
        .expect("object")
        .entry("hooks")
        .or_insert_with(|| json!({}));
    if !hooks.is_object() {
        *hooks = json!({});
    }
    let pre = hooks
        .as_object_mut()
        .expect("object")
        .entry("PreToolUse")
        .or_insert_with(|| json!([]));
    if !pre.is_array() {
        *pre = json!([]);
    }
    let arr = pre.as_array_mut().expect("array");
    let wanted = hook_entry(target, mode);
    let before = arr.clone();
    arr.retain(|e| !is_caro_entry(e));
    arr.push(wanted);
    *arr != before
}

/// The caro hook command found in a settings document, if any.
pub fn installed_command(settings: &Value) -> Option<String> {
    settings
        .get("hooks")?
        .get("PreToolUse")?
        .as_array()?
        .iter()
        .filter(|e| is_caro_entry(e))
        .find_map(|e| {
            e.get("hooks")?
                .as_array()?
                .iter()
                .find_map(|h| h.get("command")?.as_str().map(str::to_owned))
        })
}

/// What `setup` did or would do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SetupOutcome {
    pub target: SetupTarget,
    pub scope: Scope,
    pub path: PathBuf,
    /// The hook command written (or the plugin file, for OpenCode).
    pub command: String,
    pub changed: bool,
    pub dry_run: bool,
    /// The full document that was (or would be) written.
    pub rendered: String,
}

fn read_json(path: &Path) -> Result<Value, String> {
    match std::fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(json!({})),
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("{} is not valid JSON: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Install (or preview) the hook.
pub fn install(
    target: SetupTarget,
    scope: Scope,
    mode: GuardMode,
    dry_run: bool,
) -> Result<SetupOutcome, String> {
    let path = hook_file(target, scope).ok_or("no home directory on this platform")?;

    if target == SetupTarget::Opencode {
        let existing = std::fs::read_to_string(&path).ok();
        let changed = existing.as_deref() != Some(OPENCODE_PLUGIN);
        if changed && !dry_run {
            write_atomic(&path, OPENCODE_PLUGIN)?;
        }
        return Ok(SetupOutcome {
            target,
            scope,
            path,
            command:
                "caro guard --harness opencode (via plugin; set CARO_GUARD_MODE=enforce to enforce)"
                    .into(),
            changed,
            dry_run,
            rendered: OPENCODE_PLUGIN.to_string(),
        });
    }

    let mut settings = read_json(&path)?;
    let changed = merge_hook(&mut settings, target, mode);
    let rendered =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("render: {e}"))? + "\n";
    if changed && !dry_run {
        write_atomic(&path, &rendered)?;
    }
    Ok(SetupOutcome {
        target,
        scope,
        path,
        command: hook_command(target, mode),
        changed,
        dry_run,
        rendered,
    })
}

fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension("caro-tmp");
    std::fs::write(&tmp, contents).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// One verification check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// The result of `--verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyReport {
    pub target: SetupTarget,
    pub scope: Scope,
    pub path: PathBuf,
    pub checks: Vec<Check>,
}

impl VerifyReport {
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|c| c.ok)
    }

    /// Human-readable rendering. The assurance label is always "adapter
    /// verified": Caro checked its own side of the boundary, not the host's.
    pub fn format(&self) -> String {
        let mut out = format!(
            "caro guard setup --verify ({}, {} scope): {}\n",
            self.target.as_str(),
            match self.scope {
                Scope::User => "user",
                Scope::Project => "project",
            },
            self.path.display()
        );
        for c in &self.checks {
            out.push_str(&format!(
                "  {} {}: {}\n",
                if c.ok { "ok  " } else { "FAIL" },
                c.name,
                c.detail
            ));
        }
        out.push_str(if self.ok() {
            "status: ADAPTER VERIFIED (the hook entry is installed and caro answers; host ingestion is not observable from here)\n"
        } else {
            "status: NOT VERIFIED\n"
        });
        out
    }
}

/// Which `caro` binary `--verify` should drive: the one on `$PATH`, because
/// that is what the harness will spawn.
fn caro_on_path() -> Result<PathBuf, String> {
    which::which("caro").map_err(|e| format!("`caro` not found on $PATH: {e}"))
}

/// Verify the installed hook from caro's side of the boundary.
pub fn verify(target: SetupTarget, scope: Scope) -> VerifyReport {
    let path = hook_file(target, scope).unwrap_or_else(|| PathBuf::from("<no home dir>"));
    let mut checks = Vec::new();

    // 1. The hook entry (or plugin) is present.
    let installed = if target == SetupTarget::Opencode {
        match std::fs::read_to_string(&path) {
            Ok(text) if text.contains("caro guard") => Ok("plugin file present".to_string()),
            Ok(_) => Err("plugin file exists but is not caro-guard.ts".to_string()),
            Err(e) => Err(format!("plugin file missing: {e}")),
        }
    } else {
        read_json(&path).and_then(|s| {
            installed_command(&s).ok_or_else(|| "no caro entry under hooks.PreToolUse".to_string())
        })
    };
    checks.push(match &installed {
        Ok(d) => Check {
            name: "hook entry",
            ok: true,
            detail: d.clone(),
        },
        Err(e) => Check {
            name: "hook entry",
            ok: false,
            detail: e.clone(),
        },
    });

    // 2. `caro` resolves on $PATH.
    let bin = caro_on_path();
    checks.push(match &bin {
        Ok(p) => Check {
            name: "caro on $PATH",
            ok: true,
            detail: p.display().to_string(),
        },
        Err(e) => Check {
            name: "caro on $PATH",
            ok: false,
            detail: e.clone(),
        },
    });

    // 3. A synthetic catastrophic payload comes back `deny` in enforce mode.
    if let Ok(bin) = bin {
        checks.push(synthetic_check(&bin, target));
    }

    VerifyReport {
        target,
        scope,
        path,
        checks,
    }
}

fn synthetic_check(bin: &Path, target: SetupTarget) -> Check {
    use std::io::Write;
    let payload = match target {
        SetupTarget::Opencode => json!({"command": "rm -rf /"}).to_string(),
        _ => json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "rm -rf /"},
            "session_id": "caro-guard-verify"
        })
        .to_string(),
    };
    let spawned = Command::new(bin)
        .args([
            "guard",
            "--harness",
            target.harness_name(),
            "--mode",
            "enforce",
            "--no-log",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(c) => c,
        Err(e) => {
            return Check {
                name: "synthetic deny",
                ok: false,
                detail: format!("could not run caro guard: {e}"),
            }
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(payload.as_bytes());
    }
    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            return Check {
                name: "synthetic deny",
                ok: false,
                detail: format!("caro guard did not finish: {e}"),
            }
        }
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let denied = match target {
        SetupTarget::Opencode => out.status.code() == Some(2),
        _ => stdout.contains("\"permissionDecision\":\"deny\""),
    };
    Check {
        name: "synthetic deny",
        ok: denied,
        detail: if denied {
            "`rm -rf /` → deny".into()
        } else {
            format!(
                "expected deny for `rm -rf /`, got exit {:?}: {}",
                out.status.code(),
                stdout.trim()
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_adds_entry_and_is_idempotent() {
        let mut s = json!({});
        assert!(merge_hook(&mut s, SetupTarget::Claude, GuardMode::Shadow));
        assert!(!merge_hook(&mut s, SetupTarget::Claude, GuardMode::Shadow));
        let pre = s["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 1);
        assert_eq!(
            installed_command(&s).as_deref(),
            Some("caro guard --harness claude --mode shadow")
        );
    }

    #[test]
    fn merge_keeps_other_hooks_and_keys_and_replaces_caro_entry() {
        let mut s = json!({
            "permissions": {"allow": ["Bash(ls:*)"]},
            "hooks": {
                "PreToolUse": [
                    {"matcher": "Write", "hooks": [{"type": "command", "command": "lint.sh"}]},
                    {"matcher": "Bash", "hooks": [{"type": "command", "command": "caro guard --harness claude --mode shadow"}]}
                ],
                "PostToolUse": [{"matcher": "", "hooks": [{"type": "command", "command": "echo done"}]}]
            }
        });
        assert!(merge_hook(&mut s, SetupTarget::Claude, GuardMode::Enforce));
        assert_eq!(s["permissions"]["allow"][0], "Bash(ls:*)");
        assert_eq!(
            s["hooks"]["PostToolUse"][0]["hooks"][0]["command"],
            "echo done"
        );
        let pre = s["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 2, "one foreign entry + one caro entry");
        assert_eq!(pre[0]["hooks"][0]["command"], "lint.sh");
        assert_eq!(
            installed_command(&s).as_deref(),
            Some("caro guard --harness claude --mode enforce")
        );
    }

    #[test]
    fn merge_repairs_wrong_shapes_without_losing_siblings() {
        let mut s = json!({"hooks": {"PreToolUse": "oops"}, "model": "x"});
        assert!(merge_hook(&mut s, SetupTarget::Codex, GuardMode::Shadow));
        assert_eq!(s["model"], "x");
        assert!(s["hooks"]["PreToolUse"].is_array());
        assert!(installed_command(&s).unwrap().contains("--harness codex"));
    }

    #[test]
    fn hook_entry_never_sets_allow_and_has_a_timeout() {
        let e = hook_entry(SetupTarget::Claude, GuardMode::Enforce).to_string();
        assert!(!e.contains("allow"));
        assert!(e.contains("\"timeout\":10"));
    }

    #[test]
    fn opencode_plugin_is_embedded() {
        assert!(OPENCODE_PLUGIN.contains("caro"));
        assert!(OPENCODE_PLUGIN.contains("tool.execute.before"));
    }
}
