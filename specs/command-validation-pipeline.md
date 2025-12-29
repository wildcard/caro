# Command Validation Pipeline Specification

## Problem Statement

Generated commands may not be compatible with the target platform due to:
- Platform-specific flags (e.g., `netstat -ano` on Linux vs `netstat -an` on macOS)
- Missing utilities or different versions
- Shell-specific syntax differences
- OS-specific command variations

**Example failure:**
```bash
# Generated (Linux-style):
netstat -ano | grep LISTEN

# Fails on macOS:
netstat: illegal option -- o
```

## Solution: Two-Phase LLM Validation

### Architecture Overview

```
User Prompt
    ↓
[Phase 1: Command Generation]
    ↓
Generated Command
    ↓
[Platform Introspection]
  - Extract base command(s)
  - Check availability (which/where)
  - Collect help text (--help, -h, man, tldr)
    ↓
[Phase 2: Command Validation]
  - LLM validates against platform
  - Suggests fixes if needed
    ↓
Validated Command → Safety Check → User Confirmation → Execute
```

## Phase 1: Enhanced Platform Context

### Platform Information to Collect

```rust
pub struct PlatformContext {
    // Basic platform info
    pub os: String,              // "macos", "linux", "windows"
    pub os_version: String,      // "14.1.1", "Ubuntu 22.04"
    pub arch: String,            // "aarch64", "x86_64"
    
    // Shell information
    pub shell: String,           // "zsh", "bash", "fish", "powershell"
    pub shell_version: String,   // "zsh 5.9"
    
    // Common utilities availability
    pub available_tools: HashMap<String, String>,  // name → version
    
    // Environment constraints
    pub posix_compliant: bool,
    pub has_gnu_coreutils: bool,
    pub has_bsd_utils: bool,
}
```

### Enhanced System Prompt

Add platform context to initial generation:

```
You are generating shell commands for:
- OS: {os} {os_version} ({arch})
- Shell: {shell} {shell_version}
- Utilities: {gnu_coreutils|bsd_utils|busybox}

PLATFORM-SPECIFIC NOTES:
- macOS uses BSD utilities (different flags than GNU)
- netstat: use -an (not -ano) on BSD systems
- sed: use -i '' (not -i) on BSD for in-place edits
- date: different format specifiers on BSD vs GNU

Generate commands that are COMPATIBLE with this specific platform.
```

## Phase 2: Command Introspection

### Help Text Collection

For each generated command, extract base commands and collect documentation:

```rust
pub struct CommandHelp {
    pub command: String,
    pub exists: bool,
    pub path: Option<PathBuf>,
    pub help_text: Option<String>,      // from --help or -h
    pub man_page: Option<String>,       // from man (first 100 lines)
    pub tldr_summary: Option<String>,   // from tldr if available
    pub version: Option<String>,        // from --version
}

impl CommandHelp {
    pub async fn collect(command: &str) -> Self {
        // 1. Check existence: which/where
        // 2. Try --help, then -h
        // 3. Try man page (parse first section)
        // 4. Try tldr if available
        // 5. Try --version
    }
}
```

### Command Extraction

```rust
pub fn extract_base_commands(command: &str) -> Vec<String> {
    // Parse shell command to extract base commands
    // "netstat -ano | grep LISTEN" → ["netstat", "grep"]
    // "find . -name '*.rs' | xargs wc -l" → ["find", "xargs", "wc"]
}
```

## Phase 3: LLM-Based Validation

### Validation Prompt

```
You are a CRITICAL command validator. Your job is to verify if a command will work on the target platform.

PLATFORM:
{platform_context}

GENERATED COMMAND:
{command}

COMMAND DOCUMENTATION:
{collected_help_text}

TASK:
1. Check if the command will work on this platform
2. Identify any incompatibilities (wrong flags, missing utilities, syntax errors)
3. If incompatible, provide a CORRECTED command that achieves the same goal

Respond in JSON:
{
  "valid": true|false,
  "issues": ["list of issues found"],
  "corrected_command": "fixed command or null if valid",
  "confidence": 0.0-1.0,
  "explanation": "brief explanation"
}
```

### Validation Module

```rust
pub struct CommandValidator {
    platform_ctx: PlatformContext,
    backend: Arc<dyn CommandGenerator>,
}

pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<String>,
    pub corrected_command: Option<String>,
    pub confidence: f32,
    pub explanation: String,
}

impl CommandValidator {
    pub async fn validate_command(&self, command: &str) -> Result<ValidationResult> {
        // 1. Extract base commands
        let base_cmds = extract_base_commands(command);
        
        // 2. Collect help text for each command
        let help_texts = futures::future::join_all(
            base_cmds.iter().map(|cmd| CommandHelp::collect(cmd))
        ).await;
        
        // 3. Build validation prompt
        let prompt = self.build_validation_prompt(command, &help_texts);
        
        // 4. Call LLM for validation
        let response = self.backend.generate_command(&prompt).await?;
        
        // 5. Parse validation result
        self.parse_validation_response(&response)
    }
}
```

## Integration with Main Flow

### Updated Execution Flow

