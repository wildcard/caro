# ADR-012: Honggfuzz Integration for Security-Critical Fuzz Testing

**Status**: Proposed

**Date**: 2026-01-02

**Authors**: caro core team

**Target**: Community

## Context

Caro is a safety-first CLI tool that converts natural language to shell commands. The core value proposition depends on robust safety validation that blocks dangerous commands through 52+ regex patterns. This safety-critical code handles untrusted input from multiple sources:

### Current Security-Critical Code Paths

1. **Safety Pattern Matching** (`src/safety/patterns.rs`): 52 pre-compiled regex patterns detecting dangerous shell commands (rm -rf, fork bombs, privilege escalation, etc.)
2. **TOML Configuration Parsing** (`src/config/mod.rs`): User-provided configuration files with validation
3. **JSON Response Parsing** (`backends/remote/*.rs`): Streaming JSON from Ollama, vLLM, and Exo backends
4. **Prompt Resolution** (`src/main.rs`): CLI arguments, stdin, and trailing arguments with shell operator truncation
5. **Execution Context Detection** (`src/context/mod.rs`): External command outputs and environment parsing

### The Problem

While caro has good test coverage with property-based tests (proptest) and integration tests, there is **no fuzz testing** infrastructure. This creates significant gaps:

- **Regex patterns may have ReDoS vulnerabilities**: Complex patterns can cause exponential backtracking on crafted input
- **Quote detection has documented limitations**: Nested quotes, hex escapes (`\x27`), and double-escaped quotes aren't handled
- **Configuration parsing lacks boundary testing**: Edge cases in TOML parsing could cause panics
- **LLM response parsing assumes well-formed JSON**: Malformed responses could crash the application
- **Prompt injection vectors unexplored**: System prompt construction with user-controlled context

### Why Fuzz Testing Now?

1. **Safety is our moat**: Any bypass of safety validation undermines the entire product value
2. **External data exposure**: Multiple untrusted input surfaces (user prompts, config files, API responses)
3. **Regex complexity**: 52 patterns with varying complexity, any one could have ReDoS potential
4. **Prior art**: Major security vulnerabilities have been found via fuzzing in parsing code (Heartbleed, CloudFlare ReDoS incidents)

## Decision

Integrate **honggfuzz-rs** as the primary fuzzing framework for caro's security-critical code paths, with focused fuzz targets for:

1. Safety validator pattern matching (highest priority)
2. TOML configuration parsing
3. JSON response parsing from backends
4. Prompt/argument parsing and validation
5. Execution context detection

### Why Honggfuzz

Honggfuzz provides unique advantages for caro's needs:

- **Feedback-driven fuzzing**: Coverage-guided mutation for deeper exploration
- **Multi-process/multi-threaded**: Utilizes all CPU cores efficiently (~1M iterations/sec on simple targets)
- **Continues on crashes**: Collects multiple crash types in a single run (with `--exit_upon_crash` option for development)
- **Stable Rust support**: Works with stable, beta, and nightly toolchains
- **Hardware-based feedback**: Can use Intel PT for additional coverage on supported CPUs
- **Proven track record**: Discovered the only critical-rated OpenSSL vulnerability

## Rationale

### Why Not cargo-fuzz/libFuzzer?

| Feature | honggfuzz | cargo-fuzz (libFuzzer) |
|---------|-----------|------------------------|
| Rust toolchain | stable/beta/nightly | nightly only |
| Crash handling | Continues by default | Stops on first crash |
| Thread utilization | Native multi-threading | Single-threaded (run multiple instances) |
| Corpus management | Built-in shared corpus | Requires manual management |
| Platform support | Linux, macOS, FreeBSD, WSL | x86_64/aarch64 Unix only |
| Speed | ~1M iter/sec (empty target) | Similar |

cargo-fuzz requires nightly Rust, which conflicts with caro's stable toolchain policy. honggfuzz's continue-on-crash behavior is ideal for finding multiple vulnerability classes in a single fuzzing campaign.

### Why Not AFL?

AFL requires more complex setup (qemu mode for Rust, separate afl-rs crate) and has lower iteration speed compared to honggfuzz. The honggfuzz integration is simpler: add dependency, write fuzz target, run.

### Alignment with Caro's Values

- **Safety-first**: Proactively discovering vulnerabilities before release
- **Local-first**: Fuzzing runs entirely locally, no cloud services
- **Developer experience**: Simple integration with cargo workflow

