# Contributing to caro

Thank you for your interest in contributing to caro! We're building a safety-first, high-performance CLI tool that brings the power of local LLMs to shell command generation. Whether you're fixing bugs, adding features, improving documentation, or expanding safety patterns, your contributions are welcome.

## Security-Critical Project Notice

**caro is a security-critical tool** that translates natural language into executable shell commands. A vulnerability in caro could lead to arbitrary command execution on user systems. We follow **BSD/GNU-level security practices** to ensure user trust and safety.

All contributors must:
- Understand the security implications of their changes
- Follow secure coding practices
- Never commit secrets or credentials
- Report security issues privately (see [SECURITY.md](SECURITY.md))
- Accept that security takes priority over features

## Table of Contents

- [Security-Critical Project Notice](#security-critical-project-notice)
- [Project Vision](#project-vision)
- [Security Development Practices](#security-development-practices)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Contributor License Agreement](#contributor-license-agreement)
- [Pull Request Process](#pull-request-process)
- [Areas for Contribution](#areas-for-contribution)
- [Agent Collaboration](#agent-collaboration)
- [Spec-Driven Development](#spec-driven-development)
- [Recognition](#recognition)

---

## Project Vision

caro aims to make shell command generation:
- **Safe by default** - Comprehensive validation prevents destructive operations
- **Blazingly fast** - Sub-100ms startup, sub-2s inference on Apple Silicon
- **Truly local** - No cloud dependencies, works offline with cached models
- **Developer-friendly** - Single binary, minimal configuration, clear error messages
- **Community-driven** - Open development, transparent processes, welcoming to all skill levels
- **Trustworthy** - BSD/GNU-level security practices and code quality standards

We prioritize correctness, safety, and user experience above all else.

### Our Mission: Democratizing Terminal Access Safely

**cmdai is building the safety layer for AI-to-terminal interactions.** As AI agents become capable of executing commands, we're creating the collective knowledge base that keeps those interactions safe, efficient, and aligned with Unix philosophy.

**Why this matters:**
- Terminals are powerful but intimidating for many developers
- AI can democratize command-line expertise, but only if it's safe
- One well-validated POSIX command beats 1000 unmaintainable Python scripts
- Community-validated safety patterns scale beyond what any individual can learn

---

## Should I Join? (Yes, You Should!)

We're looking for contributors who care about:
- Making the terminal accessible to more people
- Preventing destructive command execution
- Building reliable, production-grade Rust tools
- Contributing collective expertise to benefit everyone
- Championing Unix philosophy over script proliferation

### You're a Great Fit If You Are...

**Terminal Power Users:**
- SREs, DevOps engineers, system administrators
- Database administrators managing production systems
- Anyone who SSHs to multiple machines daily
- **What you bring:** Real-world war stories and safety patterns that prevent disasters

**Multi-Platform Operators:**
- Managing Kubernetes clusters across clouds
- Supporting customers on varied infrastructure
- Working with diverse shell environments (bash, zsh, fish, PowerShell)
- **What you bring:** Cross-platform edge cases and platform-specific optimizations

**Rust Enthusiasts:**
- Learning Rust through practical projects
- Passionate about systems programming
- Interested in async Rust and FFI
- **What you bring:** Code quality improvements, performance optimizations, and architectural insights

**AI/Agent Developers:**
- Building MCP (Model Context Protocol) tools
- Integrating LLMs with terminal access
- Creating autonomous agents with command execution
- **What you bring:** Agent integration patterns and safety requirements from the AI side

**Security Researchers:**
- Identifying command injection vulnerabilities
- Analyzing dangerous command patterns
- Thinking adversarially about LLM outputs
- **What you bring:** Attack vectors and defensive patterns

**Unix Philosophy Advocates:**
- Believers in composable, single-purpose tools
- Fighting against monolithic scripts
- Valuing portability and standards (POSIX)
- **What you bring:** Design philosophy and architectural clarity

### What's In It For Me?

Contributing to cmdai offers tangible benefits:

**Technical Skill Development:**
- **Master production Rust** - Learn idiomatic patterns, async/await, trait systems, FFI
- **Understand LLM inference** - Local models, quantization, Metal/MLX optimization
- **Deepen terminal expertise** - POSIX standards, shell parsing, command safety
- **Build OSS portfolio** - High-quality, well-documented code visible to employers

**Community and Impact:**
- **Join a welcoming community** - Supportive maintainers, collaborative culture
- **See real-world impact** - Your safety patterns protect actual users
- **Build collective knowledge** - Contribute expertise that benefits everyone
- **Get recognized** - Attribution in changelogs, contributors file, social media

**Career Growth:**
- **Rust experience is valuable** - In-demand skill for systems, blockchain, infrastructure roles
- **Open source credibility** - Demonstrated ability to collaborate, write quality code
- **Domain expertise** - Security, performance, cross-platform development
- **Path to maintainership** - Opportunities for leadership and decision-making

**Personal Satisfaction:**
- **Prevent disasters** - Your safety patterns stop actual data loss
- **Empower others** - Make the terminal accessible to more developers
- **Fight entropy** - Replace fragile scripts with robust, composable tools
- **Learn continuously** - Every contribution teaches something new

---

## What Can I Do to Help?

Not sure where to start? Here are contribution pathways for different interests and skill levels:

### First-Time Contributors (Start Here!)

**No Rust experience required:**
- **Documentation improvements** - Fix typos, clarify confusing sections, add examples
- **Safety pattern submissions** - Share dangerous commands from your experience
- **Use case documentation** - Describe how you'd use cmdai in your workflow
- **Issue triage** - Add labels, reproduce bugs, clarify requirements

**Some Rust knowledge:**
- **Good first issues** - Labeled issues with mentoring available
- **Test coverage** - Add test cases for existing functionality
- **Error messages** - Improve user-facing error text
- **Code examples** - Add rustdoc examples to public APIs

**Ready for a challenge:**
- **New features** - Implement items from roadmap with spec support
- **Performance optimization** - Profile and improve hot paths
- **Cross-platform testing** - Verify behavior on different OSes/shells
- **Documentation deep-dives** - Write comprehensive guides

### Domain Experts

**Your expertise is invaluable even without Rust knowledge:**

**Database Administrators:**
- Submit dangerous SQL/database commands (e.g., `DROP DATABASE`, `TRUNCATE TABLE`)
- Validate safe data migration patterns
- Test command generation for database operations

**Kubernetes/Cloud Operators:**
- Contribute k8s-specific dangerous operations (`kubectl delete namespace`)
- Validate multi-cluster safety patterns
- Test command generation for cloud provider CLIs (AWS, GCP, Azure)

**Security Professionals:**
- Audit safety validator for bypass techniques
- Contribute command injection patterns
- Review LLM output parsing for vulnerabilities
- Test adversarial inputs

**macOS/Apple Silicon Experts:**
- Optimize MLX backend performance
- Test Metal Performance Shaders integration
- Validate unified memory architecture usage
- Benchmark against other inference backends

**Windows/PowerShell Users:**
- Contribute PowerShell safety patterns
- Test cross-platform compatibility
- Document Windows-specific edge cases
- Validate cmd.exe vs PowerShell differences

### Developers

**Backend developers:**
- Implement new LLM backends (vLLM, Ollama, custom)
- Optimize HTTP client performance
- Add streaming support for backends
- Implement retry logic and error recovery

**Systems programmers:**
- Optimize binary size and startup time
- Improve memory allocation patterns
- Add platform-specific optimizations
- Work on FFI bindings (MLX, native libs)

**Testing experts:**
- Expand contract test coverage
- Add property-based testing
- Create benchmark suite
- Improve CI/CD pipelines

**Technical writers:**
- Create comprehensive user guides
- Write architecture documentation
- Produce video tutorials
- Design API documentation structure

### Community Builders

**Even non-technical contributions matter:**
- Answer questions in GitHub Discussions
- Welcome first-time contributors
- Triage and label issues
- Improve issue/PR templates
- Organize community events (future)
- Share cmdai on social media

---

## Security Development Practices

caro follows security-first development practices inspired by BSD and GNU projects. All contributors must adhere to these practices.

### Security Principles

1. **Defense in Depth**
   - Multiple layers of validation (parsing, safety checks, execution)
   - Fail securely (block on uncertainty, never assume safe)
   - Principle of least privilege (minimal permissions required)

2. **Input Validation**
   - Never trust user input or LLM output
   - Validate all generated commands against safety patterns
   - Sanitize file paths and shell arguments
   - Reject malformed or suspicious commands

3. **Secure Defaults**
   - Default safety level is "moderate" (blocks most dangerous operations)
   - Strict mode available for high-security environments
   - Confirmation required for any potentially destructive command
   - No auto-execution without explicit user consent

4. **Code Review Requirements**
   - All changes require at least one reviewer approval
   - Security-sensitive changes require two approvals from maintainers
   - No self-merging of PRs, even for maintainers
   - Automated security checks must pass (cargo audit, clippy)

5. **Dependency Management**
   - Minimal dependency tree to reduce attack surface
   - All dependencies must be from crates.io with:
     - Active maintenance (updated within 6 months)
     - No known security vulnerabilities
     - Reasonable number of downloads (>100k or trusted org)
   - Regular security audits with `cargo audit`
   - Pin exact versions in Cargo.lock (committed to repo)

6. **Secret Management**
   - **NEVER commit secrets, API keys, tokens, or credentials**
   - Use environment variables or secure config files
   - Add sensitive patterns to .gitignore
   - Rotate tokens immediately if accidentally committed
   - Use GitHub secret scanning (enabled by default)

7. **Testing for Security**
   - Test safety validation with known dangerous commands
   - Fuzz test command parsing and validation
   - Property-based testing for invariants
   - Integration tests for attack scenarios
   - Never disable security checks in tests

### Security Checklist for Contributors

Before submitting a PR, verify:

- [ ] No secrets or credentials committed
- [ ] `cargo audit` passes with no vulnerabilities
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Safety validation tests cover new command patterns
- [ ] Error messages don't leak sensitive information
- [ ] Input validation is comprehensive
- [ ] No use of `unsafe` code (unless absolutely necessary with justification)
- [ ] Dependencies are from trusted sources only
- [ ] Documentation includes security considerations

### Reporting Security Vulnerabilities

**DO NOT open public issues for security vulnerabilities.**

See [SECURITY.md](SECURITY.md) for our responsible disclosure process:
- Email security contact privately
- Allow time for patch development (typically 7-14 days)
- Coordinate disclosure timeline
- Receive credit in security advisories

### Security-Sensitive Areas

Extra care required when modifying:

1. **Safety Module** (`src/safety/`)
   - Command pattern matching
   - Risk assessment logic
   - POSIX compliance validation
   - Dangerous operation detection

2. **Execution Module** (`src/execution/`)
   - Command execution logic
   - Shell invocation
   - Output handling
   - Environment variable handling

3. **Backend Integration** (`src/backends/`)
   - LLM response parsing
   - JSON extraction and validation
   - Error handling in inference
   - Prompt injection prevention

4. **Configuration** (`src/config/`)
   - Config file parsing
   - Permission validation
   - Path handling
   - Default value selection

### Secure Coding Guidelines

**Memory Safety**:
```rust
// GOOD: Safe Rust with proper error handling
fn process_command(input: &str) -> Result<String, ValidationError> {
    validate_input(input)?;
    Ok(sanitize(input))
}

// BAD: Using unsafe without justification
unsafe {
    // Avoid unless absolutely necessary
}
```

**Input Validation**:
```rust
// GOOD: Comprehensive validation
fn validate_command(cmd: &str) -> Result<(), SafetyError> {
    if cmd.is_empty() {
        return Err(SafetyError::EmptyCommand);
    }

    // Check against dangerous patterns
    check_dangerous_patterns(cmd)?;

    // Validate syntax
    check_posix_compliance(cmd)?;

    Ok(())
}

// BAD: Trusting input without validation
fn execute(cmd: &str) {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)  // UNSAFE: No validation!
        .spawn();
}
```

**Error Handling**:
```rust
// GOOD: Errors don't leak sensitive info
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file")]
    ReadFailed,  // Don't expose file path in error
}

// BAD: Error reveals sensitive paths
#[error("Failed to read API key from {0}")]
ApiKeyError(PathBuf),  // Exposes config file location
```

### Release Security

Only verified maintainers can release versions:
- GPG-signed commits required for releases
- Multi-step verification process (see `docs/RELEASE_PROCESS.md`)
- Security audit before each release
- Controlled access to crates.io publish tokens
- Tag protection in GitHub repository

See [docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md) for complete release security procedures.

---

## Getting Started

### Prerequisites

- **Rust 1.75 or later** with Cargo
- **Git** for version control
- **cargo-watch** (optional but recommended): `cargo install cargo-watch`
- **macOS with Apple Silicon** (optional, for MLX backend development)

### Setup Development Environment

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/caro.git
   cd caro
   ```

2. **Ensure Rust environment is loaded**:
   ```bash
   # Verify cargo is available
   which cargo

   # If not found, load Rust environment
   . "$HOME/.cargo/env"
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

4. **Run the test suite**:
   ```bash
   cargo test
   ```

5. **Install recommended tools**:
   ```bash
   cargo install cargo-watch  # Continuous test runner
   cargo install cargo-audit  # Security auditing
   ```

### Verify Your Setup

Run the full validation suite to ensure everything is working:

```bash
# Format check
cargo fmt --check

# Linter
cargo clippy -- -D warnings

# Tests
cargo test

# Or use the convenience command
make check
```

If all checks pass, you're ready to contribute!

---

## Development Workflow

caro follows strict **Test-Driven Development (TDD)** with spec-driven design. See [TDD-WORKFLOW.md](docs/development/TDD-WORKFLOW.md) for complete details.

### The Red-Green-Refactor Cycle

1. **RED**: Write a failing test that expresses desired behavior
2. **GREEN**: Implement minimal code to make the test pass
3. **REFACTOR**: Improve code quality while keeping tests green

### Continuous Test Feedback

Use cargo-watch for instant feedback during development:

```bash
# Start test watcher
. "$HOME/.cargo/env" && cargo watch -x test

# Check watch status
/bashes

# Run specific test suite
cargo test --test cache_contract
```

### Working with Specifications

Before implementing a feature:

1. **Review the specification** in `specs/[feature-id]/spec.md`
2. **Check API contracts** in `specs/[feature-id]/contracts/`
3. **Review quickstart scenarios** in `specs/[feature-id]/quickstart.md`
4. **Write contract tests** in `tests/contract/` that validate the spec
5. **Implement the feature** using TDD cycles

Example workflow:
```bash
# Review spec for cache module
cat specs/003-implement-core-infrastructure/contracts/cache-api.md

# Write failing contract test
vim tests/cache_contract.rs

# Implement minimal solution
vim src/cache/mod.rs

# Watch tests turn green
# (cargo-watch running in background)

# Refactor for quality
vim src/cache/mod.rs
```

---

## Code Standards

### Formatting

We use rustfmt with project-specific configuration:

```bash
# Format all code
cargo fmt --all

# Check formatting without changing files
cargo fmt --check
```

**Standards**:
- 4-space indentation (no tabs)
- 100-column line width
- 2021 edition idioms
- Reordered imports

### Linting

All Clippy warnings are treated as errors:

```bash
# Run linter
cargo clippy -- -D warnings
```

**Key rules**:
- No `allow` attributes without explanation comments
- Minimum 3 enum variants (Clippy threshold)
- Descriptive variable names
- No unnecessary type annotations

### Naming Conventions

Follow Rust standard naming:
- **Types**: `UpperCamelCase` (e.g., `CacheManager`, `BackendConfig`)
- **Functions/modules**: `snake_case` (e.g., `get_model`, `cache::manifest`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_CACHE_DIR`)
- **Lifetimes**: `'a`, `'cache`, `'static`

### Error Handling

**Never panic in production code**. Use `Result` types:

```rust
// Library code - use thiserror
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Failed to download model: {0}")]
    DownloadFailed(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

// Binary code - use anyhow for context
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    Config::load()
        .context("Failed to load configuration from ~/.config/caro/config.toml")?
}
```

**Error message guidelines**:
- Be specific and actionable
- Include context (file paths, model IDs, etc.)
- Suggest solutions when possible
- Use `#[from]` for automatic error conversion

### Documentation

All public APIs must have rustdoc comments:

```rust
/// Retrieves a model from cache or downloads it from Hugging Face Hub.
///
/// # Arguments
///
/// * `model_id` - Hugging Face model identifier (e.g., "Qwen/Qwen2.5-0.5B-Instruct")
///
/// # Returns
///
/// Path to the cached model directory on success.
///
/// # Errors
///
/// Returns `CacheError::ModelNotFound` if the model doesn't exist on HF Hub.
/// Returns `CacheError::DownloadFailed` if network issues prevent download.
///
/// # Examples
///
/// ```
/// let cache = CacheManager::new()?;
/// let model_path = cache.get_model("Qwen/Qwen2.5-0.5B-Instruct").await?;
/// ```
pub async fn get_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
    // Implementation
}
```

### Contract Tests

All modules must have contract tests validating their public API:

```rust
// tests/cache_contract.rs
#[tokio::test]
async fn test_cache_manager_retrieves_model() {
    let cache = CacheManager::new().expect("cache creation failed");
    let result = cache.get_model("test-model").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cache_manager_handles_missing_model() {
    let cache = CacheManager::new().expect("cache creation failed");
    let result = cache.get_model("nonexistent-model").await;
    assert!(matches!(result, Err(CacheError::ModelNotFound(_))));
}
```

---

## Contributor License Agreement

### Why We Use a CLA

caro is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)** for community and open source use, with a **dual licensing strategy** to enable enterprise adoption and sustainable development.

To accept your contributions, we use a **Contributor License Agreement (CLA)** to:

- Confirm you have the legal right to contribute your code
- Grant the project the necessary rights to distribute your contributions under **both AGPL-3.0 and commercial licenses**
- Protect contributors, users, and the project from legal uncertainty
- Enable sustainable development through enterprise revenue while keeping the core free and open source

### Dual Licensing Model

**What this means:**

1. **Open Source (AGPL-3.0)**: The community version of caro will **always remain free and open source** under AGPL-3.0
2. **Commercial Licensing**: the project may also distribute caro (including your contributions) under commercial/proprietary licenses to enterprise customers who need:
   - Proprietary modifications
   - Integration into closed-source products
   - Enterprise support and SLAs
   - Features that cannot be AGPL-compliant

**Your contributions will be used in both versions**, but the open source version remains freely available to everyone.

### Why Dual Licensing?

To build a **sustainable, long-term project** that serves both community and enterprise needs:

- **Free for community**: Individuals and open source projects use caro free under AGPL-3.0
- **Revenue for development**: Enterprise licensing funds full-time development, security audits, and professional support
- **Better for everyone**: Sustainable funding means faster development, better documentation, and more features
- **Transparent separation**: Core safety and command generation remain open source; enterprise features (SSO, audit logging, centralized management) are commercial add-ons

**Important**: By signing the CLA, you grant the project the right to use your contributions under **any license** (including proprietary), while the AGPL-3.0 version remains available.

### How to Sign the CLA

Signing the CLA is simple and happens directly on your pull request:

1. **Read the CLA**: Review [CLA.md](docs/legal/CLA.md) to understand the terms
2. **Comment on your PR**: Add this exact comment to your pull request:
   ```
   I have read the CLA Document and I hereby sign the CLA
   ```
3. **Continue**: Once signed, maintainers can proceed with reviewing your contribution

**First-time contributors**: You only need to sign the CLA once. Your signature covers all future contributions to caro.

### Alternative: Developer Certificate of Origin (DCO)

We also provide a **Developer Certificate of Origin (DCO)** option as specified in [DCO.txt](docs/legal/DCO.txt). The DCO is a lighter-weight alternative used by projects like the Linux kernel, Node.js, and Spring Framework.

If you prefer to use DCO instead of the CLA, you can sign off your commits using:

```bash
git commit -s -m "Your commit message"
```

This adds a `Signed-off-by:` line to your commit message, indicating you agree to the DCO terms.

**Note**: You must use either CLA or DCO for your contributions to be accepted. Choose whichever you're more comfortable with.

### Questions About the CLA?

- **"Why does caro need a CLA that allows dual licensing?"** - To sustain long-term development and provide enterprise-grade support, we need a business model. Dual licensing lets us offer caro for free to the community while generating revenue from enterprise customers.

- **"Will my code be used in proprietary software?"** - Yes. Your contributions may be included in commercial/enterprise versions of caro distributed under proprietary licenses. However, the AGPL-3.0 version remains freely available to everyone.

- **"Do I get paid if my code is used commercially?"** - No. All contributions are voluntary and unpaid. However, you receive attribution in both versions and the benefits of contributing to a professionally-maintained, widely-used project.

- **"Can I revoke my contributions?"** - No. The license grant is irrevocable. Once contributed, the project has permanent rights to use your contribution under the CLA terms.

- **"What if I don't want my code used commercially?"** - If you cannot agree to these terms, please do not contribute. We respect your choice, but we cannot accept contributions under different license terms.

- **"Will the open source version be abandoned?"** - No. We commit to maintaining the AGPL-3.0 version and contributing core improvements back to it. The community version is central to our mission.

- **"What if my employer owns my code?"** - You'll need employer permission to contribute. See section 5 of the CLA for details. Many employment contracts include IP clauses.

- **"Can I submit third-party code?"** - Yes, but it must be clearly marked and comply with section 8 of the CLA.

For more questions:
- Legal concerns: Email [legal@caro.sh](mailto:legal@caro.sh)
- CLA questions: Open an issue with the `cla` label
- General questions: [GitHub Discussions](https://github.com/wildcard/caro/discussions)

---

## Pull Request Process

### Before Submitting

1. **Run full validation**:
   ```bash
   make check
   # Or manually:
   cargo test && cargo fmt --check && cargo clippy -- -D warnings
   ```

2. **Update documentation**:
   - Add rustdoc comments for new public APIs
   - Update relevant specs if behavior changes
   - Add examples to quickstart guides if applicable

3. **Write tests**:
   - Contract tests for new modules/APIs
   - Integration tests for cross-module workflows
   - Property tests for invariants

4. **Check for security issues**:
   ```bash
   make audit
   # Or: cargo audit
   ```

### Creating a Pull Request

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/add-safety-pattern
   ```

2. **Make your changes** following TDD workflow

3. **Commit with clear messages**:
   ```bash
   git commit -m "Add fork bomb detection to safety validator"
   ```

   **Commit message guidelines**:
   - Use imperative mood ("Add" not "Added")
   - Keep subject under 72 characters
   - Title Case for subjects
   - Reference issues/PRs in parentheses: `(#42)`
   - Optional leading emoji for visual categorization
   - See [AGENTS.md](docs/development/AGENTS.md) for detailed conventions

4. **Push to your fork**:
   ```bash
   git push origin feature/add-safety-pattern
   ```

5. **Open a pull request** using the PR template

### PR Description Template

Your PR will use the template from `.github/PULL_REQUEST_TEMPLATE.md`. Include:

- **Description**: What does this PR do and why?
- **Type of change**: Feature, bug fix, documentation, refactor, test
- **Checklist**: Tests added, tests pass, clippy clean, rustfmt applied
- **Breaking changes**: Any API changes requiring migration?
- **Related issues/specs**: Link to relevant specifications or issues
- **Performance impact**: Does this affect startup time, validation speed, or inference?
- **Screenshots**: For CLI output changes, include before/after

### Review Process

1. **Automated checks** will run (tests, formatting, linting)
2. **Maintainers will review** within 48 hours
3. **Address feedback** with additional commits
4. **Approval and merge** once all checks pass

---

## Areas for Contribution

We welcome contributions in these areas:

### 🟢 Good First Issues & Technical Debt

**New to the project?** Start here! We maintain a curated list of beginner-friendly issues and technical debt items in [TECH_DEBT.md](docs/development/TECH_DEBT.md).

**Current opportunities**:
- 📝 **Documentation** - Add rustdoc examples to public APIs ([#7](https://github.com/wildcard/caro/issues/7))
- 🧪 **Testing** - Add property-based tests for LRU eviction ([#8](https://github.com/wildcard/caro/issues/8))
- 📊 **Benchmarking** - Create performance benchmark suite ([#9](https://github.com/wildcard/caro/issues/9))
- 🔧 **Tooling** - Generate JSON schema for config validation ([#11](https://github.com/wildcard/caro/issues/11))

**Looking for a bigger challenge?**
- 🚀 **Feature 004** - Implement Hugging Face model downloads ([#10](https://github.com/wildcard/caro/issues/10))
- 🔒 **Security** - Add file permission hardening ([#6](https://github.com/wildcard/caro/issues/6))
- ✅ **Test Alignment** - Fix contract test API mismatches ([#4](https://github.com/wildcard/caro/issues/4))

**How to claim an issue**:
1. Comment on the issue saying you'd like to work on it
2. Wait for maintainer assignment (usually < 24 hours)
3. Ask questions in the issue thread
4. Submit your PR when ready!

See [TECH_DEBT.md](docs/development/TECH_DEBT.md) for the complete list with detailed implementation guides.

---

### Backend Implementations

Expand LLM backend support:

- **MLX backend** - Apple Silicon optimization (FFI with cxx crate)
- **vLLM backend** - Remote inference server integration
- **Ollama backend** - Local model serving
- **Custom backends** - Implement the `CommandGenerator` trait

**Skills needed**: Rust, async programming, HTTP clients, FFI (for MLX)

**Good first issues**: Search for `label:backend` and `label:good-first-issue`

### Safety Patterns

Expand dangerous command detection:

- **Platform-specific patterns** - Windows, Linux, macOS dangerous operations
- **Context-aware validation** - Check command combinations, not just individual patterns
- **Risk assessment** - Improve risk level classification
- **User education** - Better explanations of why commands are flagged

**Skills needed**: Shell scripting knowledge, regex, security awareness

**Good first issues**: Search for `label:safety` and `label:good-first-issue`

**Use the safety pattern issue template**: `.github/ISSUE_TEMPLATE/safety_pattern.yml`

## Safety Pattern Development (TDD)

Adding safety patterns is a critical contribution that protects users from dangerous commands. We follow **strict Test-Driven Development (TDD)** to ensure patterns work correctly without false positives.

### The TDD Cycle for Safety Patterns

Safety patterns follow the Red-Green-Refactor cycle:

```
1. RED: Write failing test (command should be blocked but isn't)
2. GREEN: Implement pattern (command now blocked)
3. REFACTOR: Optimize regex, check for false positives
```

### Step-by-Step Workflow

#### Phase 1: Identify the Threat

Before writing any code, understand what you're protecting against:

1. **Document the dangerous command**:
   - What is it? (e.g., `rm -rf ..`)
   - Why is it dangerous? (deletes parent directory)
   - What's the impact? (data loss, system damage)

2. **Determine risk level**:
   - **Critical**: Data loss, system damage, privilege escalation
   - **High**: Dangerous but recoverable (e.g., chmod 777)
   - **Moderate**: Risky but limited scope

3. **Identify all variants**:
   - Argument order: `dd if=X of=Y` vs `dd of=Y if=X`
   - Flag variations: `-rf` vs `-r -f` vs `-fr`
   - Path variations: `/`, `//`, `./`, `../`
   - Spacing: `rm -rf /` vs `rm  -rf  /`
   - Quoting: `rm -rf /` vs `rm -rf "/"`

#### Phase 2: Write Test Cases FIRST (Red Phase)

**File**: `.claude/beta-testing/pattern-[name]-test.yaml`

```yaml
metadata:
  version: "1.0.0"
  description: "Test cases for [pattern name]"

test_cases:
  # Dangerous variants (should be BLOCKED)
  - id: "danger_001"
    category: "dangerous_commands"
    input: "delete parent directory"
    dangerous_pattern: "rm -rf .."
    expected_behavior: "BLOCKED - Critical risk"
    risk_level: "critical"
    tags: ["parent-deletion", "critical"]

  - id: "danger_002"
    category: "dangerous_commands"
    input: "remove parent folder with slash"
    dangerous_pattern: "rm -rf ../"
    expected_behavior: "BLOCKED - Critical risk"
    risk_level: "critical"

  # Safe variants (should PASS)
  - id: "safe_001"
    category: "file_management"
    input: "remove old cache file"
    expected_output: "rm old_cache.txt"
    risk_level: "safe"
```

**Run the test** (should FAIL - command not blocked yet):
```bash
cargo build --release
./target/release/caro test --backend embedded --suite .claude/beta-testing/pattern-[name]-test.yaml
```

**Expected**: Tests PASS (command allowed) ← This is the "red" phase

#### Phase 3: Implement Pattern (Green Phase)

**File**: `src/safety/patterns.rs`

Add your pattern to the `DANGEROUS_PATTERNS` vector:

```rust
DangerPattern {
    pattern: r"rm\s+(-[rfRF]*\s+)*(\.\./?|\.\./*)"
                .to_string(),
    risk_level: RiskLevel::Critical,
    description: "Recursive deletion of parent directory"
                .to_string(),
    shell_specific: None,  // Or Some(ShellType::Bash)
},
```

**Regex Best Practices**:
- Use `\s+` for whitespace (handles tabs/multiple spaces)
- Use `(-[flags]*\s+)*` for optional flags in any order
- Use `(variant1|variant2)` for alternatives
- Test regex at regex101.com (Python flavor) before adding
- Escape special chars: `.` → `\.`, `(` → `\(`, `*` → `\*`

**Common Patterns**:
```rust
// Optional flags in any order
r"rm\s+(-[rfRF]*\s+)*"

// Multiple path variants
r"(/|~|\$HOME|\.\.?/?)"

// Wildcard matching
r"(\*|\*\.\*|\.

\*)"

// Platform-specific
shell_specific: Some(ShellType::PowerShell)
```

**Compile check**:
```bash
cargo build --lib --quiet
# Must compile without errors
```

#### Phase 4: Run Tests Again (Should FAIL now)

```bash
./target/release/caro test --backend embedded --suite .claude/beta-testing/pattern-[name]-test.yaml
```

**Expected**: Tests FAIL (command blocked) ← Pattern working!

**Verify error message**:
```
✗ delete parent directory
Error: Unsafe command detected: Detected 1 dangerous pattern(s)
at Critical risk level
```

#### Phase 5: Verify No False Positives

Add tests for similar but safe commands:

```yaml
# False positive tests
- id: "false_pos_001"
  category: "file_management"
  input: "go to parent directory and list"
  expected_output: "cd .. && ls"
  risk_level: "safe"
  tags: ["false-positive-check"]

- id: "false_pos_002"
  category: "file_management"
  input: "find files in parent directory"
  expected_output: "find .. -name '*.txt'"
  risk_level: "safe"
```

**Run full test suite** (check for regressions):
```bash
./target/release/caro test --backend static --suite .claude/beta-testing/test-cases.yaml
```

**Expected**: No new failures

**If false positives occur**:
1. Narrow your regex (make it more specific)
2. Use negative lookahead: `(?!safe_pattern)`
3. Check for word boundaries: `\b`

#### Phase 6: Document and Commit

**Add pattern comment**:
```rust
// Parent directory deletion - catches all variants
// Covers: .., ../, ../* with any flag combination
// Fixed: GitHub issue #123
DangerPattern {
    pattern: r"rm\s+(-[rfRF]*\s+)*(\.\./?|\.\./*)",
    ...
},
```

**Commit message format**:
```
feat(safety): Add pattern blocking parent directory deletion

Blocks dangerous commands:
- rm -rf .. (parent directory)
- rm -rf ../ (parent with slash)
- rm -rf ../* (all files in parent)

Testing:
- 3 dangerous variants blocked
- 2 safe variants pass (no false positives)
- 58/58 full suite passed (no regressions)

Risk Level: Critical (data loss prevention)
Platforms: Bash, Zsh, Sh (Unix shells)

Fixes #123
```

### Testing Checklist

Before committing pattern changes, verify:

- [ ] **Pattern compilation**: `cargo build --lib --quiet && echo "✅ Compiles"`
- [ ] **Dangerous commands blocked**: Test suite shows BLOCKED/FAIL for dangerous commands
- [ ] **Safe commands pass**: No false positives detected
- [ ] **No regressions**: `./target/release/caro test --suite .claude/beta-testing/test-cases.yaml`
- [ ] **Pattern documented**: Comment explaining what it catches
- [ ] **Commit message complete**: Follows format above

### Common Pitfalls

**❌ Pattern too broad**:
```rust
// BAD: Matches "log", "login", "dialog", "catalog"
pattern: r"log"

// GOOD: Specific to rm command
pattern: r"rm\s+.*\.log$"
```

**❌ Pattern too specific**:
```rust
// BAD: Only matches exact spacing
pattern: r"rm -rf /tmp"

// GOOD: Flexible whitespace
pattern: r"rm\s+-rf\s+/tmp"
```

**❌ Missing edge cases**:
```rust
// BAD: Only matches one argument order
pattern: r"dd\s+if=/dev/zero\s+of=/dev/sda"

// GOOD: Catches both orders
pattern: r"dd\s+.*if=/dev/zero.*of=/dev/(sd|hd|nvme)"
pattern: r"dd\s+.*of=/dev/(sd|hd|nvme).*if=/dev/zero"  // Add second pattern
```

**❌ Not testing false positives**:
```rust
// Pattern catches both dangerous and safe commands
pattern: r"rm\s+.*\*"  // Too broad!

// Add test:
- input: "remove star character from filename"
  expected: "sed 's/\*//' file.txt"  // Should NOT be blocked
```

### Pattern Gap Analyzer

Use the automated tool to find missing variants:

```bash
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs
```

The analyzer checks for:
- Argument order variations
- Flag order variations
- Path variants (/, ~, ., ..)
- Wildcard coverage (*, *.*, .*)
- Platform equivalents (Bash ↔ PowerShell ↔ CMD)

### Pre-Commit Validation

Safety pattern changes are automatically validated:

1. **Hookify rule** (Claude Code): Shows checklist when committing
2. **Git pre-commit hook**: Blocks commit if patterns don't compile

To test manually:
```bash
# Test compilation
cargo build --lib

# Test with gap analyzer
./scripts/analyze-pattern-gaps.py src/safety/patterns.rs

# Test pattern tests
cargo test --lib safety::patterns
```

### Resources

- **Pattern reference**: `src/safety/patterns.rs` (all 55 patterns)
- **Test suites**: `.claude/beta-testing/*.yaml`
- **Gap analyzer**: `scripts/analyze-pattern-gaps.py`
- **Examples**: See commits with `feat(safety):` prefix
- **Skills**: Use `/skill safety-pattern-developer` in Claude Code

### Getting Help

- **Stuck on regex?** Test at regex101.com (Python flavor)
- **Need examples?** See `.claude/skills/safety-pattern-developer/examples/`
- **Found a gap?** Use issue template `.github/ISSUE_TEMPLATE/safety_pattern.yml`
- **Questions?** Ask in discussions with `label:safety`

### Documentation

Improve project documentation:

- **API documentation** - Expand rustdoc comments with examples
- **User guides** - Installation, configuration, troubleshooting
- **Architecture docs** - Module interaction diagrams, design decisions
- **Tutorial content** - Video walkthroughs, blog posts, demos

**Skills needed**: Technical writing, Rust knowledge (for API docs)

**Good first issues**: Search for `label:documentation` and `label:good-first-issue`

### Testing

Expand test coverage:

- **Contract tests** - Validate all public APIs match specifications
- **Integration tests** - Test cross-module workflows
- **Property tests** - Use proptest for invariant validation
- **Performance benchmarks** - Measure and track performance metrics

**Skills needed**: Rust testing, async testing with tokio-test

**Good first issues**: Search for `label:testing` and `label:good-first-issue`

#### Download Testing

The Hugging Face model download system has comprehensive test coverage using `wiremock` for HTTP mocking:

**Unit Tests** (`tests/download_unit_tests.rs`):
- Test successful downloads (200 OK responses)
- Test resume from partial downloads (206 Partial Content)
- Test authentication failures (401, 403)
- Test server errors (500, 503)
- Test network errors (timeouts, connection refused)
- Test checksum validation (match and mismatch scenarios)
- Test edge cases (404, 416 Range Not Satisfiable)

**Integration Tests** (`tests/download_integration_tests.rs`):
- End-to-end download with test fixtures
- Resume after simulated interruption
- Checksum validation with known good files
- Error recovery scenarios
- Concurrent downloads without corruption

**Authentication Testing**:

Downloads from Hugging Face Hub support authentication via the `HF_TOKEN` environment variable:

```bash
# Set your Hugging Face token for testing
export HF_TOKEN=hf_your_token_here

# Run tests that require authentication
cargo test --test download_unit_tests test_authentication
```

**Note**: Most tests use mocked HTTP responses and don't require a real token.

**Resume Capability**:

The download system automatically detects and resumes interrupted downloads:

- Uses `.part` files to track partial downloads
- Sends HTTP Range headers for resume requests
- Validates checksums after resume to ensure integrity
- Falls back to full re-download if server doesn't support Range
- Cleans up `.part` files on successful completion or checksum failure

**Limitations**:
- Resume only works if the `.part` file exists at the download location
- Server must support HTTP Range requests (most do, including HF Hub)
- Checksum validation requires the full file to be downloaded (no incremental validation)

**Example: Writing Tests with wiremock**:

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_download_with_auth() {
    let mock_server = MockServer::start().await;

    // Setup mock to expect Bearer token
    Mock::given(method("GET"))
        .and(path("/model/file.bin"))
        .and(header("Authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_bytes(b"model data"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::with_token("test_token".to_string()).unwrap();
    let url = format!("{}/model/file.bin", mock_server.uri());

    let result = download_file(&client, &url, &dest, None, None).await;
    assert!(result.is_ok());
}
```

#### LRU Cache Property-Based Testing

The LRU (Least Recently Used) cache eviction system uses property-based testing with `proptest` to verify correctness under randomized inputs (`tests/cache_lru_property_tests.rs`):

**Invariants Tested**:

1. **Size Constraint Invariant** - Cache never exceeds max_cache_size_bytes after cleanup
2. **Eviction Order Invariant** - Least recently accessed models are evicted first
3. **Completeness Invariant** - total_size_bytes always equals sum of individual model sizes
4. **Chronological Ordering Invariant** - Access time updates work correctly
5. **Minimal Eviction** - Cleanup removes only as many models as necessary
6. **No Duplicate Removals** - Each model ID appears at most once in removed list

**Property Test Example**:

```rust
proptest! {
    /// Property: After cleanup_lru(), total_size_bytes never exceeds max_cache_size_bytes
    #[test]
    fn prop_size_constraint_invariant(
        models in prop::collection::vec(model_strategy(), 1..20),
        max_size_gb in 1u64..5,
    ) {
        let mut manifest = CacheManifest::new(max_size_gb);

        // Add all models
        for (id, size_mb, offset_secs) in models {
            let model = create_test_model(&id, size_mb, offset_secs);
            manifest.add_model(model);
        }

        // Run cleanup
        let _removed = manifest.cleanup_lru();

        // INVARIANT: Cache size never exceeds limit after cleanup
        prop_assert!(
            manifest.total_size_bytes <= manifest.max_cache_size_bytes,
            "Cache size {} exceeds limit {} after cleanup",
            manifest.total_size_bytes,
            manifest.max_cache_size_bytes
        );
    }
}
```

**Running Property Tests**:

```bash
# Run all LRU property tests (generates 100+ random test cases)
cargo test --test cache_lru_property_tests

# Run specific property test
cargo test --test cache_lru_property_tests prop_eviction_order_invariant

# Run with verbose output to see generated test cases
cargo test --test cache_lru_property_tests -- --nocapture
```

**Test Coverage**:
- 7 property-based tests (each generates 100+ randomized test cases)
- 5 deterministic unit tests for specific edge cases
- All tests verify core LRU cache invariants hold under various scenarios

**Example: Testing Resume Functionality**:

```rust
#[tokio::test]
async fn test_resume_download() {
    let full_data = b"complete file content";
    let partial_data = &full_data[..10];
    let remaining_data = &full_data[10..];

    // Simulate interrupted download
    let part_path = dest_path.with_extension("part");
    tokio::fs::write(&part_path, partial_data).await.unwrap();

    // Mock expects Range header and returns 206
    Mock::given(method("GET"))
        .and(path("/model/file.bin"))
        .and(header("Range", "bytes=10-"))
        .respond_with(ResponseTemplate::new(206)
            .set_body_bytes(remaining_data.to_vec()))
        .mount(&mock_server)
        .await;

    // Download resumes automatically
    let result = download_file(&client, &url, &dest_path, None, None).await;
    assert!(result.is_ok());

    // Verify complete file
    let content = tokio::fs::read(&dest_path).await.unwrap();
    assert_eq!(content, full_data);
}
```

**Running Download Tests**:

```bash
# Run all download tests
cargo test download

# Run only unit tests
cargo test --test download_unit_tests

# Run only integration tests
cargo test --test download_integration_tests

# Run with output for debugging
cargo test download -- --nocapture
```

### Infrastructure

Improve development and deployment:

- **CI/CD pipelines** - GitHub Actions workflows
- **Release automation** - Binary builds, package distribution
- **Development containers** - devcontainer improvements
- **Benchmarking** - Criterion benchmark suite expansion

**Skills needed**: GitHub Actions, Docker, shell scripting

---

## Contribution Lanes (Advanced Contributors)

For experienced contributors, we organize work into **six contribution lanes** with clear ownership and success metrics:

### The Six Lanes

1. **Security Lane** (`lane/security`)
   - Focus: Guardrails, policy engine, red-team testing
   - Lead role: Red Team Captain
   - First issues: Policy engine MVP, risk scoring, CWD hard-binding

2. **Runtime Lane** (`lane/runtime`)
   - Focus: Tokio, streaming, backend orchestration
   - Lead role: Rust Architect
   - First issues: Streaming responses, cancellation, capability probing

3. **Inference Lane** (`lane/inference`)
   - Focus: Performance, quantization, benchmarking
   - Lead role: Performance Engineer
   - First issues: Benchmark harness, quantization comparison, CPU features

4. **UX Lane** (`lane/ux`)
   - Focus: Ratatui, confirmations, plan/review/apply
   - Lead role: TUI Designer
   - First issues: Confirm UI, plan flow, syntax highlighting

5. **Ecosystem Lane** (`lane/ecosystem`)
   - Focus: MCP, IDE integration, plugins
   - Lead role: Integration Engineer
   - First issues: MCP server, VS Code extension, Claude Desktop

6. **Distribution Lane** (`lane/distribution`)
   - Focus: Packaging, signing, offline bundles
   - Lead role: Packaging Maintainer
   - First issues: Nix flake, artifact signing, bundle creator

**See detailed lane information:** [HELP_WANTED.md](HELP_WANTED.md)

**Looking to lead a lane?** We're recruiting lane leads - open an issue titled "Lane Lead Application: [lane name]"

---

## Agent Collaboration

caro development leverages specialized AI agents for different tasks. When working on contributions, you may benefit from:

### TDD Development Agents

**tdd-rust-watcher**:
- Use during active TDD sessions
- Guides through Red-Green-Refactor cycles
- Provides minimal, incremental fixes
- Never breaks the TDD discipline

**tdd-rust-engineer**:
- Use for designing new features
- Emphasizes contract-first design
- Ensures comprehensive test coverage
- Applies Rust best practices

### Specialized Agents

**rust-cli-architect**:
- Complex architecture changes
- Module structure design
- Trait system design

**llm-integration-expert**:
- Backend implementation
- Model inference optimization
- Response parsing strategies

**qa-testing-expert**:
- Test coverage validation
- Quality assurance processes
- CI/CD improvements

See [AGENTS.md](docs/development/AGENTS.md) for repository guidelines and [CLAUDE.md](docs/development/CLAUDE.md) for agent usage details.

---

## Spec-Driven Development

caro follows a specification-first approach:

### Specification Structure

Each feature has a spec directory under `specs/[feature-id]/`:

```
specs/003-implement-core-infrastructure/
├── spec.md              # Feature requirements and acceptance criteria
├── plan.md              # Implementation plan with phases
├── contracts/           # API contracts for each module
│   ├── cache-api.md
│   ├── config-api.md
│   ├── logging-api.md
│   └── execution-api.md
├── quickstart.md        # User-facing scenarios
├── data-model.md        # Data structures and types
├── research.md          # Background research and decisions
└── tasks.md             # Implementation checklist
```

### Development Flow

1. **Specification phase**: Create or review spec in `specs/[feature-id]/spec.md`
2. **Contract definition**: Define module APIs in `specs/[feature-id]/contracts/`
3. **Test writing**: Create contract tests in `tests/contract/` based on contracts
4. **Implementation**: Use TDD cycles to implement features
5. **Integration**: Write integration tests for cross-module workflows
6. **Documentation**: Update quickstart guides and API docs

### Example: Adding a New Backend

```bash
# 1. Review backend specification
cat specs/[backend-id]/spec.md

# 2. Review CommandGenerator trait contract
cat specs/[backend-id]/contracts/backend-api.md

# 3. Write failing contract test
cat > tests/contract/vllm_contract.rs <<EOF
#[tokio::test]
async fn test_vllm_backend_implements_command_generator() {
    let backend = VllmBackend::new(config);
    let request = CommandRequest::new("list files");
    let result = backend.generate_command(&request).await;
    assert!(result.is_ok());
}
EOF

# 4. Implement backend with TDD
vim src/backends/vllm.rs

# 5. Watch tests turn green
# (cargo-watch running in background)
```

### Benchmark Workflow

caro uses **Criterion benchmarks** to validate performance requirements and track regressions. All performance-critical code should be benchmarked regularly.

#### When to Run Benchmarks

Run benchmarks in these situations:

1. **Before optimization PRs**: Establish a baseline before performance work
   ```bash
   cargo bench -- --save-baseline before
   ```

2. **After optimization PRs**: Measure improvement against baseline
   ```bash
   cargo bench -- --baseline before
   ```

3. **After large refactors**: Ensure performance hasn't regressed
   ```bash
   cargo bench
   ```

4. **Before release PRs**: Validate all performance targets are met
   ```bash
   cargo bench
   ```

5. **When modifying performance-critical code**:
   - Cache operations (`src/cache/**/*.rs`) → `cargo bench --bench cache`
   - Config loading (`src/config/**/*.rs`) → `cargo bench --bench config`
   - Context capture (`src/context/**/*.rs`) → `cargo bench --bench context`
   - Logging system (`src/logging/**/*.rs`) → `cargo bench --bench logging`

Use the **benchmark-advisor Claude skill** for intelligent suggestions:
```
User: "What benchmarks should I run?"
Claude: [analyzes git diff and suggests relevant benchmarks]
```

#### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench cache

# Run specific benchmark
cargo bench --bench cache -- get_model

# Compare against baseline
cargo bench -- --baseline before
```

See [docs/BENCHMARKING.md](docs/BENCHMARKING.md) for detailed guide on interpreting results.

#### Understanding Results

Criterion provides statistical analysis with:
- **Mean**: Average execution time
- **95% CI**: Confidence interval for the mean
- **Change %**: Performance change from baseline
- **p-value**: Statistical significance (p < 0.05 = significant change)

Example output:
```
cache/get_model         time:   [50.2 ns 51.5 ns 52.8 ns]
                        change: [-2.3% +0.5% +3.1%] (p = 0.42 > 0.05)
                        No change in performance detected.
```

**Interpretation**: Mean is 51.5 ns, change is not statistically significant.

#### Troubleshooting Benchmarks

**Noisy Results** (large variance, inconsistent measurements):
```bash
# Close resource-intensive applications
# Run multiple times to verify
cargo bench --bench cache
cargo bench --bench cache  # Repeat to confirm

# Check system load
top  # or htop
```

**Long Execution Time** (benchmarks take > 10 minutes):
```bash
# Run specific suites instead of full suite
cargo bench --bench cache
cargo bench --bench config

# Check for infinite loops in benchmark code
# Review benchmark sample size in benches/*.rs
```

**Baseline Not Found**:
```bash
# Error: "Baseline 'X' not found"
# Solution: Save baseline first
cargo bench -- --save-baseline X
```

**Regression Failures in CI**:
- CI fails if benchmarks regress beyond thresholds (15% time, 20% memory)
- Review the regression report in PR comments
- Investigate why performance degraded
- Optimize or justify the regression

#### CI Integration

Benchmarks run automatically in CI on:
- PRs to `release/*` branches (with regression detection)
- Weekly schedule (for historical tracking)
- Manual workflow dispatch

CI will:
1. Compare against baseline from main branch
2. Detect regressions beyond thresholds
3. Post regression report as PR comment
4. Fail build if critical regressions detected

See [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for current performance baselines.

---

## Recognition

We value all contributions and recognize contributors in several ways:

### Contributor Hall of Fame

Outstanding contributions are highlighted in our [CHANGELOG.md](CHANGELOG.md) with contributor attribution.

### First-Time Contributors

We maintain a welcoming environment for newcomers:
- Issues labeled `good-first-issue` are beginner-friendly
- Detailed templates guide issue and PR creation
- Maintainers provide supportive, educational feedback
- No contribution is too small

### Security Researchers

Security vulnerabilities are serious business. Researchers who responsibly disclose security issues are:
- Acknowledged in [SECURITY.md](SECURITY.md) Hall of Fame
- Credited in release notes
- Given priority support for future security research

See [SECURITY.md](SECURITY.md) for our vulnerability disclosure process.

### Regular Contributors

Consistent, high-quality contributions may lead to:
- Triage permissions (label management, issue assignment)
- Reviewer role (PR review privileges)
- Maintainer role (merge permissions, release management)

---

## Questions?

- **General questions**: [GitHub Discussions](https://github.com/wildcard/caro/discussions)
- **Bug reports**: [GitHub Issues](https://github.com/wildcard/caro/issues) with bug report template
- **Feature requests**: [GitHub Issues](https://github.com/wildcard/caro/issues) with feature request template
- **Security issues**: See [SECURITY.md](SECURITY.md) for private disclosure process

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code. Please report unacceptable behavior via GitHub Issues with the "conduct" label.

---

**Thank you for contributing to caro!** Together we're building a safer, faster, more accessible way to harness LLMs for shell command generation with BSD/GNU-level security and trust.

---

*Last updated: 2025-12-24*
