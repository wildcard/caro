# cmdai CLI Quality Assurance Test Cases

## Overview

This document provides comprehensive QA test cases for the cmdai CLI tool. These test cases serve as both manual testing guidelines and specifications for automated E2E tests. They are organized by functional area and include expected behaviors, edge cases, and performance requirements.

## Test Environment Setup

### Prerequisites
- Rust toolchain installed and configured
- cmdai binary built (`cargo build --release`)
- Clean temporary directory for configuration isolation
- All automated tests passing

### Environment Variables
```bash
export CMDAI_CONFIG_DIR=/tmp/cmdai-test-config
export CMDAI_CACHE_DIR=/tmp/cmdai-test-cache
export RUST_LOG=debug  # For verbose testing
```

## Test Categories

---

## Category A: Core CLI Functionality

### A1: Help System
**Objective**: Verify comprehensive help information is displayed correctly

**Test Steps**:
```bash
cmdai --help
cmdai -h
```

**Expected Results**:
- [ ] Displays tool description and purpose
- [ ] Shows correct usage syntax: `cmdai [OPTIONS] [PROMPT]`
- [ ] Lists all command-line options with descriptions
- [ ] Includes shell type options (bash, zsh, fish, sh, powershell, cmd)
- [ ] Shows safety level options (strict, moderate, permissive)
- [ ] Lists output formats (json, yaml, plain)
- [ ] Contains configuration and verbose options
- [ ] Help text is clear and user-friendly
- [ ] No formatting issues or truncation

**Edge Cases**:
- Help display in narrow terminal windows
- Help with invalid options (should still show help)

**Performance**: Help should display instantly (< 100ms)

---

### A2: Version Information
**Objective**: Verify version information is displayed correctly

**Test Steps**:
```bash
cmdai --version
cmdai -V
```

**Expected Results**:
- [ ] Displays version in format "cmdai X.Y.Z"
- [ ] Version matches Cargo.toml version
- [ ] No additional output or error messages
- [ ] Clean, single-line output

**Edge Cases**:
- Version command with other conflicting options

**Performance**: Version should display instantly (< 50ms)

---

### A3: Basic Command Generation
**Objective**: Verify natural language to shell command conversion works

**Test Cases**:
```bash
# Simple file operations
cmdai "list files in current directory"
cmdai "show current date and time"
cmdai "display disk usage"
cmdai "find Python files"

# Directory operations  
cmdai "create a backup directory"
cmdai "navigate to home directory"
cmdai "show directory structure"

# System information
cmdai "show system information"
cmdai "display environment variables"
cmdai "check available memory"
```

**Expected Results**:
- [ ] Generates appropriate shell commands
- [ ] Commands are POSIX-compliant
- [ ] Output includes command and explanation
- [ ] No crash or error for valid inputs
- [ ] Commands are safe (no destructive operations without warnings)

**Edge Cases**:
- Very short prompts ("ls", "help")
- Very long prompts (1000+ characters)
- Prompts with special characters
- Prompts in different languages
- Ambiguous prompts

**Performance**: Command generation should complete within reasonable time

---

## Category B: Output Formats

### B1: JSON Output Format
**Objective**: Verify JSON output is well-formed and contains expected fields

**Test Steps**:
```bash
cmdai "list files" --output json
cmdai "show date" --output json
cmdai "complex command with multiple parameters" --output json
```

**Expected Results**:
- [ ] Valid JSON structure (parseable with jq or JSON parser)
- [ ] Contains required fields:
  - [ ] `generated_command`: The shell command
  - [ ] `explanation`: Human-readable explanation
  - [ ] `executed`: Boolean execution status
  - [ ] `shell_used`: Shell type
  - [ ] `output_format`: Format confirmation
  - [ ] `timing_info`: Performance metrics
- [ ] Optional fields handled gracefully:
  - [ ] `alternatives`: Alternative commands
  - [ ] `warnings`: Safety warnings
  - [ ] `debug_info`: Debugging information
- [ ] Proper escaping of special characters
- [ ] Consistent field naming and types

**Edge Cases**:
- Commands with quotes, newlines, or special characters
- Empty or null values in fields
- Large command outputs

**Validation Commands**:
```bash
cmdai "test" --output json | jq .
cmdai "test" --output json | python -m json.tool
```

---

### B2: YAML Output Format
**Objective**: Verify YAML output is well-formed and human-readable

**Test Steps**:
```bash
cmdai "list files" --output yaml
cmdai "show system info" --output yaml
cmdai "multi-step process" --output yaml
```

**Expected Results**:
- [ ] Valid YAML structure
- [ ] Human-readable formatting with proper indentation
- [ ] Same data fields as JSON format
- [ ] Proper handling of strings, lists, and objects
- [ ] No JSON artifacts (no curly braces or JSON syntax)

**Edge Cases**:
- Commands with YAML-sensitive characters (colons, quotes)
- Multi-line explanations
- Lists and nested structures

---

### B3: Plain Text Output (Default)
**Objective**: Verify default plain text output is user-friendly

