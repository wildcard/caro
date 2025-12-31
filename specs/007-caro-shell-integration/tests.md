# Test Plan: Shell Integration

## Overview

This document defines the comprehensive test strategy for the Caro shell integration feature, covering unit tests, integration tests, performance benchmarks, and manual validation procedures.

## Test Categories

| Category | Scope | Automation | Frequency |
|----------|-------|------------|-----------|
| Unit Tests | Individual functions | Automated | Every commit |
| Integration Tests | Component interaction | Automated | Every PR |
| Shell Tests | Per-shell behavior | Semi-automated | Every PR |
| Performance Tests | Latency/throughput | Automated | Weekly/Release |
| Security Tests | Vulnerability scanning | Automated | Every PR |
| Manual Tests | UX validation | Manual | Pre-release |

---

## Unit Tests

### Rust Unit Tests (`src/shell/`)

#### IPC Protocol Tests

```rust
#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn test_session_start_serialization() {
        let msg = ShellMessage::SessionStart {
            shell_type: ShellType::Bash,
            pid: 12345,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"SessionStart\""));
        assert!(json.contains("\"shell_type\":\"bash\""));
        assert!(json.contains("\"pid\":12345"));

        // Round-trip
        let decoded: ShellMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            ShellMessage::SessionStart { shell_type, pid } => {
                assert_eq!(shell_type, ShellType::Bash);
                assert_eq!(pid, 12345);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_preexec_with_special_chars() {
        let msg = ShellMessage::PreExec {
            command: "echo \"hello\\nworld\" | grep 'o'".to_string(),
            cwd: "/home/user".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ShellMessage = serde_json::from_str(&json).unwrap();

        match decoded {
            ShellMessage::PreExec { command, .. } => {
                assert_eq!(command, "echo \"hello\\nworld\" | grep 'o'");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_invalid_json_handling() {
        let invalid = "{ invalid json }";
        let result: Result<ShellMessage, _> = serde_json::from_str(invalid);
        assert!(result.is_err());
    }
}
```

#### Policy Engine Tests

```rust
#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn test_safety_level_off() {
        let policy = PolicyEngine::new(SafetyLevel::Off);

        // All commands allowed in off mode
        assert!(policy.evaluate("rm -rf /").allow);
        assert!(policy.evaluate(":(){ :|:& };:").allow);
    }

    #[test]
    fn test_safety_level_passive() {
        let policy = PolicyEngine::new(SafetyLevel::Passive);

        // Critical commands warn but allow
        let result = policy.evaluate("rm -rf /");
        assert!(result.allow);
        assert!(!result.warnings.is_empty());

        // Normal commands pass silently
        let result = policy.evaluate("ls -la");
        assert!(result.allow);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_safety_level_active() {
        let policy = PolicyEngine::new(SafetyLevel::Active);

        // Critical commands blocked
        let result = policy.evaluate("rm -rf /");
        assert!(!result.allow);

        // High-risk commands require confirmation
        let result = policy.evaluate("sudo su");
        assert!(result.allow);
        assert!(result.require_confirmation);

        // Normal commands allowed
        let result = policy.evaluate("ls -la");
        assert!(result.allow);
        assert!(!result.require_confirmation);
    }

    #[test]
    fn test_blocklist() {
        let mut policy = PolicyEngine::new(SafetyLevel::Off);
        policy.add_blocklist_pattern(r"^docker\s+rm\s+-f");

        let result = policy.evaluate("docker rm -f container");
        assert!(!result.allow);

        let result = policy.evaluate("docker ps");
        assert!(result.allow);
    }

    #[test]
    fn test_allowlist() {
        let mut policy = PolicyEngine::new(SafetyLevel::Active);
        policy.add_allowlist_pattern(r"^/safe/script\.sh$");

        // Normally blocked command
        let result = policy.evaluate("rm -rf ~");
        assert!(!result.allow);

        // Allowlisted command
        let result = policy.evaluate("/safe/script.sh");
        assert!(result.allow);
    }

    #[test]
    fn test_directory_override() {
        let mut policy = PolicyEngine::new(SafetyLevel::Active);
        policy.set_directory_policy("/tmp".into(), SafetyLevel::Off);

        // In /tmp, everything allowed
        let result = policy.evaluate_in_dir("rm -rf *", Path::new("/tmp"));
        assert!(result.allow);

        // Elsewhere, normal rules apply
        let result = policy.evaluate_in_dir("rm -rf *", Path::new("/home"));
        assert!(result.require_confirmation);
    }
}
```

