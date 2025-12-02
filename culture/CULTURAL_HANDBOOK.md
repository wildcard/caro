# cmdai Cultural Handbook
## Who We Are • What We Believe • How We Work

> **Version 1.0** | Last Updated: 2025-11-19
> This is a living document that evolves with our community.

---

## 🎯 Our Purpose

### Why We Exist

**We exist to make AI-powered terminals both fast AND safe.**

In a world where AI agents can do amazing things, we're the guardrails that keep everyone safe. We believe developers deserve tools that accelerate their work without introducing existential risk. We're building the safety net for the AI automation revolution.

**Our North Star:**
*Enable every developer to run AI-generated commands with confidence.*

---

## 🧭 Our Values

These aren't just words on a wall. They're the principles that guide every decision, every line of code, every interaction.

### 1. Safety First, Always

**What it means:**
- Safety is non-negotiable. It's our core differentiator and reason for existing.
- We validate EVERY command before execution. No exceptions.
- We prioritize preventing one catastrophic failure over enabling a hundred convenient shortcuts.
- We're the responsible adults in a room full of "move fast and break things."

**In practice:**
- Code reviews must address safety implications
- Performance optimizations never compromise security
- When in doubt, block first and explain later
- Safety documentation is as important as feature documentation

**We say:**
- ✅ "Let's add another safety check to catch this edge case."
- ✅ "This might slow us down, but it's the right thing to do."
- ✅ "I'd rather be overly cautious than dangerously convenient."

**We don't say:**
- ❌ "Users can just use --force if they want."
- ❌ "That's a rare edge case, we can skip validation."
- ❌ "Safety checks add too much latency."

---

### 2. Radical Transparency

**What it means:**
- Our source code is our contract with users. AGPL-3.0 means full transparency.
- We explain our decisions, especially when we block commands.
- We document our mistakes as openly as our wins.
- Security through obscurity is not security—it's postponed failure.

**In practice:**
- All safety patterns are publicly documented
- Validation logic is readable and well-commented
- Our roadmap is public on GitHub
- We discuss tradeoffs openly in issues and PRs
- We publish postmortems when things go wrong

**We say:**
- ✅ "Here's exactly why we blocked this command..."
- ✅ "This is a known limitation, and here's our plan to fix it."
- ✅ "We made a mistake. Here's what happened and how we're preventing it."

**We don't say:**
- ❌ "Trust us, the algorithm knows best."
- ❌ "That's proprietary logic we can't share."
- ❌ "We'll fix it quietly and hope no one noticed."

---

### 3. Respect > Ego

**What it means:**
- We check our egos at the door. The best idea wins, regardless of who had it.
- We respect users' time, intelligence, and autonomy.
- We're helpful, not condescending. Expert, not elitist.
- Disagreement is good. Disrespect is not.

**In practice:**
- Code review comments focus on the code, not the coder
- We assume good intentions until proven otherwise
- We say "I don't know" instead of making stuff up
- Junior contributors' ideas get the same consideration as maintainers'
- We explain technical concepts without gatekeeping jargon

**We say:**
- ✅ "Great question! Here's how that works..."
- ✅ "I learned something new from your approach."
- ✅ "Let's explore both options and see which works better."

**We don't say:**
- ❌ "Actually, you should already know this..."
- ❌ "That's a stupid question."
- ❌ "Clearly you don't understand how this works."

---

### 4. Speed Through Simplicity

**What it means:**
- Fast doesn't mean reckless. It means efficient.
- We ship quickly by keeping things simple, not by cutting corners.
- Single binary. Zero dependencies. Minimal complexity.
- The fastest code to maintain is code that doesn't exist.

**In practice:**
- We prefer boring, proven solutions over shiny new tech
- We delete code more often than we add it
- We optimize for readability first, performance second
- We ship small, incremental changes over massive rewrites
- We measure what matters (startup time, validation speed, accuracy)

**We say:**
- ✅ "Can we solve this with less code?"
- ✅ "Let's ship this now and iterate based on feedback."
- ✅ "This dependency adds complexity. Do we really need it?"

**We don't say:**
- ❌ "Let's rewrite everything in [shiny new framework]."
- ❌ "This code is clever, no need for comments."
- ❌ "We'll optimize later." (Spoiler: later never comes)

---

### 5. Community Over Company

**What it means:**
- cmdai belongs to the community, not to any one person or organization.
- We're stewards, not owners. The code outlives us.
- The best feature request comes from a user scratching their own itch.
- Contributions are gifts, not obligations. We treat them as such.

**In practice:**
- We prioritize community-driven features
- We onboard contributors like they're joining a team, not filing a ticket
- We celebrate contributors publicly and often
- We make decisions in the open (GitHub Discussions)
- We share credit generously

**We say:**
- ✅ "Thank you for contributing! How can we help you succeed?"
- ✅ "Let's discuss this in GitHub Discussions so everyone can weigh in."
- ✅ "This wouldn't be possible without our amazing community."

