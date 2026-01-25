# Research: Self-Healing CARO

**Feature**: 005-self-healing-caro
**Date**: 2024-03-20
**Status**: Complete

## Research Questions

This document resolves the NEEDS CLARIFICATION items from the implementation plan.

---

## 1. PII Detection & Redaction

### Question
How should we detect and redact personally identifiable information from diagnostic reports?

### Research

**Options Evaluated**:

| Approach | Pros | Cons |
|----------|------|------|
| `scrub` crate | Pre-built patterns, maintained | Heavy dependency, may over-redact |
| `redact` crate | Simple API | Limited patterns, unmaintained |
| Custom regex | Full control, minimal deps | Maintenance burden |
| Hybrid | Best of both | Slightly more code |

**Patterns Needed**:
- Email: `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`
- File paths with usernames: `/Users/[^/]+/`, `/home/[^/]+/`
- IP addresses: `\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}`
- API keys: `(sk-|pk-|api[_-]?key)[a-zA-Z0-9]{20,}`
- Environment variable values: `[A-Z_]+=(.*)`

### Decision
**Custom regex with curated patterns**

**Rationale**:
- Minimal dependencies (only `regex` which we already use)
- Full control over what gets redacted
- Can be tested exhaustively
- Patterns are domain-specific to CLI tool diagnostics

**Implementation**:
```rust
pub struct Sanitizer {
    patterns: Vec<(Regex, &'static str)>,  // (pattern, replacement)
}

impl Sanitizer {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (EMAIL_REGEX, "[EMAIL]"),
                (HOME_PATH_REGEX, "[HOME]/"),
                (IP_REGEX, "[IP]"),
                (API_KEY_REGEX, "[API_KEY]"),
            ],
        }
    }

    pub fn sanitize(&self, input: &str) -> String {
        self.patterns.iter().fold(input.to_string(), |acc, (re, repl)| {
            re.replace_all(&acc, *repl).to_string()
        })
    }
}
```

---

## 2. HTTP Client Patterns

### Question
How should we handle HTTP submission with retry and offline support?

### Research

**Options Evaluated**:

| Approach | Pros | Cons |
|----------|------|------|
| `reqwest` blocking | Simple, synchronous | Blocks CLI |
| `reqwest` async | Non-blocking | Requires tokio runtime |
| `ureq` | Minimal, blocking | Less features |
| `reqwest` + queue | Best UX | More complexity |

**Retry Strategy Research**:
- Exponential backoff: 1s, 2s, 4s, 8s, 16s (max 5 retries)
- Jitter: Add random 0-500ms to prevent thundering herd
- Timeout: 10s per request, 60s total

**Offline Queue Format**:
```
~/.config/cmdai/healing/queue/
├── 2024-03-20T10-30-00-uuid1.json
├── 2024-03-20T11-45-00-uuid2.json
└── ...
```

### Decision
**`reqwest` async with file-based offline queue**

**Rationale**:
- CARO already uses tokio for backends
- Async allows UI responsiveness during submission
- File queue is simple, debuggable, survives crashes
- JSON format for easy inspection

**Implementation**:
```rust
pub async fn submit_with_retry(report: &DiagnosticReport) -> Result<CaseId, HealingError> {
    let mut delay = Duration::from_secs(1);

    for attempt in 1..=5 {
        match submit_once(report).await {
            Ok(case_id) => return Ok(case_id),
            Err(e) if e.is_retryable() => {
                if attempt < 5 {
                    let jitter = rand::thread_rng().gen_range(0..500);
                    tokio::time::sleep(delay + Duration::from_millis(jitter)).await;
                    delay *= 2;
                }
            }
            Err(e) => return Err(e),
        }
    }

    // Queue for later
    queue_report(report).await?;
    Err(HealingError::Queued)
}
```

---

## 3. Cross-Platform Diagnostics

### Question
How do we collect system information across macOS, Linux, and Windows?

### Research

**Platform-Specific Commands**:

| Info | macOS | Linux | Windows |
|------|-------|-------|---------|
| OS Version | `sw_vers` | `/etc/os-release` | `ver` |
| Architecture | `uname -m` | `uname -m` | `wmic os get osarchitecture` |
| Shell | `$SHELL` | `$SHELL` | `$PSVersionTable` / `%COMSPEC%` |
| Shell Version | `$SHELL --version` | `$SHELL --version` | PowerShell version table |

**Crates Evaluated**:

| Crate | Pros | Cons |
|-------|------|------|
| `sysinfo` | Comprehensive | Heavy (adds ~1MB) |
| `os_info` | Lightweight | Limited info |
| `whoami` | User/hostname | Very limited |
| Custom | Full control | Platform-specific code |

### Decision
**`os_info` crate + custom shell detection**

