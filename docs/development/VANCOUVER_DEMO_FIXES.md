# Vancouver.dev Demo Fixes - Implementation Plan

## Issues Found (December 15, 2024)

### Summary
The demo is generating Linux commands on macOS, causing 5 out of 6 demos to fail. Core issues:
1. **Platform detection not integrated** into command generation
2. **JSON parsing fails** with nested quotes
3. **Commands assume Linux** (ps, ss, df flags)
4. **Path assumptions** (searching from root instead of current dir)

---

## 🔴 Critical Fixes (Must Have for Tomorrow)

### Issue #1: JSON Parsing with Nested Quotes
**Problem:**
```json
{"cmd": "git log --since="2.weeks ago" --author-name"}
```
Nested quotes break the parser.

**Root Cause:**
- LLM generates commands with unescaped quotes inside JSON
- Our parser doesn't handle this gracefully

**Implementation:**
```rust
// File: src/backends/mod.rs (or response parsing module)

// BEFORE:
let response: CommandResponse = serde_json::from_str(&text)?;

// AFTER: Add fallback parsing strategies
fn parse_response(text: &str) -> Result<String, ParseError> {
    // Strategy 1: Try standard JSON parsing
    if let Ok(response) = serde_json::from_str::<CommandResponse>(text) {
        return Ok(response.cmd);
    }
    
    // Strategy 2: Extract command with regex (fallback for malformed JSON)
    // Pattern: {"cmd": "...", ...} or {"cmd": '...', ...}
    let re = Regex::new(r#"\{"cmd"\s*:\s*["']([^"']*(?:["'][^"']*["'][^"']*)*)["']\s*[,}]"#)?;
    if let Some(caps) = re.captures(text) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    // Strategy 3: Look for just the command part
    let cmd_re = Regex::new(r#"["']cmd["']\s*:\s*["'](.+?)["']"#)?;
    if let Some(caps) = cmd_re.captures(text) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }
    
    Err(ParseError::NoValidFormat)
}
```

**Priority:** 🔴 CRITICAL
**Time Estimate:** 30 minutes
**Files to Change:**
- `src/backends/mod.rs` or response parsing module
- Add tests in `tests/backend_trait_contract.rs`

---

### Issue #2: Platform-Aware Command Generation
**Problem:**
Commands generated assume Linux:
- `ps aux --sort=-pcpu` ❌ (Linux)
- `ss -tuln` ❌ (Linux only)
- `df -h --sort=size` ❌ (GNU coreutils)
- `find /` ❌ (searches root, permission errors)

**Implementation:**

#### Step 1: Enhance Platform Detection
```rust
// File: src/platform/mod.rs

pub enum OSFamily {
    Linux,
    MacOS,
    Windows,
    BSD,
    Unknown,
}

pub struct PlatformInfo {
    pub os_family: OSFamily,
    pub architecture: String,
    pub shell: String,
    pub available_commands: HashSet<String>,
}

impl PlatformInfo {
    pub fn detect() -> Self {
        let os_family = match std::env::consts::OS {
            "linux" => OSFamily::Linux,
            "macos" => OSFamily::MacOS,
            "windows" => OSFamily::Windows,
            _ if std::env::consts::OS.contains("bsd") => OSFamily::BSD,
            _ => OSFamily::Unknown,
        };
        
        // Detect available commands
        let available_commands = detect_available_commands(&[
            "ps", "ss", "netstat", "lsof", "df", "du", 
            "find", "grep", "awk", "sed"
        ]);
        
        PlatformInfo {
            os_family,
            architecture: std::env::consts::ARCH.to_string(),
            shell: detect_shell(),
            available_commands,
        }
    }
    
    pub fn get_platform_context(&self) -> String {
        match self.os_family {
            OSFamily::MacOS => "macOS (BSD-style commands, use 'ps aux' without --sort, 'lsof -i' for ports, 'df -h' without --sort)",
            OSFamily::Linux => "Linux (GNU coreutils, 'ps aux --sort=-pcpu', 'ss -tuln', 'df -h --sort=size')",
            OSFamily::Windows => "Windows (PowerShell commands preferred)",
            _ => "POSIX-compliant Unix",
        }.to_string()
    }
}

fn detect_available_commands(commands: &[&str]) -> HashSet<String> {
    commands.iter()
        .filter(|cmd| Command::new("which").arg(cmd).output().is_ok())
        .map(|s| s.to_string())
        .collect()
}
```

