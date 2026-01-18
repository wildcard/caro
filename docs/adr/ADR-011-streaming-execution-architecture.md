# ADR-011: Streaming Execution Architecture

**Status**: Proposed

**Date**: 2026-01-18

**Authors**: @wildcard

**Target**: Community

## Context

Caro currently executes shell commands synchronously using `std::process::Command::output()`, which blocks until the entire command completes. This creates a poor user experience for long-running commands:

### Current Architecture (Problem)

```rust
// src/execution/executor.rs (lines 35-80)
pub fn execute(&self, command: &str) -> Result<ExecutionResult, ExecutorError> {
    let start_time = Instant::now();
    let output = cmd.output().map_err(...)?;  // BLOCKING - waits for full completion
    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    // Timeout checked AFTER execution (cannot prevent hanging)
    if let Some(timeout) = self.timeout_ms {
        if execution_time_ms > timeout {
            return Err(ExecutorError::Timeout(timeout));
        }
    }
}
```

### Problems Identified

1. **No streaming**: Output captured only after process exit
2. **No preemptive timeout**: Cannot kill commands exceeding time limit
3. **No cancellation**: Ctrl+C kills caro, not the child process cleanly
4. **No progress feedback**: Users cannot tell if command is running
5. **Memory concerns**: Large outputs buffered entirely in memory

### Triggering Incident

User reported `find . -type f -size +100M` from home directory appeared "stuck" - the command was running but produced no output for 30+ seconds, causing user confusion and force-quit.

## Decision

Implement async streaming execution using Tokio's process module with:

1. **Real-time output streaming** via piped stdout/stderr
2. **Preemptive timeout enforcement** with process termination
3. **Graceful cancellation** via signal handling
4. **Activity indicator** for commands with no output
5. **Partial result preservation** on timeout/cancellation

### New Architecture

```rust
// Proposed: src/execution/streaming.rs

use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct StreamingExecutor {
    timeout: Duration,
    show_progress: bool,
}

pub struct StreamingResult {
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub execution_time_ms: u64,
    pub terminated_by: TerminationReason,
}

pub enum TerminationReason {
    Completed,      // Normal exit
    Timeout,        // Exceeded time limit
    Cancelled,      // User Ctrl+C
    Error(String),  // Process error
}

impl StreamingExecutor {
    pub async fn execute(&self, command: &str) -> Result<StreamingResult, ExecutorError> {
        let mut child = Command::new(self.shell_cmd())
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Stream output in real-time
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Spawn streaming tasks
        let stdout_handle = tokio::spawn(stream_lines(stdout, OutputType::Stdout));
        let stderr_handle = tokio::spawn(stream_lines(stderr, OutputType::Stderr));

        // Wait with timeout
        match timeout(self.timeout, child.wait()).await {
            Ok(Ok(status)) => { /* Command completed */ }
            Ok(Err(e)) => { /* Process error */ }
            Err(_) => {
                // Timeout - terminate process
                child.kill().await?;
                return Ok(StreamingResult {
                    terminated_by: TerminationReason::Timeout,
                    // ... partial results
                });
            }
        }
    }
}
```

## Rationale

### Why Tokio Async?

1. **Non-blocking I/O**: Can stream output while waiting for exit
2. **Native timeout support**: `tokio::time::timeout` integrates cleanly
3. **Signal handling**: `tokio::signal` for Ctrl+C
4. **Already a dependency**: Caro uses Tokio for HTTP backends
5. **Cross-platform**: Works on macOS, Linux, Windows

### Why Not Alternatives?

| Alternative | Reason Not Chosen |
|-------------|-------------------|
| Threading with std | More complex signal handling, no native timeout |
| subprocess crate | Additional dependency, less control |
| nix crate directly | Platform-specific, complex |
| Keep synchronous + threads | Poor ergonomics, complex ownership |

### Key Design Decisions

1. **Streaming by default**: Modern terminals expect real-time feedback
2. **30-second default timeout**: Balances responsiveness with utility
3. **Preserve partial output**: Users see what was found before timeout
4. **Educational messaging**: Help users understand and adapt
5. **Graceful degradation**: Fall back for non-TTY contexts