**Rationale**:
- `os_info` is lightweight (~50KB) and covers OS/version/arch
- Shell detection already exists in CARO's `platform/` module
- Reuse existing `ShellType` detection code
- Avoid heavy dependencies

**Implementation**:
```rust
pub struct PlatformInfo {
    pub os_type: String,      // "macOS", "Linux", "Windows"
    pub os_version: String,   // "14.2", "22.04", "11"
    pub arch: String,         // "arm64", "x86_64"
}

pub struct ShellInfo {
    pub shell_type: ShellType,
    pub version: Option<String>,
    pub path: String,
}

impl PlatformInfo {
    pub fn collect() -> Self {
        let info = os_info::get();
        Self {
            os_type: info.os_type().to_string(),
            os_version: info.version().to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}
```

---

## 4. Notification Storage

### Question
How should we store and display notifications to users on subsequent CLI runs?

### Research

**Storage Options**:

| Format | Pros | Cons |
|--------|------|------|
| JSON files | Human-readable, simple | Multiple files to manage |
| SQLite | Query support, ACID | Heavy dependency |
| Single JSON | Atomic updates | Could grow large |
| TOML | Config-like | Limited for lists |

**Notification Lifecycle**:
1. Created when case resolved
2. Displayed on next CLI run
3. Marked as shown
4. Expired after 30 days or user dismissal

### Decision
**Single JSON file with array of notifications**

**Rationale**:
- Simple to implement and debug
- Atomic writes prevent corruption
- Array operations are straightforward
- Can limit to last 10 notifications

**Implementation**:
```rust
// ~/.config/cmdai/healing/notifications.json
#[derive(Serialize, Deserialize)]
pub struct NotificationStore {
    pub notifications: Vec<Notification>,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub case_id: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub shown: bool,
    pub dismissed: bool,
}

impl NotificationStore {
    pub fn pending(&self) -> Vec<&Notification> {
        self.notifications.iter()
            .filter(|n| !n.shown && !n.dismissed)
            .collect()
    }
}
```

---

## 5. Consent UX Patterns

### Question
What's the best UX pattern for obtaining user consent for diagnostic sharing?

### Research

**CLI Consent Patterns**:

| Pattern | Example | Pros | Cons |
|---------|---------|------|------|
| Y/N prompt | `[Y/n]` | Quick, familiar | No preview |
| Multi-choice | `[Y/N/D]` | Shows options | More keystrokes |
| Preview first | Show data, then ask | Transparent | More output |
| Config default | Set in config, skip prompt | Fast for repeat | Less visibility |

**Privacy Best Practices**:
- Always allow preview of what will be shared
- Never share without explicit consent
- Remember preference but allow override
- Clear language about data usage

### Decision
**Multi-choice with preview option, rememberable preference**

**Rationale**:
- Respects user time (quick Y/N for repeat users)
- Provides transparency (D for details)
- Supports both one-time and persistent preferences
- Follows privacy-by-design principles

**Implementation**:
```rust
pub enum ConsentChoice {
    Yes,
    No,
    Details,
    Remember(bool),  // Remember yes/no for future
}

pub fn prompt_consent(report: &DiagnosticReport) -> ConsentChoice {
    // Check for saved preference
    if let Some(pref) = load_consent_preference() {
        return if pref { ConsentChoice::Yes } else { ConsentChoice::No };
    }

    println!("Would you like to help improve CARO by sharing this diagnostic?");
    println!("[Y] Yes  [N] No  [D] Show details  [A] Always  [V] Never");

    match read_key() {
        'y' | 'Y' => ConsentChoice::Yes,
        'n' | 'N' => ConsentChoice::No,
        'd' | 'D' => ConsentChoice::Details,
        'a' | 'A' => ConsentChoice::Remember(true),
        'v' | 'V' => ConsentChoice::Remember(false),
        _ => ConsentChoice::No,
    }
}
```

---

## Summary of Decisions

| Topic | Decision | Key Dependency |
|-------|----------|----------------|
| PII Redaction | Custom regex patterns | `regex` (existing) |
| HTTP Submission | `reqwest` async + file queue | `reqwest` (existing) |
| Platform Info | `os_info` + existing shell detection | `os_info` (new, lightweight) |
| Notification Storage | Single JSON file | `serde_json` (existing) |
| Consent UX | Multi-choice with preview, rememberable | `dialoguer` (existing) |

## New Dependencies

```toml
[dependencies]
os_info = "3"  # ~50KB, OS/version detection
```

## Alternatives Rejected

1. **Heavy PII crates**: Over-redact, add unnecessary dependencies
2. **SQLite for notifications**: Overkill for simple list storage
3. **Synchronous HTTP**: Blocks CLI during submission
4. **`sysinfo` crate**: Too heavy for our needs (~1MB)

---

*Research complete. Ready for Phase 1 design.*
