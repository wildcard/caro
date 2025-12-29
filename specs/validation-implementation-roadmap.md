# Command Validation Implementation Roadmap

## Overview

This roadmap outlines the test-first implementation of the command validation pipeline to prevent platform-incompatible commands from being executed.

## Pre-Implementation Checklist

- [x] Specification document created (`specs/command-validation-pipeline.md`)
- [ ] Review specification with team
- [ ] Identify breaking changes to existing APIs
- [ ] Plan backward compatibility strategy
- [ ] Set up feature flag for gradual rollout

## Phase 1: Platform Detection Enhancement (3-5 days)

### Goals
- Extend `PlatformContext` with shell and utility information
- Make platform detection comprehensive and reliable

### Test-First Tasks

#### 1.1: Platform Context Data Structure
**Contract Test First** (`tests/platform_detection_contract.rs`)
```rust
#[test]
fn test_platform_context_has_required_fields() {
    let ctx = PlatformContext::detect().await;
    assert!(!ctx.os.is_empty());
    assert!(!ctx.shell.is_empty());
    assert!(ctx.posix_compliant == true || ctx.posix_compliant == false);
}
```

**Implementation** (`src/platform/mod.rs`)
- Add fields to `PlatformContext` struct
- Implement detection methods

#### 1.2: Shell Detection
**Contract Test First**
```rust
#[test]
fn test_shell_detection() {
    let shell = detect_shell();
    assert!(["zsh", "bash", "fish", "sh", "powershell"].contains(&shell.as_str()));
}

#[test]
fn test_shell_version_detection() {
    let version = detect_shell_version();
    assert!(version.is_some());
}
```

**Implementation** (`src/platform/shell.rs`)
- Read `$SHELL` environment variable
- Parse shell version from `--version` or `-c 'echo $ZSH_VERSION'`
- Detect Windows PowerShell vs PowerShell Core

#### 1.3: Utility Detection
**Contract Test First**
```rust
#[test]
fn test_gnu_coreutils_detection() {
    let has_gnu = has_gnu_coreutils();
    // On Linux, should be true; on macOS, might be false
    assert!(has_gnu == true || has_gnu == false);
}

#[test]
fn test_common_utility_availability() {
    let utils = detect_available_utilities();
    assert!(utils.contains_key("ls")); // Should always have ls
}
```

**Implementation** (`src/platform/utilities.rs`)
- Check for GNU vs BSD coreutils (test `ls --version`)
- Build map of common utilities (`grep`, `sed`, `awk`, `netstat`, etc.)
- Detect versions where relevant

#### 1.4: Integration with System Prompt
**Contract Test First** (`tests/backend_trait_contract.rs`)
```rust
#[tokio::test]
async fn test_system_prompt_includes_platform_context() {
    let prompt = build_system_prompt(&platform_ctx);
    assert!(prompt.contains("OS:"));
    assert!(prompt.contains("Shell:"));
    assert!(prompt.contains(&platform_ctx.os));
}
```

**Implementation** (`src/backends/mod.rs`)
- Update system prompt builder
- Include platform-specific hints

**Deliverables:**
- [ ] `src/platform/shell.rs` - Shell detection module
- [ ] `src/platform/utilities.rs` - Utility detection module
- [ ] Updated `src/platform/mod.rs` - Enhanced `PlatformContext`
- [ ] Contract tests passing
- [ ] Documentation updated

---

## Phase 2: Help Text Collection (3-5 days)

### Goals
- Collect documentation for commands (--help, man, tldr)
- Cache help text for performance
- Handle errors gracefully

### Test-First Tasks

#### 2.1: Command Extraction
**Contract Test First** (`tests/command_extraction_contract.rs`)
```rust
#[test]
fn test_extract_simple_command() {
    let cmds = extract_base_commands("ls -la");
    assert_eq!(cmds, vec!["ls"]);
}

#[test]
fn test_extract_piped_commands() {
    let cmds = extract_base_commands("netstat -ano | grep LISTEN");
    assert_eq!(cmds, vec!["netstat", "grep"]);
}

#[test]
fn test_extract_complex_pipeline() {
    let cmds = extract_base_commands("find . -name '*.rs' | xargs wc -l | sort -n");
    assert_eq!(cmds, vec!["find", "xargs", "wc", "sort"]);
}
```

**Implementation** (`src/validation/command_parser.rs`)
- Tokenize shell command
- Extract base commands from pipes, redirects, subshells
- Handle quoted strings

