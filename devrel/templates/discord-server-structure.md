# Discord Server Structure for cmdai

Complete Discord server setup with channels, roles, rules, and moderation guidelines.

---

## Server Overview

**Server Name:** cmdai Community
**Description:** Official community for cmdai - AI-powered commands with human-level safety
**Vanity URL:** discord.gg/cmdai (if available)
**Icon:** ⚡🛡️ cmdai logo on Deep Space (#0A0E27) background

---

## Channel Structure

### 📢 INFORMATION

#### #welcome
**Purpose:** First channel newcomers see
**Permissions:** Read-only for @everyone, post-only for @Moderator
**Auto-messages:** Welcome bot greeting

**Pinned Message:**
```
⚡🛡️ Welcome to the cmdai Community!

We're building AI-powered terminal automation with built-in safety validation.

🚀 GET STARTED:
• Read the rules in #rules
• Introduce yourself in #introductions
• Check out #getting-started for installation
• Ask questions in #help

🛠️ QUICK LINKS:
• GitHub: https://github.com/wildcard/cmdai
• Docs: https://cmdai.dev/docs
• Website: https://cmdai.dev

💬 BE KIND:
We're all here to learn and help each other. Follow our Code of Conduct and enjoy the community!

Think Fast. Stay Safe. 🛡️
```

#### #rules
**Purpose:** Community guidelines and Code of Conduct
**Permissions:** Read-only for @everyone

**Content:**
```
⚡🛡️ cmdai Community Rules

These rules keep our community healthy and welcoming.

1️⃣ BE RESPECTFUL
Treat everyone with dignity. No harassment, discrimination, or personal attacks.

2️⃣ BE HELPFUL
Answer questions patiently. Share knowledge freely. Celebrate others' success.

3️⃣ STAY ON TOPIC
Use appropriate channels. Keep discussions relevant. No spam or excessive self-promotion.

4️⃣ NO DANGEROUS COMMANDS (UNLESS DISCUSSING SAFETY)
Don't post malicious code. If sharing dangerous commands for discussion, use code blocks and warn users.

5️⃣ RESPECT PRIVACY
Don't share others' personal information. Don't DM people without permission.

6️⃣ FOLLOW DISCORD TOS
https://discord.com/terms

7️⃣ MODERATORS HAVE FINAL SAY
If a mod asks you to stop, stop. Appeal via DM if you disagree.

---

VIOLATIONS:
• First offense: Warning
• Second offense: Temporary timeout (7-30 days)
• Third offense or severe violation: Ban

---

REPORT ISSUES:
• React with 🚩 on problematic messages
• DM a moderator
• Use #mod-support channel

---

Full Code of Conduct: https://github.com/wildcard/cmdai/blob/main/culture/COMMUNITY_GUIDELINES.md

Questions? Ask in #meta

Think Fast. Stay Safe. Be Kind. ⚡🛡️
```

#### #announcements
**Purpose:** Official cmdai announcements
**Permissions:** Read-only for @everyone, post-only for @Core Team
**Settings:** Enable Discord notifications for all members

**Pin announcement template:**
```
📢 Announcement Format:

**[Release/Event/Important]**

Brief description of what's happening

🔗 Relevant links
📅 Dates if applicable
💬 Where to discuss: #channel-name

---
Posted by @username
```

#### #changelog
**Purpose:** Automated feed of GitHub releases and major PRs
**Permissions:** Read-only for @everyone
**Integrations:** GitHub webhook for releases

---

### 💬 COMMUNITY

#### #general
**Purpose:** General chat about cmdai, AI, terminals, development
**Permissions:** Open to @everyone
**Slow mode:** 5 seconds (prevent spam)

**Channel description:**
```
General discussion about cmdai and related topics.

Keep it friendly and on-topic. For specific help, use #help instead.
```

#### #introductions
**Purpose:** New member introductions
**Permissions:** Open to @everyone
**Slow mode:** 30 seconds

**Channel description:**
```
👋 Introduce yourself!

Tell us:
• Your name/handle
• What you do
• How you found cmdai
• What you're excited about

Welcome! ⚡🛡️
```

**Pinned template:**
```
👋 Introduction Template (optional):

**Name:**
**Role:** (student/developer/SRE/etc.)
**Location:** (timezone helpful for community calls)
**Found cmdai via:** (Twitter/Reddit/HN/etc.)
**Interested in:** (using/contributing/learning)
**Fun fact:** (optional)

Welcome to the community! 🛡️
```

#### #show-and-tell
**Purpose:** Share projects, configurations, cool commands
**Permissions:** Open to @everyone
**Slow mode:** 60 seconds

**Channel description:**
```
🎨 Share what you're building with cmdai!

• Custom configurations
• Cool commands you've generated
• Integration projects
• Workflow improvements

Celebrate your wins! 🏆
```

#### #off-topic
**Purpose:** Non-cmdai chat, community bonding
**Permissions:** Open to @everyone
**Slow mode:** None

**Channel description:**
```
💬 Off-topic chat, memes, general discussion

Anything goes (within Discord TOS and our rules).

Not sure if it belongs here? It probably does.
```

---

### 🆘 SUPPORT

#### #help
**Purpose:** User support and troubleshooting
**Permissions:** Open to @everyone
**Slow mode:** 10 seconds
**Tags:** Enable forum-style threads

**Channel description:**
```
🆘 Get help with cmdai

BEFORE POSTING:
• Search this channel for similar issues
• Check #faq and documentation
• Include version: `cmdai --version`

WHEN ASKING:
✅ What you're trying to do
✅ What you tried
✅ What happened instead
✅ Error messages (use code blocks)
✅ Your environment (OS, cmdai version)

❌ "It doesn't work help!!!"

Be patient. Community volunteers answer in their free time. 🙏
```

**Auto-response bot triggers:**
```
Trigger: "doesn't work"
Response: "Can you provide more details? See the pinned message for what to include in help requests."

Trigger: "cmdai --version"
Response: "Great! Including version info helps us help you faster. 🚀"
```

#### #faq
**Purpose:** Common questions and answers
**Permissions:** Read-only for @everyone, post-only for @Moderator
**Format:** Q&A pairs

**Initial FAQs:**
```
❓ How do I install cmdai?

Check our getting started guide: [link]

For quick install:
• macOS: `brew install cmdai` (coming soon)
• Cargo: `cargo install cmdai`
• Binary: Download from GitHub releases

---

❓ Is my data sent to external APIs?

No. All inference runs locally. Your commands never leave your machine.

cmdai supports local backends: MLX, Ollama, vLLM.

---

❓ What if cmdai blocks a safe command?

You can:
1. Override the warning (for HIGH risk)
2. Report false positive on GitHub
3. Customize safety rules (advanced)

We err on the side of caution.

---

❓ Can I use cmdai offline?

Yes! Once models are cached, cmdai works completely offline.

---

❓ How do I contribute?

Check CONTRIBUTING.md: [link]
Ask in #development
Start with issues tagged "good-first-issue"

---

More questions? Ask in #help!
```

#### #bug-reports
**Purpose:** Bug tracking and triage
**Permissions:** Open to @everyone
**Format:** Forum-style threads
**Integration:** Sync to GitHub Issues

**Channel description:**
```
🐛 Report bugs here

BEFORE POSTING:
• Check if it's already reported on GitHub
• Make sure you're on latest version
• Try to reproduce it

INCLUDE:
• cmdai version: `cmdai --version`
• OS and version
• Steps to reproduce
• Expected vs actual behavior
• Error messages (code blocks)

Critical security bugs: Email [address] instead

Thank you for making cmdai better! 🛡️
```

#### #feature-requests
**Purpose:** Feature ideas and discussion
**Permissions:** Open to @everyone
**Format:** Forum-style threads

**Channel description:**
```
💡 Suggest features and improvements

GOOD REQUESTS:
• Describe the problem you're solving
• Explain your use case
• Suggest implementation (optional)
• Consider safety implications

AVOID:
• Vague requests ("make it better")
• Duplicates (search first)
• Demands ("you must add this")

All ideas welcome. Core team reviews regularly. 🚀
```

---

### 🛠️ DEVELOPMENT

#### #development
**Purpose:** Contributor discussion, architecture, implementation
**Permissions:** Open to @everyone (read) and @Contributor (post)
**Integration:** GitHub activity feed

**Channel description:**
```
🛠️ Developer discussion

For contributors and aspiring contributors:
• Architecture questions
• Implementation discussions
• Code review
• Development workflow

New to contributing? Introduce yourself! We're friendly. 😊
```

#### #pull-requests
**Purpose:** Automated feed of GitHub PRs
**Permissions:** Read-only for @everyone
**Integration:** GitHub webhook for PR events

#### #safety-patterns
**Purpose:** Discuss dangerous command patterns and validation logic
**Permissions:** Open to @everyone
**Slow mode:** 30 seconds

**Channel description:**
```
🛡️ Safety validator discussion

Share:
• Dangerous commands you've encountered
• Patterns cmdai should block
• False positives/negatives
• Safety validation ideas

⚠️ When sharing dangerous commands, use code blocks and explain WHY they're dangerous.

This helps us improve safety validation for everyone.
```

---

### 🎉 EVENTS

#### #community-calls
**Purpose:** Monthly community call coordination
**Permissions:** Open to @everyone

**Channel description:**
```
📞 Monthly community calls

First Tuesday of every month
30 minutes
Open to all

Agenda:
• Roadmap updates
• Community highlights
• Q&A

Recordings posted to YouTube.

Next call: [date/time]
Add to calendar: [link]
```

#### #meetups
**Purpose:** Local/virtual meetup coordination
**Permissions:** Open to @everyone

**Channel description:**
```
🌍 cmdai meetups

Organize or find:
• Local user groups
• Virtual workshops
• Conference meetups
• Hack sessions

Ping @Meetup-Organizer if you want to host one. We'll help promote and provide swag!
```

---

### 📚 RESOURCES

#### #learning-resources
**Purpose:** Tutorials, guides, articles
**Permissions:** Open to @everyone (curated)

**Channel description:**
```
📚 Learning resources

• Official docs: [link]
• Tutorials and guides
• Video walkthroughs
• Blog posts
• Conference talks

Share helpful resources you've found!
```

#### #rust-help
**Purpose:** Rust-specific help (for contributors)
**Permissions:** Open to @everyone

**Channel description:**
```
🦀 Rust programming help

For contributors learning Rust:
• Language questions
• Best practices
• Debugging help

Not cmdai-specific Rust questions welcomed!
```

---

### ⚙️ META

#### #feedback
**Purpose:** Feedback on cmdai, community, Discord server
**Permissions:** Open to @everyone

**Channel description:**
```
💬 Feedback welcome

Tell us what's working and what isn't:
• Product feedback
• Community suggestions
• Discord server improvements

All feedback appreciated. Be constructive. 🙏
```

#### #meta
**Purpose:** Discussion about the community itself
**Permissions:** Open to @everyone

**Channel description:**
```
🔄 Meta discussion

Talk about the community, processes, governance, etc.

"How should we handle X?"
"Can we add a channel for Y?"

Community-driven improvement.
```

---

### 🔧 ADMIN (Private)

#### #mod-chat
**Permissions:** @Moderator only
**Purpose:** Moderator coordination

#### #mod-support
**Permissions:** Open for reports, @Moderator can see
**Purpose:** Report rule violations

#### #mod-log
**Permissions:** @Moderator only
**Purpose:** Automated moderation log

---

## Roles

### Public Roles

#### @Core Team
- **Color:** Terminal Green (#00FF41)
- **Permissions:** Administrator
- **Description:** cmdai maintainers and core contributors
- **How to get:** Invited by existing core team

#### @Contributor
- **Color:** Cyber Cyan (#00D9FF)
- **Permissions:** Standard + ability to post in #development
- **Description:** Anyone with merged PR
- **How to get:** Automatic via GitHub integration after first merged PR

#### @Helper
- **Color:** Silver Frost (#C0C5D0)
- **Permissions:** Standard
- **Description:** Active community members who help others
- **How to get:** Nominated by mods for consistently helpful behavior

#### @Moderator
- **Color:** Warning Amber (#FFB800)
- **Permissions:** Kick, timeout, delete messages
- **Description:** Community moderators
- **How to get:** Invited by core team

### Interest Roles (Self-Assign)

#### @Release-Notify
- **Purpose:** Ping for new releases
- **Permissions:** Standard

#### @Events
- **Purpose:** Ping for community events
- **Permissions:** Standard

#### @Rust-Dev
- **Purpose:** Identify Rust developers
- **Permissions:** Standard

#### @Security-Minded
- **Purpose:** Identify security-focused users
- **Permissions:** Standard

#### @Looking-To-Contribute
- **Purpose:** Identify potential contributors
- **Permissions:** Standard

### Role Selection

Create a #roles channel with reaction roles:
```
⚡🛡️ Self-Assign Roles

React to get notifications:

🔔 @Release-Notify - New cmdai releases
📅 @Events - Community calls and meetups
🦀 @Rust-Dev - Rust development discussions
🛡️ @Security-Minded - Security and safety topics
🚀 @Looking-To-Contribute - Contribution opportunities

Remove your reaction to unassign.
```

---

## Moderation

### Moderator Guidelines

**Response Times:**
- Rule violations: Within 1 hour during active hours
- Questions: Best effort, no SLA
- Emergencies (doxxing, threats): Immediate

**Moderation Actions:**

**Warning (First offense for minor issues):**
```
Hey @user, friendly reminder about [rule].

[Specific guidance]

No action this time, but please keep our guidelines in mind. Thanks!
```

**Timeout (Second offense or moderate issue):**
```
@user, you've been timed out for [duration] for [reason].

This is following a previous warning about [issue].

You can appeal by DMing a moderator.

We want you in the community, but need everyone to follow the rules.
```

**Ban (Third offense or severe violation):**
- DM user with specific reason
- Log in #mod-log
- Note in moderation tracking sheet

**Appeal Process:**
1. User DMs moderator
2. Moderators discuss in #mod-chat
3. Decision within 48 hours
4. Final decision communicated to user

### Common Scenarios

**Scenario: Spam**
- Action: Delete message, warn user
- If repeat: Timeout 24 hours
- If persistent: Ban

**Scenario: Asking same question repeatedly**
- Action: Point to previous answer, link to #faq
- Suggest better channel if appropriate
- No punishment unless intentionally disruptive

**Scenario: Heated disagreement**
- Action: Step in early, remind about respectful discourse
- If it escalates: Move to DMs or timeout both parties
- Follow up privately to cool down

**Scenario: Posting dangerous commands**
- Action: Check context (is it for safety discussion?)
- If malicious: Delete, timeout, possibly ban
- If educational: Ask to add warning and code block

**Scenario: Self-promotion**
- Action: Assess relevance
- If relevant: Allow, maybe ask to share in #show-and-tell
- If spam: Delete, warn
- If persistent: Ban

---

## Bots & Automation

### Welcome Bot
- Greet new members in #welcome
- DM server rules on join
- Assign @everyone role

### Moderation Bot (Carl-bot or similar)
- Auto-delete spam patterns
- Timeout for excessive caps/spam
- Reaction roles for #roles channel
- Auto-mod for banned words
- Logging for moderator actions

### GitHub Integration
- Post releases to #changelog
- Post PRs to #pull-requests
- Auto-assign @Contributor role on first merged PR
- Post new issues to #bug-reports

### Activity Bot (Optional)
- Track message activity
- Highlight active helpers (for @Helper role consideration)
- Generate monthly stats

---

## Server Settings

**Verification Level:** Medium (verified email)
**Explicit Content Filter:** Scan from all members
**2FA Requirement:** For moderators
**Default Notifications:** Only @mentions
**Discovery:** Enabled (if eligible)
**Community Features:** Enable all

**Invites:**
- Permanent link: https://discord.gg/cmdai
- No temporary invites by default

**Vanity URL:** discord.gg/cmdai (if available)

---

## Launch Checklist

### Pre-Launch
- [ ] Create server
- [ ] Set up channels
- [ ] Configure roles
- [ ] Add moderators (at least 3)
- [ ] Install and configure bots
- [ ] Write welcome messages
- [ ] Test integrations
- [ ] Create #faq content
- [ ] Set up auto-moderation rules

### Launch Day
- [ ] Post announcement on Twitter, LinkedIn, GitHub
- [ ] Pin invite link in GitHub README
- [ ] Share in relevant communities
- [ ] Have moderators online for first 24 hours
- [ ] Monitor for issues
- [ ] Welcome new members personally (first 50)

### Week 1
- [ ] Daily moderator check-ins
- [ ] Respond to all messages
- [ ] Fix any channel structure issues
- [ ] Gather feedback
- [ ] Update #faq based on questions

### Month 1
- [ ] Review channel usage (archive unused channels)
- [ ] Evaluate moderator coverage
- [ ] Update rules if needed
- [ ] Host first community call
- [ ] Collect feedback survey

---

## Growth Strategy

### 0-100 Members
- Focus: Seed community with early adopters
- Tactics: Personal invites, GitHub contributors, Twitter followers
- Goal: Establish culture and norms

### 100-500 Members
- Focus: Organic growth, word of mouth
- Tactics: Quality content, helpful community, social sharing
- Goal: Self-sustaining discussions

### 500-1000 Members
- Focus: Structured programs, contributor pipeline
- Tactics: Office hours, workshops, contributor mentorship
- Goal: Community helping community (not just core team)

### 1000+ Members
- Focus: Scaling moderation, subgroups
- Tactics: Regional channels, language support, specialized topics
- Goal: Decentralized, self-organizing community

---

## Success Metrics

**Health Metrics:**
- Messages per day
- Active members (posted in last 7 days)
- Ratio of questions answered by community vs core team
- Average response time to questions
- Member retention (% still active after 30 days)

**Engagement Metrics:**
- #introductions posts
- #show-and-tell shares
- Event attendance
- Role adoption (self-assign roles)

**Community Metrics:**
- Contributors from Discord
- GitHub issues/PRs from Discord members
- Positive sentiment in feedback

**Warning Signs:**
- Declining active users
- Increased mod actions
- Unanswered questions in #help
- Drama in #general
- Moderators burning out

---

## Community Events

### Weekly: Office Hours
- Every Wednesday, 2pm PT
- 30 minutes
- Voice channel: #office-hours
- Core team answers questions live
- Casual, drop-in format

### Monthly: Community Call
- First Tuesday, 10am PT
- 30 minutes
- Voice + screen share
- Structured agenda
- Recorded and posted

### Quarterly: Hack Session
- Virtual hackathon
- 48 hours
- Prizes for best contributions
- Community judging

### Annual: Community Summit
- Virtual conference
- Talks from community
- Roadmap presentation
- Celebration

---

## Templates

### Welcome Message (DM Bot)
```
👋 Welcome to the cmdai Community!

Thanks for joining! Here's how to get started:

1️⃣ Read the rules in #rules
2️⃣ Introduce yourself in #introductions
3️⃣ Check out #getting-started

🆘 Need help? Ask in #help
💬 Want to chat? Head to #general
🛠️ Want to contribute? Check #development

We're glad you're here! ⚡🛡️

Think Fast. Stay Safe.
```

### Monthly Recap
```
📊 cmdai Community - [Month] Recap

This month:
👥 [X] new members (total: [Y])
💬 [X] messages sent
🆘 [X] questions answered in #help
🎉 [X] features shipped
🏆 [X] contributors with merged PRs

Top helpers this month:
@user1 - [X] questions answered
@user2 - [X] questions answered
@user3 - [X] questions answered

Thank you for making this community awesome! 🙏

Next month:
• Community call: [date]
• Feature focus: [topic]
• Workshop: [topic]

Keep being excellent to each other! ⚡🛡️
```

---

**Remember:** The Discord server is a community, not a support forum. Foster relationships, celebrate contributions, and make it a place people want to be.

⚡🛡️ Think Fast. Stay Safe. Be Kind.
