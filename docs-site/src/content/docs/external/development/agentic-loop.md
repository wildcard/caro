---
title: Agentic Loop Architecture
description: "Documentation: Agentic Loop Architecture"
editUrl: false
---
## Overview
Implement an iterative refinement system with context enrichment for accurate command generation.

## Core Components

### 1. Agent Loop (Max 2-3 iterations, <5s total)
```
User Request
    ↓
[Iteration 1] Generate Initial Command (with platform context)
    ↓
[Validation] Parse response, extract commands
    ↓
[Context Enrichment] Get command help/man pages
    ↓
[Iteration 2] Refine with command-specific context
    ↓
Final Command
```

### 2. Context Gathering
```rust
pub struct ExecutionContext {
    // Platform info
    pub os: String,              // "Darwin", "Linux", "Windows"
    pub arch: String,            // "arm64", "x86_64"
    pub os_version: String,      // "14.2.1" (macOS), "6.5.0" (Linux)
    pub distribution: Option<String>,  // "Ubuntu 22.04", "macOS Sonoma"
    
    // Current environment
    pub cwd: PathBuf,            // Current working directory
    pub shell: String,           // "zsh", "bash", "fish"
    pub user: String,            // Current username
    
    // Available commands
    pub available_commands: HashMap<String, CommandInfo>,
}

pub struct CommandInfo {
    pub path: PathBuf,           // Full path to command
    pub version: String,         // Output of --version
    pub help_text: String,       // Output of --help
    pub man_summary: Option<String>,  // Brief man page summary
}
```

### 3. Iterative Refinement Flow

#### Iteration 1: Initial Generation
**Input:**
- User prompt
- Platform context (OS, arch, cwd)
- Available commands list

**Output:**
- Initial command
- Confidence score
- Identified commands used

#### Iteration 2: Refinement (if needed)
**Input:**
- User prompt
- Initial command
- Command-specific help/man pages
- Platform-specific flags

**Output:**
- Refined command
- Explanation of changes

**Trigger refinement when:**
- Confidence < 0.8
- Command uses complex flags (--sort, -exec, etc.)
- Platform-specific variations detected
- Multiple commands in pipeline

### 4. Command Context Extraction

```rust
pub async fn get_command_context(command: &str) -> CommandContext {
    let commands = extract_commands(command);
    let mut context = CommandContext::new();
    
    for cmd in commands {
        // Get version
        let version = get_command_version(&cmd).await;
        
        // Get help text (truncated to relevant sections)
        let help = get_command_help(&cmd).await;
        
        // Get man page summary
        let man_summary = get_man_summary(&cmd).await;
        
        context.add(cmd, CommandInfo { version, help, man_summary });
    }
    
    context
}

fn extract_commands(command: &str) -> Vec<String> {
    // Parse: ps aux | sort -k3 | head -5
    // Returns: ["ps", "sort", "head"]
    // Ignore: |, &&, ||, ;, >, <, etc.
}
```

### 5. Enhanced System Prompt

```rust
fn build_system_prompt(iteration: u32, context: &ExecutionContext) -> String {
    match iteration {
        1 => format!(r#"
You are a shell command generator for {os} {arch}.

ENVIRONMENT:
- OS: {os} {version} ({distribution})
- Architecture: {arch}
- Shell: {shell}
- Current Directory: {cwd}

AVAILABLE COMMANDS:
{available_commands}

PLATFORM NOTES:
{platform_notes}

RULES:
1. Output ONLY valid JSON: {{"cmd": "command", "confidence": 0.95}}
2. Use ONLY commands from available list
3. Prefer relative paths (. or ~/)
4. Escape quotes properly in JSON
5. For {os}:
   {os_specific_rules}

EXAMPLES:
{platform_examples}
"#),
        
        2 => format!(r#"
REFINEMENT ITERATION

ORIGINAL REQUEST: {user_prompt}
INITIAL COMMAND: {initial_command}

COMMAND DETAILS:
{command_help_context}

ISSUES TO FIX:
{detected_issues}

Please refine the command considering:
1. Platform-specific flags for {os}
2. Command versions and available options
3. Correct syntax for piping and chaining

Output: {{"cmd": "refined_command", "changes": "what was fixed"}}
"#)
    }
}
```