**Test Steps**:
```bash
cmdai "list files"
cmdai "show date"
cmdai "complex operation"
```

**Expected Results**:
- [ ] Clean, readable format
- [ ] Clear section headers (Command:, Explanation:, etc.)
- [ ] No JSON/YAML artifacts
- [ ] Proper line breaks and spacing
- [ ] Color coding (if terminal supports it)
- [ ] Alternative commands displayed when available

**Edge Cases**:
- Output in non-color terminals
- Very long commands or explanations
- Commands with special formatting needs

---

## Category C: Shell Types and Backend Selection

### C1: Shell Type Selection
**Objective**: Verify all supported shell types are handled correctly

**Test Matrix**:
```bash
# Test each shell type
cmdai "list files" --shell bash
cmdai "list files" --shell zsh
cmdai "list files" --shell fish
cmdai "list files" --shell sh
cmdai "list files" --shell powershell  # Windows
cmdai "list files" --shell cmd         # Windows
```

**Expected Results**:
- [ ] Each shell type generates appropriate syntax
- [ ] No errors for any supported shell
- [ ] Commands are shell-specific when relevant
- [ ] Consistent behavior across shells

**Shell-Specific Validation**:
- [ ] **Bash**: Uses bash-specific features when beneficial
- [ ] **Zsh**: Compatible with zsh extensions
- [ ] **Fish**: Uses fish syntax for complex operations
- [ ] **sh**: POSIX-compliant, no bash-isms
- [ ] **PowerShell**: Uses PowerShell cmdlets and syntax
- [ ] **CMD**: Uses Windows batch syntax

**Edge Cases**:
- Invalid shell type (should show warning or error)
- Mixed case shell names ("BASH", "Zsh")
- Shell auto-detection when not specified

---

### C2: Backend Selection and Fallback
**Objective**: Verify backend selection logic works correctly

**Test Steps**:
```bash
# Test with verbose mode to see backend info
cmdai "test command" --verbose
cmdai "test command" --verbose --output json
```

**Expected Results**:
- [ ] Shows which backend is used (mock in debug, embedded in release)
- [ ] Backend info includes model name and performance metrics
- [ ] Fallback chain works when primary backend unavailable
- [ ] Clear indication of which backend generated the command

**Platform-Specific Tests**:
- [ ] **Apple Silicon**: Should prefer MLX backend when available
- [ ] **Other platforms**: Should use CPU backend
- [ ] **Remote backends**: Should fallback to embedded when network unavailable

---

## Category D: Safety Validation and Confirmation

### D1: Safety Level Configuration
**Objective**: Verify safety levels affect command generation and validation

**Test Matrix**:
```bash
# Test with potentially dangerous commands at different safety levels
cmdai "delete temporary files" --safety strict
cmdai "delete temporary files" --safety moderate
cmdai "delete temporary files" --safety permissive

# Test with clearly safe commands
cmdai "list files" --safety strict
cmdai "show date" --safety moderate
cmdai "check disk usage" --safety permissive
```

**Expected Results**:
- [ ] **Strict mode**: More conservative, extensive warnings
- [ ] **Moderate mode**: Balanced approach with reasonable warnings
- [ ] **Permissive mode**: Fewer restrictions, minimal warnings
- [ ] All modes still prevent critical dangerous operations
- [ ] Clear explanation of why commands are blocked/allowed

**Dangerous Command Tests**:
```bash
# These should be blocked or heavily warned at all safety levels
cmdai "rm -rf /" --safety permissive
cmdai "format hard drive" --safety permissive
cmdai "delete all files" --safety moderate
```

---

### D2: User Confirmation Flows
**Objective**: Verify confirmation prompts work correctly

**Test Steps**:
```bash
# Test auto-confirmation
cmdai "potentially dangerous command" --confirm

# Test with different risk levels
cmdai "rm *.tmp" --safety strict
cmdai "sudo command" --safety moderate
```

**Expected Results**:
- [ ] High-risk commands prompt for confirmation
- [ ] `--confirm` flag bypasses prompts
- [ ] Clear explanation of risks before confirmation
- [ ] Ability to cancel safely
- [ ] Different confirmation levels for different risk levels

---

## Category E: Configuration Management

### E1: Configuration Display
**Objective**: Verify configuration information is displayed correctly

**Test Steps**:
```bash
cmdai --show-config
```

**Expected Results**:
- [ ] Shows configuration file location
- [ ] Indicates if config file exists
- [ ] Displays current settings:
  - [ ] Default shell
  - [ ] Safety level
  - [ ] Log level
  - [ ] Cache settings
  - [ ] Backend preferences
- [ ] Clear, readable format

---

### E2: Configuration File Handling
**Objective**: Verify configuration files are loaded and applied correctly

**Test Steps**:
```bash
# Create test configuration
mkdir -p ~/.config/cmdai
cat > ~/.config/cmdai/config.toml << EOF
default_shell = "zsh"
safety_level = "strict"
log_level = "debug"
EOF

# Test configuration is applied
cmdai --show-config
cmdai "test command"  # Should use zsh and strict safety
```