## Consequences

### Benefits

- **Discover hidden vulnerabilities**: Find ReDoS patterns, panic conditions, and parsing bugs
- **Regression prevention**: Fuzz corpus serves as regression test suite
- **Confidence in safety claims**: Empirical validation of pattern matching robustness
- **CI integration potential**: Run fuzzing in CI/CD for continuous security testing
- **Community contribution**: Published fuzz targets enable security researchers to contribute
- **Documentation value**: Fuzz targets document expected input domains

### Trade-offs

- **Build complexity**: Additional dev dependency and build configuration
- **Disk usage**: Fuzz corpus and crash artifacts consume disk space
- **CI resources**: Long-running fuzz jobs need dedicated resources
- **Learning curve**: Team needs familiarity with fuzz testing methodology
- **Maintenance**: Fuzz targets need updates as code evolves

### Risks

1. **False sense of security**: Fuzzing doesn't guarantee absence of bugs
   - **Mitigation**: Document limitations, combine with other testing methods

2. **Build time increase**: honggfuzz instrumentation adds compilation overhead
   - **Mitigation**: Separate `fuzz` feature flag, only compile when needed

3. **Corpus management**: Large corpus sizes can slow down CI
   - **Mitigation**: Minimize corpus, store only unique crash triggers

4. **Platform-specific issues**: Linux dependencies (libbfd, libunwind, liblzma)
   - **Mitigation**: Document setup, provide Docker development environment

## Alternatives Considered

### Alternative 1: Property-Based Testing Only (proptest)
- **Description**: Extend existing proptest infrastructure without adding fuzzer
- **Pros**: Already integrated, no new dependencies, deterministic
- **Cons**: Limited mutation strategies, no coverage guidance, misses edge cases fuzzer finds
- **Why not chosen**: proptest is complementary, not a replacement for coverage-guided fuzzing

### Alternative 2: cargo-fuzz with LibFuzzer
- **Description**: Use the more popular cargo-fuzz ecosystem
- **Pros**: Larger community, OSS-Fuzz integration, more tutorials
- **Cons**: Requires nightly Rust, single-threaded, stops on first crash
- **Why not chosen**: Nightly requirement conflicts with stable toolchain policy

### Alternative 3: LibAFL
- **Description**: New Rust-native fuzzing framework
- **Pros**: Pure Rust, highly customizable, active development
- **Cons**: More complex setup, less mature ecosystem, steeper learning curve
- **Why not chosen**: Higher complexity for initial integration; consider for future

### Alternative 4: No Fuzz Testing
- **Description**: Rely on existing test infrastructure
- **Pros**: No additional complexity
- **Cons**: Known gaps in edge case coverage, security-critical code not rigorously tested
- **Why not chosen**: Unacceptable risk for safety-critical application

## Implementation Notes

### Phase 1: Infrastructure Setup

1. Add honggfuzz dependency:
```toml
# Cargo.toml
[dev-dependencies]
honggfuzz = "0.5"
```

2. Create fuzz targets directory:
```
fuzz/
├── Cargo.toml        # Workspace member for fuzz targets
├── fuzz_targets/
│   ├── safety_validator.rs
│   ├── config_parser.rs
│   ├── json_response.rs
│   └── prompt_parser.rs
└── corpus/           # Initial seed corpus
```

3. Configure in workspace:
```toml
# Cargo.toml (workspace root)
[workspace]
members = [".", "fuzz"]
```

### Phase 2: High-Priority Fuzz Targets

**Target 1: Safety Validator** (Highest Priority)
```rust
// fuzz/fuzz_targets/safety_validator.rs
use honggfuzz::fuzz;
use caro::safety::{SafetyValidator, SafetyLevel};
use caro::models::ShellType;

fn main() {
    let validator = SafetyValidator::new(SafetyLevel::Strict);

    loop {
        fuzz!(|data: &[u8]| {
            if let Ok(command) = std::str::from_utf8(data) {
                // Test pattern matching doesn't panic or hang
                let _ = validator.validate(command, ShellType::Bash);
                let _ = validator.validate(command, ShellType::PowerShell);
                let _ = validator.validate(command, ShellType::Zsh);
            }
        });
    }
}
```

