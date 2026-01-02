# ADR-004: Evaluation of cmd_lib for Command Execution

**Status**: Rejected

**Date**: 2026-01-02

**Authors**: Claude Code

**Target**: Community

## Context

Caro currently implements command execution using Rust's standard library `std::process::Command` API directly. The implementation in `src/execution/executor.rs` handles:

- Shell-specific command wrapping (bash, zsh, fish, sh, PowerShell, cmd)
- Output capture (stdout/stderr)
- Exit code handling
- Execution timing

This ADR evaluates whether adopting [cmd_lib](https://crates.io/crates/cmd_lib), a Rust crate providing shell-script-like macros and utilities, would benefit caro's command execution subsystem.

### Current Implementation Analysis

Caro's `CommandExecutor` (`src/execution/executor.rs:36-157`) uses a straightforward approach:

```rust
pub fn execute(&self, command: &str) -> Result<ExecutionResult, ExecutorError> {
    let mut cmd = self.create_shell_command(command)?;
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd.output()?;
    Ok(self.process_output(output, execution_time_ms))
}
```

Key characteristics:
- Commands are received as runtime strings from AI backends
- Shell type detection determines which shell interpreter to invoke
- Full output capture for user display and safety validation
- Platform-specific handling for Unix vs Windows

### cmd_lib Overview

cmd_lib provides ergonomic macros for shell-like command execution:

```rust
use cmd_lib::run_cmd;
// Compile-time parsed, injection-safe macro
run_cmd!(ls -la | grep foo)?;
run_cmd!(echo "Hello $name")?;
```

**Key Features:**
- `run_cmd!` and `run_fun!` macros for command execution
- Compile-time command parsing with injection protection
- Piping and redirection syntax similar to shell scripts
- Built-in commands: `cd`, `echo`, `ignore`, logging functions
- Thread-local variable support (`tls_init!`, `tls_get!`, `tls_set!`)
- Logging integration via `log` crate
- MSRV: Rust 1.88

**Statistics:**
- 1.1k GitHub stars
- 589 commits
- 7 open issues
- License: MIT OR Apache-2.0

## Decision

**We will NOT adopt cmd_lib for caro's command execution.**

The library's primary value proposition—ergonomic, compile-time-safe command scripting—does not align with caro's runtime command execution model.

## Rationale

### 1. Runtime vs Compile-Time Command Model

cmd_lib's injection protection works at **compile time** through macro expansion:

```rust
// cmd_lib parses this at compile time
run_cmd!(rm -f /var/upload/$file)?;
```

Caro receives commands as **runtime strings** from AI backends:

```rust
// Caro executes runtime-generated commands
let command: String = ai_backend.generate_command(&prompt).await?;
executor.execute(&command)?;
```

cmd_lib cannot provide its injection protection for runtime strings—the exact scenario caro handles. Caro's existing safety validation (`src/safety/`) is specifically designed for runtime command inspection.

### 2. No Meaningful API Improvement

cmd_lib excels at making Rust code that scripts system tasks more readable:

```rust
// With cmd_lib
run_cmd!(git clone $repo && cd $dir && cargo build)?;

// With std::process::Command
Command::new("bash").arg("-c").arg(&command).output()?;
```

Since caro passes a single command string to a shell interpreter, the ergonomic benefits don't apply. Both approaches effectively become:

```rust
shell.execute(command_string);
```

### 3. Additional Dependencies

cmd_lib adds several dependencies:
- `cmd_lib_macros` (proc-macro)
- `log`
- `os_pipe`
- `faccess`

Caro already uses `tracing` for logging (not `log`), and adding cmd_lib would introduce:
- Dependency complexity without clear benefit
- Potential conflicts with caro's existing logging infrastructure
- Increased binary size

### 4. Better Alternatives Exist

If caro were to migrate from `std::process::Command`, **xshell** would be a superior choice:

| Feature | cmd_lib | xshell | std::process |
|---------|---------|--------|--------------|
| Downloads | ~3.5M | ~8.7M | (stdlib) |
| Shell injection safety | Compile-time macros | Compile-time macros | Manual |
| Cross-platform | Yes | Yes | Yes |
| Runtime strings | Limited | Limited | Full |
| Proc-macro cost | Higher | Lower | None |
| MSRV | 1.88 | 1.63 | Stable |

xshell specifically advertises "no shell injection by construction" and has a more mature ecosystem, but suffers from the same fundamental limitation: compile-time safety doesn't help runtime command execution.

### 5. Current Implementation is Sufficient

The existing `CommandExecutor` implementation:
- Is well-tested with platform-specific test coverage
- Has clear error handling via `ExecutorError`
- Supports all required shell types
- Tracks execution timing
- Integrates cleanly with caro's safety validation pipeline

No user issues or performance problems have been reported with the current approach.

## Consequences

### Benefits of Rejection

- **Reduced complexity**: No new dependencies to maintain
- **Consistency**: Continue using battle-tested std::process approach
- **Focus**: Engineering effort directed at actual user needs
- **Binary size**: No additional crate overhead

### Trade-offs

- **No ergonomic scripting syntax**: If caro needed internal scripting tasks (build, test automation), cmd_lib or xshell could help—but this isn't a current requirement
- **Manual shell wrapping**: Continue maintaining shell-type-specific command construction

### Risks

- **Future requirements**: If caro adds features requiring complex internal command orchestration, this decision should be revisited
  - **Mitigation**: Document this ADR for future reference; xshell remains available

## Alternatives Considered

### Alternative 1: Adopt cmd_lib

- **Description**: Replace `std::process::Command` usage with cmd_lib macros
- **Pros**: Shell-like syntax for any internal commands; logging integration
- **Cons**: Doesn't solve runtime command execution; adds dependencies; MSRV 1.88 is very recent
- **Why not chosen**: Core benefit (compile-time safety) doesn't apply to caro's use case

### Alternative 2: Adopt xshell

- **Description**: Use xshell instead of current approach
- **Pros**: More popular; lower MSRV; explicit cross-platform design
- **Cons**: Same fundamental limitation—compile-time parsing doesn't help runtime strings
- **Why not chosen**: No clear benefit over current implementation

### Alternative 3: Adopt duct

- **Description**: Use duct for pipeline-heavy command execution
- **Pros**: Strong pipeline support; good error handling
- **Cons**: Caro doesn't build pipelines programmatically; AI generates complete commands
- **Why not chosen**: Not aligned with caro's execution model

### Alternative 4: Keep Current Implementation (Selected)

- **Description**: Continue using `std::process::Command` with shell-type wrapping
- **Pros**: Works well; tested; no new dependencies; full runtime string support
- **Cons**: Less ergonomic for hypothetical internal scripting
- **Why chosen**: Best fit for caro's actual requirements

## Implementation Notes

No implementation required—this ADR recommends maintaining the current approach.

### If Future Needs Change

Should caro require internal command scripting (e.g., automated testing, build orchestration), the evaluation priority should be:

1. **xshell**: Better ecosystem, lower MSRV, explicit cross-platform focus
2. **duct**: If complex pipelines become necessary
3. **cmd_lib**: If logging integration with `log` crate becomes relevant

## Success Metrics

This decision is successful if:

- No regressions in command execution reliability
- No user-reported issues with the current execution approach
- Engineering effort remains focused on user-facing features

## References

- [cmd_lib on crates.io](https://crates.io/crates/cmd_lib)
- [cmd_lib documentation](https://docs.rs/cmd_lib/latest/cmd_lib/)
- [cmd_lib GitHub repository](https://github.com/rust-shell-script/rust_cmd_lib)
- [xshell documentation](https://docs.rs/xshell/latest/xshell/)
- [duct documentation](https://docs.rs/duct/latest/duct/)
- [Rust users forum: shell-like commands discussion](https://users.rust-lang.org/t/is-there-a-crate-that-has-easy-shell-like-commands/58766)
- Caro execution module: `src/execution/executor.rs`
- Caro safety validation: `src/safety/mod.rs`

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | Claude Code | Initial draft - recommend rejection |
