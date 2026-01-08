//! Caro WASM - Browser-based command validation and generation
//!
//! This crate provides a WebAssembly interface to Caro's safety validation
//! and static command matching capabilities.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// ============================================================================
// DATA MODELS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub command: String,
    pub is_safe: bool,
    pub risk_level: RiskLevel,
    pub explanation: String,
    pub matched_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCommand {
    pub command: String,
    pub description: String,
    pub is_safe: bool,
    pub risk_level: RiskLevel,
    pub confidence: f32,
}

// ============================================================================
// DANGEROUS PATTERNS (52+ patterns from main crate)
// ============================================================================

struct DangerPattern {
    pattern: &'static str,
    risk_level: RiskLevel,
    description: &'static str,
}

static DANGER_PATTERNS: &[DangerPattern] = &[
    // CRITICAL: Filesystem destruction
    DangerPattern {
        pattern: r"rm\s+(-[rfRF]*\s+)*(/|~|\$HOME|/\*|~/\*|\*|\.|\.\/|\.\/\*|\.\*)",
        risk_level: RiskLevel::Critical,
        description: "Recursive deletion of root, home, or current directory",
    },
    DangerPattern {
        pattern: r"rm\s+-rf\s+/",
        risk_level: RiskLevel::Critical,
        description: "Force recursive deletion from root",
    },
    DangerPattern {
        pattern: r"rm\s+-rf\s+--no-preserve-root\s+/",
        risk_level: RiskLevel::Critical,
        description: "Bypass root protection and delete everything",
    },
    // CRITICAL: Disk operations
    DangerPattern {
        pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)",
        risk_level: RiskLevel::Critical,
        description: "Overwrite disk with random data",
    },
    DangerPattern {
        pattern: r"mkfs\.\w+\s+/dev/(sd|hd|nvme)",
        risk_level: RiskLevel::Critical,
        description: "Format disk destroying all data",
    },
    // CRITICAL: Fork bombs
    DangerPattern {
        pattern: r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:",
        risk_level: RiskLevel::Critical,
        description: "Fork bomb - exponential process creation",
    },
    // HIGH: System directory operations
    DangerPattern {
        pattern: r"(rm|mv|chmod|chown)\s+.*(/bin|/sbin|/usr/bin|/usr/sbin|/etc)",
        risk_level: RiskLevel::High,
        description: "Modification of critical system directories",
    },
    DangerPattern {
        pattern: r"chmod\s+777\s+/",
        risk_level: RiskLevel::High,
        description: "Recursive permission change from root",
    },
    // HIGH: Privilege escalation
    DangerPattern {
        pattern: r"sudo\s+su\s*$",
        risk_level: RiskLevel::High,
        description: "Switch to root user without specific command",
    },
    DangerPattern {
        pattern: r"sudo\s+.*chmod\s+u\+s",
        risk_level: RiskLevel::High,
        description: "Adding setuid bit with elevated privileges",
    },
    // HIGH: Download and execute
    DangerPattern {
        pattern: r"(curl|wget)\s+.*\|\s*(bash|sh|zsh|fish)",
        risk_level: RiskLevel::High,
        description: "Download and execute remote script without inspection",
    },
    DangerPattern {
        pattern: r"(curl|wget)\s+.*\|\s*sudo\s+(bash|sh)",
        risk_level: RiskLevel::Critical,
        description: "Download and execute remote script with root privileges",
    },
    // HIGH: Network backdoors
    DangerPattern {
        pattern: r"nc\s+.*-[a-z]*l[a-z]*\s+.*-[a-z]*e",
        risk_level: RiskLevel::Critical,
        description: "Netcat bind shell - creates network backdoor",
    },
    DangerPattern {
        pattern: r"nc\s+-[a-z]*e\s+/bin/(ba)?sh",
        risk_level: RiskLevel::Critical,
        description: "Netcat shell binding",
    },
    // CRITICAL: Direct disk writes
    DangerPattern {
        pattern: r">\s*/dev/sd[a-z]",
        risk_level: RiskLevel::Critical,
        description: "Direct write to disk device",
    },
    DangerPattern {
        pattern: r"shred\s+-[uvz]*\s+/dev/(sd|hd|nvme)",
        risk_level: RiskLevel::Critical,
        description: "Securely delete disk device",
    },
    // MODERATE: Package management
    DangerPattern {
        pattern: r"(apt|yum|dnf)\s+remove\s+.*--force",
        risk_level: RiskLevel::Moderate,
        description: "Force removal of packages bypassing dependencies",
    },
    DangerPattern {
        pattern: r"pip\s+install\s+.*--break-system-packages",
        risk_level: RiskLevel::Moderate,
        description: "Install Python packages bypassing system protections",
    },
    // MODERATE: Process manipulation
    DangerPattern {
        pattern: r"kill\s+-9\s+(-1|1)\s*$",
        risk_level: RiskLevel::Moderate,
        description: "Force kill all processes or init",
    },
    DangerPattern {
        pattern: r"killall\s+-9\s+\w+",
        risk_level: RiskLevel::Moderate,
        description: "Force kill all processes by name",
    },
    // MODERATE: Network operations
    DangerPattern {
        pattern: r"iptables\s+-F",
        risk_level: RiskLevel::Moderate,
        description: "Flush all firewall rules",
    },
    DangerPattern {
        pattern: r"ufw\s+disable",
        risk_level: RiskLevel::Moderate,
        description: "Disable firewall",
    },
    // HIGH: Sudo with system modifications
    DangerPattern {
        pattern: r"sudo\s+(systemctl|service)\s+(restart|stop|disable)",
        risk_level: RiskLevel::High,
        description: "Modify system services with elevated privileges",
    },
    DangerPattern {
        pattern: r"sudo\s+rm\s",
        risk_level: RiskLevel::High,
        description: "Delete files with elevated privileges",
    },
    // HIGH: System file modification
    DangerPattern {
        pattern: r">\s*/etc/",
        risk_level: RiskLevel::High,
        description: "Redirect output to system configuration file",
    },
    DangerPattern {
        pattern: r"(echo|cat|printf)\s+.*>\s*/etc/",
        risk_level: RiskLevel::High,
        description: "Write to system configuration directory",
    },
    // CRITICAL: Windows operations
    DangerPattern {
        pattern: r"rm\s+-r[f]*\s+[A-Z]:[/\\]",
        risk_level: RiskLevel::Critical,
        description: "Recursive deletion of Windows drive root",
    },
    DangerPattern {
        pattern: r"Remove-Item\s+-Recurse\s+-Force\s+[A-Z]:\\",
        risk_level: RiskLevel::Critical,
        description: "Recursive deletion of Windows drive root (PowerShell)",
    },
    DangerPattern {
        pattern: r"del\s+/[fFsS]\s+",
        risk_level: RiskLevel::Critical,
        description: "Windows delete with force/subdirectory flags",
    },
    DangerPattern {
        pattern: r"format\s+[A-Z]:",
        risk_level: RiskLevel::Critical,
        description: "Format disk drive",
    },
    // HIGH: Cron job manipulation
    DangerPattern {
        pattern: r"crontab\s+-r",
        risk_level: RiskLevel::High,
        description: "Remove all cron jobs",
    },
    // HIGH: Docker privileged mode
    DangerPattern {
        pattern: r"docker\s+run\s+.*--privileged",
        risk_level: RiskLevel::High,
        description: "Docker container with full host access",
    },
    // MODERATE: Permissions
    DangerPattern {
        pattern: r"chmod\s+[0-7]{3,4}\s+",
        risk_level: RiskLevel::Moderate,
        description: "Changing file permissions",
    },
    DangerPattern {
        pattern: r"chown\s+[^\s]+\s+",
        risk_level: RiskLevel::Moderate,
        description: "Changing file ownership",
    },
];

