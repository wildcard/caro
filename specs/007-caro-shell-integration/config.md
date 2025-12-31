# Configuration System Design

## Overview

The shell integration configuration system provides:
- Safety level controls (off/passive/active)
- Per-command rules (allowlist/blocklist)
- Privacy controls
- Keybinding customization
- LLM integration settings

## Configuration File

**Location**: `$XDG_CONFIG_HOME/caro/config.toml` (default: `~/.config/caro/config.toml`)

### Complete Configuration Schema

```toml
# Caro Shell Integration Configuration
# Version: 1.0
# Documentation: https://github.com/wildcard/caro

# =============================================================================
# SHELL INTEGRATION
# =============================================================================
[shell]
# Safety level determines how Caro responds to risky commands
#   "off"     - No safety checks (pure assist mode)
#   "passive" - Warn on high/critical risk commands but allow execution
#   "active"  - Require confirmation for high-risk, block critical-risk
safety_level = "passive"

# Enable fix-it suggestions after failed commands
show_suggestions = true

# Maximum number of suggestions to show
max_suggestions = 3

# Show explanation with suggestions
show_explanation = true

# Keybinding to invoke Caro interactive prompt
# Format: readline notation (bash/zsh) or fish bind notation
# Default: Ctrl+X Ctrl+C
hotkey = "\\C-x\\C-c"

# Keybinding to apply last fix suggestion
# Default: Esc Esc
fixit_hotkey = "\\e\\e"

# Auto-apply highly confident fixes (confidence > threshold)
# WARNING: Use with caution
auto_apply_threshold = 0.0  # 0.0 = disabled, 0.95 = high confidence only

# =============================================================================
# DAEMON SETTINGS
# =============================================================================
[daemon]
# Automatically start daemon when shell initializes
auto_start = true

# Socket path (leave empty for default: $XDG_RUNTIME_DIR/caro-$UID.sock)
socket_path = ""

# IPC timeout in milliseconds
# If daemon doesn't respond within this time, allow command to proceed
timeout_ms = 50

# Maximum concurrent sessions
max_sessions = 100

# Session idle timeout (seconds) - cleanup inactive sessions
session_timeout_secs = 3600

# Log level for daemon (error, warn, info, debug, trace)
log_level = "warn"

# =============================================================================
# SAFETY POLICIES
# =============================================================================
[safety]
# Inherit from base safety level, or override per-risk
# Each risk level can have: "allow", "warn", "confirm", "block"
[safety.levels]
critical = "block"    # rm -rf /, mkfs, fork bombs
high = "confirm"      # rm -rf, sudo su, chmod -R
moderate = "warn"     # curl | sh, git push -f
low = "allow"         # Safe commands

# Maximum command length (bytes) - longer commands require review
max_command_length = 5000

# Block commands matching these patterns (regex)
blocklist = [
    # Example: Block all Docker operations
    # "^docker\\s+rm\\s+-f",
]

# Allow commands matching these patterns, bypassing safety checks (regex)
# Use with caution!
allowlist = [
    # Example: Allow specific maintenance scripts
    # "^/home/user/scripts/safe-cleanup\\.sh$",
]

# Per-directory policies
# Commands in these directories have modified safety levels
[safety.directories]
# Example: Production directories are more strict
# "/srv/production" = "strict"
# Example: Personal sandbox is more permissive
# "/home/user/sandbox" = "permissive"

# Per-command overrides
# Specific commands can have custom policies
[safety.commands]
# Example: Always confirm rsync --delete
# "rsync.*--delete" = "confirm"
# Example: Block all eval usage
# "eval\\s" = "block"

# =============================================================================
# PRIVACY AND LOGGING
# =============================================================================
[privacy]
# What to log to local files
#   "none"     - No logging
#   "errors"   - Only errors and blocked commands
#   "commands" - All commands (for learning/debugging)
#   "all"      - Everything including suggestions
logging_level = "errors"

# Redact sensitive data from logs
# Patterns that look like secrets are replaced with [REDACTED]
redact_secrets = true

# Additional redaction patterns (regex)
redaction_patterns = [
    # Defaults include: API keys, tokens, passwords, credit cards
    # Add custom patterns here:
    # "MY_COMPANY_SECRET_\\w+",
]

# Command history retention (days)
# 0 = disabled, commands not stored
history_retention_days = 30

# Encrypt stored history
# Requires setting up encryption key on first use
encrypt_history = false

# Send anonymous usage statistics (helps improve Caro)
# Never includes command content, only aggregated metrics
telemetry_enabled = false

# =============================================================================
# FIX-IT ENGINE
# =============================================================================
[fixit]
# Enable the fix-it suggestion engine
enabled = true

# Learn from user corrections (when they fix their own typos)
learn_corrections = true

# Source of fix suggestions
# "patterns"  - Built-in pattern database only
# "llm"       - Use LLM for advanced suggestions
# "hybrid"    - Try patterns first, fall back to LLM
suggestion_source = "patterns"

# Confidence threshold for showing suggestions
# Lower = more suggestions, higher = fewer but more accurate
confidence_threshold = 0.5

# Common typo corrections (user-defined)
[fixit.typos]
# command_typo = "correct_command"
gti = "git"
sl = "ls"
pythno = "python"
suod = "sudo"
gerp = "grep"
fiel = "file"

# =============================================================================
# LLM INTEGRATION
# =============================================================================
[llm]
# Enable LLM-powered features (suggestions, explanations)
enabled = false

# Backend to use (mlx, ollama, vllm)
backend = "ollama"

# Model for fix suggestions
suggestion_model = "codellama:7b"

# Model for explanations (can be same or different)
explanation_model = "codellama:7b"

# Maximum tokens for LLM response
max_tokens = 100

# Temperature (0.0 = deterministic, 1.0 = creative)
temperature = 0.3

# Timeout for LLM requests (milliseconds)
# This is async and doesn't block shell
timeout_ms = 5000

# =============================================================================
# UI CUSTOMIZATION
# =============================================================================
[ui]
# Enable colored output
colors = true

# Color scheme (currently only "default" supported)
color_scheme = "default"

# Custom colors (ANSI codes or names)
[ui.colors]
warning = "yellow"
error = "red"
success = "green"
suggestion = "cyan"
prompt = "blue"

# Prompt format for Caro interactive mode
prompt_format = "Caro> "

# Show icons/emoji in output
show_icons = true

# Icons to use (requires font support)
[ui.icons]
warning = "⚠"
error = "✗"
success = "✓"
suggestion = "💡"
blocked = "🚫"

# =============================================================================
# ENVIRONMENT-SPECIFIC OVERRIDES
# =============================================================================
# These settings apply in specific environments

[environments.production]
# More strict in production-like environments
# Detected by: PWD contains "prod", CARO_ENV=production, etc.
safety_level = "active"

[environments.development]
# More relaxed in development
safety_level = "passive"

# =============================================================================
# ADVANCED
# =============================================================================
[advanced]
# Path to custom pattern database
custom_patterns_path = ""

# External command to run before execution (for custom checks)
# Receives command as argument, non-zero exit blocks
pre_exec_hook = ""

# External command to run after execution
post_exec_hook = ""

# Debug mode (verbose logging)
debug = false
```