#### Fix-It Engine Tests

```rust
#[cfg(test)]
mod fixit_tests {
    use super::*;

    #[test]
    fn test_typo_correction() {
        let engine = FixItEngine::new();

        let fix = engine.suggest_fix("gti status", 127, Some("command not found: gti"));
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().corrected_command, "git status");
    }

    #[test]
    fn test_sudo_suggestion() {
        let engine = FixItEngine::new();

        let fix = engine.suggest_fix(
            "apt install vim",
            1,
            Some("Permission denied"),
        );
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().corrected_command, "sudo apt install vim");
    }

    #[test]
    fn test_no_fix_for_success() {
        let engine = FixItEngine::new();

        let fix = engine.suggest_fix("ls -la", 0, None);
        assert!(fix.is_none());
    }

    #[test]
    fn test_custom_typos() {
        let mut engine = FixItEngine::new();
        engine.add_typo("myalais", "myalias");

        let fix = engine.suggest_fix("myalais arg", 127, Some("command not found"));
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().corrected_command, "myalias arg");
    }

    #[test]
    fn test_confidence_threshold() {
        let engine = FixItEngine::with_threshold(0.9);

        // Low confidence fix should be filtered
        let fix = engine.suggest_fix("some random command", 1, None);
        assert!(fix.is_none() || fix.unwrap().confidence >= 0.9);
    }
}
```

#### Secret Redaction Tests

```rust
#[cfg(test)]
mod redaction_tests {
    use super::*;

    #[test]
    fn test_api_key_redaction() {
        let input = "curl -H 'Authorization: Bearer sk_live_abc123'";
        let output = redact_secrets(input);
        assert!(!output.contains("sk_live_abc123"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_password_redaction() {
        let input = "mysql -u root -pMyS3cr3t!";
        let output = redact_secrets(input);
        assert!(!output.contains("MyS3cr3t!"));
    }

    #[test]
    fn test_aws_key_redaction() {
        let input = "export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let output = redact_secrets(input);
        assert!(!output.contains("wJalrXUtnFEMI"));
    }

    #[test]
    fn test_url_credential_redaction() {
        let input = "git clone https://user:password@github.com/repo.git";
        let output = redact_secrets(input);
        assert!(!output.contains("password"));
    }

    #[test]
    fn test_safe_content_unchanged() {
        let input = "ls -la && echo hello";
        let output = redact_secrets(input);
        assert_eq!(input, output);
    }
}
```

---

## Integration Tests

### IPC Integration Tests

```rust
#[cfg(test)]
mod ipc_integration_tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_daemon_lifecycle() {
        let socket_path = temp_socket_path();
        let daemon = CaroDaemon::new(&socket_path).await.unwrap();

        // Start daemon
        let handle = tokio::spawn(daemon.run());

        // Connect as client
        let client = CaroClient::connect(&socket_path).await.unwrap();

        // Send session start
        let response = client.send(ShellMessage::SessionStart {
            shell_type: ShellType::Bash,
            pid: std::process::id(),
        }).await.unwrap();

        // Verify response
        assert!(matches!(response, ShellMessage::Ack { .. }));

        // Cleanup
        client.close().await;
        handle.abort();
    }

    #[tokio::test]
    async fn test_preexec_flow() {
        let (daemon, client) = setup_test_env().await;

        // Start session
        client.start_session(ShellType::Zsh).await.unwrap();

        // Send preexec
        let response = client.preexec("ls -la", "/home/user").await.unwrap();

        assert!(response.allow);
        assert!(response.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_dangerous_command_blocked() {
        let (daemon, client) = setup_test_env_with_config(SafetyLevel::Active).await;

        client.start_session(ShellType::Bash).await.unwrap();

        let response = client.preexec("rm -rf /", "/home/user").await.unwrap();

        assert!(!response.allow);
        assert!(!response.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_postcmd_with_fix_suggestion() {
        let (daemon, client) = setup_test_env().await;

        client.start_session(ShellType::Bash).await.unwrap();

        // Simulate failed command
        let response = client.postcmd(127, 100).await.unwrap();

        // Should get suggestion for command not found
        // (Note: actual suggestion depends on command history in session)
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let socket_path = temp_socket_path();
        // Don't start daemon - socket exists but nothing listening

        let result = tokio::time::timeout(
            Duration::from_millis(100),
            CaroClient::connect(&socket_path),
        ).await;

        // Should timeout or fail gracefully
        assert!(result.is_err() || result.unwrap().is_err());
    }
}
```

### Session Management Tests