## Consequences

### Benefits

- Users see output immediately, eliminating "stuck" perception
- Commands can be cancelled cleanly with Ctrl+C
- Timeout actually prevents hanging (preemptive, not post-hoc)
- Partial results available on timeout/cancellation
- Better resource management (no unbounded memory growth)

### Trade-offs

- Slightly more complex execution path
- Requires async context propagation
- Output ordering may differ from direct shell (stdout/stderr interleaving)
- Small performance overhead for short commands (<5ms)

### Risks

| Risk | Mitigation |
|------|------------|
| Breaking change to ExecutionResult | Version the struct, maintain backward compatibility |
| Platform differences in process termination | Comprehensive cross-platform tests, graceful fallbacks |
| Output buffering issues | Configure line buffering, test with various commands |
| Signal handling complexity | Use tokio-signal, test cancellation thoroughly |

## Alternatives Considered

### Alternative 1: Polling with Non-blocking I/O

- **Description**: Use `try_wait()` in a loop with manual polling
- **Pros**: No async runtime changes
- **Cons**: Complex, CPU-intensive, harder timeout handling

### Alternative 2: Separate Thread per Execution

- **Description**: Spawn thread for each execution, join with timeout
- **Pros**: Simpler mental model
- **Cons**: Thread overhead, complex cancellation, signal handling issues

### Alternative 3: External Process Manager

- **Description**: Use a wrapper process (like `timeout` command)
- **Pros**: Simple implementation
- **Cons**: Not cross-platform, external dependency, less control

## Implementation Notes

### Key Components to Build

1. **StreamingExecutor** (`src/execution/streaming.rs`)
   - New async executor with streaming support
   - Backward-compatible API alongside existing executor

2. **Output Display** (`src/display/stream.rs`)
   - Real-time output rendering
   - Progress spinner for quiet periods
   - Elapsed time display

3. **Signal Handler** (`src/execution/signals.rs`)
   - Ctrl+C handling
   - Child process cleanup
   - Graceful shutdown

4. **Configuration** (`src/config/execution.rs`)
   - Timeout settings
   - Stream mode toggle
   - Progress display options

### Integration Points

```
main.rs
  └── execute_command()
        └── StreamingExecutor::execute()  // New
              ├── spawn child process
              ├── stream_stdout() ──────► display_line()
              ├── stream_stderr() ──────► display_error()
              ├── timeout_handler()
              └── signal_handler() ─────► cleanup()
```

### Testing Approach

1. **Unit tests**: Mock process output, test timeout logic
2. **Integration tests**: Real commands with known behavior
3. **Platform tests**: CI matrix (macOS, Linux, Windows)
4. **Stress tests**: Long-running commands, large outputs
5. **Manual tests**: TTY behavior, signal handling

### Migration Path

Phase 1: Add StreamingExecutor alongside existing executor
Phase 2: Feature flag to enable streaming by default
Phase 3: Deprecate synchronous executor
Phase 4: Remove synchronous executor in next major version

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time-to-first-output | <100ms | Instrumentation |
| Timeout enforcement accuracy | 100% | Automated tests |
| Cancellation success rate | >99% | Integration tests |
| Memory growth for large output | <10MB | Profiling |
| User "stuck" reports | -90% | Support tickets |

## Business Implications

- **User retention**: Eliminates major UX frustration point
- **Competitive advantage**: First AI shell assistant with streaming
- **Trust building**: Transparent execution increases user confidence
- **Enterprise readiness**: Predictable behavior required for enterprise adoption

## References

- PRD-001: Long-Running Command UX Improvements
- [Tokio Process Documentation](https://docs.rs/tokio/latest/tokio/process/index.html)
- [Rust async-process patterns](https://rust-lang.github.io/async-book/)
- GitHub Issue: Command appears hung on large find operations
- Similar implementation: [starship prompt async rendering](https://github.com/starship/starship)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-18 | @wildcard | Initial draft |
