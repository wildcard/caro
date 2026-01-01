# Social Media Guide & Templates

**Owner:** DevRel Team
**Last Updated:** January 1, 2025
**Version:** 1.0

---

## Brand Voice & Messaging

### Core Identity
- **Name:** Caro
- **Tagline:** "Your loyal shell companion"
- **Personality:** Loyal, protective, smart (like a Shiba Inu)
- **Origin Story:** Kyaro (beloved dog) → Caro, inspired by Portal's Caroline → GLaDOS

### Key Messages
1. **Safety-First:** AI-generated commands with comprehensive validation
2. **Local-First:** No cloud dependencies, works offline
3. **Cross-Platform:** macOS, Linux, Windows support
4. **Integration:** MCP for Claude, standalone CLI, Claude Code Skill
5. **Open Source:** AGPL-3.0, community-driven

### Tone Guidelines
- **Professional** but **approachable**
- **Technical** but **beginner-friendly**
- **Serious about safety** but not scary
- **Dog references welcome** (Caro is a Shiba!)

---

## Platform-Specific Strategies

### Twitter/X (@CaroShell)

**Character Limit:** 280 characters (4000 for Premium)
**Best Times:** Weekdays 9-11 AM, 1-3 PM PT
**Frequency:** 3-5 tweets/week
**Hashtags:** 2-3 max

**Content Mix:**
- 40% - Educational (safety tips, command examples)
- 30% - Community (contributor shoutouts, milestones)
- 20% - Product updates (releases, features)
- 10% - Fun (Shiba content, memes)

---

### Reddit

**Key Subreddits:**
- /r/rust - Rust programming
- /r/commandline - CLI tools
- /r/linux - Linux users
- /r/programming - General programming
- /r/opensource - Open source projects

**Guidelines:**
- **No self-promotion spam** - Be genuine
- **Provide value first** - Answer questions, share insights
- **10:1 Rule** - 10 comments for every 1 self-promotional post
- **Respect subreddit rules** - Read before posting

---

### Hacker News

**Best Times:** Weekdays 8-10 AM PT
**Title Format:** "Show HN: Caro – Local-first shell companion with AI safety"
**Frequency:** Major releases only (1-2x per quarter)

**Guidelines:**
- **Substantive content** - Link to GitHub, not marketing site
- **Be present** - Respond to all comments within 2-4 hours
- **Take criticism well** - HN can be harsh but valuable
- **Technical depth** - HN audience is sophisticated

---

### LinkedIn

**Audience:** Enterprise decision makers, developers, CTOs
**Frequency:** 2-3 posts/week
**Best Times:** Weekdays 7-9 AM, 12-1 PM PT

**Content Types:**
- Product updates with business value
- Security thought leadership
- Open source success stories
- Hiring/contributor opportunities

---

### Mastodon

**Server:** mastodon.social or fosstodon.org
**Audience:** Privacy-conscious, FOSS enthusiasts
**Frequency:** 3-5 toots/week
**Tone:** Community-focused, technical

---

### Dev.to

**Format:** Long-form technical articles
**Frequency:** 1-2 articles/month
**Topics:**
- Tutorial content
- Behind-the-scenes technical deep dives
- Open source journey posts

---

## Content Templates

### Product Launch Announcement

#### Twitter
```
🎉 Introducing Caro v1.0 – Your loyal shell companion!

Generate safe shell commands from natural language using local LLMs.

🛡️ Safety-first validation
🏠 Fully offline-capable
🦀 Built with Rust
🔌 MCP integration for Claude

Try it: cargo install caro

https://caro.sh

#Rust #CLI #AI #OpenSource
```

#### Reddit /r/rust
```markdown
**Title:** [Show Rust] Caro – Local-first shell command generation with AI safety

**Body:**
Hi r/rust! I've been working on Caro, a Rust CLI that converts natural language to shell commands using local LLMs, with a focus on safety.

**Key features:**
- Safety-first command validation (prevents `rm -rf /` disasters)
- Local LLM inference (MLX on Apple Silicon, Candle for CPU)
- Cross-platform (macOS, Linux, Windows)
- MCP integration with Claude Desktop
- AGPL-3.0 licensed

**Why Rust?**
- Single binary distribution
- Blazing fast (<100ms startup)
- Strong type safety for safety-critical code
- Excellent async support (tokio)

**Tech stack:**
- clap for CLI
- tokio for async runtime
- candle/MLX for inference
- comprehensive safety validator

Would love feedback from the Rust community! Especially interested in:
- Architecture review
- Performance optimization ideas
- Cross-platform testing help

**Links:**
- GitHub: https://github.com/wildcard/caro
- Website: https://caro.sh
- Contributing: https://github.com/wildcard/caro/blob/main/CONTRIBUTING.md

**Questions welcome!** I'll be here to answer throughout the day.
```

