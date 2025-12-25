# cmdai Contributor Handbook

Welcome to the cmdai community! This handbook explains how we work together, make decisions, and build a safety-first terminal command generation tool that empowers everyone to use the command line more effectively.

## Table of Contents

- [Our Mission](#our-mission)
- [Core Values](#core-values)
- [How We Work](#how-we-work)
- [Decision Making](#decision-making)
- [Communication](#communication)
- [Ownership and Responsibility](#ownership-and-responsibility)
- [Recognition and Growth](#recognition-and-growth)
- [Conflict Resolution](#conflict-resolution)
- [Community Norms](#community-norms)

---

## Our Mission

**cmdai exists to make terminal access safer and more accessible for everyone.**

We're building more than a CLI tool - we're creating a collective knowledge base of terminal expertise that gets embedded into an AI and given back to the world. Every safety pattern you contribute, every edge case you document, every test you write helps someone else work more confidently with their command line.

### What We Believe

**The terminal is powerful and should be accessible:**
- Not everyone memorizes POSIX flags or regex syntax
- Command-line mastery shouldn't be a prerequisite for productivity
- AI can democratize terminal access while maintaining safety

**Safety is non-negotiable:**
- A helpful command that destroys data is worse than no command at all
- We protect users from themselves, not just from malicious actors
- Every dangerous pattern we catch prevents real-world disasters

**Collective knowledge beats individual scripts:**
- 1000 unmaintainable Python scripts vs. 1 composable POSIX tool
- Unix philosophy: do one thing well, compose freely
- Community-validated patterns > solo experimentation

**Open source is how we scale impact:**
- Transparent development builds trust
- Diverse contributors create robust safety patterns
- Local execution respects privacy and enables offline use

---

## Core Values

Our values guide every decision, from code architecture to community interactions.

### 1. Safety First, Always

**What this means:**
- Every feature starts with "how could this go wrong?"
- We block dangerous operations by default, not as an afterthought
- Conservative validation is better than clever inference
- When in doubt, ask for confirmation

**How this shows up:**
- Comprehensive safety pattern database
- Multi-level risk assessment (Safe, Moderate, High, Critical)
- Clear explanations of why commands are flagged
- User education over silent blocking

**In practice:**
```rust
// We prefer this (explicit, safe)
if safety_validator.is_dangerous(&cmd) {
    warn_user(&cmd, &risk_explanation);
    require_confirmation();
}

// Over this (implicit, risky)
// Just execute and hope for the best
```

### 2. Ship, Learn, Iterate

**What this means:**
- Working code beats perfect documentation
- Release early, gather feedback, improve fast
- Every contribution moves us forward
- Done is better than perfect (but tested is better than done)

**How this shows up:**
- MVP implementations with clear upgrade paths
- Feature flags for experimental functionality
- Rapid PR review cycles (target: 48 hours)
- Changelog-driven development

**In practice:**
- Start with mock backends, add real ones incrementally
- Ship parsers that handle 80% of cases, improve with real data
- Document known limitations, accept contributions to fix them

### 3. Radical Transparency

**What this means:**
- All work happens in public (GitHub issues, PRs, discussions)
- Architectural decisions are documented with rationale
- Roadmap, priorities, and trade-offs are visible
- Mistakes are learning opportunities, shared openly

**How this shows up:**
- Comprehensive specs in `/specs` directory
- Public project boards and milestones
- Detailed PR descriptions explaining "why"
- Post-mortems for bugs and outages

**In practice:**
- Architecture Decision Records (ADRs) for major choices
- Issue templates that capture context
- Changelog entries written for users, not developers
- Open discussion of technical debt in TECH_DEBT.md

### 4. Async-First, Sync When Needed

**What this means:**
- Default to asynchronous communication
- Respect contributors' time zones and schedules
- Document decisions so they're searchable later
- Synchronous meetings are the exception, not the norm

**Communication Priority:**
1. **Pull Requests** - Ship code, update docs, make things better
2. **Issues** - Clarify tasks, track bugs, discuss features
3. **Discussions** - Ask questions, share ideas, build community
4. **Discord/Slack** (future) - Quick questions, real-time collaboration

**In practice:**
- Detailed PR descriptions eliminate need for Slack threads
- Issue templates capture context upfront
- Spec-driven development enables parallel work
- Time zones are features, not bugs - we're global by design

### 5. Terminal Expertise is Our Collective Strength

**What this means:**
- Your domain knowledge (k8s, DBAs, SREs, sysadmins) is invaluable
- Real-world war stories improve our safety patterns
- Multi-platform knowledge (macOS, Linux, Windows) enriches the tool
- Teaching others is as valuable as writing code

**How this shows up:**
- Safety pattern contributions from field experts
- Platform-specific validation rules
- Issue templates for use-case submissions
- Acknowledgment of domain expertise in release notes

**In practice:**
- SRE contributes Kubernetes safety patterns
- DBA adds database-specific dangerous operations
- macOS expert optimizes MLX backend performance
- Windows user documents PowerShell edge cases

### 6. macOS-First, But Not macOS-Only

**What this means:**
- We optimize for macOS development experience (MLX, Metal, Apple Silicon)
- We prioritize cross-platform compatibility (Linux, Windows)
- Platform-specific optimizations enhance, don't exclude
- Superior DX on macOS benefits all platforms

**How this shows up:**
- MLX backend for Apple Silicon, CPU backend for everything else
- Platform-specific CI runners (macOS, Linux, Windows)
- Cross-platform testing requirements for all PRs
- Documentation for all platform-specific features

**In practice:**
- MLX backend is `#[cfg(target_os = "macos")]`
- Safety patterns tested on bash, zsh, fish, PowerShell
- Binary distribution for all major platforms
- Development environment works everywhere (devcontainer)

---

## How We Work

### Asynchronous by Default

We embrace async work because:
- Contributors span time zones (US, Europe, Asia, everywhere)
- Deep work requires uninterrupted focus
- Written communication creates searchable history
- Not everyone can attend synchronous meetings

**Async workflows:**
- Code review happens in PR comments, not Slack
- Architectural discussions happen in GitHub Discussions
- Decisions are documented in specs and ADRs
- Status updates go in issues, not standups

**When to go sync:**
- Complex architectural debates (30-min Discord call beats 50-comment thread)
- Pair programming on tricky implementations
- Community office hours (future)
- Emergency bug fixes or security incidents

### Spec-Driven Development

Before writing code, we write specs. This enables:
- **Parallel work** - Multiple contributors tackle different modules
- **Clear contracts** - API boundaries defined upfront
- **Better reviews** - Compare implementation to spec
- **Future reference** - Understand "why" years later

**Spec structure** (see `/specs/[feature-id]/`):
```
spec.md              # What are we building and why?
plan.md              # How are we building it?
contracts/           # What are the API contracts?
quickstart.md        # How do users interact with it?
tasks.md             # What are the implementation steps?
```

**Workflow:**
1. Problem identified (user request, bug, limitation)
2. Spec created or existing spec updated
3. Community discussion on approach
4. Spec finalized with clear acceptance criteria
5. Implementation in phases with contract tests
6. Integration tests and documentation
7. Release with changelog entry

### Test-Driven Development (TDD)

We practice strict TDD:

**Red-Green-Refactor cycle:**
1. **RED** - Write a failing test that expresses desired behavior
2. **GREEN** - Implement minimal code to make the test pass
3. **REFACTOR** - Improve code quality while keeping tests green

**Why TDD:**
- Tests are specifications in code form
- Prevents regressions as we iterate
- Enables confident refactoring
- Documents expected behavior

**Test types:**
- **Contract tests** - Validate public API matches spec
- **Integration tests** - Test cross-module workflows
- **Property tests** - Validate invariants with random inputs
- **Benchmarks** - Track performance over time

See [TDD-WORKFLOW.md](TDD-WORKFLOW.md) for detailed guidance.

### Code Review Philosophy

Code review is:
- **Educational** - Both reviewer and author learn
- **Collaborative** - We're improving code together
- **Timely** - Target 48-hour review turnaround
- **Respectful** - Assume good intent, ask questions

**What we review:**
- Correctness (does it work as intended?)
- Safety (could this cause problems?)
- Performance (does it meet our requirements?)
- Maintainability (can others understand this?)
- Testing (are there tests? Do they cover edge cases?)
- Documentation (can users figure this out?)

**Review norms:**
- Approve when it's good enough, not perfect
- Request changes for safety issues or spec violations
- Comment for suggestions and learning opportunities
- Explain the "why" behind feedback

---

## Decision Making

### Benevolent Dictatorship with Community Input

**Structure:**
- Project maintainers have final decision authority
- Community input shapes decisions through discussion
- Transparency in decision-making process
- Bias toward action over consensus

### How Decisions Are Made

**Small decisions** (implementation details, minor refactors):
- Make the change in a PR
- Reviewers can suggest improvements
- Maintainer approves and merges

**Medium decisions** (new features, architectural changes):
- Open a GitHub Discussion or issue
- Create a spec document
- Community provides feedback
- Maintainer decides based on project goals
- Decision documented in spec or ADR

**Large decisions** (breaking changes, major pivots):
- RFC-style proposal in GitHub Discussion
- Extended community discussion (7+ days)
- Multiple maintainer review
- Decision documented with rationale
- Migration guides for users

### Architecture Decision Records (ADRs)

Major architectural decisions are captured as ADRs:
- Located in `/specs/adr/`
- Include context, decision, consequences
- Immutable once decided (new ADRs supersede old ones)
- Referenced in code comments

**Example ADR topics:**
- Why AGPL-3.0 license?
- Why trait-based backend system?
- Why JSON-only model responses?
- Why embedded models over server-only?

---

## Communication

### Communication Channels

**GitHub Issues** - Bug reports, feature requests, task tracking
- Use issue templates
- Search before creating duplicates
- Label appropriately
- Update status as work progresses

**GitHub Pull Requests** - Code changes, documentation updates
- Use PR template
- Link to related issues/specs
- Request review from relevant experts
- Respond to feedback promptly

**GitHub Discussions** - Questions, ideas, RFCs, community chat
- Ask "how do I..." questions
- Share use cases and stories
- Propose major changes
- Celebrate wins

**Discord/Slack** (future) - Real-time chat, quick questions
- For immediate questions
- Coordination during sprints
- Social connection
- Important decisions get documented in GitHub

### Communication Guidelines

**Be respectful:**
- Assume good intent
- Disagree without being disagreeable
- Attack ideas, not people
- Remember there's a human on the other side

**Be clear:**
- Provide context (links, code snippets, error messages)
- Use descriptive titles
- Break down complex ideas
- Ask questions if something is unclear

**Be inclusive:**
- Avoid jargon without explanation
- Remember not everyone's first language is English
- Use gender-neutral language
- Consider accessibility (alt text for images)

**Be responsive:**
- Respond to mentions within 48 hours
- Update issues when status changes
- Close or hand off issues you can't finish
- Acknowledge when you're stuck

---

## Ownership and Responsibility

### What "Ownership" Means

**Individual ownership:**
- Take responsibility for your contributions
- Follow through on commitments
- Ask for help when blocked
- Communicate status changes

**Collective ownership:**
- Anyone can improve any part of the codebase
- Fix bugs when you find them
- Improve documentation as you learn
- Review PRs even if not tagged

### Expectations for Contributors

**All contributors:**
- Follow Code of Conduct
- Use issue/PR templates
- Write tests for changes
- Update documentation
- Respond to review feedback

**Regular contributors:**
- Help triage new issues
- Review others' PRs
- Mentor first-time contributors
- Contribute to discussions

**Maintainers:**
- Review PRs within 48 hours
- Triage issues weekly
- Make architectural decisions
- Cut releases
- Moderate community
- Maintain project health

### Becoming a Maintainer

Path to maintainership:
1. **Consistent contributions** - Regular, quality PRs over months
2. **Domain expertise** - Deep knowledge of specific area
3. **Community engagement** - Helpful in issues, discussions, reviews
4. **Demonstrated judgment** - Good technical and social decisions
5. **Invitation** - Existing maintainer invites you

Maintainers have:
- Write access to repository
- Ability to merge PRs
- Responsibility for releases
- Authority to make decisions
- Duty to mentor others

---

## Recognition and Growth

### How We Celebrate Contributions

**Every contribution matters:**
- First-time contributors get welcome message
- All contributors listed in release notes
- Outstanding contributions highlighted in changelog
- Domain expertise acknowledged in documentation

### Types of Contributions We Recognize

**Code contributions:**
- New features
- Bug fixes
- Performance improvements
- Refactoring for clarity

**Non-code contributions:**
- Documentation improvements
- Issue triage and labels
- Safety pattern submissions
- Use case documentation
- Tutorial creation
- Community support
- Design and UX feedback
- Testing and bug reports

### CONTRIBUTORS.md

We maintain a [CONTRIBUTORS.md](CONTRIBUTORS.md) file acknowledging everyone who has contributed to cmdai, regardless of contribution size.

### Changelog Attribution

Release notes credit contributors:
```markdown
## [0.3.0] - 2025-11-30

### Added
- MLX backend for Apple Silicon (#42) - @contributor
- Fork bomb detection (#45) - @safety-expert
- Kubernetes safety patterns (#47) - @k8s-sre

### Fixed
- Path quoting on Windows (#44) - @windows-contributor

Thanks to @contributor, @safety-expert, @k8s-sre, and @windows-contributor for making this release possible!
```

### Growth Opportunities

Contributing to cmdai helps you:
- **Learn Rust** - Production-grade Rust patterns
- **Master async** - Tokio, async/await, concurrent systems
- **Understand LLMs** - Local inference, model optimization
- **Deepen terminal expertise** - POSIX, shells, safety patterns
- **Build OSS portfolio** - Visible, impactful work
- **Join a community** - Meet other terminal enthusiasts

---

## Conflict Resolution

### Handling Disagreements

Technical disagreements are normal and healthy. When they arise:

**In code review:**
1. Explain your reasoning (not just "this is wrong")
2. Link to relevant docs/specs/examples
3. Consider compromise solutions
4. Escalate to maintainer if stuck
5. Accept decision and move forward

**In discussions:**
1. State your position clearly
2. Listen to other perspectives
3. Focus on project goals, not personal preference
4. Look for win-win solutions
5. Agree to disagree and defer to maintainer

**In community:**
1. Assume good intent
2. De-escalate, don't inflame
3. Involve moderator if needed
4. Take a break if emotions run high
5. Apologize when wrong

### Code of Conduct Violations

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for full details.

**Reporting:**
- Report via GitHub issue with "conduct" label
- Email maintainers directly for sensitive issues
- Reports handled confidentially
- No retaliation against reporters

**Consequences:**
- Warning for first offense
- Temporary ban for repeated violations
- Permanent ban for severe violations
- Transparency in moderation actions (respecting privacy)

---

## Community Norms

### Cultural Practices

**We say "thank you":**
- For code reviews
- For bug reports
- For feature ideas
- For patience with beginners

**We celebrate wins:**
- First PR merged
- Milestones reached (10th PR, 100th issue)
- Major features shipped
- Performance improvements
- New contributors joining

**We share knowledge:**
- Document as we learn
- Explain "why" in comments
- Write tutorials and guides
- Answer questions patiently
- Pair program when helpful

**We maintain quality:**
- Tests are not optional
- Documentation is not optional
- Code review is not optional
- Breaking CI is not acceptable
- Technical debt is tracked and addressed

### Working in the Open

Everything happens publicly:
- Roadmap and priorities
- Architectural decisions
- Bug reports and fixes
- Performance benchmarks
- Release process

**Benefits:**
- Builds trust with users
- Attracts contributors
- Creates accountability
- Enables learning
- Shows progress

**Exceptions:**
- Security vulnerabilities (private disclosure)
- Code of Conduct violations (privacy respected)
- Personal information
- Credentials and secrets

---

## Your Role in This Community

You're not just a contributor - you're a co-creator of cmdai's culture.

**Every time you:**
- Write a helpful code review
- Document something that confused you
- Answer a question in discussions
- Submit a safety pattern from your experience
- Fix a bug you encountered
- Improve an error message

**You're making cmdai:**
- Safer for users
- Easier for contributors
- More robust in production
- More welcoming to newcomers
- More valuable to the community

---

## Questions About This Handbook?

This handbook is a living document. If something is:
- Unclear - ask in [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- Missing - open an issue or PR
- Wrong - submit a correction
- Outdated - flag it for update

The best handbooks are written by the community, for the community.

---

## Further Reading

- [CONTRIBUTING.md](CONTRIBUTING.md) - Technical contribution guide
- [CLAUDE.md](CLAUDE.md) - Project overview for AI assistants
- [TDD-WORKFLOW.md](TDD-WORKFLOW.md) - Test-driven development process
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community standards
- [SECURITY.md](SECURITY.md) - Security policy and disclosure

---

**Thank you for being part of cmdai.** Together we're making the terminal safer, more accessible, and more powerful for everyone.

*Last updated: 2025-11-28*