```rust
#[tokio::test]
async fn test_multiple_sessions() {
    let (daemon, _) = setup_test_env().await;

    // Create multiple clients (simulating multiple shell sessions)
    let clients: Vec<_> = (0..10)
        .map(|i| {
            let client = CaroClient::connect(&daemon.socket_path()).await.unwrap();
            client.start_session(ShellType::Bash).await.unwrap();
            client
        })
        .collect();

    // Each should work independently
    for client in &clients {
        let response = client.preexec("echo test", "/tmp").await.unwrap();
        assert!(response.allow);
    }

    // Cleanup
    for client in clients {
        client.close().await;
    }
}

#[tokio::test]
async fn test_session_cleanup_on_disconnect() {
    let (daemon, client) = setup_test_env().await;

    client.start_session(ShellType::Bash).await.unwrap();

    // Get session count
    let initial_sessions = daemon.session_count();
    assert_eq!(initial_sessions, 1);

    // Disconnect
    client.close().await;

    // Wait for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Session should be cleaned up
    let final_sessions = daemon.session_count();
    assert_eq!(final_sessions, 0);
}
```

---

## Shell-Specific Tests

### bash Integration Tests

```bash
#!/usr/bin/env bash
# tests/shell/bash_test.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

# Setup
setup_test_env "bash"

test_hooks_installed() {
    echo "Testing bash hooks are installed..."

    # Source our integration
    source "$CARO_CONFIG/shell/bash.init"

    # Check DEBUG trap is set
    trap_output=$(trap -p DEBUG)
    assert_contains "$trap_output" "__caro_preexec"

    # Check PROMPT_COMMAND is set
    assert_contains "$PROMPT_COMMAND" "__caro"

    echo "✓ Hooks installed correctly"
}

test_preexec_fires() {
    echo "Testing preexec fires..."

    source "$CARO_CONFIG/shell/bash.init"

    # Set a marker variable that preexec should set
    export __test_preexec_fired=0
    __caro_preexec() {
        export __test_preexec_fired=1
    }

    # Run a command (in subshell to trigger trap)
    ( true )

    # Note: DEBUG trap fires in current shell, so we check directly
    assert_equals "$__test_preexec_fired" "1"

    echo "✓ Preexec fires correctly"
}

test_postcmd_captures_exit_code() {
    echo "Testing postcmd captures exit code..."

    source "$CARO_CONFIG/shell/bash.init"

    # Override postcmd to capture
    __captured_exit_code=""
    __caro_postcmd() {
        __captured_exit_code=$?
    }
    PROMPT_COMMAND="__caro_postcmd"

    # Run failing command
    (exit 42)
    eval "$PROMPT_COMMAND"

    assert_equals "$__captured_exit_code" "42"

    echo "✓ Exit code captured correctly"
}

test_keybindings() {
    echo "Testing keybindings..."

    source "$CARO_CONFIG/shell/bash.init"

    # Check bindings exist
    bindings=$(bind -X 2>/dev/null || true)
    # Note: bind -X requires bash 4.3+

    echo "✓ Keybindings configured (manual verification recommended)"
}

test_non_interactive_safe() {
    echo "Testing non-interactive shells don't load hooks..."

    # Run in non-interactive mode
    output=$(bash -c 'source '"$CARO_CONFIG/shell/bash.init"' 2>&1; echo $PROMPT_COMMAND')

    # PROMPT_COMMAND should be empty or not contain caro
    if [[ "$output" == *"__caro"* ]]; then
        fail "Hooks loaded in non-interactive shell!"
    fi

    echo "✓ Non-interactive shells are safe"
}

test_disable_env_var() {
    echo "Testing CARO_DISABLE works..."

    CARO_DISABLE=1 bash -i -c 'source '"$CARO_CONFIG/shell/bash.init"' 2>&1; echo $PROMPT_COMMAND' | {
        read output
        if [[ "$output" == *"__caro"* ]]; then
            fail "Hooks loaded despite CARO_DISABLE=1!"
        fi
    }

    echo "✓ CARO_DISABLE works correctly"
}

# Run tests
test_hooks_installed
test_preexec_fires
test_postcmd_captures_exit_code
test_keybindings
test_non_interactive_safe
test_disable_env_var

echo ""
echo "All bash tests passed! ✓"
```

### zsh Integration Tests