// Compiled patterns for performance
static COMPILED_PATTERNS: Lazy<Vec<(Regex, RiskLevel, &'static str)>> = Lazy::new(|| {
    DANGER_PATTERNS
        .iter()
        .filter_map(|p| {
            Regex::new(p.pattern)
                .ok()
                .map(|r| (r, p.risk_level, p.description))
        })
        .collect()
});

// ============================================================================
// STATIC COMMAND PATTERNS
// ============================================================================

struct CommandPattern {
    keywords: &'static [&'static str],
    command: &'static str,
    description: &'static str,
}

static COMMAND_PATTERNS: &[CommandPattern] = &[
    // File searches
    CommandPattern {
        keywords: &["find", "python", "files"],
        command: r#"find . -name "*.py" -type f"#,
        description: "Find all Python files in current directory",
    },
    CommandPattern {
        keywords: &["find", "python", "modified", "today"],
        command: r#"find . -name "*.py" -type f -mtime 0"#,
        description: "Find Python files modified today",
    },
    CommandPattern {
        keywords: &["find", "large", "files", "100"],
        command: "find . -type f -size +100M",
        description: "Find files larger than 100MB",
    },
    CommandPattern {
        keywords: &["find", "large", "files", "1gb"],
        command: "find . -type f -size +1G",
        description: "Find files larger than 1GB",
    },
    CommandPattern {
        keywords: &["find", "files", "50mb"],
        command: "find . -type f -size +50M",
        description: "Find files larger than 50MB",
    },
    CommandPattern {
        keywords: &["list", "large", "files"],
        command: "find . -type f -size +100M -exec ls -lh {} \\;",
        description: "List large files with details",
    },
    // Disk usage
    CommandPattern {
        keywords: &["disk", "usage"],
        command: "df -h",
        description: "Show disk usage summary",
    },
    CommandPattern {
        keywords: &["disk", "usage", "directory", "sorted"],
        command: "du -h --max-depth=1 | sort -hr",
        description: "Show disk usage by directory, sorted",
    },
    CommandPattern {
        keywords: &["show", "disk", "usage"],
        command: "du -sh */ | sort -rh | head -10",
        description: "Show disk usage by folder",
    },
    // Memory and processes
    CommandPattern {
        keywords: &["memory", "usage"],
        command: "free -h",
        description: "Show memory usage",
    },
    CommandPattern {
        keywords: &["running", "processes"],
        command: "ps aux --sort=-%cpu | head -10",
        description: "Show top processes by CPU",
    },
    CommandPattern {
        keywords: &["process", "cpu"],
        command: "ps aux --sort=-%cpu | head -10",
        description: "Show processes sorted by CPU usage",
    },
    // Network
    CommandPattern {
        keywords: &["network", "connections"],
        command: "netstat -tuln",
        description: "Show network connections",
    },
    CommandPattern {
        keywords: &["listening", "ports"],
        command: "netstat -tuln | grep LISTEN",
        description: "Show listening ports",
    },
    // File content search
    CommandPattern {
        keywords: &["find", "text", "files"],
        command: r#"grep -r "pattern" . --include="*.txt""#,
        description: "Search for text in files",
    },
    CommandPattern {
        keywords: &["search", "content"],
        command: "grep -rn \"pattern\" .",
        description: "Search file contents recursively",
    },
    // Archive operations
    CommandPattern {
        keywords: &["compress", "folder"],
        command: "tar -czvf archive.tar.gz folder/",
        description: "Compress a folder",
    },
    CommandPattern {
        keywords: &["extract", "tar"],
        command: "tar -xzvf archive.tar.gz",
        description: "Extract tar.gz archive",
    },
    // Time-based file searches
    CommandPattern {
        keywords: &["modified", "today"],
        command: "find . -type f -mtime 0",
        description: "Find files modified today",
    },
    CommandPattern {
        keywords: &["modified", "last", "hour"],
        command: "find . -type f -mmin -60",
        description: "Find files modified in the last hour",
    },
    CommandPattern {
        keywords: &["modified", "week"],
        command: "find . -type f -mtime -7",
        description: "Find files modified in the last week",
    },
    // System info
    CommandPattern {
        keywords: &["system", "info"],
        command: "uname -a",
        description: "Show system information",
    },
    CommandPattern {
        keywords: &["kernel", "version"],
        command: "uname -r",
        description: "Show kernel version",
    },
];

