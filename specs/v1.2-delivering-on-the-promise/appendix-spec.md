# v1.2.0 Appendix: Selected Task Specifications

Detailed implementation specs for the 8 most complex tasks in v1.2.0. Simpler tasks (config keys, CLI flags) follow standard patterns and don't need dedicated specs.

---

## A1: Safer Alternatives When Blocking (T1-01)

### Problem
When safety validation blocks a command (e.g., `rm -rf /`), the user gets an error but no guidance on what to do instead. The `alternatives` field in `GeneratedCommand` is hardcoded to `vec![]` in every backend.

### Design

#### Data Structure
The `alternatives` field already exists in `GeneratedCommand` (type: `Vec<String>`). We need to populate it.

#### Generation Logic

**Option A: Pattern-mapped alternatives** (recommended for v1.2)

Add a static mapping in `src/safety/validator.rs` that maps dangerous patterns to safer alternatives:

```rust
// src/safety/alternatives.rs (new file)
pub struct SafetyAlternative {
    pub pattern_name: &'static str,
    pub safer_command: &'static str,
    pub explanation: &'static str,
}

pub const SAFETY_ALTERNATIVES: &[SafetyAlternative] = &[
    SafetyAlternative {
        pattern_name: "full_system_deletion",
        safer_command: "ls -la ./specific-directory",
        explanation: "Preview files before deleting. Use 'rm -rf ./specific-directory' for targeted deletion.",
    },
    SafetyAlternative {
        pattern_name: "home_directory_deletion",
        safer_command: "find ~/projects -name '*.tmp' -type f",
        explanation: "Target specific file types in subdirectories instead of deleting everything.",
    },
    // ... more patterns
];
```

**Option B: LLM-generated alternatives** (deferred to v1.3)

Use the inference backend to generate a safer alternative on-the-fly. This adds latency and complexity. Not suitable for v1.2.

#### Integration Points

1. **`src/safety/validator.rs`**: When `validate()` returns a `Critical` or `High` severity result, also look up and return a `SafetyAlternative`.

2. **`src/main.rs`**: After safety validation fails, print the alternative:
```
🔴 CRITICAL: This command would destroy your filesystem.
   Command blocked for safety.

   Try this instead:
     find ~/projects -name '*.tmp' -type f -mtime +30 -ls

   Preview what would be deleted first, then use -delete if it looks correct.
```

3. **JSON output**: Populate `alternatives` field:
```json
{
  "command": "rm -rf /",
  "safety_level": "critical",
  "blocked": true,
  "alternatives": ["find ~/projects -name '*.tmp' -type f -mtime +30 -ls"]
}
```

#### Acceptance Tests
```
#[test]
fn test_safer_alternative_for_full_deletion() {
    let result = validate("rm -rf /");
    assert!(result.is_blocked);
    assert!(!result.alternatives.is_empty());
    assert!(result.alternatives[0].contains("find") || result.alternatives[0].contains("./"));
}

#[test]
fn test_safer_alternative_for_fork_bomb() {
    let result = validate(":(){ :|:& };:");
    assert!(result.is_blocked);
    assert!(!result.alternatives.is_empty());
}
```

#### Estimated Effort
- Create `src/safety/alternatives.rs` with 15-20 pattern mappings: **2 hours**
- Integrate into `validator.rs`: **2 hours**
- Wire into `main.rs` output: **1 hour**
- Wire into JSON serialization: **1 hour**
- Tests: **2 hours**
- **Total: 1-2 days**

---

## A2: Configuration Key Expansion (T0-09, T0-10, T0-11)

### Problem
Only 4 config keys are accepted (`backend`, `model-name`, `shell`, `safety`). The website and skill docs reference 14+ keys that all produce "Unknown config key" errors.

### Current Code

```rust
// src/config/mod.rs (approximate)
match key {
    "backend" => { /* set backend */ }
    "model-name" => { /* set model name */ }
    "shell" => { /* set shell */ }
    "safety" => { /* set safety level */ }
    _ => Err(format!("Unknown config key '{}'. Valid keys: backend, model-name, shell, safety", key))
}
```

### Design

#### Option A: Flat key namespace (simplest, recommended for v1.2)

Extend the existing flat key match to support all documented keys:

```rust
match key {
    // Existing
    "backend" => { user_config.backend = value; }
    "model-name" => { user_config.model_name = value; }
    "shell" => { user_config.shell = value; }
    "safety" => { user_config.safety = value; }

    // New: Telemetry
    "telemetry.enabled" => { user_config.telemetry.enabled = parse_bool(&value)?; }
    "telemetry.level" => { user_config.telemetry.level = value; }
    "telemetry.air_gapped" => { user_config.telemetry.air_gapped = parse_bool(&value)?; }

    // New: Safety
    "safety.level" => { user_config.safety.level = value; }
    "safety.require_confirmation" => { user_config.safety.require_confirmation = parse_bool(&value)?; }

    // New: Output
    "output.format" => { user_config.output.format = value; }
    "output.color" => { user_config.output.color = parse_bool(&value)?; }

    // New: Backend
    "backend.primary" => { user_config.backend.primary = value; }
    "backend.enable_fallback" => { user_config.backend.enable_fallback = parse_bool(&value)?; }
    "backend.ollama.base_url" => { user_config.backend.ollama.base_url = value; }
    "backend.ollama.model_name" => { user_config.backend.ollama.model_name = value; }
    "backend.vllm.base_url" => { user_config.backend.vllm.base_url = value; }
    "backend.vllm.model_name" => { user_config.backend.vllm.model_name = value; }

    _ => Err(...)
}
```

#### Option B: TOML path parsing (more robust, deferred)

Parse `safety.level` as a TOML path and set nested values. More flexible but adds complexity. Not needed for v1.2.

#### Data Model Additions

```rust
pub struct UserConfig {
    // existing fields...
    pub telemetry: TelemetryConfig,
    pub output: OutputConfig,
    pub backend: BackendConfig,
}

pub struct TelemetryConfig {
    pub enabled: bool,
    pub level: String,  // "off", "basic", "detailed"
    pub air_gapped: bool,
}

pub struct OutputConfig {
    pub format: String,  // "plain", "json", "yaml"
    pub color: bool,
}

pub struct BackendConfig {
    pub primary: String,
    pub enable_fallback: bool,
    pub ollama: BackendEndpoint,
    pub vllm: BackendEndpoint,
}

pub struct BackendEndpoint {
    pub base_url: String,
    pub model_name: String,
}
```

#### Acceptance Tests
```
#[test]
fn test_telemetry_config_keys() {
    assert_ok(config_set("telemetry.enabled", "false"));
    assert_ok(config_set("telemetry.level", "off"));
    assert_ok(config_set("telemetry.air_gapped", "true"));
}

#[test]
fn test_safety_config_keys() {
    assert_ok(config_set("safety.level", "strict"));
    assert_ok(config_set("safety.require_confirmation", "true"));
}

#[test]
fn test_backend_config_keys() {
    assert_ok(config_set("backend.primary", "ollama"));
    assert_ok(config_set("backend.ollama.base_url", "http://localhost:11434"));
}

#[test]
fn test_backward_compat() {
    // Old keys still work
    assert_ok(config_set("backend", "embedded"));
    assert_ok(config_set("safety", "moderate"));
}
```

#### Estimated Effort
- Struct additions: **2 hours**
- Key parsing expansion: **3 hours**
- Env var override support: **2 hours**
- `caro config get` for nested keys: **2 hours**
- Tests: **2 hours**
- **Total: 1-2 days**

---

## A3: Telemetry Subcommands (T0-04, T0-05)

### Problem
`caro telemetry show` and `caro telemetry export` are documented on caro.sh/telemetry but silently fall through to command generation. The code exists but is commented out.

### Current State (src/main.rs, commented out)

```rust
// Lines ~447-451:
// /// Manage telemetry data and settings
// Telemetry {
//     #[command(subcommand)]
//     command: caro::cli::telemetry::TelemetryCommands,
// },

// Lines ~1893-1896:
//     _ => {
//         match caro::cli::telemetry::handle_telemetry(command, storage_path).await {
//             ...
//         }
//     }
```

### Implementation Plan

1. **Uncomment** the telemetry subcommand in `Commands` enum
2. **Verify** `TelemetryCommands` enum exists in `src/cli/telemetry.rs` (or equivalent)
3. **Wire** `handle_telemetry()` to the storage path
4. **Implement `show`**: Read from SQLite telemetry database, format for display
5. **Implement `export`**: Read from SQLite, serialize to JSON, write to file