---

## Policy Toggles Detail

### Safety Levels

```rust
pub enum SafetyLevel {
    /// No safety checks - Caro only assists when asked
    Off,

    /// Warn on high/critical risk but allow execution
    /// Good for experienced users who want awareness
    Passive,

    /// Require confirmation for high-risk, block critical
    /// Recommended for most users
    Active,
}
```

### Risk Levels and Default Actions

| Risk Level | Examples | Off | Passive | Active |
|------------|----------|-----|---------|--------|
| Critical | `rm -rf /`, fork bomb | Allow | Warn | Block |
| High | `rm -rf ~`, `sudo su` | Allow | Warn | Confirm |
| Moderate | `curl \| sh`, `git push -f` | Allow | Warn | Warn |
| Low | Most normal commands | Allow | Allow | Allow |

### Per-Risk Actions

```rust
pub enum PolicyAction {
    /// Allow without any notification
    Allow,

    /// Show warning but allow
    Warn,

    /// Require explicit confirmation
    Confirm,

    /// Block execution entirely
    Block,
}
```

---

## Per-Command Rules

### Blocklist

Commands matching blocklist patterns are always blocked:

```toml
[safety]
blocklist = [
    # Block Docker force remove
    "^docker\\s+rm\\s+-f",

    # Block kubectl delete without --dry-run
    "^kubectl\\s+delete(?!.*--dry-run)",

    # Block production database commands
    "DROP\\s+DATABASE.*production",
]
```

### Allowlist

Commands matching allowlist patterns bypass safety checks:

```toml
[safety]
allowlist = [
    # Allow specific backup script
    "^/opt/scripts/backup\\.sh$",

    # Allow commands in safe directory
    "^cd /home/user/sandbox && ",

    # Allow git operations in personal repos
    "^git\\s+(push|pull)\\s+origin\\s+(main|master)$",
]
```

### Processing Order

1. Check allowlist first (if match → allow)
2. Check blocklist (if match → block)
3. Apply safety level rules
4. Apply per-directory overrides
5. Apply per-command overrides

---

## Directory-Based Policies

