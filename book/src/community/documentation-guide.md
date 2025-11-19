# Documenting Your Work

> 📝 **Make Your Contribution Visible** - A complete guide to documenting your work in cmdai

Whether you're adding a feature, fixing a bug, or improving documentation, **your work deserves to be seen and celebrated**. This guide shows you exactly how to document your contributions.

---

## 🎯 Why Document Your Work?

### Benefits for You
- ✨ **Recognition** - Get credit for your contribution
- 🎓 **Portfolio** - Showcase your work to employers
- 🤝 **Collaboration** - Help others build on your work
- 💬 **Feedback** - Get valuable input from community

### Benefits for the Project
- 📚 **Knowledge sharing** - Help others understand the codebase
- 🚀 **Onboarding** - Make it easier for new contributors
- 🔍 **Visibility** - Show progress to users and stakeholders
- 📖 **Maintainability** - Future developers will thank you

---

## 🗺️ Documentation Workflow

### Quick Overview

```
1. Start Work
   ├─→ Create branch
   ├─→ Add to "What's Being Built"
   └─→ Link to issue/discussion

2. During Development
   ├─→ Update progress %
   ├─→ Write inline code docs
   └─→ Add examples as you go

3. Before PR
   ├─→ Write/update user docs
   ├─→ Add to changelog
   └─→ Update roadmap if needed

4. After Merge
   ├─→ Move to "completed" section
   ├─→ Update contributor showcase
   └─→ Share in community channels
```

---

## 📋 Step-by-Step Guide

### Step 1: Announce Your Work

**As soon as you start**, add your work to the docs!

#### A. Update "What's Being Built"

1. Open `book/src/community/active-development.md`
2. Add your feature under the appropriate priority section
3. Use this template:

```markdown
#### Your Feature Name
**Branch:** `feature/your-feature-name`
**Contributors:** @your-github-username
**Status:** 🌱 Planning (10% complete)

**What it does:**
- User-facing description (what problem it solves)
- Key benefits for users

**Progress:**
- [x] Initial research
- [ ] Implementation
- [ ] Testing
- [ ] Documentation

**Help Needed:**
- List any areas where you'd like help
- Specific skills needed

**PR:** #XXX (when created) | **Discussion:** #YYY
```

#### B. Update Project Roadmap (if applicable)

If your feature is new to the roadmap:
1. Open `book/src/community/roadmap.md`
2. Add to the appropriate phase
3. Link to your entry in "What's Being Built"

#### C. Create/Link Discussion

