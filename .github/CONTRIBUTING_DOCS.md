# Contributing to cmdai Documentation

Welcome! 👋 We're excited that you want to help improve cmdai's documentation.

Good documentation is essential for making cmdai accessible to everyone - from beginners just getting started to experienced developers building complex workflows. Your contribution matters!

## Quick Start

### 1. Choose Your Contribution Type

Pick what fits your interest and time:

| Type | Time Needed | Best For |
|------|------------|----------|
| **Fix typo/error** | 5-10 minutes | Quick improvements |
| **Improve existing doc** | 30-60 minutes | Clarity and completeness |
| **Add example** | 1-2 hours | Practical demonstrations |
| **Write tutorial** | 3-5 hours | Teaching workflows |
| **Create guide** | 5-10 hours | Comprehensive documentation |

### 2. Set Up Your Environment

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Install mdBook
cargo install mdbook

# Preview documentation locally
cd book
mdbook serve --open
```

The documentation will open in your browser at `http://localhost:3000` with live reload enabled.

### 3. Make Your Changes

Documentation lives in `book/src/`:

```
book/src/
├── user-guide/        # How to use cmdai
├── dev-guide/         # How to contribute code
├── technical/         # Deep technical details
├── tutorial/          # Step-by-step guides
└── community/         # Community resources
```

**Common tasks:**

- **Fix typo**: Edit the `.md` file directly
- **Add example**: Edit relevant guide page
- **New page**: Create `.md` file and add to `SUMMARY.md`
- **Code docs**: Edit Rust source files with `///` comments

### 4. Submit Your Contribution

```bash
# Create a branch
git checkout -b docs/your-improvement

# Add your changes
git add .
git commit -m "docs: improve installation instructions"

# Push and create PR
git push origin docs/your-improvement
```

Then open a Pull Request on GitHub!

## Documentation Standards

### Writing Style

✅ **DO:**
- Use clear, simple language
- Provide examples for everything
- Explain the "why", not just the "how"
- Test all code examples
- Use active voice: "Run the command" not "The command should be run"

❌ **DON'T:**
- Assume prior knowledge
- Use unexplained jargon
- Leave incomplete sections
- Skip examples
- Write without testing

### Code Examples

Every code example should:
1. **Work** - Test it before submitting
2. **Be complete** - Include necessary context
3. **Show output** - What should users see?
4. **Explain** - What does it demonstrate?

**Good example:**

```markdown
Generate a command to find large files:

```bash
cmdai "find files larger than 100MB"
```

**Generated command:**
```bash
find . -type f -size +100M -ls
```

**Sample output:**
```
-rw-r--r--  1 user  staff  256M video.mp4
-rw-r--r--  1 user  staff  150M archive.zip
```
\```
```

### Structure

Use consistent heading hierarchy:

```markdown
# Page Title (H1 - only one per page)

Brief introduction

## Main Section (H2)

Content

### Subsection (H3)

Details

#### Minor Point (H4)

Specifics
```

## Types of Documentation Contributions

### Quick Fixes (10 minutes)

Found a typo? Broken link? Unclear sentence? Fix it!

1. Click "Edit this file" on the page
2. Make your change
3. Submit a PR

No issue needed for obvious fixes.

### Improving Existing Documentation (30-60 minutes)

Making existing docs better:

1. **Identify the problem** - What's unclear?
2. **Research the answer** - Test in code if needed
3. **Update the docs** - Be clear and thorough
4. **Add examples** - Show, don't just tell
5. **Submit PR** - Reference what you improved

### Adding Examples (1-2 hours)

Great examples help users learn faster:

1. **Choose a real use case** - What do users actually do?
2. **Write working code** - Test it thoroughly
3. **Show expected output** - What should they see?
4. **Explain key points** - Why does it work this way?
5. **Submit PR** - Use "example" label

See our [example request template](.github/ISSUE_TEMPLATE/example-request.yml) for ideas.

### Writing Tutorials (3-5 hours)

Tutorials teach complete workflows:

1. **Define learning goals** - What will users achieve?
2. **Outline the steps** - Break into manageable chunks
3. **Write step-by-step** - One task at a time
4. **Test thoroughly** - Follow your own tutorial
5. **Get feedback** - Ask someone to try it
6. **Submit PR** - Use "tutorial" label

See our [tutorial request template](.github/ISSUE_TEMPLATE/tutorial-request.yml) for guidance.

