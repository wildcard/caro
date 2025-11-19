# Welcome to cmdai! 🎉
## Your Guide to Getting Started as a Contributor

> *We're so glad you're here! This guide will help you make your first contribution.*

---

## 👋 Hello!

First off, **thank you** for considering contributing to cmdai!

Whether you're:
- Fixing a typo in the docs
- Adding a new safety pattern
- Building a major feature
- Answering questions in discussions

**Your contribution matters.**

This guide will help you get started, regardless of your experience level.

---

## 🎯 Quick Start (5 Minutes)

### If You Want to Contribute Code

```bash
# 1. Fork the repo on GitHub (click "Fork" button)

# 2. Clone your fork
git clone https://github.com/YOUR-USERNAME/cmdai.git
cd cmdai

# 3. Set up Rust (if you haven't)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. Build the project
. "$HOME/.cargo/env"  # Load Rust into your shell
cargo build

# 5. Run tests to make sure everything works
cargo test

# 6. Create a branch for your work
git checkout -b fix/my-awesome-contribution

# 7. Make your changes!
# 8. Test them: cargo test
# 9. Format code: cargo fmt
# 10. Lint code: cargo clippy

# 11. Commit with a clear message
git commit -m "fix: describe what you fixed"

# 12. Push to your fork
git push origin fix/my-awesome-contribution

# 13. Open a Pull Request on GitHub!
```

**That's it!** We'll take it from there.

---

### If You Want to Contribute Without Code

**Documentation:**
- Fix typos or unclear explanations
- Add examples
- Improve getting started guides
- Translate docs (coming soon)

**Community:**
- Answer questions in GitHub Discussions
- Help newcomers on Discord
- Share your cmdai experiences
- Report bugs with clear reproduction steps

**Ideas:**
- Suggest new safety patterns
- Propose features
- Share use cases
- Provide feedback

**No code required! These are equally valuable.**

---

## 🗺️ Finding Something to Work On

### For First-Time Contributors