#### 2.2: Help Text Collection
**Contract Test First** (`tests/help_collection_contract.rs`)
```rust
#[tokio::test]
async fn test_collect_help_for_existing_command() {
    let help = CommandHelp::collect("ls").await;
    assert!(help.exists);
    assert!(help.path.is_some());
    assert!(help.help_text.is_some() || help.man_page.is_some());
}

#[tokio::test]
async fn test_collect_help_for_missing_command() {
    let help = CommandHelp::collect("nonexistent_cmd_12345").await;
    assert!(!help.exists);
    assert!(help.path.is_none());
}

#[tokio::test]
async fn test_help_text_truncation() {
    let help = CommandHelp::collect("git").await; // git has huge help
    if let Some(text) = help.help_text {
        assert!(text.len() < 10000); // Should be truncated
    }
}
```

**Implementation** (`src/validation/help_collector.rs`)
- Check command existence (`which` / `where`)
- Try `--help`, then `-h`, capture stdout
- Try `man <cmd>`, parse first section
- Try `tldr <cmd>` if available
- Truncate long output
- Handle timeouts (2s per command)

#### 2.3: Help Text Caching
**Contract Test First** (`tests/help_cache_contract.rs`)
```rust
#[tokio::test]
async fn test_cache_stores_help_text() {
    let cache = HelpCache::new();
    cache.set("ls", &help_data).await.unwrap();
    let cached = cache.get("ls").await.unwrap();
    assert_eq!(cached, Some(help_data));
}

#[tokio::test]
async fn test_cache_respects_ttl() {
    let cache = HelpCache::new();
    cache.set_with_ttl("ls", &help_data, Duration::from_millis(100)).await;
    tokio::time::sleep(Duration::from_millis(150)).await;
    let cached = cache.get("ls").await.unwrap();
    assert!(cached.is_none()); // Should be expired
}
```

**Implementation** (`src/validation/help_cache.rs`)
- Store help text in `~/.cache/caro/help/{command}.json`
- Include TTL (default 7 days)
- Validate cache freshness
- Handle cache corruption gracefully

**Deliverables:**
- [ ] `src/validation/command_parser.rs` - Command extraction
- [ ] `src/validation/help_collector.rs` - Help text collection
- [ ] `src/validation/help_cache.rs` - Caching layer
- [ ] Contract tests passing
- [ ] Performance benchmarks (<2s for 3 commands)

---

## Phase 3: Validation Module (5-7 days)

### Goals
- Implement LLM-based command validation
- Parse validation responses
- Handle corrections and suggestions

### Test-First Tasks

#### 3.1: Validation Prompt Builder
**Contract Test First** (`tests/validation_prompt_contract.rs`)
```rust
#[test]
fn test_build_validation_prompt() {
    let prompt = ValidationPrompt::build(
        &platform_ctx,
        "netstat -ano | grep LISTEN",
        &help_texts
    );
    
    assert!(prompt.contains("PLATFORM:"));
    assert!(prompt.contains("macos") || prompt.contains("linux"));
    assert!(prompt.contains("netstat -ano"));
    assert!(prompt.contains("COMMAND DOCUMENTATION:"));
}

#[test]
fn test_prompt_includes_relevant_help_text() {
    let prompt = ValidationPrompt::build(...);
    assert!(prompt.contains("netstat") || prompt.contains("man page"));
}
```

**Implementation** (`src/validation/prompt.rs`)
- Template for validation prompt
- Include platform context
- Include command and help text
- Format for JSON response

#### 3.2: Validation Response Parsing
**Contract Test First** (`tests/validation_response_contract.rs`)
```rust
#[test]
fn test_parse_valid_command_response() {
    let json = r#"{"valid": true, "issues": [], "corrected_command": null, 
                   "confidence": 0.95, "explanation": "Command is compatible"}"#;
    let result = parse_validation_response(json).unwrap();
    assert!(result.valid);
    assert_eq!(result.confidence, 0.95);
}

#[test]
fn test_parse_invalid_command_response() {
    let json = r#"{"valid": false, "issues": ["Flag -o not supported"], 
                   "corrected_command": "netstat -an | grep LISTEN", 
                   "confidence": 0.9, "explanation": "BSD netstat uses -an"}"#;
    let result = parse_validation_response(json).unwrap();
    assert!(!result.valid);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.corrected_command.unwrap(), "netstat -an | grep LISTEN");
}

#[test]
fn test_handle_malformed_response() {
    let json = "invalid json";
    let result = parse_validation_response(json);
    assert!(result.is_err());
}
```