### Creating Guides (5-10 hours)

Comprehensive guides cover entire topics:

1. **Plan structure** - Outline all sections
2. **Research thoroughly** - Understand deeply
3. **Write systematically** - One section at a time
4. **Add examples throughout** - Practical demonstrations
5. **Review and refine** - Multiple passes
6. **Get team review** - Tag @docs-team
7. **Submit PR** - Mention in description

## Documentation Checklist

Before submitting your PR:

### Content
- [ ] Information is accurate and tested
- [ ] Examples work as shown
- [ ] All links are valid
- [ ] No TODOs or placeholders
- [ ] Follows documentation style guide

### Structure
- [ ] Proper heading hierarchy
- [ ] Added to `SUMMARY.md` (if new page)
- [ ] Clear navigation path
- [ ] Related pages linked

### Code Examples
- [ ] All commands tested
- [ ] Output shown where helpful
- [ ] Errors handled appropriately
- [ ] Platform-specific notes included

### Project Documentation
- [ ] CHANGELOG.md updated (if applicable)
- [ ] "What's Being Built" updated (if applicable)
- [ ] PR description explains changes
- [ ] Linked to related issues

## Getting Help

Need assistance with your documentation contribution?

### Resources
- **Full Documentation Guide**: [book/src/community/documentation-guide.md](../book/src/community/documentation-guide.md)
- **Examples**: Browse existing documentation for inspiration
- **Templates**: Use our issue templates for ideas

### Community Support
- **GitHub Discussions**: Ask questions or share ideas
- **PR Reviews**: Request feedback from @docs-team
- **Issues**: Create a documentation issue if you're not sure how to fix something

### Common Questions

**Q: I'm not a native English speaker. Can I still contribute?**

A: Absolutely! Write what you can, and we'll help polish it. Technical accuracy is most important.

**Q: Should I open an issue before submitting a PR?**

A: For small fixes (typos, broken links), just submit a PR. For larger changes (new pages, major rewrites), opening an issue first helps us coordinate.

**Q: How do I know if my documentation is good enough?**

A: If it:
1. Provides accurate information
2. Includes working examples
3. Is clear to you when you read it back

Then it's good enough! We'll help refine it during review.

**Q: Can I contribute documentation for features that don't exist yet?**

A: Yes! Add to "What's Being Built" to track in-progress work. User-facing docs should wait until the feature is ready to merge.

## Examples of Great Documentation PRs

### Small Fix
```
Title: docs: fix broken link in installation guide
Description: The link to Rust installation was outdated.
Changed: Updated URL to current rustup.rs site
```

### Medium Improvement
```
Title: docs: add troubleshooting section for MLX backend
Description: Users were confused about MLX setup on Apple Silicon
Added:
- System requirements
- Common errors and solutions
- Performance tuning tips
Changed: Installation guide now links to troubleshooting
```

### Large Contribution
```
Title: docs: add comprehensive tutorial for CI/CD integration
Description: New tutorial showing how to use cmdai in automated workflows
Added:
- Complete GitHub Actions tutorial
- GitLab CI example
- Best practices section
- Troubleshooting guide
Updated:
- SUMMARY.md with new page
- CHANGELOG.md with new tutorial
- "What's Being Built" with completion
```

## Recognition

We celebrate documentation contributions! Great documentation gets you:

- **Credit**: Listed in contributor showcase
- **Mentions**: Featured in release notes for significant contributions
- **Badges**: "Documentation Champion" recognition
- **Community**: Appreciation from users you've helped

## Next Steps

Ready to contribute?

1. **Browse issues**: Look for [documentation label](https://github.com/wildcard/cmdai/labels/documentation)
2. **Pick a task**: Start with `good-first-doc` label
3. **Ask questions**: No question is too small
4. **Submit PR**: We're here to help!

Thank you for helping make cmdai documentation better! 📚

---

## Related Resources

- **Full Documentation Guide**: [book/src/community/documentation-guide.md](../book/src/community/documentation-guide.md)
- **General Contributing Guide**: [CONTRIBUTING.md](../CONTRIBUTING.md)
- **Code of Conduct**: [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)
- **Project Roadmap**: [book/src/community/roadmap.md](../book/src/community/roadmap.md)

---

**Questions?** Open a [discussion](https://github.com/wildcard/cmdai/discussions) or create an [issue](https://github.com/wildcard/cmdai/issues/new/choose)!