Start here:
- Look for [`good first issue`](https://github.com/wildcard/cmdai/labels/good%20first%20issue) label
- Check [`documentation`](https://github.com/wildcard/cmdai/labels/documentation) label
- Browse `help wanted` issues

**Perfect first contributions:**
- Fix a typo in README.md
- Add a safety pattern to the docs
- Improve error messages
- Add tests for existing code
- Update outdated documentation

---

### For Experienced Contributors

Try these:
- [`help wanted`](https://github.com/wildcard/cmdai/labels/help%20wanted) - Features we'd love help with
- [`performance`](https://github.com/wildcard/cmdai/labels/performance) - Optimization opportunities
- [`enhancement`](https://github.com/wildcard/cmdai/labels/enhancement) - New features

**Bigger projects:**
- New backend integrations
- Platform support (Windows, etc.)
- Plugin system development
- Performance optimizations

---

### Not Sure Where to Start?

**Ask!** We're here to help:
- Comment on an issue: "I'd like to work on this!"
- Open a discussion: "I want to contribute but not sure where to start"
- Join Discord and ask in #help channel

**We will:**
- Suggest good issues for your skill level
- Answer questions
- Provide context and guidance
- Review your work constructively

---

## 🎓 Understanding the Codebase

### Project Structure

```
cmdai/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── backends/         # LLM backend implementations
│   │   ├── mod.rs       # Backend trait
│   │   ├── mlx.rs       # Apple Silicon backend
│   │   ├── vllm.rs      # vLLM backend
│   │   └── ollama.rs    # Ollama backend
│   ├── safety/          # Command validation
│   │   └── mod.rs       # Safety validator
│   ├── config/          # Configuration management
│   ├── cli/             # CLI argument parsing
│   └── models/          # Data structures
├── tests/               # Tests
│   ├── integration/     # End-to-end tests
│   └── unit/            # Component tests
├── docs/                # Documentation
├── culture/             # Community & culture docs
└── brand-assets/        # Brand guidelines
```

---

### Key Concepts

**1. Backend Trait System**
All LLM backends implement `CommandGenerator` trait:
```rust
#[async_trait]
trait CommandGenerator {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

**2. Safety Validator**
Checks commands against dangerous patterns:
- System destruction (`rm -rf /`)
- Fork bombs
- Privilege escalation
- Path traversal

**3. Risk Levels**
- **SAFE** (Green) - Execute freely
- **MODERATE** (Yellow) - Ask confirmation
- **HIGH** (Orange) - Require explicit approval
- **CRITICAL** (Red) - Blocked by default

---

## 🛠️ Development Workflow

### Setting Up Your Environment

**Required:**
- Rust 1.75+ ([install here](https://rustup.rs/))
- Git
- Your favorite editor (VSCode, vim, etc.)

**Optional:**
- `cargo-watch` for auto-rebuilds
- `cargo-audit` for security checks
- MLX (if developing on Apple Silicon)

**Recommended VSCode Extensions:**
- rust-analyzer
- Even Better TOML
- GitLens

---

### Making Changes

**1. Create a branch**
```bash
git checkout -b category/description
```

**Categories:**
- `fix/` - Bug fixes
- `feat/` - New features
- `docs/` - Documentation
- `test/` - Test additions
- `refactor/` - Code improvements

**Examples:**
- `fix/safety-validator-regex`
- `feat/windows-support`
- `docs/ollama-setup-guide`

---

**2. Make your changes**

Follow these principles:
- **Start with tests** (TDD approach)
- **Keep it simple** (boring code is good code)
- **Consider safety** (every change impacts security)
- **Document as you go** (future you will thank you)

---

**3. Test your changes**

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Test a specific file
cargo test --test integration_test
```

**Make sure:**
- All existing tests still pass
- New code has tests
- Tests are meaningful (not just for coverage)

---

**4. Format and lint**

```bash
# Format code (required before PR)
cargo fmt

# Check formatting without changing
cargo fmt -- --check

# Run linter (fix all warnings)
cargo clippy -- -D warnings

# Security audit
cargo audit
```

**These must pass for PR to be merged.**

---

**5. Commit your changes**

**Good commit messages:**
```
fix: prevent panic when config file is missing

- Add validation for config file existence
- Return helpful error message with example path
- Add test for missing config scenario

Closes #123
```

**Bad commit messages:**
```
fixed stuff
WIP
asdf
```

**Format:**
```
type: short description (50 chars max)

Longer explanation if needed (72 chars max per line).
Explain WHY, not just WHAT.

Closes #issue-number
```

**Types:** fix, feat, docs, test, refactor, chore

---

**6. Push and create PR**

```bash
# Push to your fork
git push origin your-branch-name

# Open Pull Request on GitHub
# Fill out the PR template
# Wait for review (usually within 1 week)
```

---

## 📝 Pull Request Guidelines

### Before You Submit

**Checklist:**
- [ ] Tests added/updated
- [ ] All tests pass (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated (if needed)
- [ ] Commit messages are clear
- [ ] PR description explains the change

---

### PR Description Template

```markdown
## What does this PR do?
Brief description of the changes.

## Why is this needed?
Explain the problem this solves or feature it adds.

## How was this tested?
Describe how you verified the changes work.

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] All CI checks passing

## Screenshots (if applicable)
[Add screenshots for UI changes]

Closes #issue-number
```

---

### Review Process

**What happens next:**
1. **Automated checks run** (CI tests, linting, etc.)
2. **Maintainers review** (usually within 1 week)
3. **You address feedback** (if any)
4. **PR gets merged!** 🎉

**During review, maintainers may:**
- Ask questions (to understand your approach)
- Suggest improvements
- Request tests or docs
- Approve immediately (if it's great!)

**We will NOT:**
- Be rude or dismissive
- Reject without explanation
- Ghost you

---

### If Your PR Needs Changes

**Don't worry!** This is normal and expected.

```bash
# Make requested changes
# Commit them
git add .
git commit -m "address review feedback"

# Push to same branch
git push origin your-branch-name
```

**The PR updates automatically!**

**Tips:**
- Ask questions if feedback is unclear
- Take your time to get it right
- Learn from the process
- Say thank you (maintainers are volunteers too!)

---

## 🎯 Types of Contributions

### Code Contributions

**Backend Development:**
- New inference backends (Anthropic, OpenAI, etc.)
- Backend performance improvements
- Error handling enhancements

**Safety Validation:**
- New dangerous command patterns
- Improved pattern matching
- False positive reduction
- Risk assessment improvements

**CLI Features:**
- New command-line options
- Output format improvements
- Interactive mode enhancements
- Shell integration

**Performance:**
- Startup time optimization
- Inference speed improvements
- Memory usage reduction
- Caching strategies

---

### Documentation Contributions

**User Documentation:**
- Getting started guides
- Configuration examples
- Troubleshooting guides
- FAQ additions

**Developer Documentation:**
- Architecture explanations
- API documentation
- Contributing guides
- Code comments

**Examples:**
- Real-world use cases
- Integration guides
- Best practices
- Video tutorials

---

### Community Contributions

**Support:**
- Answer questions in Discussions
- Help troubleshoot issues
- Welcome new contributors
- Share your experiences

**Advocacy:**
- Write blog posts
- Give conference talks
- Create tutorials
- Share on social media

**Feedback:**
- Report bugs with clear reproduction
- Suggest features
- Share use cases
- Provide UX feedback

---

## 🧪 Testing Guidelines

### What to Test

**Unit Tests:**
- Individual functions
- Edge cases
- Error handling
- Input validation

**Integration Tests:**
- Full workflows
- Backend communication
- Configuration loading
- Command generation + validation

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_dangerous_rm_rf() {
        let validator = SafetyValidator::new();
        let result = validator.validate("rm -rf /");

        assert!(result.is_blocked());
        assert_eq!(result.risk_level(), RiskLevel::Critical);
    }
}
```

---

### Test-Driven Development (TDD)

**Our preferred workflow:**

1. **Write a failing test** (Red)
```rust
#[test]
fn test_new_safety_pattern() {
    // This will fail initially
    assert!(validator.detects_fork_bomb(":(){ :|:& };:"));
}
```

2. **Write code to make it pass** (Green)
```rust
impl SafetyValidator {
    fn detects_fork_bomb(&self, cmd: &str) -> bool {
        FORK_BOMB_PATTERN.is_match(cmd)
    }
}
```

3. **Refactor** (Clean)
- Improve code quality
- Remove duplication
- Add documentation

---

## 🤝 Getting Help

### Where to Ask

**Technical questions:**
- GitHub Discussions (best for async)
- Discord #help channel (real-time)
- Issue comments (context-specific)

**Process questions:**
- "How do I...?"
- "What's the best way to...?"
- "Should I...?"

**We're here to help!**

---

### How to Ask

**Good question:**
```
I'm trying to add a safety pattern for detecting SQL injection attempts.
I've added the pattern here: [link to code]

But the test is failing with this error: [error message]

I expected [X] but got [Y]. Am I missing something about how the
validator parses commands?
```

**Needs more context:**
```
My code doesn't work. Help?
```

**Include:**
1. What you're trying to do
2. What you've tried
3. What happened
4. Relevant code/errors
5. Your environment (OS, Rust version)

---

## 🎉 Your First Contribution

### Suggested First PRs

**Super Easy (15 minutes):**
1. Fix a typo in README.md
2. Add an example to documentation
3. Improve a code comment
4. Add yourself to CONTRIBUTORS.md (after first PR merges!)

**Easy (30 minutes):**
1. Add a dangerous command pattern to safety validator
2. Improve an error message
3. Add a test for existing functionality
4. Update outdated documentation

**Medium (1-2 hours):**
1. Implement a small CLI feature
2. Add configuration option
3. Write a troubleshooting guide
4. Create an integration test

**Start small!** Build confidence before tackling big features.

---

## 📚 Learning Resources

### Rust

**New to Rust?**
- [The Rust Book](https://doc.rust-lang.org/book/) (free, comprehensive)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) (interactive exercises)

**Rust Concepts We Use:**
- Traits and trait objects
- Async/await
- Error handling with `Result`
- Testing with `cargo test`

---

### cmdai-Specific

**Read these:**
1. [CLAUDE.md](../CLAUDE.md) - Project overview
2. [Cultural Handbook](CULTURAL_HANDBOOK.md) - Our values
3. [Architecture docs](../docs/) - System design

**Watch these:**
(Coming soon: Video walkthroughs)

---

## ⚡ Tips for Success

### From Experienced Contributors

**1. Start small and ship often**
Many small PRs > One massive PR

**2. Communicate early**
Open a draft PR or discussion before investing days of work.

**3. Ask questions**
There are no stupid questions. We were all new once.

**4. Read existing code**
See how similar features are implemented.

**5. Don't let perfect be the enemy of good**
Ship it, get feedback, iterate.

**6. Have fun!**
This is a community, not a job. Enjoy the process.

---

### Common Pitfalls to Avoid

**1. Not reading the PR template**
We provide it for a reason! It helps reviewers.

**2. Skipping tests**
Untested code will be sent back for tests.

**3. Making PRs too large**
Smaller PRs get reviewed faster.

**4. Not asking for help**
Stuck for 3 hours? Ask! We'll unstick you in 3 minutes.

**5. Taking rejection personally**
Sometimes we can't merge a PR. It's not about you!

---

## 🏆 Recognition

### We Celebrate Contributors!

**When your PR merges:**
- You're added to CONTRIBUTORS.md
- We thank you on Twitter
- You get a Discord role
- You're eligible for swag!

**After 5 contributions:**
- Invited to contributor calls
- Priority on feature requests
- Special Discord perks

**After 20 contributions:**
- Considered for maintainer role
- cmdai swag package
- Listed on website

**Your work matters. We make sure people know it.**

---

## 🌱 Growing as a Contributor

### The Contributor Journey

**Level 1: First PR**
- Fix typo, add test, improve docs
- **Goal:** Get comfortable with the process

**Level 2: Regular Contributor**
- Fix bugs, add features, help others
- **Goal:** Understand the codebase

**Level 3: Trusted Contributor**
- Review others' PRs, mentor newcomers
- **Goal:** Help the community grow

**Level 4: Maintainer**
- Merge PRs, make decisions, set direction
- **Goal:** Steward the project

**There's no rush. Contribute at your own pace.**

---

## 💬 Final Thoughts

You made it to the end! Here's what to remember:

**You belong here.**
- No contribution is too small
- Questions are welcome
- Mistakes are how we learn

**We're here to help.**
- Ask questions
- Request feedback
- Seek mentorship

**Start now.**
- Pick a "good first issue"
- Fix that typo you noticed
- Answer a question in Discussions

**Your first contribution is one PR away.**

---

## 🚀 Ready to Contribute?

**Choose your path:**

**I want to code:**
→ Browse [good first issues](https://github.com/wildcard/cmdai/labels/good%20first%20issue)

**I want to write docs:**
→ Check [documentation issues](https://github.com/wildcard/cmdai/labels/documentation)

**I want to help the community:**
→ Answer questions in [Discussions](https://github.com/wildcard/cmdai/discussions)

**I'm not sure yet:**
→ Join [Discord](link-tbd) and introduce yourself!

---

**Welcome to cmdai. We're better because you're here.**

⚡🛡️ **Think Fast. Stay Safe. Build Together.**

---

*Questions about contributing?*
*Open a discussion or ask in #help on Discord.*
*We're excited to work with you!*
