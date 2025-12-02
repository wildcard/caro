# GitHub Discussions Structure for cmdai

Complete setup for GitHub Discussions as cmdai's primary Q&A and community forum.

---

## Why GitHub Discussions?

**Advantages:**
- Already where developers are (lower friction than Discord)
- Searchable and indexed by Google
- Integrates with Issues and PRs
- Markdown support for code examples
- No separate login required
- Permanent archive of knowledge

**Use Cases:**
- Technical Q&A
- Feature discussions
- Roadmap planning
- Announcements
- Community showcase

---

## Discussion Categories

### 📢 Announcements

**Description:**
```
Official announcements from the cmdai core team.

New releases, important updates, and community news.
```

**Settings:**
- Format: Announcement (core team can post, everyone can comment)
- Pinned: Latest release announcement
- Locked after: 30 days (prevent necro-posting)

**Pin Template:**
```
# cmdai [Version] Released 🚀

We're excited to announce cmdai [version]!

## What's New

### Features
- [Feature 1 with brief description]
- [Feature 2 with brief description]

### Safety Improvements
- [New dangerous patterns detected]
- [Safety validator enhancements]

### Performance
- [Performance improvements with benchmarks]

### Bug Fixes
- [Major bug fixes]

## Breaking Changes

[List any breaking changes with migration guide]

## Installation

\`\`\`bash
# Cargo
cargo install cmdai

# From source
git clone https://github.com/wildcard/cmdai
cd cmdai
cargo build --release
\`\`\`

## Full Changelog

See [CHANGELOG.md](link)

## Thank You

Special thanks to our contributors:
@user1, @user2, @user3

[X] PRs merged this release. You're amazing! 🙏

## What's Next

Check out the [roadmap discussion](link) for what's coming in the next release.

---

⚡🛡️ Think Fast. Stay Safe.
```

---

### 💡 Ideas & Feature Requests

**Description:**
```
Propose new features, improvements, and ideas.

Before posting:
- Search existing discussions
- Review the roadmap
- Consider safety implications

Template provided to structure your idea.
```

**Settings:**
- Format: Discussion
- Labels: `feature-request`, `enhancement`, `needs-triage`
- Requires: Issue template

**Discussion Template:**
```
### The Problem

Describe the problem you're trying to solve.

What workflow is currently difficult or impossible?

### Proposed Solution

How would you solve this?

Be specific. Mockups, code examples, and use cases help!

### Alternatives Considered

What other approaches did you consider?

Why is your proposed solution better?

### Safety Considerations

How does this interact with the safety validator?

Are there any security implications?

### Additional Context

Any other context, screenshots, or examples.
```

**Pinned Example:**
```
# How to Write a Great Feature Request

Good feature requests:
✅ Describe a specific problem
✅ Explain the use case
✅ Suggest implementation (optional)
✅ Consider safety implications

Example:

**Title:** "Add dry-run mode for high-risk commands"

**Problem:** I want to see what a command would do without executing it, especially for operations flagged as HIGH or CRITICAL risk.

**Solution:** Add a `--dry-run` flag that:
- Shows what files would be affected
- Estimates changes
- Requires explicit confirmation to proceed

**Safety:** Actually makes things safer by allowing inspection before execution.

---

Not-so-good requests:
❌ "Make it better"
❌ "Add AI" (we already have AI)
❌ "This should work like [other tool]" (without explaining why)

---

All ideas welcome! We review feature requests regularly and prioritize based on:
- User impact
- Alignment with safety-first philosophy
- Implementation complexity
- Community interest

⚡🛡️
```

---

### ❓ Q&A (Help)

**Description:**
```
Ask questions and get help from the community.

For bugs, use Issues instead.
For features, use Ideas & Feature Requests.

Include:
- cmdai version: \`cmdai --version\`
- OS and version
- What you tried
- What happened
```

**Settings:**
- Format: Q&A (can mark answer as accepted)
- Labels: `question`, `help-wanted`, `answered`
- Encourages: Best answer selection

**Auto-Response Templates:**

**For "doesn't work" posts:**
```
Thanks for posting! To help us help you faster, could you provide:

- cmdai version: \`cmdai --version\`
- Operating system and version
- What you tried (exact commands)
- What happened (error messages, unexpected behavior)
- What you expected to happen

The more details, the faster we can solve this! 🚀
```