**We don't say:**
- ❌ "We'll consider your suggestion." (And then ghost them)
- ❌ "That's not on our roadmap." (Without explaining why)
- ❌ "We're the maintainers, we know best."

---

### 6. Excellence with Empathy

**What it means:**
- We have high standards for code quality, but we're kind to people.
- Perfect is the enemy of good, but "good enough" isn't good enough.
- We ship production-quality code, but we don't sacrifice people to do it.
- Burnout ships bugs. Sustainable pace ships quality.

**In practice:**
- We do thorough code reviews, but we're constructive, never cruel
- We acknowledge effort even when we can't merge a PR
- We set realistic timelines and communicate delays early
- We encourage breaks and time off
- We fix bugs quickly but don't blame the person who introduced them

**We say:**
- ✅ "This is close! Here are some suggestions to get it over the line."
- ✅ "I really appreciate the effort here. Let me explain why we can't merge this as-is."
- ✅ "Take your time. Quality matters more than speed."

**We don't say:**
- ❌ "This code is garbage. Rewrite it."
- ❌ "Why didn't you think of this obvious edge case?"
- ❌ "Ship it now, we'll fix the bugs later."

---

## 🛠️ How We Work

### Our Development Philosophy

**Test-Driven Development (TDD)**
- We write tests first. Not sometimes. Always.
- Red → Green → Refactor is our rhythm
- Contract-based tests ensure reliability
- If it's not tested, it's not done

**Safety-First Architecture**
- Every feature must pass the "safety validator" test
- New backend? Prove it validates dangerous commands.
- New feature? Show me the safety checks.
- Performance optimization? Not at the cost of security.

**Boring Technology**
- Rust, not Rust + 47 dependencies
- Standard library first, external crates when necessary
- MLX for Apple Silicon (proven), not experimental frameworks
- TOML for config, not YAML/JSON/INI/XML/custom format

**Ship Small, Ship Often**
- Small PRs > Large rewrites
- Incremental improvements > Grand redesigns
- Weekly releases > Quarterly milestones
- User feedback > Long-term plans

---

### Our Communication Principles

**Async-First**
- We default to asynchronous communication (GitHub, Discussions)
- We document decisions so future contributors can understand context
- We respect time zones and personal schedules
- Meetings are the exception, not the rule

**Written > Verbal**
- We write things down. Slack messages disappear. GitHub issues don't.
- We link to docs instead of explaining the same thing repeatedly
- We create RFCs for major decisions
- We update docs when things change

**Disagree and Commit**
- We encourage healthy debate
- We listen to dissent and consider it seriously
- Once a decision is made, we commit fully
- We can revisit decisions, but we don't sabotage them

**No Surprises**
- We communicate early and often
- We share bad news immediately, not at the last minute
- We update the community on delays
- We're honest about what we don't know

---

### Our Code Quality Standards

**Every PR Must:**
- ✅ Include tests (unit + integration where applicable)
- ✅ Pass all CI checks (clippy, fmt, tests)
- ✅ Update documentation if changing user-facing behavior
- ✅ Consider safety implications
- ✅ Have clear commit messages

**Every Release Must:**
- ✅ Be thoroughly tested on target platforms
- ✅ Include changelog with user-facing changes
- ✅ Pass security audit (cargo-audit)
- ✅ Have rollback plan documented
- ✅ Be announced to community

**Code Review Guidelines:**
- Review for correctness, not style (clippy handles style)
- Ask questions, don't make demands
- Approve quickly, block rarely
- Explain the "why" behind requested changes
- Celebrate good code as much as you catch bugs

---

## 🤝 Working Together

### For Maintainers

**Your role is to:**
- Empower contributors, not gatekeep contributions
- Make decisions, but explain them
- Ship code, but maintain quality
- Grow the community, not just the codebase

**You are expected to:**
- Respond to issues/PRs within 48 hours (acknowledgment, not resolution)
- Write clear, helpful code review comments
- Update documentation when you change things
- Mentor new contributors
- Model the values in all interactions

**You are NOT expected to:**
- Be online 24/7
- Merge every PR that comes in
- Say yes to every feature request
- Sacrifice your well-being for the project

---

### For Contributors

**Your contributions are valued:**
- Code, docs, bug reports, design, ideas—all matter
- Your time is a gift. We treat it as such.
- You don't need permission to start. Fork and experiment.
- Questions are contributions. Ask them.

**We will:**
- Review your PRs within a week
- Explain our decisions (approval or rejection)
- Help you improve your contribution if it's not quite ready
- Credit you publicly for your work
- Support you as you learn

**We ask that you:**
- Read CONTRIBUTING.md before your first PR
- Test your changes before submitting
- Be patient with review cycles
- Accept feedback graciously
- Respect the community guidelines

---

### For Users

**You are why we exist:**
- Your safety is our priority
- Your feedback shapes our roadmap
- Your trust is earned, not assumed
- Your use cases matter, even edge cases