```rust
pub async fn execute_workflow(prompt: &str) -> Result<()> {
    // 1. Detect platform
    let platform_ctx = PlatformContext::detect().await?;
    
    // 2. Generate command (Phase 1)
    println!("Generating command...");
    let generated_cmd = generator.generate_command(prompt, &platform_ctx).await?;
    
    // 3. Validate command (Phase 2)
    println!("🔍 Validating command compatibility...");
    let validation = validator.validate_command(&generated_cmd.command).await?;
    
    // 4. Handle validation result
    let final_cmd = if validation.valid {
        println!("✓ Command validated");
        generated_cmd.command
    } else {
        println!("⚠ Issues found: {}", validation.issues.join(", "));
        if let Some(corrected) = validation.corrected_command {
            println!("📝 Suggested fix:\n  {}", corrected);
            // Ask user if they want to use corrected command
            corrected
        } else {
            return Err(anyhow!("Command cannot be executed on this platform"));
        }
    };
    
    // 5. Safety validation
    let safety_result = safety_validator.validate(&final_cmd)?;
    
    // 6. User confirmation
    // 7. Execute
}
```

## Configuration

### User Configuration (`~/.config/caro/config.toml`)

```toml
[validation]
enabled = true
collect_help_text = true        # Collect --help, man, tldr
collect_man_pages = true        # Can be slow, make optional
collect_tldr = true             # Only if tldr is installed
max_help_text_size = 5000       # Truncate large help texts
validation_timeout_ms = 5000    # Timeout for validation phase
auto_fix = false                # Auto-apply corrections without asking
show_validation_details = true  # Show what's being checked

[validation.skip_commands]
# Commands to skip validation (always trusted)
commands = ["ls", "cd", "pwd", "echo"]
```

## Performance Considerations

1. **Caching**: Cache help text for common commands
   - `~/.cache/caro/help_text/{command}.json`
   - TTL: 7 days (commands don't change often)

2. **Parallel collection**: Collect help text for all commands concurrently

3. **Timeouts**: 
   - Help text collection: 2s per command
   - Validation LLM call: 5s
   - Total validation phase: <10s

4. **Optimization**:
   - Skip validation for simple commands (configurable whitelist)
   - Only collect help for unknown/complex commands
   - Use shortened man pages (first 100 lines)

## Testing Strategy

### Contract Tests

```rust
// tests/validation_contract.rs

#[tokio::test]
async fn test_platform_context_detection() {
    let ctx = PlatformContext::detect().await.unwrap();
    assert!(!ctx.os.is_empty());
    assert!(!ctx.shell.is_empty());
}

#[tokio::test]
async fn test_help_text_collection() {
    let help = CommandHelp::collect("ls").await;
    assert!(help.exists);
    assert!(help.help_text.is_some() || help.man_page.is_some());
}

#[tokio::test]
async fn test_command_extraction() {
    let cmds = extract_base_commands("netstat -ano | grep LISTEN");
    assert_eq!(cmds, vec!["netstat", "grep"]);
}

#[tokio::test]
async fn test_validation_pipeline() {
    let validator = CommandValidator::new(platform_ctx, backend);
    
    // Test invalid command
    let result = validator.validate_command("netstat -ano").await.unwrap();
    if cfg!(target_os = "macos") {
        assert!(!result.valid);
        assert!(result.corrected_command.is_some());
    }
}
```

### Integration Tests

```rust
// tests/validation_integration.rs

#[tokio::test]
async fn test_full_validation_workflow() {
    // Generate → Validate → Execute
}

#[tokio::test]
async fn test_validation_with_missing_command() {
    // Should detect command doesn't exist
}

#[tokio::test]
async fn test_validation_caching() {
    // Second call should use cached help text
}
```

## Implementation Phases

### Phase 1: Platform Detection (Week 1)
- [ ] Enhance `PlatformContext` struct
- [ ] Implement shell detection
- [ ] Implement utility detection
- [ ] Add platform info to system prompt
- [ ] Contract tests

### Phase 2: Help Text Collection (Week 1-2)
- [ ] Implement `CommandHelp` struct
- [ ] Add help text collectors (--help, man, tldr)
- [ ] Implement command extraction
- [ ] Add caching layer
- [ ] Contract tests

### Phase 3: Validation Module (Week 2)
- [ ] Create `CommandValidator` struct
- [ ] Implement validation prompt builder
- [ ] Implement LLM-based validation
- [ ] Add correction handling
- [ ] Contract tests

### Phase 4: Integration (Week 2-3)
- [ ] Update main execution flow
- [ ] Add user feedback/progress indicators
- [ ] Add configuration options
- [ ] Integration tests
- [ ] E2E tests

### Phase 5: Optimization (Week 3)
- [ ] Add help text caching
- [ ] Add command whitelist
- [ ] Performance benchmarks
- [ ] Documentation

## Success Metrics

1. **Correctness**: 95%+ of generated commands work on target platform
2. **Performance**: Validation adds <5s to total execution time
3. **User Experience**: Clear feedback during validation phase
4. **Reliability**: No false positives (valid commands marked invalid)

## Alternative Approaches Considered

1. **Static validation**: Parse command syntax without LLM
   - Rejected: Too many edge cases, hard to maintain

2. **Dry-run execution**: Run command with `--dry-run` flag
   - Rejected: Not all commands support dry-run, could still have side effects

3. **Platform-specific models**: Train separate models per platform
   - Rejected: Too expensive, current approach is more flexible

## Open Questions

1. Should we validate every command or only on first failure?
   - **Decision**: Always validate initially, add option to skip for trusted commands

2. How to handle commands that don't have help text?
   - **Decision**: Mark as "unknown" and proceed with caution, show warning

3. Should we auto-apply corrections or always ask user?
   - **Decision**: Configurable, default to showing suggestion and asking

4. What about commands that are deprecated on newer OS versions?
   - **Decision**: Validation should catch this and suggest modern alternatives

## References

- GNU Coreutils: https://www.gnu.org/software/coreutils/
- BSD Commands: https://man.freebsd.org/
- POSIX Utilities: https://pubs.opengroup.org/onlinepubs/9699919799/
- Shell Command Language: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/contents.html