**Implementation** (`src/validation/response_parser.rs`)
- Parse JSON response
- Handle missing fields gracefully
- Fallback strategies for malformed responses

#### 3.3: Command Validator
**Contract Test First** (`tests/command_validator_contract.rs`)
```rust
#[tokio::test]
async fn test_validate_compatible_command() {
    let validator = CommandValidator::new(platform_ctx, backend);
    let result = validator.validate_command("ls -la").await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_validate_incompatible_command() {
    let validator = CommandValidator::new(platform_ctx, backend);
    // On macOS, this should fail
    if cfg!(target_os = "macos") {
        let result = validator.validate_command("netstat -ano").await.unwrap();
        assert!(!result.valid);
        assert!(result.corrected_command.is_some());
    }
}

#[tokio::test]
async fn test_validation_timeout() {
    let validator = CommandValidator::with_timeout(Duration::from_millis(100));
    // Should timeout and return error
}
```

**Implementation** (`src/validation/validator.rs`)
- Implement `CommandValidator` struct
- Coordinate help collection and LLM validation
- Handle timeouts and errors
- Return structured validation result

**Deliverables:**
- [ ] `src/validation/prompt.rs` - Prompt builder
- [ ] `src/validation/response_parser.rs` - Response parsing
- [ ] `src/validation/validator.rs` - Main validation logic
- [ ] `src/validation/mod.rs` - Module exports
- [ ] Contract tests passing
- [ ] Integration tests passing

---

## Phase 4: Main Flow Integration (3-5 days)

### Goals
- Integrate validation into main execution flow
- Add user feedback and progress indicators
- Handle validation results (corrections, failures)

### Test-First Tasks

#### 4.1: Execution Flow Update
**Integration Test First** (`tests/validation_integration.rs`)
```rust
#[tokio::test]
async fn test_full_workflow_with_validation() {
    // Generate → Validate → Execute
    let result = execute_command_workflow("show listening ports").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflow_with_correction() {
    // Generate incompatible → Validate → Correct → Execute
    if cfg!(target_os = "macos") {
        // Mock backend to return "netstat -ano"
        let result = execute_command_workflow("show listening ports").await;
        // Should auto-correct to "netstat -an"
    }
}
```

**Implementation** (`src/main.rs`, `src/lib.rs`)
- Update `execute_workflow` function
- Add validation step between generation and safety check
- Handle validation results

#### 4.2: User Feedback
**Implementation** (`src/ui/feedback.rs` or inline in main.rs)
```rust
println!("🔍 Validating command compatibility...");
// Collect help text
println!("  → Checking command availability...");
// Validate with LLM
println!("  → Verifying platform compatibility...");
// Result
println!("✓ Command validated");
// Or
println!("⚠  Issues found: {}", issues.join(", "));
println!("📝 Suggested fix:\n  {}", corrected);
```

#### 4.3: Correction Handling
**E2E Test First** (`tests/e2e_cli_tests.rs`)
```rust
#[tokio::test]
async fn test_cli_with_auto_correction() {
    // Run CLI with a command that needs correction
    // Verify user is prompted
    // Verify corrected command is offered
}
```

**Implementation** (`src/cli/mod.rs`)
- Add `--auto-fix` flag for automatic corrections
- Add `--skip-validation` flag to bypass validation
- Interactive prompt for accepting corrections

**Deliverables:**
- [ ] Updated `src/main.rs` with validation step
- [ ] User feedback messages
- [ ] CLI flags for validation control
- [ ] Integration tests passing
- [ ] E2E tests passing

---

## Phase 5: Configuration & Optimization (2-3 days)

### Goals
- Add configuration options for validation
- Optimize performance with caching
- Add command whitelist for skipping validation

### Tasks

#### 5.1: Configuration
**Contract Test First** (`tests/config_contract.rs`)
```rust
#[test]
fn test_validation_config_defaults() {
    let config = ValidationConfig::default();
    assert!(config.enabled);
    assert_eq!(config.timeout_ms, 5000);
}

#[test]
fn test_load_validation_config_from_file() {
    let config = Config::load_from_file("test_config.toml").unwrap();
    assert_eq!(config.validation.enabled, true);
}
```

**Implementation** (`src/config/validation.rs`)
- Add `ValidationConfig` struct
- Add to main `Config` struct
- Parse from TOML file

