# Project Roadmap

> 🗺️ **Where We're Going** - The vision and development plan for cmdai

This roadmap shows our **vision** for cmdai, what's **in progress**, and what's **planned**. Every item here represents work by our amazing community of contributors.

---

## 🎯 Project Vision

Build the **safest, fastest, and most user-friendly** natural language to shell command tool, powered by local LLMs and designed with a safety-first philosophy.

### Core Principles

1. **Safety First** - Never execute dangerous commands without clear warnings
2. **Privacy-Preserving** - Local LLM inference, no data leaves your machine
3. **Developer Experience** - Fast, reliable, and intuitive
4. **Open Source** - Built by and for the community
5. **Cross-Platform** - Works everywhere (macOS, Linux, Windows)

---

## 📊 Development Phases

### ✅ Phase 1: Foundation (COMPLETED)

**Goal:** Establish core architecture and safety framework

- [x] CLI argument parsing with clap
- [x] Modular backend trait system
- [x] Safety validation framework
- [x] Configuration management
- [x] Initial test infrastructure

**Contributors:** @contributor1, @contributor2, @contributor3

---

### 🚧 Phase 2: Backend Integration (IN PROGRESS)

**Goal:** Support multiple LLM backends with graceful fallback

#### Completed ✅
- [x] Backend trait definition
- [x] Ollama integration (PR #004)
- [x] vLLM integration (PR #004)
- [x] Backend fallback mechanism

#### In Progress 🔨
- [ ] **Embedded model backend** (Branch: `feature/embedded-model`)
  - MLX support for Apple Silicon
  - CPU fallback using Candle
  - Model quantization (4-bit)
  - **Contributors:** @mlx-team
  - **Status:** Testing phase
  - **PR:** #XXX (draft)

- [ ] **Model caching system** (Branch: `feature/model-cache`)
  - Hugging Face integration
  - Cache management
  - Offline mode support
  - **Contributors:** @cache-dev
  - **Status:** Implementation
  - **PR:** Coming soon

#### Planned 📅
- [ ] OpenAI/Anthropic API support
- [ ] Custom model loading
- [ ] Model registry system

**Track Progress:** [What's Being Built](./active-development.md)

---

### 📅 Phase 3: Safety & Validation (NEXT UP)

**Goal:** Comprehensive command safety and user protection

#### Planned Features
- [ ] **Advanced pattern detection**
  - Machine learning-based risk assessment
  - Context-aware validation
  - User-specific safety profiles

- [ ] **Execution sandboxing**
  - Restricted environment for command execution
  - Resource limits (CPU, memory, network)
  - Filesystem isolation

- [ ] **Command explanation**
  - Break down complex commands
  - Show what each part does
  - Educational mode

- [ ] **Undo/rollback system**
  - Track command effects
  - Automatic rollback on errors
  - File versioning integration

**Want to help?** See [Contributing Guide](./contributing.md)

---

### 📅 Phase 4: Enhanced User Experience

**Goal:** Make cmdai delightful to use

#### Planned Features
- [ ] **Multi-step workflows**
  - Complex goal decomposition
  - Step-by-step execution
  - Progress tracking

- [ ] **Context awareness**
  - Shell history integration
  - Environment variable awareness
  - Git repository context

- [ ] **Interactive mode**
  - REPL-style interface
  - Command history
  - Auto-completion

- [ ] **Shell script generation**
  - Generate complete scripts
  - Comment generation
  - Error handling

---

### 📅 Phase 5: Performance & Scale

**Goal:** Optimize for speed and efficiency

#### Planned Features
- [ ] **Startup optimization**
  - Target: <100ms cold start
  - Lazy loading improvements
  - Binary size reduction (<50MB)

- [ ] **Inference optimization**
  - Streaming responses
  - Batch processing
  - Model quantization

- [ ] **Caching strategies**
  - Response caching
  - Prompt caching
  - Model weight caching

---

### 📅 Phase 6: Community & Ecosystem

**Goal:** Build a thriving community and ecosystem

#### Planned Features
- [ ] **Plugin system**
  - Custom backend plugins
  - Safety rule plugins
  - Output formatter plugins

- [ ] **Community examples**
  - Shared prompt library
  - Best practices repository
  - Use case documentation

- [ ] **Integration ecosystem**
  - Shell integration (zsh, bash, fish)
  - Editor plugins (VS Code, Vim)
  - CI/CD integration

- [ ] **Web playground**
  - WebAssembly-powered demo
  - Interactive tutorials
  - Shareable examples

**In Development:** [Playground Spec](../tutorial/playground.md)

---

## 🎯 Current Sprint (Updated Weekly)

### This Week's Focus
- Complete embedded model backend
- Improve test coverage to 80%
- Documentation improvements
- Bug fixes from community feedback

### Active Work Items
1. **MLX Backend** - Testing on M1/M2/M3 Macs
2. **Model Caching** - Implementing download manager
3. **Documentation** - Interactive tutorials
4. **Testing** - Contract test expansion

See live updates: [What's Being Built](./active-development.md)

---

## 📈 Long-Term Vision

### 6 Months
- ✨ Production-ready v1.0 release
- 🎯 All Phase 2 & 3 features complete
- 📚 Comprehensive documentation
- 🌍 Growing community of contributors

### 1 Year
- 🚀 10,000+ active users
- 🔌 Rich plugin ecosystem
- 🌐 Multi-language support
- 🏆 Industry recognition as best-in-class tool

### 2 Years
- 🧠 Advanced AI capabilities
- 🌍 Enterprise adoption
- 📊 Research contributions
- 🎓 Educational partnerships

---

## 🤝 How to Contribute

Every phase needs help! Here's how to get involved:

### 1. Pick a Phase
Choose an area that interests you:
- **Backend work?** → Phase 2
- **Security?** → Phase 3
- **UX?** → Phase 4
- **Performance?** → Phase 5

### 2. Check Active Work
See what's already being built: [What's Being Built](./active-development.md)

### 3. Find Your Task
- **Issues labeled "good first issue"** - Great for beginners
- **Issues labeled "help wanted"** - Need contributors
- **Feature requests** - Propose new ideas

### 4. Document Your Work
Follow our guide: [Documenting Your Work](./documentation-guide.md)

### 5. Submit Your PR
Read: [Contributing Guide](./contributing.md)

---

## 🗳️ Community Input

### Vote on Features
We use GitHub Discussions for feature voting:
- 👍 React to features you want
- 💬 Comment with use cases
- 🎯 Help prioritize the roadmap

**Cast your vote:** [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)

### Propose Features
Have an idea? We want to hear it!
1. Open a discussion
2. Describe the use case
3. Community feedback
4. Maintainer review
5. Added to roadmap if approved

---

## 📊 Metrics & Goals

### Quality Metrics
- **Test Coverage:** 80%+ (Current: 65%)
- **Documentation Coverage:** 100% of public APIs
- **Build Time:** <3 minutes
- **Binary Size:** <50MB

### Performance Targets
- **Cold Start:** <100ms
- **Inference (MLX):** <2s on M1 Mac
- **Inference (Remote):** <5s with network
- **Memory Usage:** <200MB

### Community Goals
- **Contributors:** 50+ active contributors
- **PRs/Month:** 20+ merged PRs
- **Response Time:** <48h for issues
- **Release Cadence:** Every 2 weeks

---

## 🔄 Roadmap Updates

This roadmap is updated **weekly** based on:
- Community feedback
- Technical discoveries
- Resource availability
- Strategic priorities

**Last updated:** 2024-11-19
**Next review:** 2024-11-26

### How to Propose Changes
1. Open a GitHub Discussion
2. Tag as "roadmap"
3. Explain rationale
4. Community discussion
5. Maintainer decision

---

## 📚 Related Pages

- **[What's Being Built](./active-development.md)** - Current active work
- **[Contributing Guide](./contributing.md)** - How to contribute
- **[Documentation Guide](./documentation-guide.md)** - Document your work
- **[Contributor Showcase](./contributors.md)** - Meet the team

---

## 🌟 Special Initiatives

### Hacktoberfest 2024
- 20+ issues labeled for Hacktoberfest
- Mentorship program for new contributors
- Documentation sprints

### Google Summer of Code 2025
Planning to apply! Potential projects:
- WebAssembly playground
- Advanced safety ML models
- Multi-language support

---

**Questions about the roadmap?** [Open a Discussion](https://github.com/wildcard/cmdai/discussions)

**Want to help shape it?** All contributors can influence priorities!