#### Hacker News
```
Title: Show HN: Caro – Local-first shell companion with AI command generation

URL: https://github.com/wildcard/caro

Text:
Hi HN! I built Caro, a Rust CLI that generates shell commands from natural language using local LLMs, with a strong focus on safety.

The core problem: AI-generated shell commands can be dangerous. A single "rm -rf /" can destroy your system. Caro addresses this with comprehensive safety validation:

- Pattern matching for dangerous operations
- Risk scoring (safe → critical)
- User confirmation for risky commands
- Audit logging

It runs entirely locally (no cloud dependencies) using MLX on Apple Silicon or Candle for CPU inference. Supports Ollama/vLLM backends too.

Built in Rust for performance (<100ms startup, <2s inference on M1). Single binary, no dependencies.

Also available as an MCP server for Claude Desktop and a Claude Code Skill.

Open questions I'm exploring:
- What safety patterns should be default-deny vs warn-only?
- How to balance protection vs user autonomy?
- Best practices for local LLM deployment in CLI tools?

Would love feedback from the HN community, especially on security and UX tradeoffs.

GitHub: https://github.com/wildcard/caro
Website: https://caro.sh
```

#### LinkedIn
```
Excited to announce Caro v1.0 - a local-first shell companion for enterprise developers who need AI assistance without compromising security. 🛡️

**Why enterprises are adopting Caro:**

✅ Fully offline operation - No cloud dependencies, works in air-gapped environments
✅ Comprehensive safety validation - Prevents catastrophic command execution
✅ Auditable and transparent - AGPL-3.0 open source, no telemetry
✅ Cross-platform support - macOS, Linux, Windows

**Perfect for:**
• Regulated industries (finance, healthcare, government)
• Security-conscious development teams
• Air-gapped or compliance-restricted environments
• Organizations with data sovereignty requirements

Built with Rust for performance and reliability. Integrates seamlessly with Claude Desktop via MCP.

Learn more: https://caro.sh
Enterprise inquiries: [contact info]

#EnterpriseAI #DevOps #Security #OpenSource
```

---

### Feature Announcement

#### Twitter
```
New in Caro v1.1: Windows PowerShell support! 🎉

Now detects dangerous PowerShell patterns:
• Remove-Item -Recurse -Force
• Set-ExecutionPolicy Unrestricted
• Disable-WindowsDefender

Cross-platform safety, unified experience.

Thanks to contributors @username1 @username2!

https://github.com/wildcard/caro/releases/v1.1

#PowerShell #Security
```

#### Dev.to Article
```markdown
Title: Building a Safety Validator for AI-Generated Shell Commands

Subtitle: How we prevent "rm -rf /" disasters in Caro

Tags: rust, ai, security, cli

[Article content exploring:
- The problem space
- Design decisions
- Implementation details
- Lessons learned
- Future improvements]
```

---

### Contributor Spotlight

#### Twitter
```
Contributor Spotlight 🌟

Big thanks to @contributorname for implementing the MCP integration!

This brought Caro to Claude Desktop users, enabling safe command generation directly in chat.

250+ commits, 3 weeks of work, flawless execution.

This is what great OSS contributions look like. 🙏

https://github.com/wildcard/caro/pull/XXX
```

#### LinkedIn
```
Celebrating Outstanding Open Source Contribution 🎉

[Contributor Name] recently contributed the MCP integration to Caro, enabling seamless Claude Desktop integration.

This is what exceptional open source collaboration looks like:
• Clear communication
• Thorough testing
• Complete documentation
• Community-focused design

Looking to build your OSS portfolio? Caro has contribution lanes for all skill levels: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md

#OpenSource #DevCommunity #Rust
```

---

### Security Update

#### Twitter
```
🚨 Security Update: Caro v1.0.4

Fixed prompt injection vulnerability (CVE-XXXX-XXXXX).

All users should upgrade immediately:
cargo install caro --force

Thanks to @security_researcher for responsible disclosure.

Full advisory: https://github.com/wildcard/caro/security/advisories/XXX

#Security #InfoSec
```

#### Reddit /r/netsec
```markdown
**Title:** [Disclosure] Prompt Injection Vulnerability in Caro Shell Agent (CVE-XXXX-XXXXX)

**Body:**
Posting here for visibility. We discovered and patched a prompt injection vulnerability in Caro, a local-first shell command generator.

**Vulnerability:**
[Technical description]

**Impact:**
[What could happen]

**Mitigation:**
Upgrade to v1.0.4 or later: `cargo install caro --force`

**Timeline:**
- Dec 15: Discovered during internal security audit
- Dec 16: Patch developed
- Dec 17: Released v1.0.4
- Dec 20: Public disclosure (72hr after patch)

**Credit:**
[Security researcher] for discovering and responsibly reporting

Full advisory: https://github.com/wildcard/caro/security/advisories/XXX

Questions welcome.
```