### 6. Platform-Specific Rules

```rust
fn get_platform_rules(os: &str) -> String {
    match os {
        "Darwin" => r#"
macOS (BSD-style):
- ps: Use 'ps aux' (no --sort), pipe to sort
- Network: Use 'lsof -iTCP' or 'netstat' (NOT ss)
- df: Use 'df -h' (no --sort), pipe to sort
- find: Use 'find .' for current dir
- sed: Use 'sed -i ""' for in-place (not -i.bak)
- xargs: Use 'xargs -I {}' for placeholder
"#,
        "Linux" => r#"
Linux (GNU coreutils):
- ps: Can use 'ps aux --sort=-pcpu'
- Network: Use 'ss -tuln'
- df: Can use 'df -h --sort=size'
- find: Use 'find .' for current dir
- sed: Use 'sed -i' for in-place
- xargs: Use 'xargs -I {}' for placeholder
"#,
        _ => ""
    }
}
```

### 7. Command Help Cache

```rust
pub struct CommandHelpCache {
    cache: Arc<RwLock<HashMap<String, CachedHelp>>>,
    cache_dir: PathBuf,
}

struct CachedHelp {
    command: String,
    version: String,
    help_text: String,
    timestamp: SystemTime,
    ttl: Duration,  // 24 hours
}

impl CommandHelpCache {
    pub async fn get_or_fetch(&self, command: &str) -> Result<CommandInfo> {
        // Check cache first
        if let Some(cached) = self.get_cached(command).await {
            return Ok(cached);
        }
        
        // Fetch and cache
        let info = self.fetch_command_info(command).await?;
        self.cache_info(command, &info).await?;
        Ok(info)
    }
    
    async fn fetch_command_info(&self, command: &str) -> Result<CommandInfo> {
        // Run: command --version
        let version = Command::new(command)
            .arg("--version")
            .output()
            .await?
            .stdout;
        
        // Run: command --help (truncate to 2KB)
        let help = Command::new(command)
            .arg("--help")
            .output()
            .await?
            .stdout
            .truncate(2048);
        
        // Get man page summary (first paragraph only)
        let man_summary = Command::new("man")
            .arg(command)
            .output()
            .await
            .ok()
            .and_then(|out| extract_man_summary(&out.stdout));
        
        Ok(CommandInfo { version, help, man_summary })
    }
}
```

### 8. Agent Loop Implementation

```rust
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    context: ExecutionContext,
    help_cache: CommandHelpCache,
    max_iterations: usize,  // Default: 2
    timeout: Duration,      // Default: 5s
}

impl AgentLoop {
    pub async fn generate_command(&self, prompt: &str) -> Result<GeneratedCommand> {
        let start = Instant::now();
        let mut iteration = 1;
        
        // Iteration 1: Initial generation with platform context
        let initial = self.generate_initial(prompt).await?;
        
        // Check if refinement needed
        if !self.should_refine(&initial) || start.elapsed() > self.timeout / 2 {
            return Ok(initial);
        }
        
        // Iteration 2: Refine with command context
        let commands = extract_commands(&initial.command);
        let command_context = self.get_command_context(&commands).await?;
        
        let refined = self.refine_command(prompt, &initial, &command_context).await?;
        
        Ok(refined)
    }
    
    fn should_refine(&self, command: &GeneratedCommand) -> bool {
        // Refine if:
        // - Confidence < 0.8
        // - Uses platform-specific commands (ps, ss, df)
        // - Has complex pipes or flags
        // - Uses sed/awk/xargs
        
        command.confidence < 0.8 ||
        command.command.contains("ps ") ||
        command.command.contains("ss ") ||
        command.command.contains("df ") ||
        command.command.contains("sed ") ||
        command.command.contains("xargs ") ||
        command.command.split('|').count() > 2
    }
    
    async fn generate_initial(&self, prompt: &str) -> Result<GeneratedCommand> {
        let system_prompt = build_system_prompt(1, &self.context);
        
        let request = CommandRequest {
            prompt: prompt.to_string(),
            system_prompt,
            context: self.context.clone(),
        };
        
        self.backend.generate_command(&request).await
    }
    
    async fn refine_command(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &CommandContext,
    ) -> Result<GeneratedCommand> {
        let system_prompt = build_refinement_prompt(
            prompt,
            &initial.command,
            command_context,
            &self.context,
        );
        
        let request = CommandRequest {
            prompt: format!("REFINE: {}", prompt),
            system_prompt,
            context: self.context.clone(),
        };
        
        self.backend.generate_command(&request).await
    }
}
```