#### Step 2: Update System Prompt with Platform Context
```rust
// File: src/backends/mod.rs or prompt building module

fn build_system_prompt(platform: &PlatformInfo) -> String {
    format!(r#"You are a shell command generator for {platform_context}.

CRITICAL RULES:
1. Output ONLY valid JSON: {{"cmd": "your command here"}}
2. Platform: {platform_details}
3. Use only available commands: {available_cmds}
4. For macOS:
   - Use 'ps aux' (no --sort flag, pipe to sort instead)
   - Use 'lsof -iTCP -sTCP:LISTEN' for listening ports (NOT ss)
   - Use 'df -h' (no --sort flag, pipe to sort instead)
   - Use 'find .' to search current directory (NOT find /)
5. Always use relative paths (. or ~/) unless user specifies absolute
6. Escape quotes in JSON properly

Examples:
- "show processes": {{"cmd": "ps aux | sort -nrk 3,3 | head -5"}}
- "listening ports": {{"cmd": "lsof -iTCP -sTCP:LISTEN -n -P"}}
- "disk usage": {{"cmd": "df -h | tail -n +2 | sort -k5 -hr"}}
"#,
        platform_context = platform.get_platform_context(),
        platform_details = format!("{:?}", platform.os_family),
        available_cmds = platform.available_commands.iter().join(", ")
    )
}
```

#### Step 3: Pass Platform Info to Backends
```rust
// File: src/models/mod.rs

pub struct CommandRequest {
    pub prompt: String,
    pub shell_type: ShellType,
    pub safety_level: SafetyLevel,
    pub platform_info: PlatformInfo,  // ADD THIS
}

// File: src/backends/mod.rs
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(
        &self, 
        request: &CommandRequest
    ) -> Result<GeneratedCommand, GeneratorError>;
}
```

**Priority:** 🔴 CRITICAL
**Time Estimate:** 2 hours
**Files to Change:**
- `src/platform/mod.rs` (enhance detection)
- `src/models/mod.rs` (add platform_info field)
- `src/backends/mod.rs` (update system prompt)
- `src/backends/embedded/mod.rs` (use platform context)
- `src/cli/mod.rs` (detect platform on startup)

---

### Issue #3: Fix Demo Queries for Cross-Platform
**Problem:**
Queries are too generic, don't hint at platform-specific needs.

**Implementation:**
Update the demo script queries to be more platform-aware or add context.

```bash
# BEFORE (Demo 2):
caro "show top 5 processes by CPU usage"
# Generates: ps aux --sort=-pcpu | head -n 5  ❌

# AFTER (more explicit):
caro "show top 5 processes by CPU usage with ps command"
# OR let platform detection handle it automatically

# BEFORE (Demo 4):
caro "show all listening TCP ports"
# Generates: ss -tuln  ❌ (Linux only)

# AFTER (platform-agnostic):
caro "show all listening TCP ports on this system"
# Should detect macOS → lsof -iTCP -sTCP:LISTEN
# Should detect Linux → ss -tuln

# BEFORE (Demo 3):
caro "find all Rust files modified in the last 7 days"
# Generates: find / ...  ❌ (searches from root)

# AFTER (explicit context):
caro "find all Rust files in current directory modified in the last 7 days"
# Generates: find . -name '*.rs' -mtime -7
```

**Priority:** 🟡 HIGH (Easy quick win)
**Time Estimate:** 15 minutes
**Files to Change:**
- `demos/asciinema/vancouver-dev-demo.sh`