---

### Tutorial/Educational Content

#### Twitter Thread
```
🧵 Thread: 5 Ways Caro Keeps Your Terminal Safe

1/ Safety Patterns

Caro blocks 50+ dangerous command patterns out of the box:
• rm -rf /
• :(){ :|:& };:
• mkfs.*
• dd if=/dev/zero

Each pattern is tested against OWASP command injection vectors.

2/ Risk Scoring

Commands are scored on 4 levels:
🟢 Safe - Execute immediately
🟡 Moderate - User confirmation
🟠 High - Typed confirmation required
🔴 Critical - Blocked in strict mode

3/ Path Protection

Caro validates paths against:
• System directories (/bin, /usr, /etc)
• Home directory (~)
• Current working directory (.)

Prevents accidental traversal outside allowed roots.

4/ Audit Logging

Every generated command is logged with:
• Timestamp
• User prompt
• Generated command
• Safety level
• Execution result

Full audit trail for security compliance.

5/ User Education

Caro explains WHY commands are dangerous, teaching users to:
• Recognize risky patterns
• Understand command implications
• Build safer habits

Safety through education, not just blocking.

---

Learn more about Caro's safety architecture:
https://caro.sh/ai-command-safety

Or contribute safety patterns:
https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#security-lane
```

---

### Community Milestone

#### Twitter
```
🎉 Milestone Alert!

Caro just hit 1,000 GitHub stars! ⭐

Thank you to our amazing community:
• 50+ contributors
• 6 active lane leads
• 200+ closed issues
• 15 countries represented

This is just the beginning. Local-first AI is the future.

Join us: https://github.com/wildcard/caro

#OpenSource #Community
```

#### Reddit /r/opensource
```markdown
**Title:** Celebrating 1,000 Stars on Caro - Local-First Shell Companion

We just hit 1,000 GitHub stars on Caro! 🎉

**What is Caro?**
A Rust CLI that converts natural language to shell commands using local LLMs, with comprehensive safety validation.

**Community Highlights:**
- 50+ contributors from 15 countries
- 6 specialized contribution lanes (Security, Runtime, Inference, UX, Ecosystem, Distribution)
- 200+ issues closed with community help
- 15 releases in first 6 months

**What's Next:**
- Advanced policy engine for enterprise
- Expanded platform support
- Enhanced MCP capabilities
- Performance optimizations

**Contributing:**
We have a two-tier contribution system:
1. Beginner-friendly first-time issues
2. Professional contribution lanes for experienced devs

Check it out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md

Thanks to everyone who's contributed, starred, and shared Caro! This community is what makes open source amazing.
```

---

## Content Calendar Template

### Weekly Rhythm

**Monday:**
- Community highlight (contributor spotlight, issue closed, PR merged)

**Wednesday:**
- Educational content (safety tip, command example, feature deep dive)

**Friday:**
- Fun content (Shiba meme, behind-the-scenes, community story)

### Monthly Themes

**Week 1:** Product & Features
**Week 2:** Community & Contributors
**Week 3:** Education & Tutorials
**Week 4:** Security & Best Practices

### Special Events

**Quarterly:**
- Major release announcement
- Community meetup/AMA
- Roadmap update

**Annually:**
- Year in review
- Kyaro's birthday (Caro's inspiration)
- Open source anniversary

---

## Hashtag Strategy

### Primary Hashtags (Always Use)
- #Rust
- #CLI
- #OpenSource

### Secondary Hashtags (Rotate)
- #AI #MachineLearning #LLM
- #DevOps #SRE #Sysadmin
- #Security #InfoSec #CyberSecurity
- #TerminalTools #ShellScripting
- #LocalFirst #Privacy

### Community Hashtags
- #RustLang #RustProgramming
- #OpenSourceContributors
- #DevCommunity

### Event Hashtags
- #Hacktoberfest (October)
- #RustConf (when attending)
- Relevant conference tags

---

## Visual Assets

### Logo Usage
- **Full logo:** For headers, featured images
- **Icon only:** For profile pictures, favicons
- **ASCII art:** For terminal screenshots