**Target 2: TOML Configuration**
```rust
// fuzz/fuzz_targets/config_parser.rs
use honggfuzz::fuzz;
use caro::config::UserConfiguration;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Ok(toml_str) = std::str::from_utf8(data) {
                // Should handle any malformed TOML gracefully
                let _ = toml::from_str::<UserConfiguration>(toml_str);
            }
        });
    }
}
```

**Target 3: JSON Response Parsing**
```rust
// fuzz/fuzz_targets/json_response.rs
use honggfuzz::fuzz;
use caro::backends::ollama::OllamaResponse;
use caro::backends::vllm::VllmResponse;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Ok(json_str) = std::str::from_utf8(data) {
                let _ = serde_json::from_str::<OllamaResponse>(json_str);
                let _ = serde_json::from_str::<VllmResponse>(json_str);
            }
        });
    }
}
```

### Phase 3: CI Integration

```yaml
# .github/workflows/fuzz.yml
name: Fuzz Testing
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y binutils-dev libunwind-dev liblzma-dev
      - name: Install honggfuzz
        run: cargo install honggfuzz
      - name: Run fuzzer (safety_validator)
        run: |
          cd fuzz
          HFUZZ_RUN_ARGS="--timeout 5 --exit_upon_crash -N 100000" \
          cargo hfuzz run safety_validator
      - name: Upload crashes
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: crash-artifacts
          path: fuzz/hfuzz_workspace/*/crashes/
```

### Running Locally

```bash
# Install honggfuzz
cargo install honggfuzz

# Install Linux dependencies (Ubuntu/Debian)
sudo apt-get install binutils-dev libunwind-dev liblzma-dev

# Run safety validator fuzzer
cd fuzz
cargo hfuzz run safety_validator

# Run with specific options
HFUZZ_RUN_ARGS="--threads 8 --timeout 10 --exit_upon_crash" \
cargo hfuzz run safety_validator

# Debug a crash
cargo hfuzz run-debug safety_validator hfuzz_workspace/safety_validator/crashes/CRASH_FILE
```

### Seed Corpus Strategy

Create initial corpus with known edge cases:

```
fuzz/corpus/safety_validator/
├── basic_rm.txt          # "rm file.txt"
├── nested_quotes.txt     # "echo \"it's rm -rf /\""
├── unicode.txt           # "рм -рф /"  (Cyrillic)
├── long_command.txt      # 10KB+ command
├── shell_operators.txt   # "cmd > out | other"
├── escape_sequences.txt  # "\x72\x6d -rf /"
└── fork_bomb.txt         # ":(){:|:&};:"
```

## Success Metrics

### Short-term (1 month)
- **Metric**: Fuzz targets for safety validator and config parser operational
- **Target**: No crashes found in 1M iterations per target

### Medium-term (3 months)
- **Metric**: CI integration running daily fuzzing
- **Target**: Zero regression crashes in release candidates

### Long-term (6 months)
- **Metric**: Fuzz corpus coverage
- **Target**: 80%+ line coverage of safety validation code under fuzzing

### Security Metrics
- **Metric**: Time to detect injected bugs (mutation testing)
- **Target**: Fuzzer finds injected ReDoS within 10K iterations

## Business Implications

### Community Trust
- **Value**: Demonstrates commitment to security through proactive testing
- **Differentiation**: Few CLI tools have this level of security validation
- **Transparency**: Open-source fuzz targets enable community security contributions

### Enterprise Readiness
- **Compliance**: Fuzz testing is increasingly required for security-sensitive tools
- **Due diligence**: Answers "how do you test your safety validation?" definitively
- **Incident prevention**: Proactive bug discovery vs. reactive incident response

### Risk Reduction
- **Cost of vulnerabilities**: Single safety bypass could destroy product reputation
- **Investment**: ~2 weeks engineering time for initial setup
- **ROI**: Prevents potential critical vulnerabilities in production

## References

- [honggfuzz-rs GitHub](https://github.com/rust-fuzz/honggfuzz-rs)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [Fuzzing comparison: cargo-fuzz vs honggfuzz](https://www.wzdftpd.net/blog/rust-fuzzers.html)
- [Introduction to Rust Fuzzing - FuzzingLabs](https://fuzzinglabs.com/introduction-rust-fuzzing-tutorial/)
- [Honggfuzz documentation](https://docs.rs/honggfuzz/)
- Related ADRs:
  - ADR-001: Enterprise vs Community Architecture (context on safety moat)

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | caro core team | Initial draft |
