# Welcome to cmdai!

Thank you for your interest in contributing to cmdai! We're excited to have you here.

This guide will help you get started, whether you're making your first open-source contribution or you're a seasoned developer looking to help build the future of safe, AI-powered command generation.

## Quick Start Checklist

- [ ] Read this welcome guide
- [ ] Explore the [What's Being Built](../book/src/community/active-development.md) page
- [ ] Set up your development environment
- [ ] Run the tests to verify your setup
- [ ] Choose something to work on
- [ ] Join the community and say hello!

## What is cmdai?

cmdai is a Rust CLI tool that converts natural language descriptions into safe, POSIX-compliant shell commands using local LLMs. We prioritize:

- **Safety First** - Comprehensive validation prevents dangerous operations
- **Privacy** - Local inference, no cloud dependencies
- **Performance** - <100ms startup, <2s inference on Apple Silicon
- **Extensibility** - Multiple backend support (MLX, Ollama, vLLM)
- **Community** - Welcoming, inclusive, and transparent development

## Your First Steps

### 1. Explore the Project (15 minutes)

**Read the basics:**
- [README.md](../README.md) - Project overview
- [What's Being Built](../book/src/community/active-development.md) - Current work and opportunities
- [Contributor Showcase](../book/src/community/contributors.md) - Meet the community

**Try the tool** (if available):
```bash
# Build from source
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release

# Try it out
./target/release/cmdai --version
./target/release/cmdai "list all files"
```

### 2. Set Up Development Environment (20 minutes)

**Prerequisites:**
- Rust 1.75+ ([install here](https://rustup.rs/))
- Git
- A code editor (we recommend VS Code with rust-analyzer)

**Clone and setup:**
```bash
# Fork the repository on GitHub first, then clone your fork
git clone https://github.com/YOUR_USERNAME/cmdai.git
cd cmdai

# Add upstream remote
git remote add upstream https://github.com/wildcard/cmdai.git

# Install dependencies and build
cargo build

# Run tests to verify setup
cargo test

# Check formatting and linting
cargo fmt --check
cargo clippy
```

**Optional but recommended:**
```bash
# Install helpful tools
cargo install cargo-watch  # Auto-rebuild on changes
cargo install cargo-audit  # Security auditing
cargo install mdbook      # Build documentation
```

### 3. Understand the Architecture (15 minutes)

Read these key documents:
- [Architecture Overview](../book/src/dev-guide/architecture.md) - System design
- [TDD Workflow](../book/src/dev-guide/tdd-workflow.md) - Development process
- [Testing Strategy](../book/src/dev-guide/testing.md) - How we test

**Key concepts:**
- **Backend Trait System** - All LLM backends implement `CommandGenerator`
- **Safety Validator** - Validates commands before execution
- **Configuration** - TOML-based user and system configuration
- **Async Runtime** - Tokio for asynchronous operations

### 4. Choose What to Work On

We have opportunities for all skill levels!

**Good First Issues:**
Browse issues labeled [`good first issue`](https://github.com/wildcard/cmdai/labels/good%20first%20issue) - these are:
- Well-defined and scoped
- Don't require deep codebase knowledge
- Have mentorship available
- Great for learning the project

**Documentation:**
- Fix typos or improve clarity
- Add examples to guides
- Create new tutorials
- Update screenshots or diagrams

**Testing:**
- Add test cases
- Improve test coverage
- Test on different platforms
- Report bugs with reproduction steps

**Code:**
- Implement features from the roadmap
- Fix bugs
- Performance improvements
- New backend implementations

**Design:**
- Improve CLI output formatting
- Create diagrams and visuals
- Design error messages
- UI/UX improvements

**Community:**
- Answer questions in discussions
- Help other contributors
- Write blog posts
- Share on social media

### 5. Make Your Contribution

**Development workflow:**

```bash
# 1. Update your fork
git checkout main
git pull upstream main

# 2. Create a feature branch
git checkout -b feature/your-feature-name

# 3. Make your changes
# - Write code
# - Add tests
# - Update documentation

# 4. Run tests and checks
cargo test
cargo fmt
cargo clippy

# 5. Commit your changes
git add .
git commit -m "feat: add your feature description"

# 6. Push to your fork
git push origin feature/your-feature-name

# 7. Open a Pull Request on GitHub
```

**Commit message conventions:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions or changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

## Getting Help

**Stuck? Need guidance? We're here to help!**

### Where to Ask

**Questions about the code:**
- Open a [GitHub Discussion](https://github.com/wildcard/cmdai/discussions)
- Ask in your pull request
- Check existing issues for similar questions

**General help:**
- Read the [Contributing Guide](../book/src/community/contributing.md)
- Check the [Documentation](https://wildcard.github.io/cmdai/)
- Review the [FAQ](../book/src/user-guide/faq.md) (if available)

**Bug reports:**
- Search [existing issues](https://github.com/wildcard/cmdai/issues)
- Use the bug report template
- Include reproduction steps

**Feature ideas:**
- Check the [roadmap](../book/src/community/roadmap.md)
- Open a [feature request](https://github.com/wildcard/cmdai/issues/new)
- Discuss in GitHub Discussions first for major changes

### Getting Faster Responses

To get help quickly:

1. **Be specific** - Include error messages, code snippets, steps to reproduce
2. **Show effort** - Share what you've tried
3. **Be patient** - Maintainers are volunteers (typically respond within 24-48 hours)
4. **Follow up** - Add more info if needed
5. **Say thanks** - We appreciate kindness!

## Contribution Ideas by Skill Level

### Beginner-Friendly

**No coding required:**
- Fix typos in documentation
- Improve README clarity
- Report bugs with reproduction steps
- Test on different platforms
- Share cmdai on social media

**Light coding:**
- Add test cases
- Improve error messages
- Add code comments
- Enhance logging

**Learning Rust:**
- Add simple safety patterns
- Implement configuration options
- Write integration tests
- Improve CLI output formatting

### Intermediate

**Rust developers:**
- Implement new features from roadmap
- Refactor code for clarity
- Improve error handling
- Add comprehensive tests
- Performance optimization

**LLM/AI experience:**
- Improve prompt engineering
- Add new backend integrations
- Optimize model selection
- Enhance response parsing

**Systems programming:**
- Cross-platform compatibility fixes
- Shell-specific features
- Process management improvements
- Performance profiling

### Advanced

**Experienced Rust developers:**
- Architecture improvements
- Complex feature implementation
- Performance optimization
- FFI work (MLX integration)
- Advanced async patterns

**Security focus:**
- Security audits
- Safety validator improvements
- Threat modeling
- Penetration testing

**ML/AI experts:**
- Model optimization
- Custom backend implementations
- Inference performance tuning
- Advanced prompt strategies

## Your Impact

Every contribution matters! Here's how your work creates impact:

### Immediate Impact

- **Code merged** - Your work goes live for all users
- **Showcased** - Featured in [What's Being Built](../book/src/community/active-development.md)
- **Recognized** - Added to [Contributor Showcase](../book/src/community/contributors.md)

### Learning Opportunities

- **Rust skills** - Real-world Rust development
- **LLM integration** - Practical AI/ML experience
- **Open source** - Collaboration and code review
- **CLI design** - User interface expertise

### Career Benefits

- **Portfolio** - Showcase your work
- **Experience** - Practical project experience
- **Network** - Connect with other developers
- **Reputation** - Build your open-source profile

### Community Building

- **Help others** - Enable safer shell command usage
- **Mentorship** - Learn from and teach others
- **Recognition** - Get credit for your contributions
- **Belonging** - Be part of something bigger

## Community Values

We're committed to creating a welcoming, inclusive community:

### We Value

- **All contributions** - Code, docs, testing, design, community help
- **All skill levels** - Beginners and experts both welcome
- **Diverse perspectives** - Different backgrounds and experiences
- **Respectful collaboration** - Kindness and professionalism
- **Continuous learning** - Growth mindset and experimentation

### We Don't Tolerate

- Harassment or discrimination
- Dismissive or condescending behavior
- Spam or self-promotion
- Code of Conduct violations

See our [Code of Conduct](../book/src/reference/code-of-conduct.md) for details.

## Recognition System

**Your work won't go unnoticed!**

### Automatic Recognition

When you contribute, you'll be:

1. **Featured in "What's Being Built"** - Work visible from day one
2. **Added to Contributor Showcase** - Profile with your contributions
3. **Mentioned in release notes** - Credit in version announcements
4. **Eligible for monthly awards** - Top contributor recognition

### Types of Recognition

- **First Contribution** - Special welcome and recognition
- **Regular Contributor** - Consistency and dedication
- **Impact Awards** - Significant contributions
- **Community Champion** - Helping others
- **Documentation Hero** - Doc improvements
- **Bug Hunter** - Finding and fixing bugs
- **Feature Builder** - New functionality

## Next Steps

Ready to contribute? Here's what to do next:

### This Week

1. **Set up your environment** - Follow the setup instructions
2. **Run the tests** - Verify everything works
3. **Pick an issue** - Start with a `good first issue`
4. **Say hello** - Introduce yourself in discussions

### This Month

1. **Make your first contribution** - Open a PR
2. **Help someone else** - Answer a question
3. **Learn something new** - Read a technical deep dive
4. **Share your experience** - Blog or tweet about it

### Ongoing

1. **Stay engaged** - Regular small contributions
2. **Grow your skills** - Take on bigger challenges
3. **Help newcomers** - Pay it forward
4. **Share feedback** - Help improve the project

## Frequently Asked Questions

### "I'm new to Rust. Can I still contribute?"

Absolutely! We have many non-code contribution opportunities, and we welcome Rust learners. Start with documentation, testing, or good first issues.

### "I'm new to open source. Where do I start?"

Start by reading through this guide, exploring the codebase, and picking a `good first issue`. Don't hesitate to ask questions!

### "How long does PR review take?"

We aim to provide initial feedback within 24-48 hours. Full review and merge may take a few days depending on complexity.

### "Can I work on something not in the issues?"

Yes! But for significant changes, please open an issue or discussion first to align with project goals and avoid duplicate work.

### "I made a mistake in my PR. What do I do?"

No problem! Push another commit to fix it, or ask for help. Everyone makes mistakes - it's part of learning.

### "Can I contribute if I disagree with design decisions?"

Yes! Respectful discussion is welcome. Open an issue or discussion to talk about alternative approaches.

## Resources

### Documentation
- [Full Documentation](https://wildcard.github.io/cmdai/)
- [Getting Started Guide](../book/src/user-guide/getting-started.md)
- [Contributing Guide](../book/src/community/contributing.md)
- [Architecture Overview](../book/src/dev-guide/architecture.md)

### Learning Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

### Project Links
- [GitHub Repository](https://github.com/wildcard/cmdai)
- [Issue Tracker](https://github.com/wildcard/cmdai/issues)
- [Discussions](https://github.com/wildcard/cmdai/discussions)
- [Roadmap](../book/src/community/roadmap.md)

## Thank You!

We're grateful you're interested in contributing to cmdai. Every contribution, no matter how small, makes a difference.

Whether you fix a typo, add a test, implement a feature, or help someone in discussions - you're making cmdai better for everyone.

**Welcome to the community! We can't wait to see what you'll build.**

---

**Questions?** Open a [GitHub Discussion](https://github.com/wildcard/cmdai/discussions) or check our [Contributing Guide](../book/src/community/contributing.md).

**Ready to contribute?** Check out [What's Being Built](../book/src/community/active-development.md) to see current opportunities!