**We promise to:**
- Never block a safe command
- Explain why we block dangerous commands
- Listen to your feature requests
- Fix security issues immediately
- Communicate changes clearly

**We ask that you:**
- Report bugs with clear reproduction steps
- Understand that safety sometimes means slower
- Participate in discussions when you have context
- Share cmdai if it's useful to you

---

## 🌱 How We Grow

### Personal Growth

**We believe:**
- Learning in public makes us all better
- Mistakes are tuition for growth
- Mentoring is how knowledge compounds
- Diversity of thought makes us stronger

**We encourage:**
- Writing blog posts about what you learned
- Giving talks at meetups/conferences
- Mentoring newcomers
- Trying new things and sharing results
- Asking for help when you need it

### Project Growth

**We measure success by:**
- **Safety accuracy:** How many dangerous commands did we catch?
- **User trust:** Do people feel safe using cmdai?
- **Community health:** Are people helping each other?
- **Code quality:** Is the codebase maintainable?
- **Performance:** Are we fast enough for daily use?

**We don't measure:**
- GitHub stars (vanity metric)
- Lines of code (more code = more bugs)
- Number of features (bloat is real)
- Revenue (we're open source, not a startup)

---

## 🚫 What We Don't Tolerate

This is an inclusive, welcoming community. Some behaviors are incompatible with that:

**Absolutely Not:**
- ❌ Harassment, bullying, or discrimination
- ❌ Disrespect toward people (ideas can be criticized, people can't)
- ❌ Bad faith arguments or trolling
- ❌ Spam or self-promotion without value
- ❌ Violating someone's privacy or safety
- ❌ Plagiarism or license violations

**Consequences:**
- First offense: Warning and explanation
- Second offense: Temporary ban from community
- Third offense or severe violation: Permanent ban

We'd rather lose a contributor than make the community unsafe.

---

## 📖 Our Story (So Far)

**Where we started:**
AI coding assistants were suggesting `rm -rf /` and other catastrophic commands. Developers wanted the speed but feared the consequences. Security teams blocked AI tools entirely.

**What we realized:**
The problem isn't AI. It's the lack of guardrails. We needed a safety validator that works at machine speed.

**What we built:**
cmdai - a Rust-powered CLI that generates commands with AI but validates them before execution. Fast enough for daily use. Safe enough for production.

**Where we're going:**
A world where every developer can use AI terminal tools with confidence. Where "AI-powered" doesn't mean "dangerously unpredictable." Where speed and safety aren't tradeoffs.

---

## 🔮 Our Future

### What We're Building Toward

**Short-term (2025):**
- ✅ Stable 1.0 release with MLX backend
- ✅ Comprehensive safety pattern library
- ✅ Multi-backend support (Ollama, vLLM, MLX)
- 🎯 Package manager distribution (Homebrew, apt, cargo)
- 🎯 Windows support with cross-platform testing

**Medium-term (2026):**
- 🎯 Plugin system for custom safety patterns
- 🎯 Multi-step command workflows
- 🎯 Learning from user feedback (privacy-first)
- 🎯 Integration with popular dev tools (VSCode, etc.)

**Long-term Vision:**
- 🎯 Industry-standard safety validation that other tools adopt
- 🎯 Community-driven safety pattern database
- 🎯 cmdai becomes the reference implementation for safe AI commands

**What we'll never become:**
- ❌ A closed-source commercial product
- ❌ A data harvesting operation
- ❌ A "move fast and break things" culture
- ❌ A tool that prioritizes convenience over safety

---

## 💬 How to Use This Handbook

**When making a decision, ask:**
1. Does this align with our values?
2. Does this serve our users?
3. Does this make us safer?
4. Can we explain this transparently?
5. Would we be proud of this in a year?

**When in doubt:**
- Bias toward safety
- Communicate openly
- Ask the community
- Sleep on big decisions
- Choose kindness

---

## 📞 Living Document

This handbook evolves as we learn and grow.

**Suggest changes:**
- Open an issue with "culture" label
- Start a discussion in GitHub Discussions
- Submit a PR with your proposed changes

**Major changes require:**
- Community discussion (at least 7 days)
- Consensus from core maintainers
- Clear explanation of reasoning

---

## 🙏 Acknowledgments

This handbook is inspired by companies and communities that got culture right:

- **GitLab** - Transparent handbook-first culture
- **Hugging Face** - Open, collaborative AI community
- **Rust Project** - Inclusive, high-quality standards
- **Oxide Computer** - Engineering excellence with values

We stand on the shoulders of giants.

---

## ⚡🛡️ Our Promise

**To users:** We'll keep your terminals safe.

**To contributors:** We'll respect your time and treat you well.

**To the community:** We'll build this in the open and share what we learn.

**To ourselves:** We'll maintain our values even when it's hard.

---

**This is cmdai.**

We're not just building a tool. We're building a culture of responsible AI automation.

We're the guardrails for the fast lane.

**Think Fast. Stay Safe.**

---

*Last updated: 2025-11-19*
*Version 1.0 - Living document*
*Feedback: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)*