### Brand Colors
- **Primary:** Orange (#ff8c42 to #ff6b35 gradient)
- **Background:** Warm white (#fff8f0)
- **Text:** Dark blue-gray (#2c3e50)
- **Accent:** Gray (#7f8c8d)

### Image Guidelines
- **Screenshots:** Clean terminal with Caro branding
- **Diagrams:** Simple, clear, on-brand colors
- **Memes:** Shiba-related, on-brand humor
- **Code:** Syntax highlighted, readable

---

## Engagement Guidelines

### Responding to Comments

**Positive Comments:**
```
Thanks for trying Caro! Let us know if you have any questions. 🐕
```

**Questions:**
```
Great question! [Answer]. You can find more details here: [link]
```

**Bug Reports:**
```
Thanks for reporting! I've created an issue to track this: [GitHub link]. We'll investigate and update you there.
```

**Criticism:**
```
Appreciate the feedback! You're right about [acknowledge valid point]. We're working on [solution]. Want to help? Here's the issue: [link]
```

**Spam/Trolls:**
- Don't engage
- Report if necessary
- Move on

---

## Analytics & Tracking

### Metrics to Track

**Engagement:**
- Likes, shares, comments
- Click-through rate (CTR)
- Follower growth

**Website:**
- Referral traffic from social
- Bounce rate from social visitors
- Conversion to GitHub stars/installs

**Community:**
- New contributors from social
- Issue activity from social posts
- Community sentiment

### Tools
- Twitter Analytics (native)
- Reddit Analytics (third-party)
- Google Analytics (website)
- GitHub Insights (stars, clones, traffic)

---

## Crisis Communication

### If Something Goes Wrong

**Security Vulnerability:**
1. Acknowledge immediately
2. Provide timeline for fix
3. Communicate clearly and often
4. Post-mortem after resolution

**Bad Press:**
1. Don't panic
2. Gather facts
3. Respond factually, professionally
4. Learn and improve

**Community Conflict:**
1. Refer to Code of Conduct
2. De-escalate privately first
3. Enforce consequences if needed
4. Document for transparency

---

## Templates Library

### GitHub → Social Cross-Post

**When:** New release, major PR merged, milestone reached

**Template:**
```
[Accomplishment] 🎉

[Brief description]

[Key highlights]
• Bullet 1
• Bullet 2
• Bullet 3

[Call to action]

GitHub: [link]
Docs: [link]

#Rust #OpenSource
```

### Contributor Thank You

**When:** PR merged, issue resolved, significant contribution

**Template:**
```
Huge thanks to @username for [contribution]! 🙏

[Impact of contribution]

This is what great OSS looks like.

PR: [link]

Want to contribute? Check out: [HELP_WANTED link]
```

### Security Advisory

**When:** Security issue discovered and patched

**Template:**
```
🚨 Security Update: Caro v[X.Y.Z]

[Brief description of vulnerability]

**Action Required:**
cargo install caro --force

**Details:**
• [CVE if applicable]
• [Impact]
• [Mitigation]

Full advisory: [link]

Thanks to @researcher for responsible disclosure.

#Security #InfoSec
```

---

## Approval Workflow

### Post Approval

**No Approval Needed:**
- Routine updates (releases, features)
- Community highlights
- Educational content
- Responses to comments

**Approval Required:**
- Security announcements
- Controversial topics
- Policy changes
- Partnerships

**Approval Process:**
1. Draft in Google Doc or GitHub issue
2. Tag @maintainers for review
3. Wait for 2+ approvals
4. Schedule and post
5. Monitor and respond

---

## Best Practices

### Do's ✅
- ✅ Be authentic and transparent
- ✅ Respond to comments within 24 hours
- ✅ Give credit generously
- ✅ Admit mistakes and learn
- ✅ Celebrate community wins
- ✅ Use inclusive language
- ✅ Share behind-the-scenes content
- ✅ Cross-promote community members

### Don'ts ❌
- ❌ Auto-post identical content everywhere
- ❌ Ignore negative feedback
- ❌ Over-promote without value
- ❌ Make promises you can't keep
- ❌ Engage in flame wars
- ❌ Steal content without attribution
- ❌ Post when angry or defensive
- ❌ Neglect platform-specific norms

---

## Resources

**Tools:**
- **Buffer/Hootsuite:** Social media scheduling
- **Canva:** Graphics creation
- **Grammarly:** Copy editing
- **TweetDeck:** Twitter management
- **Analytics:** Platform-native tools

**Learning:**
- [Twitter Marketing Guide](https://business.twitter.com/en/basics/create-a-marketing-strategy.html)
- [Reddit Marketing Guide](https://www.reddit.com/r/changelog/comments/6xfyfg/an_update_on_the_state_of_the_redditreddit_and/)
- [Dev.to Writing Guide](https://dev.to/devteam/how-to-write-great-technical-articles-2i66)

---

## Questions?

Contact: DevRel team via GitHub Discussions

**Last Updated:** January 1, 2025