**For questions answered elsewhere:**
```
Great question! This has been answered before:

[Link to previous discussion or docs]

Does that help? If you have follow-up questions, feel free to ask!
```

**Pinned: Getting Better Answers**
```
# How to Ask Questions (And Get Great Answers Fast)

## Good Question Example

**Title:** "cmdai validation fails for valid find command on macOS"

**Body:**
I'm trying to use cmdai to generate a find command, but it's being flagged as HIGH risk when I don't think it should be.

**Command:**
\`\`\`bash
find ~/Downloads -type f -name "*.tmp" -mtime +30 -delete
\`\`\`

**cmdai version:** 0.1.0
**OS:** macOS 13.4 (Ventura)
**Backend:** MLX

**What I expected:** GREEN or YELLOW (seems safe - just deleting old tmp files in my Downloads)

**What happened:** Flagged as HIGH with warning about `-delete` flag

**Error message:**
\`\`\`
⚠ HIGH RISK: Delete operation with find command
Review carefully before executing
\`\`\`

Is this a false positive? Or am I missing something dangerous about this command?

---

## Why This Is A Good Question

✅ Specific title
✅ Code blocks for commands
✅ Version info included
✅ Expected vs actual behavior
✅ Polite and clear
✅ Shows you tried to understand the issue

## Response Time Expectations

- Simple questions: Usually < 24 hours
- Complex questions: 2-3 days
- Deep technical questions: May need core team input

## If You're Not Getting Answers

- Make sure title is specific (not "Help needed")
- Add more context
- Share what you've already tried
- Bump after 48 hours with additional info

We want to help! Make it easy for us. 🙏

---

⚡🛡️ Think Fast. Stay Safe.
```

---

### 🎨 Show and Tell

**Description:**
```
Share what you've built with cmdai!

Cool configurations, integrations, workflows, or projects.

Celebrate your work and inspire others!
```

**Settings:**
- Format: Discussion
- Labels: `showcase`, `integration`, `workflow`
- Encourages: Screenshots, code examples, demos

**Pinned Template:**
```
# Show Us What You Built!

We love seeing cmdai in action. Share:

🔧 **Configurations**
- Custom safety rules
- Backend setups
- Shell integrations

🏗️ **Projects**
- Tools built on cmdai
- Integrations with other software
- Workflow automations

💡 **Creative Uses**
- Unexpected use cases
- Productivity hacks
- Teaching materials

## Template (Optional)

**What I built:** [Brief description]

**Why:** [Problem you solved]

**How:** [Code/configuration]

**Screenshots:** [If applicable]

**Try it:** [Link or instructions]

---

## Community Favorites

We'll highlight exceptional showcases here monthly!

Keep building amazing things! 🚀

⚡🛡️
```

**Example Post:**
```
# VSCode Integration: cmdai Command Palette

I built a VSCode extension that integrates cmdai into the command palette!

## What It Does

- Select text in your editor describing a shell command
- Hit Cmd+Shift+P → "Generate cmdai command"
- See the generated command with safety validation
- One-click to copy to terminal or execute in integrated terminal

## Why I Built It

I wanted cmdai in my editor workflow without switching to terminal.

## How It Works

Uses cmdai CLI via VSCode tasks:
\`\`\`json
{
  "tasks": [
    {
      "label": "cmdai generate",
      "type": "shell",
      "command": "cmdai",
      "args": ["${selectedText}"],
      "presentation": {
        "reveal": "always",
        "panel": "new"
      }
    }
  ]
}
\`\`\`

## Try It

\`\`\`bash
# Install the extension
code --install-extension cmdai-vscode

# Or build from source
git clone https://github.com/username/cmdai-vscode
cd cmdai-vscode
npm install
vsce package
\`\`\`

## Demo

![Screenshot](link)

## Feedback Welcome!

What other editor integrations would be useful?

---

⚡🛡️
```

---

### 🗺️ Roadmap & Planning

**Description:**
```
Discuss cmdai's future direction, roadmap, and priorities.

Maintainers post roadmap proposals here for community feedback.

Your input shapes cmdai's future!
```