---

## 🟡 High Priority Fixes (Should Have)

### Issue #4: Better Error Messages
**Problem:**
When commands fail, errors are verbose and confusing.

**Implementation:**
```rust
// File: src/cli/mod.rs or error handling

pub fn format_execution_error(error: &ExecutionError, platform: &PlatformInfo) -> String {
    match error {
        ExecutionError::CommandNotFound(cmd) => {
            let suggestion = suggest_alternative(cmd, platform);
            format!("Command '{}' not found. {}", cmd, suggestion)
        }
        ExecutionError::PermissionDenied(path) => {
            format!("Permission denied: {}. Try running from a different directory.", path)
        }
        ExecutionError::ExitCode(code, stderr) => {
            let hint = parse_error_hint(stderr, platform);
            format!("Command failed (exit {}): {}", code, hint)
        }
        _ => format!("{}", error)
    }
}

fn suggest_alternative(cmd: &str, platform: &PlatformInfo) -> String {
    match (cmd, &platform.os_family) {
        ("ss", OSFamily::MacOS) => "Try using 'lsof' or 'netstat' on macOS.".to_string(),
        ("ps", OSFamily::MacOS) if cmd.contains("--sort") => 
            "macOS ps doesn't support --sort. Pipe to 'sort' instead.".to_string(),
        _ => "".to_string()
    }
}
```

**Priority:** 🟡 HIGH
**Time Estimate:** 1 hour
**Files to Change:**
- `src/cli/mod.rs`
- `src/execution/mod.rs`

---

### Issue #5: Command Validation Before Execution
**Problem:**
Commands execute even when they're clearly wrong for the platform.

**Implementation:**
```rust
// File: src/safety/mod.rs

impl SafetyValidator {
    pub fn validate_platform_compatibility(
        &self, 
        command: &str, 
        platform: &PlatformInfo
    ) -> ValidationResult {
        let warnings = vec![];
        
        // Check for Linux-specific commands on macOS
        if matches!(platform.os_family, OSFamily::MacOS) {
            if command.contains("--sort=") && command.contains("ps") {
                warnings.push("'ps --sort' is not supported on macOS. Use pipe to sort.");
            }
            if command.contains("ss ") && !command.contains("less") {
                warnings.push("'ss' command not available on macOS. Use 'lsof' or 'netstat'.");
            }
        }
        
        // Check for macOS-specific issues
        if command.starts_with("find /") && !command.contains("sudo") {
            warnings.push("Searching from root (/) will cause permission errors. Use 'find .' instead.");
        }
        
        ValidationResult {
            is_valid: warnings.is_empty(),
            warnings,
            risk_level: if warnings.is_empty() { RiskLevel::Safe } else { RiskLevel::Moderate },
        }
    }
}
```

**Priority:** 🟡 HIGH
**Time Estimate:** 1 hour
**Files to Change:**
- `src/safety/mod.rs`
- `src/models/mod.rs`

---

## 🟢 Nice to Have (Future)

### Issue #6: Command History & Learning
Store failed commands and improve suggestions over time.

**Implementation:**
```rust
// File: src/cache/command_history.rs

pub struct CommandHistory {
    successful: Vec<HistoryEntry>,
    failed: Vec<FailedEntry>,
}

pub struct HistoryEntry {
    pub prompt: String,
    pub command: String,
    pub platform: String,
    pub timestamp: DateTime<Utc>,
    pub execution_time: Duration,
}

pub struct FailedEntry {
    pub prompt: String,
    pub command: String,
    pub error: String,
    pub platform: String,
    pub timestamp: DateTime<Utc>,
}

impl CommandHistory {
    pub fn suggest_from_history(&self, prompt: &str, platform: &str) -> Option<String> {
        // Find similar successful commands
        self.successful.iter()
            .filter(|entry| entry.platform == platform)
            .find(|entry| similarity(prompt, &entry.prompt) > 0.8)
            .map(|entry| entry.command.clone())
    }
}
```