#### Show Output Format
```
Caro Telemetry Summary
======================
Total sessions: 147
Commands generated: 382
Commands executed: 291
Safety blocks: 12
Last session: 2026-03-25T14:32:00Z
Average session duration: 4m 12s

Backend usage:
  embedded: 312 (81.7%)
  ollama: 68 (17.8%)
  vllm: 2 (0.5%)
```

#### Export Format
```json
{
  "version": "1.1.3",
  "export_date": "2026-03-26T10:00:00Z",
  "total_sessions": 147,
  "total_commands_generated": 382,
  "total_commands_executed": 291,
  "total_safety_blocks": 12,
  "sessions": [
    {
      "session_id": "hash123",
      "start_time": "2026-03-25T14:30:00Z",
      "duration_seconds": 120,
      "commands_generated": 5,
      "commands_executed": 3,
      "backend_used": "embedded",
      "safety_blocks": 1
    }
  ]
}
```

#### Acceptance Tests
```
#[test]
fn test_telemetry_show() {
    let output = run_command(&["telemetry", "show"]);
    assert!(output.contains("Total sessions"));
    assert!(output.contains("Commands generated"));
}

#[test]
fn test_telemetry_export() {
    let path = temp_file("telemetry-export.json");
    run_command(&["telemetry", "export", "-o", path.to_str().unwrap()]);
    let content = read_file(&path);
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("version").is_some());
}
```

#### Estimated Effort
- Uncomment + wire: **2 hours**
- Implement `show` formatting: **3 hours**
- Implement `export` to JSON: **2 hours**
- SQLite read queries: **2 hours**
- Tests: **2 hours**
- **Total: 1-2 days**

---

## A4: Static Matcher Expansion (T1-04, T3-05)

### Problem
The static matcher returns `ls -la` or `echo 'Unable to generate command'` for too many common queries. This is the primary cause of the low pass rate (31%) and the most visible quality issue.

### Current Behavior

```rust
// src/backends/static_matcher.rs
fn match_command(&self, prompt: &str) -> Option<String> {
    // Matches against a set of regex patterns
    // Falls back to generic responses for unmatched queries
}
```

Queries that currently fail:
- "delete all log files" → `echo 'Unable to generate command'` ❌
- "check disk space" → `ls -la` ❌  
- "show processes by memory" → `ls -la` ❌
- "find large files" → `find . -type f -size +100M` ✅ (this one works)

### Design

Add 30-50 high-traffic query patterns. Organize by category:

#### Category: File Operations (extend existing)
```rust
// "delete all log files" or "remove log files" or "clear logs"
Pattern {
    regex: r"(?i)delete.*log|remove.*log|clear.*log",
    command: "find . -name '*.log' -type f -mtime +30 -delete",
    risk_level: High,
    preview_command: "find . -name '*.log' -type f -mtime +30 -ls",
}

// "delete old files" or "remove old files"
Pattern {
    regex: r"(?i)delete.*old|remove.*old|clean.*old",
    command: "find . -type f -mtime +30 -delete",
    risk_level: High,
    preview_command: "find . -type f -mtime +30 -ls",
}

// "delete temp files" or "clean temp"
Pattern {
    regex: r"(?i)delete.*temp|remove.*temp|clean.*temp|clean.*tmp",
    command: "find /tmp -type f -user $(whoami) -mtime +7 -delete",
    risk_level: Moderate,
    preview_command: "find /tmp -type f -user $(whoami) -mtime +7 -ls",
}
```

#### Category: System Information (NEW)
```rust
// "check disk space" or "show disk usage" or "disk space"
Pattern {
    regex: r"(?i)disk.*(space|usage|free)|show.*disk|df",
    command: "df -h",
    risk_level: Safe,
}

// "show processes" or "top processes" or "process by memory"
Pattern {
    regex: r"(?i)top.*process|process.*memory|memory.*process|show.*process",
    command: "ps aux --sort=-%mem | head -20",
    risk_level: Safe,
}

// "show cpu usage" or "cpu top"
Pattern {
    regex: r"(?i)cpu.*(usage|top)|show.*cpu",
    command: "top -l 1 | head -10",
    risk_level: Safe,
}

// "check network" or "show connections" or "network connections"
Pattern {
    regex: r"(?i)network.*(connect|status)|show.*connect|netstat",
    command: "netstat -tln",
    risk_level: Safe,
}

// "show environment" or "print env" or "list env"
Pattern {
    regex: r"(?i)(print|show|list).*env|env.*var",
    command: "env | sort",
    risk_level: Safe,
}

// "show memory usage" or "check ram"
Pattern {
    regex: r"(?i)memory.*(usage|free|avail)|check.*ram|show.*mem",
    command: "free -h",
    risk_level: Safe,
}

// "check uptime" or "how long running"
Pattern {
    regex: r"(?i)uptime|how.*long.*(running|up)|system.*up",
    command: "uptime",
    risk_level: Safe,
}
```