**Settings:**
- Format: Discussion
- Labels: `roadmap`, `planning`, `community-input`
- Pinned: Current quarter roadmap

**Pinned: Q1 2025 Roadmap**
```
# cmdai Q1 2025 Roadmap

Community input requested! This roadmap is a proposal, not set in stone.

## Themes

### 1. Safety Enhancements
**Goal:** Catch more dangerous patterns with fewer false positives

- [ ] Add domain-specific validators (k8s, terraform, docker)
- [ ] User-configurable safety rules
- [ ] Explain mode (why was this flagged?)
- [ ] Safety pattern marketplace (community-contributed)

**Community question:** What dangerous patterns are we missing?

### 2. Performance
**Goal:** Make cmdai invisible (faster than typing)

- [ ] Reduce startup time to <50ms
- [ ] Streaming LLM responses
- [ ] Better caching strategies
- [ ] Parallel validation

**Community question:** What's your tolerance for latency?

### 3. Integrations
**Goal:** Meet developers where they are

- [ ] VSCode extension
- [ ] Vim/Neovim plugin
- [ ] Warp terminal integration
- [ ] Raycast extension
- [ ] Alfred workflow

**Community question:** What tools do you want integrated?

### 4. Developer Experience
**Goal:** Make cmdai easier to use and contribute to

- [ ] Better error messages
- [ ] Interactive tutorial mode
- [ ] Configuration wizard
- [ ] Improved documentation
- [ ] Video tutorials

**Community question:** What's confusing about cmdai right now?

## Timeline

**January:**
- Safety enhancements
- VSCode extension MVP

**February:**
- Performance optimizations
- Better docs

**March:**
- Additional integrations
- Community validator marketplace

## How to Influence the Roadmap

1. **Upvote items you care about** (use 👍 reactions)
2. **Comment with your use case** (helps us prioritize)
3. **Volunteer to help** (speeds up delivery)
4. **Propose new items** (in Ideas category)

## What's NOT on the Roadmap (And Why)

❌ **Cloud/SaaS version** - Privacy-first means local-first
❌ **Windows-first features** - Cross-platform support, but Unix focus
❌ **Paid features** - Open source, always will be

## Questions?

Ask here! This is a living roadmap.

Last updated: 2025-01-15

---

⚡🛡️ Think Fast. Stay Safe.
```

---

### 🛡️ Safety & Security

**Description:**
```
Discuss safety patterns, validation logic, and security considerations.

Report security vulnerabilities privately (see SECURITY.md).
Discuss general safety philosophy here.
```

**Settings:**
- Format: Discussion
- Labels: `safety`, `security`, `validation`

**Pinned: Safety Philosophy**
```
# cmdai's Safety Philosophy

Understanding how and why we validate commands.

## Core Principles

### 1. Conservative by Default
**Better false positive than false negative.**

If we're unsure, we warn. Annoying >> catastrophic.

### 2. Transparent Validation
**You can read the validation logic.**

Open source means you can audit every safety rule.

See: [src/safety/mod.rs](link)

### 3. User Maintains Control
**We inform, you decide.**

Even CRITICAL warnings can be overridden (with explicit confirmation).

You're the human. We're the guardrail.

### 4. Continuous Improvement
**Community makes us better.**

Every new dangerous pattern makes the validator stronger.

## Current Validation Layers

### Layer 1: Pattern Matching
Match known dangerous operations:
- Filesystem destruction: `rm -rf /`, `mkfs`, `dd`
- Privilege escalation: `sudo su`, `chmod 777 /etc`
- Resource exhaustion: fork bombs
- Data exfiltration: suspicious network operations

### Layer 2: POSIX Compliance
Catch syntax errors and bash-specific features.

### Layer 3: Risk Assessment
Categorize commands:
- 🟢 SAFE - Common operations, low risk
- 🟡 MODERATE - Potentially destructive, needs review
- 🟠 HIGH - Dangerous, requires confirmation
- 🔴 CRITICAL - Auto-blocked, extremely dangerous

## What We DON'T Do

❌ **Simulate execution** - Too slow and complex
❌ **AI-based validation** - Not deterministic enough
❌ **Network-based checks** - Privacy and speed concerns
❌ **Block without explanation** - Always show WHY

## Contribute to Safety

### Report Dangerous Patterns We're Missing

Found a command that should be flagged but isn't?

Share it here:
1. The dangerous command (in code blocks)
2. Why it's dangerous
3. What risk level you think it should be

### Report False Positives

Got flagged for a safe command?

Help us tune the validator:
1. The safe command that was flagged
2. Why it's actually safe
3. Your use case

### Propose New Validation Strategies

Ideas for better safety validation:
- Static analysis approaches
- Domain-specific validators
- User-configurable rules

## Examples

### Good Safety Discussion
```
**Command:** `find / -type f -exec rm {} \;`
**Risk:** HIGH (should probably be CRITICAL)