**Priority:** 🟢 NICE TO HAVE
**Time Estimate:** 3 hours
**Files to Create:**
- `src/cache/command_history.rs`

---

## 📋 Implementation Checklist for Tomorrow

### Must Do (Before Demo):
- [ ] **Fix JSON parsing** with fallback strategies (30 min)
- [ ] **Add platform context** to system prompt (1 hour)
- [ ] **Update demo queries** to be more explicit (15 min)
- [ ] **Test all 6 demos** on macOS (30 min)
- [ ] **Create macOS-specific demo** variant if needed (30 min)

### Should Do (If Time):
- [ ] Enhance platform detection with available commands (1 hour)
- [ ] Add platform compatibility validation (1 hour)
- [ ] Better error messages with suggestions (1 hour)

### Total Time: ~3-5 hours

---

## 🧪 Testing Plan

### Test Commands (macOS):
```bash
# Demo 1: Git (fix JSON parsing)
./target/release/caro "show git commits from last 2 weeks with author names"
# Expected: git log --since='2 weeks ago' --pretty=format:'%an %s'

# Demo 2: Processes (fix macOS ps)
./target/release/caro "show top 5 processes by CPU usage"
# Expected: ps aux | sort -nrk 3,3 | head -5

# Demo 3: File search (fix path)
./target/release/caro "find all Rust files in current directory modified in the last 7 days"
# Expected: find . -name '*.rs' -mtime -7

# Demo 4: Network (fix ss → lsof)
./target/release/caro "show all listening TCP ports on this system"
# Expected: lsof -iTCP -sTCP:LISTEN -n -P

# Demo 5: Disk (fix df flags)
./target/release/caro "show disk usage sorted by size"
# Expected: df -h | tail -n +2 | sort -k5 -hr

# Demo 6: Logs (fix path)
./target/release/caro "find all log files in current directory and count total lines"
# Expected: find . -name '*.log' -type f | xargs wc -l
```

---

## 🎯 Success Criteria

### Before Demo Tomorrow:
1. ✅ All 6 demos run successfully on macOS without errors
2. ✅ Commands are platform-appropriate (lsof not ss, etc.)
3. ✅ No permission errors from searching /
4. ✅ JSON parsing handles nested quotes
5. ✅ Execution time < 2s per command
6. ✅ Clean, professional output

### Presentation Ready:
- [ ] Demo script runs start-to-finish without failures
- [ ] Each command completes in < 5 seconds
- [ ] Output is relevant and useful
- [ ] Error handling is graceful (if any)
- [ ] Platform detection is invisible to user

---

## 📝 Quick Fix Script

```bash
#!/bin/bash
# Quick test of all fixes

echo "Testing Demo Fixes..."

cd /Users/kobi/personal/caro

# Test 1: JSON parsing
echo "1. Testing JSON parsing..."
./target/release/caro "show git commits from today" -x

# Test 2: macOS ps
echo "2. Testing macOS process listing..."
./target/release/caro "show top 5 processes by CPU" -x

# Test 3: Relative path find
echo "3. Testing file search..."
./target/release/caro "find Rust files in current directory" -x

# Test 4: macOS networking
echo "4. Testing network ports..."
./target/release/caro "show listening TCP ports" -x

# Test 5: macOS df
echo "5. Testing disk usage..."
./target/release/caro "show disk usage" -x

# Test 6: Relative log search
echo "6. Testing log analysis..."
./target/release/caro "count log files in current directory" -x

echo "All tests complete!"
```

---

## Priority Order for Implementation:

1. **JSON parsing fallback** (30 min) - Unblocks Demo 1
2. **Platform context in system prompt** (1 hour) - Fixes Demos 2, 4, 5
3. **Update demo queries** (15 min) - Clarifies intent for Demos 3, 6
4. **Test & iterate** (30 min) - Verify all fixes work

**Total minimum time: 2.25 hours**

This will get the demo working for tomorrow! 🚀