**Expected Results**:
- [ ] Configuration file is loaded automatically
- [ ] Settings are applied to command generation
- [ ] CLI arguments override config file settings
- [ ] Invalid config values are handled gracefully

---

## Category F: Error Handling and Edge Cases

### F1: Input Validation
**Objective**: Verify robust handling of invalid inputs

**Test Cases**:
```bash
# Empty inputs
cmdai ""
cmdai "   "  # Whitespace only

# Very long inputs
cmdai "$(printf 'a%.0s' {1..10000})"  # 10,000 character input

# Special characters
cmdai "test with unicode: 🚀 ñ é 中文"
cmdai "test with control chars: $(printf '\x00\x01\x02')"
```

**Expected Results**:
- [ ] No crashes or panics
- [ ] Meaningful error messages for invalid inputs
- [ ] Graceful handling of edge cases
- [ ] Proper encoding handling for unicode

---

### F2: System Resource Handling
**Objective**: Verify tool handles system constraints gracefully

**Test Cases**:
```bash
# Disk space constraints
# Memory constraints  
# Network timeouts (for remote backends)
# File permission issues
```

**Expected Results**:
- [ ] Graceful degradation when resources are limited
- [ ] Clear error messages for resource issues
- [ ] No data corruption or partial writes

---

## Category G: Performance and Reliability

### G1: Performance Benchmarks
**Objective**: Verify tool meets performance requirements

**Performance Targets**:
- [ ] **Startup time**: < 200ms for --help or --version
- [ ] **Command generation**: < 2s for simple commands
- [ ] **Memory usage**: < 100MB for typical operations
- [ ] **Concurrent requests**: Handle 5+ simultaneous requests

**Test Commands**:
```bash
time cmdai --version
time cmdai "list files"
time cmdai "complex multi-step operation"
```

---

### G2: Reliability and Consistency
**Objective**: Verify consistent behavior across runs

**Test Steps**:
```bash
# Run same command multiple times
for i in {1..10}; do
    cmdai "show current directory" --output json
done | jq -r '.generated_command' | sort -u | wc -l
# Should show 1 (identical outputs)
```

**Expected Results**:
- [ ] Consistent command generation for identical inputs
- [ ] Stable performance across multiple runs
- [ ] No memory leaks or resource accumulation

---

## Test Automation and CI Integration

### Automated Test Execution
```bash
# Run all E2E tests
cargo test e2e_

# Run specific test categories
cargo test e2e_core_functionality
cargo test e2e_output_formats
cargo test e2e_safety_validation

# Run performance tests
cargo test e2e_performance --release

# Run smoke tests (critical functionality only)
cargo test e2e_smoke_test_suite
```

### CI/CD Pipeline Integration
- [ ] All E2E tests pass before merging
- [ ] Performance regression detection
- [ ] Cross-platform testing (Linux, macOS, Windows)
- [ ] Different Rust versions testing

---

## Test Data and Scenarios

### Common Test Prompts
```bash
# File operations
"list all files in current directory"
"find all Python files recursively"
"create a backup of important files"
"show hidden files and directories"

# System information
"show current date and time"
"display system information"
"check available disk space"
"show running processes"

# Development tasks
"show git status"
"find files modified in last 24 hours"
"search for text in source code"
"run tests and show results"

# Potentially dangerous operations
"delete temporary files"
"remove all log files"
"change file permissions"
"kill running processes"
```

### Expected Safe Commands
```bash
ls -la                    # list files
date                      # show date
df -h                     # disk usage
ps aux                    # show processes
git status               # git status
find . -name "*.py"      # find Python files
```

### Commands Requiring Warnings
```bash
rm -rf /tmp/*            # delete temp files
chmod 777 file           # change permissions
sudo su                  # privilege escalation
killall process          # kill processes
```

---

## Maintenance and Updates

### Adding New Test Cases
1. **Identify the functional area** (A-G categories)
2. **Define objective and success criteria**
3. **Create both manual and automated tests**
4. **Update this documentation**
5. **Add corresponding E2E test in `tests/e2e_cli_tests.rs`**

### Test Case Review Schedule
- **Weekly**: Review failed tests and update expectations
- **Monthly**: Add new test cases based on user feedback
- **Before releases**: Full regression testing of all categories
- **After major changes**: Update affected test categories

### Tools and Utilities
- **JSON validation**: `jq`, `python -m json.tool`
- **YAML validation**: `yq`, `python -c "import yaml; yaml.safe_load(open('file'))"`
- **Performance measurement**: `time`, `hyperfine`
- **Memory profiling**: `valgrind`, `heaptrack`
- **Load testing**: Custom scripts for concurrent execution

---

## Conclusion

These QA test cases provide comprehensive coverage of cmdai CLI functionality and serve as both manual testing guidelines and specifications for automated E2E tests. They should be updated regularly as the tool evolves and new features are added.

For questions or updates to these test cases, please refer to the project's issue tracker or development documentation.