#### Category: Text Processing (extend existing)
```rust
// "count files" or "how many files"
Pattern {
    regex: r"(?i)count.*file|how.*many.*file|number.*file",
    command: "find . -type f | wc -l",
    risk_level: Safe,
}

// "find text in files" or "search in files" or "grep"
Pattern {
    regex: r"(?i)(find|search).*text.*file|grep",
    command: "grep -r 'SEARCH_TERM' .",
    risk_level: Safe,
    explanation: "Replace SEARCH_TERM with your search text",
}

// "show file size" or "largest files"
Pattern {
    regex: r"(?i)largest.*file|big.*file|file.*size|show.*size",
    command: "du -sh * | sort -hr | head -20",
    risk_level: Safe,
}

// "compare files" or "diff files"
Pattern {
    regex: r"(?i)compare.*file|diff.*file",
    command: "diff file1 file2",
    risk_level: Safe,
    explanation: "Replace file1 and file2 with your files",
}
```

#### Category: Process Management
```rust
// "kill process" or "stop process"
Pattern {
    regex: r"(?i)kill.*process|stop.*process|end.*process",
    command: "pkill PROCESS_NAME",
    risk_level: Moderate,
    explanation: "Replace PROCESS_NAME with the process name to kill",
}

// "show running services" or "list services"
Pattern {
    regex: r"(?i)(show|list).*service|service.*run",
    command: "systemctl list-units --type=service --state=running",
    risk_level: Safe,
}
```

#### Category: Git Operations
```rust
// "git status" or "check git"
Pattern {
    regex: r"(?i)git.*status|check.*git|repo.*status",
    command: "git status",
    risk_level: Safe,
}

// "git log recent" or "show commits"
Pattern {
    regex: r"(?i)git.*log|show.*commit|recent.*commit",
    command: "git log --oneline -10",
    risk_level: Safe,
}

// "git diff" or "what changed"
Pattern {
    regex: r"(?i)git.*diff|what.*change",
    command: "git diff",
    risk_level: Safe,
}
```

#### Acceptance Tests
```
#[test]
fn test_delete_all_logs() {
    let cmd = static_matcher("delete all log files");
    assert!(cmd.contains("find"));
    assert!(cmd.contains("*.log"));
}

#[test]
fn test_check_disk_space() {
    let cmd = static_matcher("check disk space");
    assert_eq!(cmd, "df -h");
}

#[test]
fn test_show_processes_by_memory() {
    let cmd = static_matcher("show top processes by memory");
    assert!(cmd.contains("ps") || cmd.contains("top"));
    assert!(cmd.contains("mem"));
}
```

#### Estimated Effort
- Pattern research (common queries): **2 hours**
- Implement 30-50 new patterns: **6 hours**
- Test each pattern: **3 hours**
- Integrate preview commands: **2 hours**
- **Total: 2-3 days**

---

## A5: Color-Coded Output (T1-03)

### Problem
Safety levels (Safe/Moderate/High/Critical) are plain text in CLI output. The skill docs and website show emoji/color indicators (🟢🟡🟠🔴) that don't appear in actual output.

### Design

#### Output Modes

```rust
enum OutputMode {
    Colored,   // TTY: ANSI colors + emoji
    Plain,     // Piped/redirected: no colors, no emoji
    Json,      // Machine-readable JSON
}
```

Detection logic:
```rust
fn detect_output_mode(args: &Args) -> OutputMode {
    if args.output == Some("json") {
        return OutputMode::Json;
    }
    if atty::is(Stream::Stdout) {
        OutputMode::Colored
    } else {
        OutputMode::Plain
    }
}
```

#### Color Implementation

Use `colored` crate (already a dependency or add it):

