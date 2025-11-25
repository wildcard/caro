# What's Being Built

> 🔨 **Live Development Dashboard** - See what's happening right now across all branches and PRs

This page showcases **ALL active development work** happening in the cmdai project, regardless of branch or merge status. Every contributor's work is valuable and deserves visibility!

**Last Updated:** 2024-11-19 | **Auto-updates:** Weekly

---

## 🌟 How to Get Your Work Listed Here

Working on something for cmdai? **We want to feature it!**

### Quick Steps:
1. Create your feature branch
2. Add a brief description to this page via PR
3. Link to your branch/PR
4. Update status as you progress

See: [Documentation Guide](./documentation-guide.md) for details

---

## 🚧 Active Feature Branches

### 🔥 High Priority

#### 1. Embedded Model Backend (MLX + CPU)
**Branch:** `feature/embedded-model`
**Contributors:** @contributor-mlx-lead, @contributor-mlx-2
**Status:** 🔨 Testing Phase (80% complete)

**What it does:**
- Runs LLM models directly in cmdai (no external server!)
- Optimized MLX backend for Apple Silicon (M1/M2/M3)
- CPU fallback using Candle framework
- 4-bit quantized models for speed

**Progress:**
- [x] MLX integration with FFI
- [x] Model loading system
- [x] Inference pipeline
- [ ] CPU fallback implementation (in progress)
- [ ] Performance benchmarks
- [ ] Integration tests

**Try it:**
```bash
git checkout feature/embedded-model
cargo build --release --features mlx
./target/release/cmdai "list files"
```

**Docs:** Will be added to [MLX Integration](../technical/mlx-integration.md)

**Help Needed:**
- Testing on M1/M2/M3 Macs
- CPU backend implementation
- Performance benchmarking

**PR:** #XXX (draft) | **Discussion:** #YYY

---

#### 2. Model Caching & Download Manager
**Branch:** `feature/model-cache`
**Contributors:** @cache-developer
**Status:** 🔨 Implementation (60% complete)

**What it does:**
- Downloads models from Hugging Face
- Caches models locally for offline use
- Manages model versions and updates
- Automatic fallback to cached models

**Progress:**
- [x] Cache directory structure
- [x] Download manager (basic)
- [ ] Hugging Face API integration
- [ ] Progress indicators
- [ ] Integrity verification
- [ ] Cache cleanup utilities

**Docs:** Will be added to User Guide

**Help Needed:**
- UI/UX for download progress
- Error handling for network failures
- Testing on slow connections

**PR:** Coming Soon | **Discussion:** #ZZZ

---

### 🎯 Medium Priority

#### 3. Advanced Safety Patterns
**Branch:** `feature/advanced-safety`
**Contributors:** @security-researcher, @safety-contributor
**Status:** 🔨 Design Phase (30% complete)

**What it does:**
- ML-based command risk assessment
- Context-aware safety validation
- User-specific safety profiles
- Learning from user corrections

**Progress:**
- [x] Research phase
- [x] Pattern database expansion
- [ ] ML model selection
- [ ] Training data collection
- [ ] Integration design

**Docs:** Will be added to [Safety Validation](../technical/safety-validation.md)

**Help Needed:**
- ML engineers for risk model
- Dataset of dangerous commands
- Safety pattern contributions

**Discussion:** #AAA

---

#### 4. Shell Integration (zsh/bash/fish)
**Branch:** `feature/shell-integration`
**Contributors:** @shell-hacker
**Status:** 🔨 Prototype (40% complete)

**What it does:**
- Keyboard shortcuts in your shell (e.g., Ctrl+E)
- Inline command suggestions
- Shell history integration
- Auto-completion

**Progress:**
- [x] zsh plugin prototype
- [ ] bash integration
- [ ] fish support
- [ ] Installation script

**Docs:** Will be added to User Guide

**Try it (zsh):**
```bash
git checkout feature/shell-integration
source shell/cmdai.zsh
# Press Ctrl+E to activate
```

**Help Needed:**
- bash and fish implementations
- Windows PowerShell support
- Installation automation

**PR:** #BBB (draft)

---