### 9. Response Format

```rust
// Iteration 1 response:
{
    "cmd": "ps aux | sort -nrk 3,3 | head -5",
    "confidence": 0.75,
    "commands_used": ["ps", "sort", "head"]
}

// Iteration 2 response:
{
    "cmd": "ps aux | sort -nrk 3,3 | head -6",
    "confidence": 0.95,
    "changes": "Changed head -5 to head -6 to account for header line",
    "commands_used": ["ps", "sort", "head"]
}
```

### 10. Context Detection Functions

```rust
pub fn detect_execution_context() -> ExecutionContext {
    ExecutionContext {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        os_version: get_os_version(),
        distribution: detect_distribution(),
        cwd: std::env::current_dir().unwrap_or_default(),
        shell: detect_shell(),
        user: std::env::var("USER").unwrap_or_default(),
        available_commands: scan_available_commands(),
    }
}

fn get_os_version() -> String {
    if cfg!(target_os = "macos") {
        // sw_vers -productVersion
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_default()
    } else if cfg!(target_os = "linux") {
        // uname -r
        Command::new("uname")
            .arg("-r")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

fn detect_distribution() -> Option<String> {
    if cfg!(target_os = "macos") {
        // sw_vers -productName + productVersion
        Some(format!("macOS {}", get_os_version()))
    } else if cfg!(target_os = "linux") {
        // Check /etc/os-release
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .map(|line| line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
            })
    } else {
        None
    }
}

fn scan_available_commands() -> HashMap<String, CommandInfo> {
    let common_commands = vec![
        "ps", "top", "kill", "find", "grep", "sed", "awk",
        "sort", "head", "tail", "cut", "tr", "wc", "xargs",
        "ls", "cat", "df", "du", "lsof", "netstat", "ss",
        "git", "curl", "wget", "tar", "gzip", "unzip",
    ];
    
    common_commands.iter()
        .filter_map(|cmd| {
            which::which(cmd).ok().map(|path| {
                (cmd.to_string(), CommandInfo::minimal(path))
            })
        })
        .collect()
}
```

## Performance Targets

- **Iteration 1**: <2s (command generation)
- **Context fetching**: <500ms (cached after first use)
- **Iteration 2**: <1.5s (refinement)
- **Total**: <4s for full loop

## Testing Strategy

```bash
# Test iteration 1 (basic)
caro "list files" --debug

# Test iteration 2 (complex)
caro "show top 5 processes by CPU" --debug

# Test context enrichment
caro "find large files using xargs" --debug --verbose

# Measure timing
caro "complex query" --timing
```

## Implementation Order

1. ✅ Create branch
2. [ ] Update Cargo.toml for embedded-mlx default
3. [ ] Implement ExecutionContext detection
4. [ ] Implement CommandHelpCache
5. [ ] Update system prompt with context
6. [ ] Implement AgentLoop with 2 iterations
7. [ ] Add response parsing with confidence
8. [ ] Test on Vancouver demo scenarios
9. [ ] Optimize performance (<5s total)

## Success Criteria

- [ ] All 6 Vancouver demos work on macOS
- [ ] Commands are platform-appropriate
- [ ] Total time <5s per query
- [ ] Context is accurate and helpful
- [ ] Iteration improves accuracy >20%