```toml
[safety.directories]
# Production paths get strict treatment
"/var/www/production" = "active"
"/srv/app" = "active"

# Home sandbox is permissive
"/home/user/sandbox" = "off"
"/tmp" = "off"

# System paths are always strict
"/usr" = "active"
"/etc" = "active"
"/bin" = "active"
```

Implementation:
```rust
fn get_effective_policy(&self, command: &str, cwd: &Path) -> SafetyLevel {
    // Check directory-specific override
    for (dir, level) in &self.config.safety.directories {
        if cwd.starts_with(dir) {
            return level;
        }
    }

    // Fall back to global setting
    self.config.shell.safety_level
}
```

---

## Environment Variables

All configuration can be overridden via environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `CARO_DISABLE` | Completely disable Caro | `CARO_DISABLE=1` |
| `CARO_SAFETY_LEVEL` | Override safety level | `CARO_SAFETY_LEVEL=off` |
| `CARO_SOCKET` | Custom socket path | `CARO_SOCKET=/tmp/my.sock` |
| `CARO_DEBUG` | Enable debug logging | `CARO_DEBUG=1` |
| `CARO_NO_COLOR` | Disable colored output | `CARO_NO_COLOR=1` |
| `CARO_CONFIG` | Custom config file path | `CARO_CONFIG=/path/to/config.toml` |
| `CARO_ENV` | Environment name | `CARO_ENV=production` |

Environment variables take precedence over config file values.

---

## Configuration API

### Rust Implementation

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default)]
    pub shell: ShellSettings,

    #[serde(default)]
    pub daemon: DaemonSettings,

    #[serde(default)]
    pub safety: SafetySettings,

    #[serde(default)]
    pub privacy: PrivacySettings,

    #[serde(default)]
    pub fixit: FixItSettings,

    #[serde(default)]
    pub llm: LlmSettings,

    #[serde(default)]
    pub ui: UiSettings,

    #[serde(default)]
    pub environments: HashMap<String, EnvironmentOverride>,

    #[serde(default)]
    pub advanced: AdvancedSettings,
}

impl ShellConfig {
    /// Load config from default location
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        Self::load_from(&path)
    }

    /// Load config from specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: ShellConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Apply environment variable overrides
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(level) = std::env::var("CARO_SAFETY_LEVEL") {
            self.shell.safety_level = level.parse().unwrap_or(self.shell.safety_level);
        }

        if let Ok(socket) = std::env::var("CARO_SOCKET") {
            self.daemon.socket_path = Some(socket);
        }

        if std::env::var("CARO_DEBUG").is_ok() {
            self.advanced.debug = true;
        }

        if std::env::var("CARO_NO_COLOR").is_ok() {
            self.ui.colors = false;
        }

        self
    }

    /// Get effective safety level for a specific context
    pub fn effective_safety_level(&self, cwd: &Path) -> SafetyLevel {
        // Check environment-based override
        if let Ok(env) = std::env::var("CARO_ENV") {
            if let Some(override_settings) = self.environments.get(&env) {
                if let Some(level) = override_settings.safety_level {
                    return level;
                }
            }
        }

        // Check directory-based override
        for (dir, level) in &self.safety.directories {
            if cwd.starts_with(dir) {
                return *level;
            }
        }

        // Return global setting
        self.shell.safety_level
    }

    /// Default config file path
    pub fn config_path() -> PathBuf {
        if let Ok(custom) = std::env::var("CARO_CONFIG") {
            return PathBuf::from(custom);
        }

        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join(".config")
            });

        config_home.join("caro").join("config.toml")
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell: ShellSettings::default(),
            daemon: DaemonSettings::default(),
            safety: SafetySettings::default(),
            privacy: PrivacySettings::default(),
            fixit: FixItSettings::default(),
            llm: LlmSettings::default(),
            ui: UiSettings::default(),
            environments: HashMap::new(),
            advanced: AdvancedSettings::default(),
        }
    }
}
```

---

## Configuration Commands

```bash
# Show current configuration
caro config show

# Edit configuration in $EDITOR
caro config edit

# Set a specific value
caro config set shell.safety_level passive

# Get a specific value
caro config get shell.safety_level

# Reset to defaults
caro config reset

# Validate configuration
caro config validate

# Show effective config for current environment
caro config effective
```

---

## Migration Strategy

When configuration format changes between versions:

```rust
pub fn migrate_config(config_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(config_path)?;

    // Detect version
    let version = extract_version(&content).unwrap_or("1.0");

    // Apply migrations
    let migrated = match version {
        "1.0" => migrate_1_0_to_1_1(&content)?,
        "1.1" => content,
        _ => return Err(anyhow!("Unknown config version")),
    };

    // Backup and write
    let backup = config_path.with_extension("toml.bak");
    std::fs::copy(config_path, &backup)?;
    std::fs::write(config_path, migrated)?;

    Ok(())
}
```