```zsh
#!/usr/bin/env zsh
# tests/shell/zsh_test.zsh

set -e

SCRIPT_DIR="${0:A:h}"
source "$SCRIPT_DIR/test_helpers.zsh"

setup_test_env "zsh"

test_hooks_installed() {
    echo "Testing zsh hooks are installed..."

    source "$CARO_CONFIG/shell/zsh.init"

    # Check preexec hook
    if (( ${+functions[__caro_preexec]} == 0 )); then
        fail "preexec hook not defined"
    fi

    # Check precmd hook
    if (( ${+functions[__caro_precmd]} == 0 )); then
        fail "precmd hook not defined"
    fi

    echo "✓ Hooks installed correctly"
}

test_zle_widgets() {
    echo "Testing ZLE widgets..."

    source "$CARO_CONFIG/shell/zsh.init"

    # Check widget exists
    zle -l | grep -q "__caro_invoke_widget" || fail "Widget not defined"

    echo "✓ ZLE widgets configured"
}

test_add_zsh_hook_used() {
    echo "Testing add-zsh-hook is used..."

    source "$CARO_CONFIG/shell/zsh.init"

    # Check hooks are registered via add-zsh-hook
    [[ "${preexec_functions[(r)__caro_preexec]}" == "__caro_preexec" ]] || \
        fail "preexec not in hook array"

    [[ "${precmd_functions[(r)__caro_precmd_hook]}" == "__caro_precmd_hook" ]] || \
        fail "precmd not in hook array"

    echo "✓ add-zsh-hook used correctly"
}

# Run tests
test_hooks_installed
test_zle_widgets
test_add_zsh_hook_used

echo ""
echo "All zsh tests passed! ✓"
```

### fish Integration Tests

```fish
#!/usr/bin/env fish
# tests/shell/fish_test.fish

set SCRIPT_DIR (dirname (status filename))
source "$SCRIPT_DIR/test_helpers.fish"

function setup
    setup_test_env "fish"
end

function test_events_registered
    echo "Testing fish events are registered..."

    source "$CARO_CONFIG/shell/fish/conf.d/caro.fish"

    # Check preexec function exists
    if not functions -q __caro_preexec
        fail "preexec function not defined"
    end

    # Check postexec function exists
    if not functions -q __caro_postexec
        fail "postexec function not defined"
    end

    echo "✓ Events registered correctly"
end

function test_keybindings
    echo "Testing fish keybindings..."

    source "$CARO_CONFIG/shell/fish/conf.d/caro.fish"

    # Check binding exists
    if not bind | grep -q "caro_invoke"
        fail "Invoke keybinding not set"
    end

    echo "✓ Keybindings configured"
end

function test_cmd_duration
    echo "Testing CMD_DURATION available..."

    # fish provides this automatically
    if not set -q CMD_DURATION
        # CMD_DURATION only set after a command runs
        echo "Note: CMD_DURATION checked (only available after command execution)"
    end

    echo "✓ CMD_DURATION accessible"
end

# Run tests
setup
test_events_registered
test_keybindings
test_cmd_duration

echo ""
echo "All fish tests passed! ✓"
```

---

## Performance Tests

### Startup Latency Benchmark

```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{criterion_group, criterion_main, Criterion};

    fn benchmark_shell_init(c: &mut Criterion) {
        c.bench_function("bash_init_source", |b| {
            b.iter(|| {
                // Measure time to source bash.init
                std::process::Command::new("bash")
                    .arg("-c")
                    .arg("source ~/.config/caro/shell/bash.init")
                    .output()
                    .expect("Failed to run bash")
            })
        });

        c.bench_function("zsh_init_source", |b| {
            b.iter(|| {
                std::process::Command::new("zsh")
                    .arg("-c")
                    .arg("source ~/.config/caro/shell/zsh.init")
                    .output()
                    .expect("Failed to run zsh")
            })
        });
    }

    fn benchmark_ipc_roundtrip(c: &mut Criterion) {
        // Setup daemon in background
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (daemon, client) = rt.block_on(setup_test_env());

        c.bench_function("ipc_preexec_roundtrip", |b| {
            b.iter(|| {
                rt.block_on(client.preexec("ls -la", "/tmp"))
            })
        });
    }

    criterion_group!(benches, benchmark_shell_init, benchmark_ipc_roundtrip);
    criterion_main!(benches);
}
```

### Performance Acceptance Criteria

| Metric | Target | Maximum |
|--------|--------|---------|
| Shell init time | < 20ms | < 50ms |
| IPC roundtrip | < 5ms | < 10ms |
| Safety validation | < 2ms | < 5ms |
| Fix suggestion | < 50ms | < 100ms |
| LLM suggestion | < 2000ms | < 5000ms |

### Load Testing

