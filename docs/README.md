# cmdai Documentation

> Complete guide for understanding and contributing to cmdai

## 📚 Documentation Index

This directory contains comprehensive documentation explaining what cmdai is, how it works, and how to contribute.

### For New Collaborators

**Start here** → Read documents in this order:

1. **[ONE_PAGER.md](ONE_PAGER.md)** (5 min read)
   - Quick overview of the entire project
   - Perfect for understanding the basics
   - Great for sharing with others

2. **[PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md)** (15 min read)
   - Detailed explanation of the problem and solution
   - Use cases and examples
   - Core philosophy and future roadmap

3. **[ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md)** (20 min read)
   - System components broken down simply
   - Sub-agent roles and responsibilities
   - Code organization and design principles

4. **[WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md)** (10 min read)
   - Visual flow diagrams
   - Step-by-step processing
   - Decision trees and state machines

### For Different Audiences

#### 👥 **I'm a Product Manager / Stakeholder**
- Start: [ONE_PAGER.md](ONE_PAGER.md)
- Read: [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "Why This Matters" section
- Skip: Technical architecture details

**What you'll learn:**
- What problem we're solving
- Target users and use cases
- Business value and differentiation
- Roadmap and future plans

---

#### 💻 **I'm a Developer / Contributor**
- Start: [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md)
- Deep dive: [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md)
- Reference: [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) when implementing
- Check: `../CLAUDE.md` for technical standards

**What you'll learn:**
- How the system works internally
- Sub-agent architecture and responsibilities
- Where to find specific components in code
- How to contribute effectively

---

#### 🎨 **I'm a UX Designer**
- Start: [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "User Interaction" sections
- Focus: [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) → "User Interaction States"
- Reference: [ONE_PAGER.md](ONE_PAGER.md) → Example scenarios

**What you'll learn:**
- User journey and interaction flow
- Safety communication patterns
- Color coding and risk presentation
- Confirmation workflows

---

#### 🔒 **I'm a Security Researcher**
- Start: [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → "Security Analyst" section
- Deep dive: `../src/safety/mod.rs` (source code)
- Reference: [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "Safety Features"

**What you'll learn:**
- Security threat model
- Pattern matching rules
- Risk assessment methodology
- Safety validation layers

---

#### 📖 **I'm Writing Documentation**
- Read all documents for consistency
- Follow tone and structure patterns
- Reference: [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) for style guide
- Check: `../README.md` for project status

**Key principles:**
- Simple language, avoid jargon
- Visual diagrams where possible
- Real examples with output
- Clear section hierarchy

---

## 📋 Document Summaries

### [ONE_PAGER.md](ONE_PAGER.md)
**Purpose**: Quick reference and elevator pitch
**Length**: ~2 pages
**Audience**: Everyone (non-technical friendly)

**Contents:**
- Problem statement
- Solution overview
- Safety levels table
- Quick examples
- Technology stack
- TL;DR section

**When to use:**
- Introducing someone to the project
- Sharing on social media
- Quick team reference
- Investor/stakeholder briefings

---

### [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md)
**Purpose**: Complete product understanding
**Length**: ~8 pages
**Audience**: All collaborators

**Contents:**
- Detailed problem analysis
- Complete solution flow
- Core features breakdown
- System architecture overview
- Example scenarios (safe/risky/critical)
- Future enhancements
- Philosophy and principles

**When to use:**
- Onboarding new team members
- Understanding project scope
- Product planning discussions
- Feature prioritization meetings

---

### [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md)
**Purpose**: Technical system design
**Length**: ~10 pages
**Audience**: Developers and architects

**Contents:**
- Sub-agent detailed breakdown
- Component responsibilities
- Code organization
- Communication patterns
- Design principles
- Contribution areas by component

**When to use:**
- Planning implementation
- Code reviews
- Architecture discussions
- Finding where to contribute

---

### [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md)
**Purpose**: Visual system flow reference
**Length**: ~6 pages (mostly diagrams)
**Audience**: Visual learners, all roles

**Contents:**
- Complete system flow diagram
- Decision trees
- Backend selection logic
- Risk assessment process
- User interaction states
- Example walkthroughs

**When to use:**
- Understanding data flow
- Debugging issues
- Designing new features
- Explaining to visual learners

---

## 🎯 Quick Navigation by Topic

### Understanding the Problem
- [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "The Problem We're Solving"
- [ONE_PAGER.md](ONE_PAGER.md) → "The Problem"

### How It Works
- [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "Simple Flow"
- [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) → "Complete System Flow"

### Safety System
- [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → "Security Analyst Sub-Agent"
- [ONE_PAGER.md](ONE_PAGER.md) → "Safety Levels"
- [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) → "Safety Pattern Matching"

### Architecture
- [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → Full document
- [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "Key Components"

### User Experience
- [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "Example Scenarios"
- [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) → "User Interaction States"
- [ONE_PAGER.md](ONE_PAGER.md) → "Example Session"

### Contributing
- [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → "For New Contributors"
- [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md) → "Getting Started as a Collaborator"

### Technical Details
- `../CLAUDE.md` → Technical implementation guide
- `../README.md` → Project status and setup
- [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → Code organization

---

## 🏗️ Document Evolution

These documents were created based on a conversation about designing safe CLI inference with:
- Security threat modeling
- Rule-based validation
- Community-driven guidance (future)
- Multi-agent architecture

**Key Concepts Captured:**
1. **Safety First**: Multiple validation layers before execution
2. **User Agency**: Always confirm risky operations
3. **Transparency**: Explain what commands do and why
4. **Local Execution**: Privacy and performance
5. **Modular Design**: Sub-agents with clear responsibilities

---

## 📝 Documentation Standards

When updating or adding documentation:

### ✅ Do:
- Use clear, simple language
- Include practical examples
- Add visual diagrams where helpful
- Structure with clear headings
- Provide "Quick Start" sections
- Cross-reference related documents

### ❌ Don't:
- Assume technical knowledge
- Use unexplained jargon
- Write walls of text without breaks
- Duplicate information (link instead)
- Leave outdated information

### Format Guidelines:
- Use emoji for visual scanning (sparingly)
- Code blocks with syntax highlighting
- Tables for comparison data
- Boxes/borders for important callouts
- Hierarchical numbering for steps

---

## 🔄 Keeping Docs Updated

### When to Update:

| Event | Documents to Update |
|-------|---------------------|
| New feature added | [PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md), [ONE_PAGER.md](ONE_PAGER.md) |
| Architecture change | [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md), [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) |
| Safety pattern added | [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → Security section |
| Backend added | [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → Backend section |
| UX change | [WORKFLOW_VISUAL.md](WORKFLOW_VISUAL.md) → User interaction |
| Release | [ONE_PAGER.md](ONE_PAGER.md) → Status/Version |

### Review Checklist:
- [ ] All examples still work
- [ ] Screenshots/diagrams up to date
- [ ] Links not broken
- [ ] Consistent terminology
- [ ] Version numbers correct
- [ ] New features documented

---

## 💬 Feedback and Questions

**Found an issue with documentation?**
- Open an issue: GitHub Issues
- Suggest improvements: GitHub Discussions
- Direct updates: Submit PR with changes

**Documentation gaps?**
- What's unclear?
- What's missing?
- What examples would help?

**Want to contribute docs?**
- See `../CONTRIBUTING.md` (if exists)
- Follow the standards above
- Reference existing document style
- Add your doc to this index

---

## 🗺️ Related Resources

### In This Repository:
- `../README.md` - Project README with status and setup
- `../CLAUDE.md` - Technical guide for AI assistants
- `../LICENSE` - AGPL-3.0 license
- `../src/` - Source code (well-commented)
- `../tests/` - Test examples

### External:
- MLX Framework: https://github.com/ml-explore/mlx
- Ollama: https://ollama.ai
- vLLM: https://github.com/vllm-project/vllm
- Rust Book: https://doc.rust-lang.org/book/

---

## 📊 Documentation Statistics

| Document | Lines | Words (approx) | Read Time | Diagrams |
|----------|-------|----------------|-----------|----------|
| ONE_PAGER.md | ~280 | ~2,000 | 5 min | 1 |
| PRODUCT_OVERVIEW.md | ~350 | ~3,500 | 15 min | 2 |
| ARCHITECTURE_SIMPLE.md | ~450 | ~4,500 | 20 min | 3 |
| WORKFLOW_VISUAL.md | ~600 | ~2,500 | 10 min | 9 |
| **Total** | ~1,680 | ~12,500 | 50 min | 15 |

**Total documentation**: ~50 minutes of reading for complete understanding

---

## 🎓 Learning Path

### Beginner (Never heard of cmdai)
```
1. ONE_PAGER.md (5 min)
   ↓
2. PRODUCT_OVERVIEW.md → "Simple Flow" section (5 min)
   ↓
3. Try the tool: cargo run -- "list files"
```

### Intermediate (Want to contribute)
```
1. PRODUCT_OVERVIEW.md (full) (15 min)
   ↓
2. ARCHITECTURE_SIMPLE.md (20 min)
   ↓
3. Pick a sub-agent from "For New Contributors"
   ↓
4. Read relevant source code
   ↓
5. Write tests first (TDD)
```

### Advanced (Architecture decisions)
```
1. All documentation (50 min)
   ↓
2. Source code review
   ↓
3. CLAUDE.md for standards
   ↓
4. Test suite examination
   ↓
5. Design proposals and PRs
```

---

## 🚀 Next Steps

After reading documentation:

1. **Try the tool**
   ```bash
   cargo build --release
   cargo run -- "your prompt here"
   ```

2. **Run tests**
   ```bash
   cargo test
   ```

3. **Pick a contribution area**
   - See [ARCHITECTURE_SIMPLE.md](ARCHITECTURE_SIMPLE.md) → "For New Contributors"

4. **Join the community**
   - GitHub Issues for bugs
   - GitHub Discussions for ideas
   - PRs for contributions

---

**Built with care** | **Documented thoroughly** | **Open for contributions**

*Last updated: 2025-11-19*
*Documentation version: 1.0*