#### 5. WebAssembly Playground
**Branch:** `feature/wasm-playground`
**Contributors:** @wasm-wizard, @ui-designer
**Status:** 🔨 Early Development (20% complete)

**What it does:**
- Run cmdai in your browser!
- Interactive tutorials
- Shareable examples
- No installation required

**Progress:**
- [x] Architecture design (see [Playground](../tutorial/playground.md))
- [x] WASM compilation research
- [ ] WASM build setup
- [ ] Basic UI prototype
- [ ] Model loading in browser

**Docs:** [Playground Documentation](../tutorial/playground.md)

**Help Needed:**
- React/TypeScript developers
- WASM optimization
- UI/UX design
- Example content

**Discussion:** #CCC | **Design Doc:** [Link]

---

### 🌱 Early Stage / Experimental

#### 6. Command Explanation Engine
**Branch:** `feature/explain-mode`
**Contributors:** @educator
**Status:** 🌱 Concept (10% complete)

**What it does:**
```bash
cmdai explain "find . -name '*.pdf' -mtime +30"
```

Outputs:
```
This command finds files:
├── find .              - Search starting from current directory
├── -name '*.pdf'       - Match files ending in .pdf
└── -mtime +30          - Modified more than 30 days ago

Safety: ✅ Safe (read-only operation)
```

**Progress:**
- [x] Feature proposal
- [ ] Parser implementation
- [ ] Explanation templates

**Help Wanted:** Anyone interested in educational features!

**Discussion:** #DDD

---

#### 7. Multi-Step Workflow Engine
**Branch:** `feature/workflows`
**Contributors:** @workflow-dev
**Status:** 🌱 Research (5% complete)

**What it does:**
- Break complex goals into steps
- Execute multi-command workflows
- Progress tracking
- Rollback on failure

**Example:**
```bash
cmdai workflow "backup my project and deploy to server"
```

Generates:
```
Step 1/4: Create backup
Step 2/4: Compress files
Step 3/4: Upload to server
Step 4/4: Restart server
```

**Progress:**
- [x] Use case collection
- [ ] Architecture design

**Help Wanted:** Designers, workflow experts

**Discussion:** #EEE

---

## 📝 Recent Pull Requests

### Merged This Week ✅
- **PR #012** - Add interactive tutorials (@doc-writer) ✅ Merged
- **PR #011** - Improve error messages (@ux-improver) ✅ Merged
- **PR #010** - Fix safety validation bug (@bug-hunter) ✅ Merged

### Under Review 👀
- **PR #013** - Add configuration examples (@config-guru)
  - Status: Awaiting review
  - Reviewers needed: 2 more

- **PR #014** - Improve test coverage (@test-master)
  - Status: Changes requested
  - Feedback: Add integration tests

### Draft PRs 📝
- **PR #015** - MLX backend implementation (@mlx-dev)
- **PR #016** - Model caching (@cache-dev)

---

## 🎨 Documentation Work in Progress

### Active Documentation PRs
- **PR #020** - Tutorial improvements (@tutorial-writer)
- **PR #021** - Architecture diagrams (@diagram-artist)
- **PR #022** - API documentation (@api-doc)

### Documentation Branches
- `docs/api-reference` - Complete API docs
- `docs/advanced-tutorials` - Advanced use cases
- `docs/troubleshooting` - Common issues & solutions

---

## 🐛 Active Bug Fixes

### High Priority Bugs
- **Issue #100** - Startup crash on Windows (@windows-dev)
  - Branch: `fix/windows-crash`
  - Status: Testing fix

- **Issue #101** - Memory leak in long sessions (@memory-detective)
  - Branch: `fix/memory-leak`
  - Status: Root cause identified

### Medium Priority
- **Issue #102** - Config parsing error (@config-fixer)
- **Issue #103** - Safety false positives (@safety-tuner)

---

## 🎯 Help Wanted

### Good First Issues
Perfect for new contributors:

1. **Add more safety patterns** (Issue #200)
   - Area: Safety validation
   - Skills: Regex, shell knowledge
   - Time: 2-4 hours

2. **Improve error messages** (Issue #201)
   - Area: User experience
   - Skills: Writing, empathy
   - Time: 1-3 hours

3. **Add examples to docs** (Issue #202)
   - Area: Documentation
   - Skills: Technical writing
   - Time: 1-2 hours

### Advanced Contributions Needed

1. **ML model for safety** (Issue #300)
   - Area: Machine learning
   - Skills: Python, ML, Rust
   - Time: Ongoing project

2. **WASM optimization** (Issue #301)
   - Area: WebAssembly
   - Skills: WASM, optimization
   - Time: 2-3 weeks

3. **Cross-platform testing** (Issue #302)
   - Area: CI/CD
   - Skills: GitHub Actions, testing
   - Time: 1 week

---

## 📊 Development Statistics

### This Month
- **Active Branches:** 15
- **Open PRs:** 8
- **Contributors:** 23
- **Commits:** 147
- **Issues Closed:** 34

### Velocity Trends
- **PR Merge Rate:** ~10 PRs/week
- **Average Review Time:** 2.3 days
- **Contributor Growth:** +15% month-over-month

---

## 🗓️ Upcoming Milestones

### v0.2.0 (Next Release - Dec 2024)
**Target:** December 15, 2024
- [ ] Embedded model backend (80% done)
- [ ] Model caching (60% done)
- [ ] Improved safety (30% done)
- [ ] Shell integration (40% done)

**On Track:** ✅ Yes

### v0.3.0 (Future - Jan 2025)
**Target:** January 30, 2025
- [ ] WebAssembly playground
- [ ] Advanced safety ML
- [ ] Multi-step workflows

---

## 💬 Community Highlights

### Shoutouts This Week 🌟
- **@mlx-dev** - Incredible progress on Apple Silicon support!
- **@doc-writer** - Amazing tutorials that help everyone
- **@bug-hunter** - Found and fixed 3 critical bugs
- **@ui-designer** - Beautiful playground mockups

### New Contributors Welcome! 👋
This week we welcomed:
- @new-contributor-1 - First PR on documentation
- @new-contributor-2 - Reported valuable bug
- @new-contributor-3 - Added test cases

**Welcome to the team!** 🎉

---

## 📢 How to Stay Updated

### Real-Time Updates
- **GitHub Watch:** Star & watch the repo
- **Discussions:** [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- **Discord:** Join our community server
- **Twitter:** [@cmdai_dev](https://twitter.com/cmdai_dev)

### Weekly Digest
Subscribe to our weekly newsletter:
- Latest merged PRs
- New contributors
- Feature highlights
- Help wanted items

**Subscribe:** [Link to newsletter]

---

## 🤝 How to Add Your Work Here

### For Active Development
1. Create your feature branch
2. Open a PR to update this page
3. Add your branch under appropriate priority
4. Include:
   - Branch name
   - Contributors (tag yourself!)
   - Status and progress
   - What it does (user-facing description)
   - Help needed
   - Link to PR/discussion

### Template
```markdown
#### Feature Name
**Branch:** `feature/your-feature`
**Contributors:** @you, @collaborator
**Status:** 🔨 Implementation (X% complete)

**What it does:**
- User-facing description
- Key benefits

**Progress:**
- [x] Completed item
- [ ] In progress item

**Help Needed:**
- What you need help with

**PR:** #XXX | **Discussion:** #YYY
```

### For Documentation Work
Add to the "Documentation Work in Progress" section

### For Bug Fixes
Add to the "Active Bug Fixes" section

---

## 📚 Related Pages

- **[Project Roadmap](./roadmap.md)** - Long-term vision
- **[Contributing Guide](./contributing.md)** - How to contribute
- **[Documentation Guide](./documentation-guide.md)** - Document your work
- **[Contributor Showcase](./contributors.md)** - Meet everyone

---

## 🎉 Want to Be Featured?

**Every contribution matters!** Whether you're:
- Writing code
- Improving docs
- Reporting bugs
- Answering questions
- Designing UI
- Testing features

**Your work belongs here.** Don't be shy - add yourself!

---

**Questions?** Ask in [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)

**Ready to contribute?** Pick an item above and get started! 🚀