**Why dangerous:**
- Deletes ALL files on system
- No confirmation
- No way to undo

**Recommendation:** Treat `find` + `-exec rm` as CRITICAL pattern

**Safer alternative:** `find /specific/dir -type f -name "*.tmp" -delete`
```

### Security Vulnerability (DON'T POST HERE)
```
If you found a security vulnerability:

1. DON'T post it publicly
2. Email security@cmdai.dev (or see SECURITY.md)
3. We'll acknowledge within 24 hours
4. We'll credit you in the fix

Responsible disclosure protects users.
```

---

Discuss safety philosophy, validation strategies, and dangerous patterns here.

Together, we make cmdai safer for everyone.

⚡🛡️
```

---

### 🤝 Contributing

**Description:**
```
Discuss contributing to cmdai: code, docs, community, design.

New contributors welcome!

Read CONTRIBUTING.md before your first PR.
```

**Settings:**
- Format: Discussion
- Labels: `contributing`, `good-first-issue`, `help-wanted`

**Pinned: Start Here**
```
# Contributing to cmdai: Start Here

Thank you for considering contributing! 🙏

## Ways to Contribute

### 1. Code
- Fix bugs
- Add features
- Improve performance
- Write tests

**Start:** Issues labeled `good-first-issue`

### 2. Documentation
- Fix typos
- Improve clarity
- Add examples
- Write tutorials

**Start:** Issues labeled `docs`

### 3. Community
- Answer questions in Q&A
- Review PRs
- Welcome new members
- Share cmdai

**Start:** Right now in Discussions!

### 4. Design
- ASCII art improvements
- Terminal UI design
- Brand applications
- Documentation layout

**Start:** Open a discussion with ideas

### 5. Testing
- Try edge cases
- Report bugs
- Validate safety patterns
- Cross-platform testing

**Start:** Use cmdai and report what breaks

## Your First Contribution

### 1. Read the Docs
- [CONTRIBUTING.md](link)
- [CLAUDE.md](link) (development guide)
- [Code of Conduct](link)

### 2. Set Up Development Environment
\`\`\`bash
# Clone the repo
git clone https://github.com/wildcard/cmdai
cd cmdai

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- "your command description"
\`\`\`

### 3. Find an Issue
- Browse [good-first-issue](link)
- Comment that you're working on it
- Ask questions if unclear

### 4. Make Your Changes
- Create a branch
- Write code
- Add tests
- Update docs

### 5. Submit PR
- Clear title and description
- Link to issue
- Include tests
- Follow Rust style guide (`cargo fmt`, `cargo clippy`)

### 6. Code Review
- Respond to feedback
- Make requested changes
- Be patient (we're volunteers too!)

## Questions?

Ask here! No question is too basic.

We were all beginners once. 😊

---

⚡🛡️ Think Fast. Stay Safe. Contribute.
```

---

### 🌍 General Discussion

**Description:**
```
Anything else related to cmdai, terminals, AI, or development.

Off-topic chat, community building, and general discussion.
```

**Settings:**
- Format: Discussion
- Labels: `discussion`, `community`

**Welcome Post:**
```
# Welcome to General Discussion!

This is your space for:
- Off-topic (but friendly) chat
- Terminal tips and tricks
- AI and LLM discussions
- Development workflows
- Community building

## Ground Rules

✅ Be kind and respectful
✅ Keep it SFW
✅ Follow Code of Conduct
✅ Have fun!

❌ No spam
❌ No self-promotion (unless relevant and not excessive)
❌ No drama

---

This is YOUR community. Make it awesome!

⚡🛡️
```

