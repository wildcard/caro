# Contributing to cmdai

Thank you for your interest in contributing to cmdai! We're building a safety-first, high-performance CLI tool that brings the power of local LLMs to shell command generation. Whether you're fixing bugs, adding features, improving documentation, or expanding safety patterns, your contributions are welcome.

## Table of Contents

- [Project Vision](#project-vision)
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

cmdai aims to make shell command generation:
- **Safe by default** - Comprehensive validation prevents destructive operations
- **Blazingly fast** - Sub-100ms startup, sub-2s inference on Apple Silicon
- **Truly local** - No cloud dependencies, works offline with cached models
- **Developer-friendly** - Single binary, minimal configuration, clear error messages
- **Community-driven** - Open development, transparent processes, welcoming to all skill levels

We prioritize correctness, safety, and user experience above all else.

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
   git clone https://github.com/YOUR_USERNAME/cmdai.git
   cd cmdai
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

cmdai follows strict **Test-Driven Development (TDD)** with spec-driven design. See [TDD-WORKFLOW.md](docs/development/TDD-WORKFLOW.md) for complete details.

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
        .context("Failed to load configuration from ~/.config/cmdai/config.toml")?
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

cmdai is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**, a strong copyleft license that ensures the software and all contributions remain free and open source. To accept your contributions, we use a **Contributor License Agreement (CLA)** to:

- Confirm you have the legal right to contribute your code
- Grant the project the necessary rights to distribute your contributions under AGPL-3.0
- Protect contributors, users, and the project from legal uncertainty
- Maintain the copyleft nature of the project

**Important**: Unlike some projects that use CLAs for dual licensing, cmdai commits to keeping all contributions under AGPL-3.0. We will not use your contributions under proprietary licenses.

### How to Sign the CLA

Signing the CLA is simple and happens directly on your pull request:

1. **Read the CLA**: Review [CLA.md](CLA.md) to understand the terms
2. **Comment on your PR**: Add this exact comment to your pull request:
   ```
   I have read the CLA Document and I hereby sign the CLA
   ```
3. **Continue**: Once signed, maintainers can proceed with reviewing your contribution

**First-time contributors**: You only need to sign the CLA once. Your signature covers all future contributions to cmdai.

### Alternative: Developer Certificate of Origin (DCO)

We also provide a **Developer Certificate of Origin (DCO)** option as specified in [DCO.txt](DCO.txt). The DCO is a lighter-weight alternative used by projects like the Linux kernel, Node.js, and Spring Framework.

If you prefer to use DCO instead of the CLA, you can sign off your commits using:

```bash
git commit -s -m "Your commit message"
```

This adds a `Signed-off-by:` line to your commit message, indicating you agree to the DCO terms.

**Note**: You must use either CLA or DCO for your contributions to be accepted. Choose whichever you're more comfortable with.

### Questions About the CLA?

- **"Why does an AGPL project need a CLA?"** - While AGPL-3.0 itself provides strong protections, the CLA ensures we have clear documentation that contributors have the right to submit their code and agree to its use under AGPL-3.0.
- **"Will my code be used in proprietary software?"** - No. cmdai commits to maintaining AGPL-3.0 licensing for all contributions.
- **"What if my employer owns my code?"** - You'll need employer permission to contribute. See section 4 of the CLA for details.
- **"Can I submit third-party code?"** - Yes, but it must be clearly marked and comply with section 7 of the CLA.

For more questions, open an issue with the `question` label or start a discussion on GitHub.

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
- 📝 **Documentation** - Add rustdoc examples to public APIs ([#7](https://github.com/wildcard/cmdai/issues/7))
- 🧪 **Testing** - Add property-based tests for LRU eviction ([#8](https://github.com/wildcard/cmdai/issues/8))
- 📊 **Benchmarking** - Create performance benchmark suite ([#9](https://github.com/wildcard/cmdai/issues/9))
- 🔧 **Tooling** - Generate JSON schema for config validation ([#11](https://github.com/wildcard/cmdai/issues/11))

**Looking for a bigger challenge?**
- 🚀 **Feature 004** - Implement Hugging Face model downloads ([#10](https://github.com/wildcard/cmdai/issues/10))
- 🔒 **Security** - Add file permission hardening ([#6](https://github.com/wildcard/cmdai/issues/6))
- ✅ **Test Alignment** - Fix contract test API mismatches ([#4](https://github.com/wildcard/cmdai/issues/4))

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

### Infrastructure

Improve development and deployment:

- **CI/CD pipelines** - GitHub Actions workflows
- **Release automation** - Binary builds, package distribution
- **Development containers** - devcontainer improvements
- **Benchmarking** - Criterion benchmark suite expansion

**Skills needed**: GitHub Actions, Docker, shell scripting

---

## Agent Collaboration

cmdai development leverages specialized AI agents for different tasks. When working on contributions, you may benefit from:

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

See [AGENTS.md](docs/development/AGENTS.md) for repository guidelines and [CLAUDE.md](CLAUDE.md) for agent usage details.

---

## Spec-Driven Development

cmdai follows a specification-first approach:

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

- **General questions**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- **Bug reports**: [GitHub Issues](https://github.com/wildcard/cmdai/issues) with bug report template
- **Feature requests**: [GitHub Issues](https://github.com/wildcard/cmdai/issues) with feature request template
- **Security issues**: See [SECURITY.md](SECURITY.md) for private disclosure process

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code. Please report unacceptable behavior via GitHub Issues with the "conduct" label.

---

**Thank you for contributing to cmdai!** Together we're building a safer, faster, more accessible way to harness LLMs for shell command generation.

---

*Last updated: 2025-10-03*
