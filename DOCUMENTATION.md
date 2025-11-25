# Documentation Guide

Welcome to the cmdai documentation! This guide provides an overview of our documentation structure, how to navigate it, and how to contribute.

## Quick Navigation

- **[Full Documentation Site](https://wildcard.github.io/cmdai/)** - Complete documentation built with mdBook
- **[Getting Started](book/src/user-guide/getting-started.md)** - Start here if you're new to cmdai
- **[Contributing](book/src/community/contributing.md)** - Learn how to contribute to the project

## Documentation Structure

Our documentation is organized into several main sections:

### 📚 Learn by Example

**Best for:** New users who prefer hands-on learning

- **[Your First Command](book/src/tutorial/first-command.md)** - Generate your first shell command
- **[Working with Files](book/src/tutorial/working-with-files.md)** - File operations and manipulation
- **[System Operations](book/src/tutorial/system-operations.md)** - System administration tasks
- **[Try It Online](book/src/tutorial/playground.md)** - Interactive playground (coming soon)

These tutorials are interactive and example-driven, allowing you to learn by doing.

### 👤 User Guide

**Best for:** Users who want to understand cmdai's features and configuration

- **[Getting Started](book/src/user-guide/getting-started.md)** - Installation and initial setup
- **[Installation](book/src/user-guide/installation.md)** - Platform-specific installation guides
- **[Quick Start](book/src/user-guide/quick-start.md)** - Common use cases and patterns
- **[Safety & Security](book/src/user-guide/safety.md)** - Understanding cmdai's safety features
- **[Configuration](book/src/user-guide/configuration.md)** - Customizing cmdai for your needs

### 👨‍💻 Developer Guide

**Best for:** Contributors and developers extending cmdai

- **[Contributing](book/src/dev-guide/contributing.md)** - How to contribute to cmdai
- **[Architecture](book/src/dev-guide/architecture.md)** - System design and components
- **[Backend Development](book/src/dev-guide/backends.md)** - Building LLM integration backends
- **[Testing Strategy](book/src/dev-guide/testing.md)** - Testing approach and guidelines
- **[TDD Workflow](book/src/dev-guide/tdd-workflow.md)** - Test-driven development process

### 🔬 Technical Deep Dives

**Best for:** Advanced developers and architects

- **[Rust Learnings](book/src/technical/rust-learnings.md)** - Lessons learned building cmdai in Rust
- **[Safety Validation](book/src/technical/safety-validation.md)** - Command safety system design
- **[MLX Integration](book/src/technical/mlx-integration.md)** - Apple Silicon optimization
- **[Performance Optimization](book/src/technical/performance.md)** - Performance tuning strategies

### 📖 Reference

**Best for:** Looking up specific information

- **[QA Test Cases](book/src/reference/qa-test-cases.md)** - Complete test case catalog
- **[Technical Debt](book/src/reference/tech-debt.md)** - Known issues and future improvements
- **[Changelog](book/src/reference/changelog.md)** - Version history and changes
- **[Security Policy](book/src/reference/security.md)** - Security practices and reporting
- **[Code of Conduct](book/src/reference/code-of-conduct.md)** - Community guidelines

### 🌍 Community

**Best for:** Getting involved and connecting with others

- **[Contributing Guide](book/src/community/contributing.md)** - Detailed contribution guidelines
- **[Development Agents](book/src/community/agents.md)** - Specialized AI agents for development
- **[Project Roadmap](book/src/community/roadmap.md)** - Long-term vision and plans
- **[What's Being Built](book/src/community/active-development.md)** - Current work and opportunities
- **[Documenting Your Work](book/src/community/documentation-guide.md)** - How to document contributions
- **[Contributor Showcase](book/src/community/contributors.md)** - Meet the community

## Building Documentation Locally

### Prerequisites

Install mdBook:

```bash
cargo install mdbook
```

### Building and Serving

```bash
# Navigate to the book directory
cd book/

# Build the documentation
mdbook build

# Serve with hot-reload for development
mdbook serve

# Open in browser (usually http://localhost:3000)
```

The documentation will automatically rebuild when you make changes to markdown files.

### Building for Production

```bash
cd book/
mdbook build --dest-dir ../docs
```

This builds static HTML files suitable for deployment to GitHub Pages or other hosting.

## Documentation Philosophy

Our documentation follows these principles:

### 1. Example-Driven Learning

We believe the best way to learn is by doing. Every concept is illustrated with practical examples that users can try immediately.

### 2. Progressive Disclosure

Documentation is structured to reveal complexity gradually:
- **Tutorials** - Learn basics through guided examples
- **User Guide** - Understand features and usage
- **Developer Guide** - Contribute and extend
- **Technical Deep Dives** - Master advanced concepts

### 3. Multiple Learning Paths

Different users have different needs:
- **"I want to try it"** → Start with tutorials
- **"I need to install it"** → Go to installation guide
- **"I want to contribute"** → Check contributing guide
- **"I need to understand how it works"** → Read architecture docs

### 4. Living Documentation

Documentation evolves with the project:
- **Up-to-date** - Reflects current implementation
- **Versioned** - Historical versions available
- **Community-driven** - Anyone can contribute improvements

### 5. Accessible and Inclusive

- Clear, concise language
- No assumed knowledge
- Helpful error messages and troubleshooting
- Examples for different skill levels

## Contributing to Documentation

We welcome documentation contributions! Here's how to help:

### Types of Documentation Contributions

1. **Fix Typos and Errors** - Spotted a mistake? Fix it!
2. **Improve Clarity** - Make explanations clearer
3. **Add Examples** - More examples are always helpful
4. **Fill Gaps** - Document undocumented features
5. **Create Tutorials** - Help others learn
6. **Update Screenshots** - Keep visuals current
7. **Translate** - Help non-English speakers (future)

### Documentation Workflow

1. **Find What to Work On**
   - Check [documentation issues](https://github.com/wildcard/cmdai/labels/documentation)
   - Look for "TODO" or "FIXME" comments in docs
   - Identify gaps or unclear sections

2. **Make Changes**
   - Fork the repository
   - Edit markdown files in `book/src/`
   - Test locally with `mdbook serve`
   - Ensure links work and formatting is correct

3. **Submit Changes**
   - Create a pull request
   - Describe what you improved and why
   - Reference any related issues
   - Add yourself to the [Contributor Showcase](book/src/community/contributors.md)

4. **Review Process**
   - Maintainers will review your changes
   - Address any feedback
   - Once approved, your changes will be merged and deployed

### Documentation Standards

#### Writing Style

- **Clear and concise** - Get to the point quickly
- **Active voice** - "Run this command" not "This command should be run"
- **Present tense** - "cmdai generates" not "cmdai will generate"
- **Second person** - "You can use" not "One can use"

#### Formatting

- **Code blocks** - Always specify language for syntax highlighting
- **Commands** - Show full commands including `$` prompt
- **File paths** - Use `code formatting` for paths
- **Emphasis** - Use **bold** for important points, *italics* sparingly

#### Examples

- **Complete** - Show full working examples
- **Realistic** - Use real-world scenarios
- **Explained** - Add comments or explanations
- **Tested** - Verify examples actually work

### Documentation File Structure

```
book/
├── book.toml              # mdBook configuration
├── src/                   # Source markdown files
│   ├── SUMMARY.md        # Table of contents
│   ├── introduction.md   # Landing page
│   ├── tutorial/         # Interactive tutorials
│   ├── user-guide/       # User documentation
│   ├── dev-guide/        # Developer documentation
│   ├── technical/        # Technical deep dives
│   ├── reference/        # Reference materials
│   └── community/        # Community resources
└── theme/                # Custom styling (optional)
```

## Getting Help

If you have questions about documentation:

1. **Check existing docs** - The answer might already be there
2. **Search issues** - Someone might have asked already
3. **Ask in discussions** - Use [GitHub Discussions Q&A](https://github.com/wildcard/cmdai/discussions/categories/q-a)
4. **Open an issue** - For documentation bugs or gaps

## Documentation Roadmap

### Current Status

- ✅ Core documentation structure established
- ✅ Interactive tutorials created
- ✅ Developer and user guides written
- ✅ Community and contribution pages complete
- ✅ GitHub Pages deployment configured

### Coming Soon

- 🚧 Interactive playground
- 🚧 Video tutorials
- 🚧 API documentation (rustdoc)
- 🚧 Troubleshooting guide expansion
- 🚧 Performance benchmarks

### Future Plans

- 📅 Multi-language support
- 📅 Advanced search functionality
- 📅 Version-specific documentation
- 📅 Community-contributed tutorials
- 📅 Integration examples

## Recognition

Documentation contributors are celebrated in our [Contributor Showcase](book/src/community/contributors.md) with special recognition for:

- First documentation contribution
- Significant documentation improvements
- New tutorial creation
- Translation work
- Maintaining documentation quality

## Additional Resources

- **[mdBook Documentation](https://rust-lang.github.io/mdBook/)** - Learn about mdBook features
- **[Markdown Guide](https://www.markdownguide.org/)** - Markdown syntax reference
- **[Documentation Style Guide](book/src/community/documentation-guide.md)** - Our specific guidelines
- **[GitHub Pages](https://pages.github.com/)** - Deployment platform

---

**Questions about documentation?** Open a [GitHub Discussion](https://github.com/wildcard/cmdai/discussions) or check our [Contributing Guide](book/src/community/contributing.md).

Thank you for helping make cmdai's documentation better for everyone!