---

## Discussion Labels

### Status Labels
- `answered` - Question has been answered
- `needs-triage` - Needs maintainer review
- `in-progress` - Actively being worked on
- `blocked` - Waiting on something
- `wontfix` - Won't implement (with explanation)

### Type Labels
- `question`
- `feature-request`
- `enhancement`
- `bug-report` (should probably be an Issue)
- `discussion`
- `showcase`
- `documentation`

### Priority Labels
- `high-priority`
- `low-priority`
- `good-first-issue`
- `help-wanted`

### Topic Labels
- `safety`
- `security`
- `performance`
- `integration`
- `backend`
- `validation`

---

## Moderation Guidelines

### Response Times
- Announcements: Posted when ready
- Feature requests: Triage within 1 week
- Questions: Best effort, community-driven
- Security: Within 24 hours

### Moving Discussions
If posted in wrong category, gently redirect:
```
Thanks for posting! This would be better in [Category] - mind if we move it?

[Explain why]

No worries - just helps keep things organized! 😊
```

### Closing Discussions
Close when:
- Question answered and marked as such
- Feature request moved to Issue
- Discussion exhausted
- Duplicate of existing discussion

Always explain why:
```
Closing this as the question has been answered by @user.

If you have follow-up questions, feel free to open a new discussion!

Thanks for being part of the community! ⚡🛡️
```

---

## Integration with Issues

### When to Use Discussions vs Issues

**Use Discussions for:**
- Questions ("How do I...?")
- Feature ideas (before implementation)
- General feedback
- Community chat
- Roadmap planning

**Use Issues for:**
- Confirmed bugs
- Planned features (after discussion)
- Tasks and tracking
- Release planning

### Moving from Discussion to Issue
```
Great idea! This has enough support to move to implementation.

I've opened an issue to track: #123

Let's continue technical discussion there. Thanks for the proposal! 🚀
```

---

## Templates for Common Responses

### Thank You for Contributing
```
Thanks for the thoughtful post! 🙏

[Specific response to their content]

We really appreciate community input like this - it makes cmdai better for everyone.

⚡🛡️
```

### That's a Known Issue
```
Good catch! This is a known issue.

We're tracking it here: #123

You can subscribe to that issue for updates. We'll post here when it's resolved.

Thanks for reporting! 🛡️
```

### That's Out of Scope
```
Thanks for the suggestion!

This is interesting, but outside cmdai's current scope.

[Explain why]

That said, cmdai is open source - you're welcome to fork and build this yourself!

Or if the community really wants it, we might reconsider in the future.

Appreciate you thinking about this! 🙏
```

### Needs More Information
```
Can you provide a bit more detail?

Specifically:
- [What info you need]
- [What info you need]

This will help us understand the use case better!

Thanks! 🚀
```

---

## Metrics to Track

### Health Metrics
- Active discussions per week
- Response time to questions
- % of questions answered by community (vs maintainers)
- Discussion-to-Issue conversion rate

### Engagement Metrics
- Upvotes on feature requests
- Comments per discussion
- Community showcase frequency
- Cross-discussion participation

### Quality Metrics
- Duplicate discussions
- Off-topic discussions
- Moderation actions needed
- Positive sentiment

---

## Launch Checklist

- [ ] Enable GitHub Discussions on repo
- [ ] Create all categories
- [ ] Write pinned posts for each category
- [ ] Create labels
- [ ] Write discussion templates
- [ ] Add moderators
- [ ] Link from README
- [ ] Announce on social media
- [ ] Seed with initial discussions
- [ ] Monitor daily for first week

---

## Success Patterns

### What Good Looks Like
- Community answers questions before maintainers
- Feature requests are well-thought-out
- Showcases inspire others
- Respectful disagreements
- Gratitude expressed
- New contributors feel welcome

### Red Flags
- Unanswered questions piling up
- Combative tone
- Spam or off-topic posts
- Maintainer burnout
- Declining participation

---

**Remember:** Discussions are where community happens. Make it welcoming, helpful, and inspiring.

⚡🛡️ Think Fast. Stay Safe. Discuss.