```rust
fn format_safety_level(level: &SafetyLevel, mode: &OutputMode) -> String {
    match mode {
        OutputMode::Colored => match level {
            SafetyLevel::Safe => "\x1b[32m✅ Safe\x1b[0m".green().to_string(),
            SafetyLevel::Moderate => "\x1b[33m⚠️ Moderate\x1b[0m".yellow().to_string(),
            SafetyLevel::High => "\x1b[91m🟠 High\x1b[0m".bright_red().to_string(),
            SafetyLevel::Critical => "\x1b[31m🔴 Critical\x1b[0m".red().bold().to_string(),
        },
        OutputMode::Plain => match level {
            SafetyLevel::Safe => "Safe".to_string(),
            SafetyLevel::Moderate => "Moderate".to_string(),
            SafetyLevel::High => "High".to_string(),
            SafetyLevel::Critical => "Critical".to_string(),
        },
        OutputMode::Json => unreachable!(),
    }
}
```

#### Estimated Effort
- Output mode detection: **1 hour**
- Color formatting: **2 hours**
- Integration in main.rs output: **2 hours**
- Tests (pipe vs TTY): **2 hours**
- **Total: 1 day**

---

## A6: PowerShell Command Generation (T1-07)

### Problem
`--shell powershell` generates POSIX commands (`ls -la`, `find . -type f`) instead of Windows syntax (`Get-ChildItem`, `Get-ChildItem -Recurse`).

### Design

#### Static Matcher PowerShell Variant

Add PowerShell equivalents for common patterns:

```rust
// src/backends/static_matcher.rs

const POSIX_TO_POWERSHELL: &[(&str, &str)] = &[
    ("ls -la", "Get-ChildItem"),
    ("ls -l", "Get-ChildItem"),
    ("ls", "Get-ChildItem"),
    ("cat ", "Get-Content "),
    ("rm ", "Remove-Item "),
    ("rm -rf ", "Remove-Item -Recurse -Force "),
    ("cp ", "Copy-Item "),
    ("mv ", "Move-Item "),
    ("find . -type f", "Get-ChildItem -Recurse -File"),
    ("find . -name", "Get-ChildItem -Recurse -Filter"),
    ("grep ", "Select-String "),
    ("grep -r ", "Get-ChildItem -Recurse | Select-String "),
    ("chmod ", "# Use icacls or Set-Acl on Windows"),
    ("df -h", "Get-PSDrive -PSProvider FileSystem"),
    ("du -sh", "(Get-ChildItem -Recurse | Measure-Object -Property Length -Sum).Sum"),
];
```

#### Prompt Modification for Embedded Model

When `--shell powershell` is passed, prepend to the system prompt:

```
You are generating PowerShell commands for Windows. 
ALWAYS use PowerShell syntax:
- Get-ChildItem instead of ls
- Get-Content instead of cat  
- Remove-Item instead of rm
- Copy-Item instead of cp
- Move-Item instead of mv
- Select-String instead of grep
- ForEach-Object instead of xargs
- | (pipe) for chaining
```

#### Estimated Effort
- Static matcher PowerShell patterns: **2 hours**
- Prompt modification for embedded model: **1 hour**
- Integration with shell detection: **1 hour**
- Tests: **2 hours**
- **Total: 1 day**

---

## A7: Embedded Model Quality Improvement (T1-06)

### Problem
The embedded model (qwen2.5-coder-1.5b) generates `ls -la` or `echo 'Unable to generate command'` for 3 out of 5 common queries. This is the #1 quality complaint.

### Root Cause Analysis

1. **Model size**: 1.5B parameters is very small for command generation
2. **Prompt engineering**: System prompt may not be optimized for this model
3. **Timeout issues**: Model may not have enough inference time for complex queries
4. **Temperature/top-p**: Sampling parameters may be too conservative

### Design

#### Prompt Engineering Improvements

**Current prompt** (approximate):
```
Generate a shell command for: {user_query}
```