// ============================================================================
// VALIDATION LOGIC
// ============================================================================

fn validate_command_internal(command: &str) -> ValidationResult {
    let mut matched_patterns = Vec::new();
    let mut highest_risk = RiskLevel::Safe;

    for (regex, risk_level, description) in COMPILED_PATTERNS.iter() {
        if regex.is_match(command) {
            matched_patterns.push(description.to_string());
            if *risk_level > highest_risk {
                highest_risk = *risk_level;
            }
        }
    }

    let is_safe = highest_risk == RiskLevel::Safe;
    let explanation = if is_safe {
        "Command appears safe to execute".to_string()
    } else {
        format!(
            "Command blocked: {}",
            matched_patterns.first().unwrap_or(&"Unknown risk".to_string())
        )
    };

    ValidationResult {
        command: command.to_string(),
        is_safe,
        risk_level: highest_risk,
        explanation,
        matched_patterns,
    }
}

fn generate_command_internal(prompt: &str) -> Option<GeneratedCommand> {
    let prompt_lower = prompt.to_lowercase();
    let words: Vec<&str> = prompt_lower.split_whitespace().collect();

    let mut best_match: Option<(&CommandPattern, usize)> = None;

    for pattern in COMMAND_PATTERNS.iter() {
        let matches = pattern
            .keywords
            .iter()
            .filter(|kw| words.iter().any(|w| w.contains(*kw)))
            .count();

        if matches >= 2 {
            if let Some((_, best_count)) = best_match {
                if matches > best_count {
                    best_match = Some((pattern, matches));
                }
            } else {
                best_match = Some((pattern, matches));
            }
        }
    }

    best_match.map(|(pattern, match_count)| {
        let confidence = (match_count as f32 / pattern.keywords.len() as f32).min(1.0);

        // Validate the generated command
        let validation = validate_command_internal(pattern.command);

        GeneratedCommand {
            command: pattern.command.to_string(),
            description: pattern.description.to_string(),
            is_safe: validation.is_safe,
            risk_level: validation.risk_level,
            confidence,
        }
    })
}

// ============================================================================
// WASM EXPORTS
// ============================================================================

#[wasm_bindgen]
pub fn validate_command(command: &str) -> JsValue {
    let result = validate_command_internal(command);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn generate_command(prompt: &str) -> JsValue {
    match generate_command_internal(prompt) {
        Some(result) => serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL),
        None => JsValue::NULL,
    }
}

#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen(start)]
pub fn init() {
    // Force lazy initialization
    let _ = COMPILED_PATTERNS.len();
}