#### 5.2: Performance Optimization
- [ ] Implement help text caching (already in Phase 2)
- [ ] Add command whitelist (skip validation for `ls`, `cd`, etc.)
- [ ] Parallel help text collection
- [ ] Benchmark validation performance
- [ ] Target: <5s for validation phase

#### 5.3: Error Handling
- [ ] Graceful degradation when help text unavailable
- [ ] Fallback when validation times out
- [ ] Clear error messages
- [ ] Logging for debugging

**Deliverables:**
- [ ] Configuration options implemented
- [ ] Performance benchmarks passing
- [ ] Error handling comprehensive
- [ ] Documentation updated

---

## Phase 6: Testing & Documentation (2-3 days)

### Tasks

#### 6.1: Comprehensive Testing
- [ ] All contract tests passing
- [ ] All integration tests passing
- [ ] Property-based tests for validation
- [ ] E2E scenarios for validation failures
- [ ] Performance benchmarks

#### 6.2: Documentation
- [ ] Update README.md with validation feature
- [ ] Add examples of validation in action
- [ ] Document configuration options
- [ ] Update CHANGELOG.md
- [ ] Add to user guide / docs site

#### 6.3: Code Review & Polish
- [ ] Code review by team
- [ ] Address feedback
- [ ] Run `make check` (fmt + lint + audit + test)
- [ ] Ensure <50MB binary size still met

**Deliverables:**
- [ ] Full test coverage
- [ ] Complete documentation
- [ ] Code review approved
- [ ] Ready for merge

---

## Rollout Strategy

### Stage 1: Feature Flag (Week 1)
- Merge code with feature flag disabled by default
- Allow early adopters to enable via config
- Monitor for issues

### Stage 2: Opt-In Beta (Week 2-3)
- Enable by default for beta users
- Collect feedback
- Fix critical issues

### Stage 3: General Availability (Week 4)
- Enable by default for all users
- Provide `--skip-validation` flag for override
- Monitor performance metrics

---

## Success Criteria

- [ ] 95%+ of platform-incompatible commands caught before execution
- [ ] Validation adds <5s to total execution time
- [ ] No false positives (valid commands marked invalid)
- [ ] User satisfaction: Clear feedback, helpful corrections
- [ ] Performance benchmarks met
- [ ] All tests passing
- [ ] Documentation complete

---

## Risk Mitigation

### Risk: Validation is too slow
**Mitigation:**
- Aggressive caching of help text
- Command whitelist for common safe commands
- Timeout with fallback to "skip validation"
- Parallel help text collection

### Risk: LLM provides incorrect corrections
**Mitigation:**
- Always show user the correction before executing
- Allow user to reject and use original command
- Collect feedback on correction quality
- Add `--skip-validation` escape hatch

### Risk: Help text collection is unreliable
**Mitigation:**
- Graceful fallback when help text unavailable
- Try multiple sources (--help, man, tldr)
- Don't block on help text collection failures

### Risk: Breaking existing users' workflows
**Mitigation:**
- Feature flag for gradual rollout
- Configuration to disable validation
- Clear error messages
- Backward compatibility maintained

---

## Future Enhancements (Post-MVP)

1. **Learning from corrections**: Store successful corrections to improve future generations
2. **Command equivalence database**: Pre-computed mappings (e.g., GNU → BSD flags)
3. **Offline validation**: Use local database of command signatures
4. **Shell-specific validation**: Different rules for bash vs zsh vs fish
5. **Version-aware validation**: Consider specific OS/utility versions
6. **Community-contributed patterns**: Allow users to share validation patterns

---

## Questions for Discussion

1. Should validation be enabled by default or opt-in initially?
   - **Recommendation**: Opt-in for first release, default-on after stabilization

2. How aggressive should auto-correction be?
   - **Recommendation**: Always show user, require confirmation unless `--auto-fix`

3. Should we validate every command or only after first failure?
   - **Recommendation**: Always validate, but use whitelist to skip common safe commands

4. What's the acceptable performance overhead?
   - **Recommendation**: <5s for validation, <10s total for generate + validate

5. How to handle edge cases (custom scripts, aliases)?
   - **Recommendation**: Document limitations, provide `--skip-validation` flag

---

## Implementation Notes

- Follow TDD strictly: RED → GREEN → REFACTOR
- Commit tests before implementation
- Run `make check` before every commit
- Update todo list as you progress
- Ask for help early if blocked
- Document assumptions and decisions

---

**Start Date**: TBD  
**Target Completion**: 3-4 weeks  
**Owner**: TBD