**Improved prompt**:
```
You are a shell command generator. Given a natural language description, output EXACTLY ONE POSIX shell command.

RULES:
- Output ONLY the command, no explanation
- No markdown, no backticks
- Use standard POSIX utilities (find, grep, awk, sed, sort, etc.)
- For "list files" → ls -la
- For "find files" → find with appropriate flags
- For "disk space" → df -h
- For "processes" → ps aux
- For "delete" → find with -delete
- For "search text" → grep with appropriate flags
- NEVER output "ls -la" as a generic fallback
- NEVER output "echo 'Unable to generate command'"
- NEVER include comments or explanations

Examples:
User: "find all PDF files larger than 10MB"
Command: find . -name "*.pdf" -type f -size +10M

User: "delete all log files older than 30 days"  
Command: find . -name "*.log" -type f -mtime +30 -delete

User: "check disk space"
Command: df -h

User: "show top 10 processes by memory usage"
Command: ps aux --sort=-%mem | head -11

Now generate a command for: {user_query}
Command:
```

#### Fallback Chain

When embedded model returns `ls -la` or `echo 'Unable to'`:
1. Try static matcher (expanded in A4)
2. If static matcher fails, try with `--force-llm` (re-infer with lower temperature)
3. If still fails, return error with guidance

```rust
// In embedded_backend.rs
fn post_process(&self, command: &str, prompt: &str) -> Result<String, GeneratorError> {
    // Reject generic fallbacks
    if command.trim() == "ls -la" && !prompt.contains("list") && !prompt.contains("ls") {
        // Try static matcher fallback
        if let Some(matched) = self.static_matcher.match_command(prompt) {
            return Ok(matched);
        }
    }
    if command.contains("Unable to generate") {
        if let Some(matched) = self.static_matcher.match_command(prompt) {
            return Ok(matched);
        }
        return Err(GeneratorError::NoCommandGenerated);
    }
    Ok(command.to_string())
}
```

#### Estimated Effort
- Prompt rewrite: **3 hours**
- Fallback chain implementation: **4 hours**
- Eval testing with new prompts: **3 hours**
- Parameter tuning (temperature, top-p): **2 hours**
- **Total: 2-3 days**

---

## A8: CLI Flag Implementation (T0-01, T0-02, T0-03, T0-08)

### Problem
Five documented CLI flags don't exist: `--quiet`, `-e`, `--no-telemetry`, `--backend-info`, `--explain`.

### Implementation (all in `src/cli/mod.rs`)

```rust
// Add to Cli struct:
#[derive(Parser, Debug)]
pub struct Cli {
    // ... existing fields ...

    /// Suppress non-essential output (timing, debug info)
    #[arg(long)]
    pub quiet: bool,

    /// Execute the generated command directly (short form of --execute)
    #[arg(short = 'e', long)]
    pub execute_e: bool,

    /// Disable telemetry for this session
    #[arg(long)]
    pub no_telemetry: bool,

    /// Show backend status and available backends
    #[arg(long)]
    pub backend_info: bool,

    /// Enable explanation mode: shows detailed breakdowns of commands
    #[arg(long)]
    pub explain: bool,
}
```

#### Wiring in main.rs

```rust
// --quiet: suppress timing output
if cli.quiet {
    // Set a global quiet flag
    QUIET_MODE.store(true, Ordering::Relaxed);
}

// -e: equivalent to --execute
let execute = cli.execute || cli.execute_e;

// --no-telemetry: disable for this session
if cli.no_telemetry {
    user_config.telemetry.enabled = false;
}

// --backend-info: list backends
if cli.backend_info {
    print_available_backends();
    return Ok(());
}

// --explain: enable explanation mode
if cli.explain {
    // Enable explanation output
}
```

#### Acceptance Tests
```
#[test]
fn test_quiet_flag() {
    let output = run_command(&["--quiet", "list files"]);
    assert!(!output.contains("Generated in"));  // Timing suppressed
}

#[test]
fn test_execute_short_flag() {
    // -e should execute immediately
    let output = run_command(&["-e", "echo hello"]);
    assert!(output.contains("hello"));
}

#[test]
fn test_no_telemetry_flag() {
    // Should not error
    let output = run_command(&["--no-telemetry", "list files"]);
    assert!(!output.contains("error"));
}

#[test]
fn test_backend_info() {
    let output = run_command(&["--backend-info"]);
    assert!(output.contains("embedded"));
    assert!(output.contains("ollama"));
}
```

#### Estimated Effort
- Add flags to Cli struct: **1 hour**
- Wire each flag in main.rs: **2 hours**
- Tests: **2 hours**
- **Total: 1 day** (can be parallelized across contributors)