```bash
#!/usr/bin/env bash
# tests/performance/load_test.sh

# Simulate many concurrent shell sessions

SESSIONS=50
COMMANDS_PER_SESSION=100

start_session() {
    local id=$1
    for i in $(seq 1 $COMMANDS_PER_SESSION); do
        # Send preexec
        echo '{"type":"PreExec","command":"ls -la","cwd":"/tmp"}' | \
            timeout 0.1 nc -U "$CARO_SOCKET" >/dev/null 2>&1

        # Small delay to simulate typing
        sleep 0.01
    done
}

# Start sessions in parallel
for i in $(seq 1 $SESSIONS); do
    start_session $i &
done

wait

echo "Load test completed: $SESSIONS sessions × $COMMANDS_PER_SESSION commands"
```

---

## Security Tests

### Static Analysis

```bash
# Run in CI
cargo audit                    # Check for vulnerabilities
cargo clippy -- -D warnings    # Lint for security issues
cargo deny check               # Check dependencies
```

### Fuzz Testing

```rust
#[cfg(test)]
mod fuzz_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_command_validation(command in ".*") {
            // Should never panic
            let _ = validate_command(&command);
        }

        #[test]
        fn fuzz_redaction(input in ".*") {
            // Should never panic
            let _ = redact_secrets(&input);
        }

        #[test]
        fn fuzz_json_parsing(json in ".*") {
            // Should handle invalid JSON gracefully
            let _: Result<ShellMessage, _> = serde_json::from_str(&json);
        }
    }
}
```

### Permission Tests

```bash
#!/usr/bin/env bash
# tests/security/permissions_test.sh

test_socket_permissions() {
    local socket="$XDG_RUNTIME_DIR/caro-$UID.sock"

    if [[ -S "$socket" ]]; then
        local perms=$(stat -c %a "$socket" 2>/dev/null || stat -f %Lp "$socket")
        assert_equals "$perms" "600" "Socket permissions should be 600"
    fi
}

test_config_dir_permissions() {
    local config_dir="$XDG_CONFIG_HOME/caro"

    if [[ -d "$config_dir" ]]; then
        local perms=$(stat -c %a "$config_dir" 2>/dev/null || stat -f %Lp "$config_dir")
        assert_equals "$perms" "700" "Config dir permissions should be 700"
    fi
}

test_no_world_readable() {
    find "$XDG_CONFIG_HOME/caro" -perm -004 -type f | while read file; do
        fail "World-readable file found: $file"
    done
}
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Shell Integration Tests

on:
  push:
    paths:
      - 'src/shell/**'
      - 'shell-scripts/**'
      - 'tests/shell/**'
  pull_request:
    paths:
      - 'src/shell/**'
      - 'shell-scripts/**'
      - 'tests/shell/**'

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --package caro --lib shell

  shell-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        shell: [bash, zsh, fish]
    steps:
      - uses: actions/checkout@v4
      - name: Install shells
        run: |
          sudo apt-get update
          sudo apt-get install -y ${{ matrix.shell }}
      - name: Run ${{ matrix.shell }} tests
        run: tests/shell/${{ matrix.shell }}_test.*

  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --package caro -- shell
      - name: Check performance thresholds
        run: scripts/check_perf_thresholds.sh

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo audit
      - run: cargo clippy -- -D warnings
```

---

## Manual Test Checklist

### Pre-Release Validation

#### Installation
- [ ] Fresh install on clean system works
- [ ] Upgrade from previous version preserves config
- [ ] Uninstall removes all traces
- [ ] Re-install after uninstall works

#### Shells
- [ ] bash 4.x: hooks fire correctly
- [ ] bash 5.x: hooks fire correctly
- [ ] zsh 5.x: hooks fire correctly
- [ ] fish 3.x: hooks fire correctly
- [ ] Non-interactive scripts unaffected

#### UX
- [ ] Ctrl+X Ctrl+C opens Caro prompt
- [ ] Esc Esc applies fix suggestion
- [ ] Dangerous command shows warning
- [ ] Blocked command shows clear message
- [ ] Fix suggestions appear after failures

#### Integration
- [ ] Works with oh-my-zsh
- [ ] Works with prezto
- [ ] Works with fisher
- [ ] Custom prompts not broken
- [ ] PS1/PROMPT unchanged

#### Edge Cases
- [ ] Very long commands handled
- [ ] Commands with special characters work
- [ ] UTF-8 content preserved
- [ ] Network timeout degrades gracefully
- [ ] Daemon restart doesn't break shells