1. Go to [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
2. Create a discussion for your feature
3. Tag it appropriately (e.g., "feature", "backend", "safety")
4. Link it in your documentation entry

---

### Step 2: Document As You Code

#### A. Inline Documentation

Every public function/struct needs docs:

```rust
/// Generates a shell command from natural language input
///
/// # Arguments
/// * `prompt` - Natural language description of desired command
/// * `config` - Configuration for generation behavior
///
/// # Returns
/// * `Ok(GeneratedCommand)` - Successfully generated command
/// * `Err(GeneratorError)` - Generation failed
///
/// # Examples
/// ```rust
/// let prompt = "list all files";
/// let result = generate_command(prompt, &config).await?;
/// assert_eq!(result.command, "ls -la");
/// ```
///
/// # Safety
/// The generated command is validated for safety before returning.
/// See `SafetyValidator` for details.
pub async fn generate_command(
    prompt: &str,
    config: &Config,
) -> Result<GeneratedCommand, GeneratorError> {
    // Implementation
}
```

**Documentation checklist:**
- [ ] Summary line (first line)
- [ ] Detailed description
- [ ] All parameters documented
- [ ] Return values documented
- [ ] Error cases documented
- [ ] Examples provided
- [ ] Safety considerations (if applicable)

#### B. Module Documentation

Add to the top of each module:

```rust
//! Backend system for LLM inference
//!
//! This module provides a unified interface for multiple LLM backends,
//! including embedded models (MLX, CPU) and remote APIs (Ollama, vLLM).
//!
//! # Architecture
//!
//! All backends implement the `CommandGenerator` trait, allowing for
//! polymorphic backend selection and graceful fallback.
//!
//! # Examples
//!
//! ```rust
//! use cmdai::backends::create_backend;
//!
//! let backend = create_backend(BackendType::Embedded, &config)?;
//! let command = backend.generate_command(&request).await?;
//! ```
```

#### C. Tests as Documentation

Write descriptive test names:

```rust
#[tokio::test]
async fn test_safety_validator_blocks_rm_rf_root() {
    // Test implementation
}

#[tokio::test]
async fn test_ollama_backend_retries_on_network_failure() {
    // Test implementation
}
```

---

### Step 3: Create User Documentation

#### A. Choose the Right Location

Depends on what you're documenting:

| What You Built | Where to Document |
|---|---|
| New feature for users | `book/src/user-guide/` |
| Backend/architecture | `book/src/dev-guide/` |
| Technical deep dive | `book/src/technical/` |
| Tutorial/example | `book/src/tutorial/` |
| API reference | Code comments → auto-generated |

#### B. Create Your Documentation Page

1. Create a new `.md` file in the appropriate directory
2. Add it to `book/src/SUMMARY.md`
3. Use this template:

```markdown
# Your Feature Name

Brief introduction explaining what this is and why it matters.

## What It Does

Clear explanation of the feature from a user perspective.

## How to Use It

### Basic Usage

```bash
cmdai your-feature "example"
```

### Advanced Usage

[More complex examples]

## Configuration

[If applicable]

## Examples

### Example 1: Common Use Case

[Step-by-step example]

### Example 2: Advanced Scenario

[Another example]

## Troubleshooting

Common issues and solutions

## Next Steps

Links to related documentation
```

#### C. Add Examples

**Good examples include:**
- Clear input/output
- Expected behavior
- Error cases
- Real-world scenarios

Example:
```markdown
### Example: Find Large Files

```bash
cmdai "find files larger than 100MB"
```

**Generated:**
```bash
find . -type f -size +100M -ls
```

**Output:**
```
-rw-r--r--  1 user  staff  256M video.mp4
-rw-r--r--  1 user  staff  150M archive.zip
```
```

---

### Step 4: Update Progress

Keep your entry in "What's Being Built" up to date:

```markdown
**Status:** 🔨 Implementation (60% complete)

**Progress:**
- [x] Initial research
- [x] Core implementation
- [ ] Edge case handling (in progress)
- [ ] Testing
- [ ] Documentation
```

**Status Emojis:**
- 🌱 Planning/Research (0-20%)
- 🔨 Implementation (20-70%)
- 🧪 Testing (70-90%)
- 📝 Documentation (90-95%)
- ✅ Complete/Merged (100%)

---

### Step 5: Before Submitting PR

#### A. Documentation Checklist

Before opening your PR, ensure:

**Code Documentation:**
- [ ] All public functions have rustdoc comments
- [ ] Module-level documentation complete
- [ ] Examples provided in code comments
- [ ] Tests are well-named and documented

**User Documentation:**
- [ ] Feature documented in appropriate guide
- [ ] Examples provided for common use cases
- [ ] Configuration options explained (if any)
- [ ] Troubleshooting section added

**Project Documentation:**
- [ ] Entry in "What's Being Built" is complete
- [ ] Roadmap updated (if needed)
- [ ] CHANGELOG.md updated
- [ ] Related docs updated

#### B. Update CHANGELOG.md

Add your changes to the unreleased section:

```markdown
## [Unreleased]

### Added
- New embedded model backend with MLX support (@your-name)
- Model caching system (@your-name)

### Changed
- Improved error messages for backend failures (@your-name)

### Fixed
- Memory leak in long-running sessions (@your-name)
```

#### C. Link Documentation in PR

In your PR description:

```markdown
## Documentation

User docs: [MLX Integration](link-to-docs)
Developer docs: [Backend Development](link-to-docs)
Examples: [Quick Start](link-to-docs)

## Changelog Entry

Added to CHANGELOG.md under "Unreleased"
```

---

### Step 6: After PR is Merged

#### A. Move to Completed

Update "What's Being Built":
```markdown
### Recently Completed ✅

#### Your Feature Name
**Merged:** 2024-11-19
**PR:** #123
**Contributors:** @you, @collaborator
**Docs:** [Link to docs]

[Brief description of what was accomplished]
```

#### B. Update Contributor Showcase

Add yourself to `book/src/community/contributors.md`:

```markdown
### @your-name
**Contributions:** MLX backend, model caching, documentation
**Joined:** November 2024
**Areas of Interest:** Performance, Apple Silicon, LLMs

[Brief bio if you want]
```

#### C. Celebrate! 🎉

Share your achievement:
- GitHub Discussions
- Discord/Slack
- Twitter with #cmdai
- LinkedIn

---

## 📝 Documentation Templates

### Feature Documentation Template

```markdown
# Feature Name

> Brief tagline explaining the feature

## Overview

What this feature does and why it matters.

## Quick Start

Minimal example to get started:

```bash
cmdai --feature "example"
```

## How It Works

Detailed explanation with diagrams if helpful.

## Configuration

[If applicable]

## Examples

### Example 1: Basic Usage
[Step-by-step]

### Example 2: Advanced Usage
[More complex scenario]

## Troubleshooting

### Issue: Common Problem
**Solution:** How to fix it

## Best Practices

Tips for using this feature effectively

## Next Steps

- [Related Feature A](link)
- [Related Feature B](link)
```

### API Documentation Template

```rust
/// Brief summary (one line)
///
/// Longer description explaining what this does, when to use it,
/// and any important considerations.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// * `Ok(Type)` - Description of success case
/// * `Err(Error)` - Description of error cases
///
/// # Errors
///
/// This function will return an error if:
/// - Condition 1
/// - Condition 2
///
/// # Examples
///
/// ```rust
/// use cmdai::module::function;
///
/// let result = function(arg1, arg2)?;
/// assert_eq!(result, expected);
/// ```
///
/// # Panics
///
/// This function will panic if [condition]
///
/// # Safety
///
/// [If unsafe or security-relevant]
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

---

## 🎨 Documentation Best Practices

### Writing Style

#### DO:
- ✅ Use active voice: "The function validates..." not "Validation is performed..."
- ✅ Be concise but complete
- ✅ Include examples for everything
- ✅ Explain the "why", not just the "what"
- ✅ Use proper terminology consistently

#### DON'T:
- ❌ Assume knowledge - explain concepts
- ❌ Use jargon without explanation
- ❌ Write walls of text - use headings
- ❌ Forget examples
- ❌ Leave TODOs or incomplete sections

### Code Examples

#### Good Example:
```rust
// Show actual usage with context
let validator = SafetyValidator::new(SafetyConfig::strict());
let result = validator.validate("rm -rf /");

if result.is_dangerous() {
    println!("⚠️  Dangerous command detected!");
}
```

#### Bad Example:
```rust
// Too abstract, no context
validate(command);
```

### Visual Aids

Use when helpful:
- Diagrams for architecture
- Tables for comparisons
- Flowcharts for logic
- Screenshots for UI

---

## 🛠️ Tools and Automation

### Building the Docs Locally

```bash
# Install mdBook
cargo install mdbook

# Build docs
cd book && mdbook build

# Serve with live reload
mdbook serve --open
```

### Check Documentation Coverage

```bash
# Run rustdoc
cargo doc --no-deps --document-private-items

# Open in browser
open target/doc/cmdai/index.html
```

### Lint Documentation

```bash
# Check for broken links
mdbook test

# Check code examples compile
cargo test --doc
```

---

## 🤝 Getting Help with Documentation

### Need Help?

- **Writing:** Ask in #documentation on Discord
- **Technical:** Tag @docs-team in discussions
- **Review:** Request review in your PR
- **Examples:** See existing docs for inspiration

### Documentation Reviews

Request a documentation review:
1. Tag @docs-team in your PR
2. Mention in #documentation channel
3. Ask for specific feedback

We'll help with:
- Structure and organization
- Writing clarity
- Example quality
- Completeness

---

## 📊 Documentation Metrics

We track documentation quality:

- **Coverage:** % of public APIs documented
- **Examples:** Code examples per feature
- **Freshness:** Days since last update
- **Completeness:** Required sections present

**Goal:** 100% coverage of public APIs with examples

---

## 🏆 Documentation Awards

We recognize great documentation!

### Documentation Champions
Contributors with exceptional docs get featured:
- Contributor showcase highlight
- "Documentation Champion" badge
- Featured in monthly newsletter

### Recent Champions
- **@doc-master** - Comprehensive backend docs
- **@example-guru** - Amazing tutorial examples
- **@diagram-wizard** - Architectural diagrams

---

## 📚 Resources

### Templates
- [Feature Documentation Template](#feature-documentation-template)
- [API Documentation Template](#api-documentation-template)
- [Tutorial Template](../tutorial/first-command.md) - See source

### Examples of Great Docs
- [Rust Book](https://doc.rust-lang.org/book/)
- [React Documentation](https://react.dev/)
- [MDN Web Docs](https://developer.mozilla.org/)

### Tools
- [mdBook](https://rust-lang.github.io/mdBook/)
- [rustdoc](https://doc.rust-lang.org/rustdoc/)
- [Mermaid](https://mermaid.js.org/) - Diagrams

---

## ❓ FAQ

### Q: I'm just fixing a small bug. Do I need to document it?
**A:** Update the CHANGELOG.md and add a note in "What's Being Built". Full docs only if it changes user-facing behavior.

### Q: My feature isn't complete. Should I still document it?
**A:** Yes! Add it to "What's Being Built" as in-progress. This helps others see what you're working on and potentially collaborate.

### Q: I'm not a native English speaker. Can I still write docs?
**A:** Absolutely! Write what you can, and we'll help polish it. The technical content is most important.

### Q: Where do I document internal/private APIs?
**A:** Use rustdoc comments in the code. These show up in `cargo doc` for developers.

### Q: Can I add my feature to the docs before it's merged?
**A:** Yes, in "What's Being Built"! User guides should be added when the feature is ready to merge.

---

## 🎯 Next Steps

Ready to document your work?

1. **Start:** Add your work to [What's Being Built](./active-development.md)
2. **Code:** Write rustdoc comments as you develop
3. **Guide:** Create user documentation
4. **Share:** Submit your PR with docs included

**Questions?** Ask in [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)

---

## 📖 Related Pages

- **[What's Being Built](./active-development.md)** - Add your work here
- **[Project Roadmap](./roadmap.md)** - See the big picture
- **[Contributing Guide](./contributing.md)** - General contribution guidelines
- **[Contributor Showcase](./contributors.md)** - Get recognized!

---

**Your documentation matters!** It helps users, helps contributors, and showcases your amazing work. Don't be shy - document everything! 📝
