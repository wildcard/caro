# Discord Server Setup Guide for cmdai

> **Complete setup for building a thriving Discord community from 0 to 1,000+ members**

This document provides copy-paste ready instructions for setting up the cmdai Discord server. Follow these steps in order for optimal onboarding and community growth.

---

## Table of Contents

1. [Server Setup Checklist](#server-setup-checklist)
2. [Channel Structure](#channel-structure)
3. [Roles & Permissions](#roles--permissions)
4. [Welcome Flow & Onboarding](#welcome-flow--onboarding)
5. [Community Rules](#community-rules)
6. [Moderation Guidelines](#moderation-guidelines)
7. [Bot Configuration](#bot-configuration)
8. [GitHub Integration](#github-integration)
9. [Weekly Schedule](#weekly-schedule)
10. [Growth Milestones](#growth-milestones)

---

## Server Setup Checklist

### Phase 1: Initial Setup (Day 1)

- [ ] Create Discord server: "cmdai Community"
- [ ] Upload server icon (cmdai logo)
- [ ] Set server description: "AI-native CLI for safe shell command generation. Open source, Rust-powered, local-first."
- [ ] Enable Community features (Server Settings > Enable Community)
- [ ] Create all channels (see Channel Structure below)
- [ ] Set up roles and permissions (see Roles & Permissions below)
- [ ] Configure welcome channel with onboarding message
- [ ] Enable member screening (Rules acceptance)
- [ ] Set up Discord bots (MEE6 or Carl-bot for moderation)

### Phase 2: Pre-Launch Prep (Day 2-3)

- [ ] Write and pin welcome message in #welcome
- [ ] Create #rules channel with code of conduct
- [ ] Set up GitHub webhook in #github-activity
- [ ] Configure auto-role assignment (New Member role on join)
- [ ] Create invite link and pin in GitHub README
- [ ] Test all channels and permissions
- [ ] Invite 3-5 beta testers to validate setup

### Phase 3: Launch Day

- [ ] Announce Discord in HN "Show HN" post
- [ ] Pin Discord invite in GitHub README
- [ ] Share invite link on Twitter, Reddit
- [ ] Monitor #general for first questions
- [ ] Greet first 20 members personally

---

## Channel Structure

### Category: WELCOME

**#welcome** (Read-only for members, post-only for admins)
```
Description: New member onboarding and community introduction
Purpose: First impression, rules acceptance, role selection
Emoji: 👋
```

**#rules** (Read-only)
```
Description: Community guidelines and code of conduct
Purpose: Set expectations, outline moderation policies
Emoji: 📜
```

**#introductions**
```
Description: Introduce yourself to the community!
Purpose: Help members get to know each other
Emoji: 🎤
Slow Mode: 60 seconds
```

**#announcements** (Read-only for members)
```
Description: Official cmdai updates, releases, and events
Purpose: Keep community informed about project milestones
Emoji: 📢
```

---

### Category: COMMUNITY

**#general**
```
Description: General discussion about cmdai and AI-assisted workflows
Purpose: Main community hangout, questions, sharing wins
Emoji: 💬
```

**#help**
```
Description: Get help with cmdai installation, configuration, and usage
Purpose: User support, troubleshooting, how-to questions
Emoji: 🆘
Tags: Enable forum-style threads for organized help
```

**#show-and-tell**
```
Description: Share your cmdai workflows, automations, and cool commands!
Purpose: Community showcases, user-generated content, inspiration
Emoji: 🚀
```

**#feedback**
```
Description: Feature requests, bug reports, and product feedback
Purpose: Capture user insights, prioritize roadmap
Emoji: 💡
```

**#off-topic**
```
Description: Casual chat, memes, water cooler talk (keep it respectful!)
Purpose: Build community bonds, humanize the space
Emoji: 🎮
```

---

### Category: DEVELOPMENT

**#development**
```
Description: For contributors: architecture, PRs, technical discussions
Purpose: Technical collaboration, implementation questions
Emoji: 🛠️
```

**#pull-requests**
```
Description: Discuss open PRs, request reviews, share implementation updates
Purpose: PR visibility, collaboration on contributions
Emoji: 🔀
```

**#roadmap**
```
Description: Discuss future features, prioritization, and vision
Purpose: Community input on product direction
Emoji: 🗺️
```

**#testing**
```
Description: Test new features, report bugs, validate releases
Purpose: Beta testing, QA collaboration
Emoji: 🧪
```

**#docs**
```
Description: Documentation improvements, tutorials, guides
Purpose: Collaborate on improving documentation
Emoji: 📚
```

---

### Category: AUTOMATION & LOGS

**#github-activity** (Read-only, webhook posts)
```
Description: Automated feed of GitHub events (PRs, issues, releases)
Purpose: Keep community informed about repository activity
Emoji: 🤖
```

**#releases** (Read-only)
```
Description: Automated announcements of new cmdai versions
Purpose: Notify users of updates and changelogs
Emoji: 🎉
```

**#moderation-log** (Admin-only)
```
Description: Private channel for moderation actions and team discussion
Purpose: Track warnings, bans, policy discussions
Emoji: 🛡️
```

---

### Category: VOICE & EVENTS

**General Voice**
```
Description: Open voice channel for casual hangouts
Purpose: Informal voice chat, pair programming
```

**Office Hours** (Stage Channel preferred)
```
Description: Weekly office hours with maintainers (Fridays 2pm PT)
Purpose: Live Q&A, PR reviews, technical discussions
```

**Events & Workshops**
```
Description: Special events, workshops, hackathons
Purpose: Structured events, guest speakers, training
```

---

### Optional Channels (Add when community grows)

**#jobs** (Add at 250+ members)
```
Description: Job postings, freelance opportunities, hiring
Purpose: Connect community members with career opportunities
Emoji: 💼
Slow Mode: 300 seconds (5 minutes)
```

**#showcase-cloud** (Add when cloud product launches)
```
Description: Share your cmdai cloud team setups, dashboards, analytics
Purpose: Highlight cloud/enterprise features
Emoji: ☁️
```

**#integrations** (Add at 500+ members)
```
Description: Discuss and share cmdai integrations (Warp, VS Code, etc.)
Purpose: Expand ecosystem, share integration ideas
Emoji: 🔌
```

---

## Roles & Permissions

### Role Hierarchy (Top to Bottom)

#### 1. Founder
```
Color: Gold (#FFD700)
Permissions: Administrator
Who: Project creator
Badge: 👑
Responsibilities:
- Ultimate decision-making authority
- Server ownership and management
- Strategic direction and vision
```

#### 2. Core Team
```
Color: Purple (#9B59B6)
Permissions: Administrator
Who: Co-founders, CTO, early employees
Badge: 💎
Responsibilities:
- Technical leadership
- Major architectural decisions
- Hiring and team building
- Product roadmap ownership
```

#### 3. Maintainer
```
Color: Blue (#3498DB)
Permissions: Manage Channels, Manage Messages, Kick Members, Ban Members
Who: Trusted contributors with consistent quality
Badge: 🔧
Requirements:
- 20+ merged PRs OR
- 3+ months of consistent contributions OR
- Invited by Core Team for expertise
Responsibilities:
- Merge PRs and review code
- Triage issues and manage labels
- Moderate community discussions
- Mentor new contributors
```

#### 4. Contributor
```
Color: Green (#2ECC71)
Permissions: Default + Send Messages in #development
Who: Anyone with a merged PR
Badge: ✅
Auto-assigned: When first PR is merged (via GitHub Actions webhook)
Responsibilities:
- Contribute code, docs, or designs
- Participate in technical discussions
- Help review PRs (optional)
```

#### 5. Active Member
```
Color: Teal (#1ABC9C)
Permissions: Default + Priority in voice channels
Who: Highly engaged community members
Badge: ⭐
Requirements:
- 50+ messages in past 30 days OR
- Consistent help in #help channel OR
- Regular participation in office hours
Auto-assigned: Via MEE6 or Carl-bot based on activity
Responsibilities:
- Help newcomers in #help
- Share knowledge and workflows
- Provide product feedback
```

#### 6. Member
```
Color: Gray (default)
Permissions: Read Messages, Send Messages, Embed Links, Attach Files, Add Reactions
Who: Everyone who has accepted rules
Badge: None
Auto-assigned: After rules acceptance in member screening
```

#### 7. New Member
```
Color: Light Gray
Permissions: Read #welcome, #rules, #introductions only
Who: Just joined, hasn't accepted rules yet
Badge: 🆕
Auto-assigned: On server join
Removed: After rules acceptance
```

#### 8. Moderator (Add when community > 200 members)
```
Color: Orange (#E67E22)
Permissions: Manage Messages, Timeout Members, Kick Members
Who: Trusted community members who help moderate
Badge: 🛡️
Requirements:
- Active Member for 2+ months
- Demonstrated judgment and helpfulness
- Invited by Core Team
Responsibilities:
- Enforce community rules
- Handle conflicts and warnings
- Monitor for spam and abuse
- Report to Core Team
```

#### 9. Bot
```
Color: Red (#E74C3C)
Permissions: Custom based on bot needs
Who: MEE6, Carl-bot, GitHub webhooks
Badge: 🤖
```

---

### Permissions Matrix

| Permission | New Member | Member | Active Member | Contributor | Maintainer | Core Team | Founder |
|-----------|-----------|--------|---------------|-------------|-----------|-----------|---------|
| Read all channels | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Send messages | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Embed links | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Attach files | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Post in #development | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Create threads | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Priority speaker (voice) | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Timeout members | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Kick members | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Ban members | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Manage channels | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Manage messages | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Administrator | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |

---

### Role Progression Path

```
New Member (join)
    ↓ (accept rules)
Member (everyone)
    ↓ (merge 1 PR)
Contributor (technical)
    ↓ (20+ merged PRs OR invited)
Maintainer (leadership)
    ↓ (hired full-time)
Core Team (employees)
```

OR

```
New Member (join)
    ↓ (accept rules)
Member (everyone)
    ↓ (50+ messages/month OR help others)
Active Member (community)
    ↓ (demonstrate judgment + invited)
Moderator (community leadership)
```

---

## Welcome Flow & Onboarding

### Member Screening Setup

**Enable Community > Member Screening > Edit**

**Rules Prompt:**
```
Welcome to cmdai Community! 👋

Before you enter, please agree to our community rules:

1. Be respectful and inclusive
2. Stay on topic in technical channels
3. No spam, self-promotion, or solicitation
4. Help newcomers and share knowledge
5. Follow the Code of Conduct: https://github.com/wildcard/cmdai/blob/main/CODE_OF_CONDUCT.md

By clicking "Submit" below, you agree to follow these rules.
```

**Verification Level:** Medium (Verified email required)

---

### Welcome Message (#welcome)

**Pin this message in #welcome:**

```markdown
# Welcome to cmdai Community! 👋

We're building the future of AI-native command-line operations. Whether you're here to use cmdai, contribute code, report bugs, or just explore, we're excited to have you!

## 🚀 What is cmdai?

cmdai is an open-source Rust CLI that converts natural language into safe shell commands using local LLMs.

Example:
\`\`\`
$ cmdai "find all PDFs larger than 10MB"
Generated: find . -name "*.pdf" -size +10M
Execute? (y/N)
\`\`\`

- ✅ Works offline (local AI via MLX/Ollama, no API keys)
- ✅ Safety-first (blocks dangerous commands like `rm -rf /`)
- ✅ Open source (MIT/Apache 2.0)
- ✅ Blazingly fast (<100ms startup, <2s inference on Apple Silicon)

GitHub: https://github.com/wildcard/cmdai

---

## 📍 Start Here

**New to cmdai?**
→ Go to <#help> to ask questions
→ Check out our README: https://github.com/wildcard/cmdai#readme

**Want to contribute?**
→ Read CONTRIBUTING.md: https://github.com/wildcard/cmdai/blob/main/CONTRIBUTING.md
→ Find `good-first-issue` tasks: https://github.com/wildcard/cmdai/issues?q=is%3Aissue+is%3Aopen+label%3Agood-first-issue
→ Join <#development> for technical discussions

**Want to stay updated?**
→ Follow <#announcements> for releases and events
→ Join office hours every Friday 2pm PT in <#office-hours>

**Just want to hang out?**
→ Introduce yourself in <#introductions>
→ Share cool workflows in <#show-and-tell>
→ Chat about anything in <#off-topic>

---

## 🎯 Quick Links

- **GitHub:** https://github.com/wildcard/cmdai
- **Roadmap:** https://github.com/wildcard/cmdai/blob/main/ROADMAP.md
- **Docs:** https://github.com/wildcard/cmdai/blob/main/CONTRIBUTING.md
- **Code of Conduct:** https://github.com/wildcard/cmdai/blob/main/CODE_OF_CONDUCT.md
- **Security Policy:** https://github.com/wildcard/cmdai/blob/main/SECURITY.md

---

## 🤝 How to Get Roles

**Contributor** ✅ - Merge your first PR (auto-assigned via GitHub)
**Active Member** ⭐ - Be helpful and engaged (50+ messages/month or help in #help)
**Maintainer** 🔧 - Consistent high-quality contributions (20+ merged PRs or invited by Core Team)

---

## 💬 Join the Conversation

Say hi in <#introductions> and let us know:
- Where you're from
- What you're working on
- Why cmdai excites you
- What you'd like to contribute (code, docs, feedback, ideas)

**Welcome to the community!** 🎉
```

---

### Auto-Reply in #introductions

**Set up MEE6 or Carl-bot to auto-react with 👋 on every new message in #introductions**

**Optional: Auto-reply template (first message only):**
```
Thanks for introducing yourself, {user}! 🎉

Here are some great next steps:
→ Check out our `good-first-issue` tasks: https://github.com/wildcard/cmdai/labels/good-first-issue
→ Join office hours on Friday to chat with maintainers
→ Ask questions anytime in <#help>

We're excited to have you here!
```

---

## Community Rules

### Rules Channel Content (#rules)

**Pin this in #rules:**

```markdown
# cmdai Community Rules 📜

Our community is built on respect, collaboration, and a shared passion for building the future of AI-native CLI tools. These rules ensure everyone has a positive experience.

---

## Core Principles

1. **Be Respectful and Inclusive**
   - Treat everyone with kindness and empathy
   - Welcome people of all backgrounds, experience levels, and identities
   - Assume good intentions; give others the benefit of the doubt
   - Disagree constructively; attack ideas, not people

2. **Stay On Topic**
   - Keep technical discussions in technical channels (#development, #pull-requests)
   - Keep general chat in #general, casual stuff in #off-topic
   - Use threads to keep conversations organized
   - Move long discussions to appropriate channels if needed

3. **No Spam or Self-Promotion**
   - Don't spam links, repeated messages, or mass DMs
   - No unsolicited promotion of products, services, or personal projects
   - Exceptions: Relevant tools/libraries that integrate with cmdai (ask first in #general)
   - Job postings: Use #jobs (when available) with moderator approval

4. **Help Newcomers**
   - Remember you were new once too
   - Answer questions patiently, even if they seem basic
   - Point people to documentation rather than just answering
   - Celebrate first contributions and small wins

5. **Follow the Code of Conduct**
   - Our full Code of Conduct applies: https://github.com/wildcard/cmdai/blob/main/CODE_OF_CONDUCT.md
   - Report violations to maintainers via DM or in #moderation-log (if you have access)
   - Serious violations: email conduct@cmdai.dev (if established)

---

## Channel-Specific Rules

### #help
- Search before asking (use Discord search + GitHub issues)
- Provide context (OS, cmdai version, error messages)
- Mark your question as solved with ✅ reaction
- Help others when you can

### #development
- Only for cmdai contributors and technical discussions
- Link to relevant issues, PRs, or specs when discussing implementation
- Code snippets should use markdown formatting
- Keep discussions constructive and solution-focused

### #show-and-tell
- Share your own work (not others' without credit)
- Include screenshots, videos, or code examples
- Explain what makes your workflow interesting
- React with ❤️ to support community creations

### #off-topic
- Keep it light and respectful
- No politics, religion, or controversial topics that derail conversation
- Memes are welcome (within reason)
- Still follow core community principles

---

## What's Not Allowed

❌ **Harassment or Abuse**
- Personal attacks, bullying, or targeted harassment
- Discriminatory language or behavior
- Sexual or violent content
- Doxxing or sharing private information

❌ **Spam and Scams**
- Mass mentions (@everyone, @here without permission)
- Repeated messages or link spam
- Phishing, malware, or scam links
- Cryptocurrency/NFT promotion

❌ **Inappropriate Content**
- NSFW content of any kind
- Pirated software or illegal content
- Excessive profanity or offensive language
- Content that violates Discord ToS

❌ **Disruption**
- Trolling or intentionally derailing conversations
- Raiding or brigading
- Impersonating maintainers, staff, or bots
- Evading bans with alternate accounts

---

## Enforcement

### Warnings
- First offense: Friendly reminder from moderator
- Second offense: Official warning (logged)
- Third offense: Temporary timeout (1-7 days)

### Immediate Bans
- Harassment, hate speech, or threats
- Spamming or scams
- NSFW content or illegal activity
- Ban evasion

### Appeals
- DM a Core Team member to appeal a ban
- Include context and acknowledge what went wrong
- Appeals are reviewed within 48 hours

---

## Questions?

If you're unsure whether something is allowed, ask in #general or DM a maintainer. We're here to help!

**Let's build an amazing community together.** 🚀
```

---

## Moderation Guidelines

### For Maintainers and Moderators

#### Moderation Philosophy

**Assume Good Intent First**
- Most violations are accidental or from misunderstanding
- Educate before punishing
- Be friendly but firm

**Progressive Enforcement**
- Escalate slowly unless it's a severe violation
- Document warnings in #moderation-log
- Give people chances to improve

**Transparency**
- Explain why moderation actions were taken
- Be consistent in enforcement
- Admit mistakes if you over-react

---

#### Common Scenarios & Responses

##### Scenario 1: Off-Topic in #development

**Violation:** User posts general question in #development instead of #help

**Response:**
```
Hey @user! This is a better fit for <#help> since it's about usage rather than contribution.
Could you repost there? Thanks! 😊
```

**Action:** Move message to #help (right-click > Apps > Move to #help if bot enabled)

---

##### Scenario 2: Self-Promotion

**Violation:** User posts "Check out my new CLI tool!" in #general

**Moderate Response:**
```
Hey @user! We keep self-promotion minimal here to avoid spam. If your tool integrates with cmdai or solves a related problem, you can mention it briefly with context. Otherwise, please hold off on promotion. Thanks for understanding!
```

**Severe Response (repeated):**
```
@user, this is your second warning about self-promotion. Per our rules, we don't allow unsolicited promotion. Next violation will result in a timeout. Please review <#rules>.
```

**Action:** Delete message, log warning in #moderation-log

---

##### Scenario 3: Disrespectful Behavior

**Violation:** User says "That's a stupid idea" or "You clearly don't know Rust"

**Response:**
```
Hey @user, let's keep feedback constructive. Instead of "stupid idea," try "I see challenges with this approach because..." or "Have you considered X instead?" We're here to learn together.
```

**Action:** Verbal warning, log in #moderation-log

---

##### Scenario 4: Spam

**Violation:** User posts the same message 3+ times or mass mentions

**Response:**
```
@user, please don't spam. I've removed duplicate messages. If you need urgent help, just post once in <#help> and be patient. Thanks!
```

**Action:** Delete spam messages, timeout for 10 minutes if extreme

---

##### Scenario 5: Harassment or Hate Speech

**Violation:** Personal attacks, slurs, threats, discriminatory language

**Response:**
```
@user has been banned for violating our Code of Conduct. We have zero tolerance for harassment. If you have concerns, DM a Core Team member.
```

**Action:** Immediate ban, document in #moderation-log, notify Core Team

---

#### Escalation Procedure

**Level 1: Friendly Reminder**
- Public or private message
- Explain the rule and why it exists
- No formal record

**Level 2: Official Warning**
- Public message with clear rule reference
- Log in #moderation-log with date, user, violation, response
- User gets a "⚠️ Warned" note in their Discord profile (manual or bot)

**Level 3: Temporary Timeout**
- 10 minutes to 7 days depending on severity
- DM user explaining timeout reason and duration
- Log in #moderation-log
- Remove timeout early if user acknowledges and apologizes

**Level 4: Permanent Ban**
- Severe violations (harassment, scams, hate speech)
- 3+ warnings for repeated minor violations
- DM user with ban reason (if possible)
- Log in #moderation-log with full context
- Core Team vote required for appeal

---

#### Moderation Log Template

**Post in #moderation-log after any formal action:**

```
**Moderation Action**

User: @username (user#1234)
Date: 2025-11-19
Violation: [Brief description]
Rule: [Which rule was violated]
Action: [Warning / Timeout (duration) / Ban]
Moderator: @moderator_name
Context: [Link to message or brief explanation]
Notes: [Any additional context]
```

---

#### Handling Conflicts Between Members

**Step 1: Assess**
- Is it a disagreement or harassment?
- Disagreement → Let them work it out (monitor)
- Harassment → Intervene immediately

**Step 2: Intervene**
```
Hey @user1 and @user2, let's keep this respectful. If you disagree, explain why constructively. If this continues, please take it to DMs or step away. Thanks!
```

**Step 3: Separate if Needed**
- Timeout both parties for 10 minutes to cool down
- DM each person separately to mediate
- Explain expectations for re-engagement

**Step 4: Follow Up**
- Check in with both parties via DM
- Ensure they understand expectations
- Document in #moderation-log

---

## Bot Configuration

### Recommended Bots

#### 1. **MEE6** (Moderation & Leveling)
**Purpose:** Auto-moderation, role assignment, welcome messages, leveling system

**Setup:**
1. Add MEE6: https://mee6.xyz/
2. Enable Moderation plugin
   - Auto-delete spam
   - Banned words filter (configure conservatively)
   - Auto-timeout on 3 warnings
3. Enable Leveling plugin
   - Award XP for messages (1 XP per message, 60 second cooldown)
   - Role rewards:
     - Level 5 (50 messages) → Active Member
     - Level 10 (200 messages) → Special color role
4. Enable Welcome plugin
   - Send welcome message in #welcome
   - Auto-assign "New Member" role on join
   - Remove "New Member" role on rules acceptance

**Commands:**
- `!rank` - Check your level
- `!leaderboard` - Top contributors
- `!warn @user [reason]` - Warn a user
- `!timeout @user [duration]` - Timeout a user

---

#### 2. **GitHub Webhook Bot** (GitHub Integration)
**Purpose:** Post GitHub activity to #github-activity

**Setup:**
1. Go to GitHub repo settings → Webhooks → Add webhook
2. Payload URL: `https://discord.com/api/webhooks/[webhook-id]/[webhook-token]`
   - Get this from Discord: #github-activity → Edit Channel → Integrations → Webhooks
3. Content type: `application/json`
4. Events to send:
   - ✅ Pull requests
   - ✅ Issues
   - ✅ Issue comments
   - ✅ Pull request reviews
   - ✅ Releases
   - ❌ Push events (too noisy)
5. Active: ✅

**Format:**
```
🔔 New Pull Request in wildcard/cmdai
#42: Add fork bomb detection to safety validator
by @username
https://github.com/wildcard/cmdai/pull/42
```

---

#### 3. **Carl-bot** (Advanced Automation)
**Purpose:** Reaction roles, auto-moderation, custom commands

**Setup:**
1. Add Carl-bot: https://carl.gg/
2. Set up reaction roles in #welcome:
   - React with 🦀 to get notifications for Rust discussions
   - React with 🤖 to get notifications for AI/ML discussions
   - React with 📢 to get notifications for releases and events
3. Auto-responder for common questions:
   - Trigger: "how to install"
   - Response: "Installation guide: https://github.com/wildcard/cmdai#installation"
4. Auto-thread creation in #help:
   - All messages in #help auto-create threads for organization

**Commands:**
- `!help` - List all commands
- `!rr` - Reaction role setup
- `!autoresponder` - Set up auto-replies

---

#### 4. **ProBot** (Optional: Advanced Moderation)
**Purpose:** Advanced spam protection, anti-raid, auto-moderation

**Setup (only if spam becomes an issue):**
1. Add ProBot: https://probot.io/
2. Enable anti-spam:
   - Auto-delete duplicate messages
   - Auto-timeout on mass mentions
3. Enable anti-raid:
   - Lock server if 10+ joins in 10 seconds
   - Require phone verification for new accounts

---

### Custom Bot (Future: Post-Seed Funding)

Build a custom cmdai bot for:
- **Contributor stats:** `!stats @user` → Show GitHub contributions
- **Issue lookup:** `!issue 42` → Fetch and display GitHub issue details
- **Command testing:** `!cmdai "find large files"` → Test cmdai inference in Discord
- **Onboarding automation:** DM new contributors with setup guide

**Tech Stack:** Rust + serenity (Discord API library)

---

## GitHub Integration

### Webhook Events

**Set up in GitHub repo settings → Webhooks**

**Webhook URL:** Discord webhook URL from #github-activity channel

**Events to post:**

✅ **Pull Requests**
- Opened
- Merged
- Closed

✅ **Issues**
- Opened
- Closed
- Labeled (if label is `good-first-issue` or `help-wanted`)

✅ **Releases**
- Published

✅ **Pull Request Reviews**
- Approved
- Requested changes

❌ **Don't post:**
- Push events (too noisy)
- Comments (too noisy)
- Stars (not valuable in Discord)

---

### Auto-Role Assignment for Contributors

**Use GitHub Actions to assign "Contributor" role when PR is merged:**

**Create `.github/workflows/discord-contributor-role.yml`:**

```yaml
name: Discord Contributor Role

on:
  pull_request:
    types: [closed]

jobs:
  assign-role:
    if: github.event.pull_request.merged == true
    runs-on: ubuntu-latest
    steps:
      - name: Assign Contributor Role
        env:
          DISCORD_WEBHOOK: ${{ secrets.DISCORD_CONTRIBUTOR_WEBHOOK }}
          GITHUB_USER: ${{ github.event.pull_request.user.login }}
        run: |
          curl -X POST "$DISCORD_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d "{\"content\": \"🎉 Congrats @$GITHUB_USER on your first merged PR! You've earned the **Contributor** role. Welcome to the team!\"}"
```

**Setup:**
1. Create Discord webhook in #announcements
2. Add webhook URL as GitHub secret: `DISCORD_CONTRIBUTOR_WEBHOOK`
3. Manually assign "Contributor" role to the user in Discord (or use Discord API)

---

### PR Discussion Flow

**Encourage contributors to discuss PRs in #pull-requests:**

**Template message when opening PR:**
```
📢 New PR opened! #42: Add fork bomb detection

I'd love feedback on my implementation approach before final review.
See PR: https://github.com/wildcard/cmdai/pull/42

Questions:
1. Is regex the right approach for fork bomb detection?
2. Should we make the pattern list user-configurable?

cc @maintainers
```

---

## Weekly Schedule

### Monday: Community Check-In
**#announcements - 9am PT**

Template:
```
☀️ Good morning cmdai community!

This week's focus: [Feature/milestone]

Last week's wins:
- 🎉 [Achievement 1]
- 🚀 [Achievement 2]
- ❤️ [Community highlight]

This week's goals:
- 🎯 [Goal 1]
- 🎯 [Goal 2]

Open tasks: https://github.com/wildcard/cmdai/labels/good-first-issue

Let's build! 🔨
```

---

### Wednesday: Mid-Week Sync
**#development - 2pm PT**

Template:
```
🛠️ Mid-week dev sync!

Active PRs needing review:
- #42 - Fork bomb detection (@username)
- #43 - MLX backend improvements (@username)

Blockers? Questions? Drop them here 👇
```

---

### Friday: Office Hours
**Office Hours Voice Channel - 2pm PT (1 hour)**

**Format:**
1. **First 10 minutes:** Maintainer shares updates, roadmap progress
2. **Next 30 minutes:** Open Q&A, technical discussions
3. **Final 20 minutes:** Pair programming on a `good-first-issue` with a new contributor

**Announcement (Friday 12pm PT):**
```
📢 Office Hours in 2 hours!

Today's agenda:
- Review of v1.2 release progress
- Q&A on MLX backend architecture
- Live coding: Implementing new safety pattern

Join us in <Office Hours> at 2pm PT!

Can't make it? We'll post notes afterward.
```

**Post-Event Summary (Friday 4pm PT):**
```
📝 Office Hours Notes (2025-11-19)

Attendees: 12
Topics covered:
- MLX backend status (80% complete)
- Safety pattern expansion roadmap
- First-time contributor walkthrough

Recording: [YouTube link]
Next week: TBD

Thanks for joining! 🎉
```

---

### Monthly: Contributor of the Month
**#announcements - First Monday of Month**

Template:
```
🏆 Contributor of the Month: October 2025

This month's winner: @username!

Contributions:
- 8 merged PRs
- 15 code reviews
- Helped 20+ people in #help
- Created comprehensive MLX documentation

Thank you for making cmdai better! 🙏

Your prize:
- Special "Contributor of the Month" role (1 month)
- cmdai swag pack (when available)
- Featured in next release notes

Want to be next month's winner? Keep contributing! 🚀
```

---

## Growth Milestones

### Milestone 1: First 100 Members (Launch → Month 1)

**Goals:**
- [ ] 100 Discord members
- [ ] 10+ active daily users
- [ ] 5+ contributors who merged PRs
- [ ] 2-3 office hours held with 5+ attendees each

**Tactics:**
- Post Discord invite in HN launch, Reddit posts, Twitter
- Pin invite in GitHub README
- Personal welcome for first 50 members
- Host weekly office hours consistently
- Celebrate first contributions publicly

**Success Metrics:**
- 20% of members introduced themselves in #introductions
- 5+ messages/day in #general (organic, not just announcements)
- 3+ questions answered in #help per week

---

### Milestone 2: 500 Members (Month 1 → Month 3)

**Goals:**
- [ ] 500 Discord members
- [ ] 30+ active daily users
- [ ] 20+ contributors with merged PRs
- [ ] 5+ Maintainers promoted
- [ ] Self-sustaining #help channel (community answers > maintainer answers)

**Tactics:**
- Add #jobs channel for recruitment
- Start weekly Dev Digest newsletter (cross-post to Discord)
- Host monthly "Contributor Showcase" event
- Invite guest speakers (Rust experts, AI researchers)
- Run first hackathon or bounty program

**Success Metrics:**
- 50+ messages/day in #general
- 80% of #help questions answered within 24 hours
- 10+ PRs opened per week from community

**When to Add:**
- Moderator roles (2-3 trusted community members)
- #jobs channel
- #integrations channel
- Regional voice channels (EU, APAC hours)

---

### Milestone 3: 1,000 Members (Month 3 → Month 6)

**Goals:**
- [ ] 1,000+ Discord members
- [ ] 100+ active daily users
- [ ] 50+ contributors with merged PRs
- [ ] Regional community leaders (EU, APAC)
- [ ] Self-organizing community events

**Tactics:**
- Add regional channels (#general-eu, #general-apac)
- Host 24-hour global hackathon
- Launch cmdai swag store (stickers, shirts)
- Create "cmdai Champions" program (community advocates)
- Partner with conferences for Discord promotions

**Success Metrics:**
- 200+ messages/day across all channels
- 90% of #help questions answered by community (not maintainers)
- 5+ community-led events per month
- 50% of contributors discovered via Discord (not GitHub)

**When to Add:**
- Regional moderators
- #showcase-cloud (for cloud product users)
- #partnerships channel
- Dedicated voice channels per topic

---

### Beyond 1,000: Scaling Strategies

**Community Segments:**
- **Users:** Casual users who ask questions → #help, #general
- **Contributors:** Active developers → #development, #pull-requests
- **Customers:** Paying cloud/enterprise users → #showcase-cloud, priority support
- **Advocates:** Community leaders → Special role, exclusive events

**Decentralization:**
- Promote 10+ Moderators to handle global timezones
- Create Working Groups for major features (WG-MLX, WG-Safety, WG-Cloud)
- Each WG has dedicated channel and weekly sync

**Advanced Automation:**
- Custom cmdai bot for stats, testing, onboarding
- Auto-thread creation in all channels
- Sentiment analysis to detect unhappy users early
- AI-powered FAQ bot (RAG over docs)

**Premium Features (Optional):**
- Paid "cmdai Pro" Discord tier for priority support
- Exclusive channels for enterprise customers
- Private office hours for Pro members

---

## Launch Checklist (Day 0)

### Pre-Launch (Day -1)

- [ ] All channels created and organized
- [ ] Roles configured with correct permissions
- [ ] Welcome message written and pinned in #welcome
- [ ] Rules written and pinned in #rules
- [ ] MEE6 or Carl-bot configured and tested
- [ ] GitHub webhook tested (create a test issue to verify)
- [ ] Invite link created: https://discord.gg/cmdai (custom alias)
- [ ] Invite link pinned in GitHub README
- [ ] 3-5 friends invited to test and provide feedback

### Launch Day (Day 0)

- [ ] Post HN "Show HN" with Discord invite
- [ ] Post Reddit (r/rust, r/commandline) with Discord invite
- [ ] Tweet about Discord launch
- [ ] Monitor #general for first questions
- [ ] Greet first 20 members personally with:
  ```
  Welcome @user! 👋 Excited to have you here. Let us know in #introductions what brought you to cmdai!
  ```
- [ ] Answer all questions in #help within 30 minutes
- [ ] Post first #announcements message about roadmap

### Post-Launch (Day 1-7)

- [ ] Daily check-ins in #general (morning + evening)
- [ ] Answer all #help questions same-day
- [ ] Highlight community contributions in #announcements
- [ ] Schedule and announce first office hours (Friday)
- [ ] Celebrate first 50 members milestone
- [ ] Collect feedback: "What do you want to see in this Discord?"

---

## Maintenance Checklist (Ongoing)

### Daily
- [ ] Check #help for unanswered questions
- [ ] Greet new members in #introductions
- [ ] Monitor #moderation-log for any issues
- [ ] React to community content with emojis (shows engagement)

### Weekly
- [ ] Post Monday community check-in
- [ ] Post Wednesday dev sync
- [ ] Host Friday office hours
- [ ] Review and update #announcements with week's progress

### Monthly
- [ ] Announce Contributor of the Month
- [ ] Review and update server rules if needed
- [ ] Analyze growth metrics (members, messages, engagement)
- [ ] Plan next month's events and focuses

### Quarterly
- [ ] Promote deserving members to Maintainer or Moderator
- [ ] Review channel structure (add/remove channels)
- [ ] Survey community for feedback
- [ ] Update role progression requirements

---

## Metrics to Track

### Community Health

**Weekly:**
- New members (growth rate)
- Active members (posted in last 7 days)
- Messages per day (by channel)
- #help response time (average)

**Monthly:**
- Member retention (% still active after 30 days)
- Contributor conversion (% of members who submitted a PR)
- Sentiment (positive vs. negative discussions)
- Office hours attendance

**Tools:**
- Discord Analytics (Server Settings > Analytics)
- MEE6 leaderboard
- Custom tracking spreadsheet

---

## Success Criteria

**After 1 Month:**
- ✅ 100+ members
- ✅ 10+ daily active members
- ✅ 5+ contributors from Discord
- ✅ 80% of #help questions answered within 24 hours
- ✅ 4/4 office hours held with 5+ attendees each

**After 3 Months:**
- ✅ 500+ members
- ✅ 30+ daily active members
- ✅ 20+ contributors from Discord
- ✅ Community answers 60% of #help questions (not maintainers)
- ✅ 1+ community-led event (meetup, study group, hackathon)

**After 6 Months:**
- ✅ 1,000+ members
- ✅ 100+ daily active members
- ✅ 50+ contributors from Discord
- ✅ Community answers 90% of #help questions
- ✅ 3+ Moderators actively moderating
- ✅ Self-sustaining community (less maintainer intervention needed)

---

## Common Mistakes to Avoid

### 1. Too Many Channels Too Soon
**Wrong:** Create 30 channels on Day 1
**Right:** Start with 8-10 essential channels, add more as community grows

### 2. No Active Moderation
**Wrong:** Let spam and off-topic run wild
**Right:** Actively moderate, enforce rules consistently, be visible

### 3. Maintainer-Only Conversations
**Wrong:** Only maintainers talk, members are silent observers
**Right:** Ask questions, encourage discussion, highlight community contributions

### 4. Ignoring New Members
**Wrong:** No welcome, no guidance, members feel lost
**Right:** Personal greetings, clear onboarding, answer questions promptly

### 5. Over-Automation
**Wrong:** Bots answer everything, feels impersonal
**Right:** Use bots for routine tasks, but maintainers stay human and engaged

---

## Resources & Inspiration

### Example Discord Servers to Study

**Supabase Discord:**
- Excellent onboarding with role selection
- Active #help channel with community-driven support
- Clear separation of user vs. contributor channels

**PostHog Discord:**
- Great use of threads for organized discussions
- Weekly "Ask Me Anything" events
- Transparent roadmap discussions

**Rust Discord:**
- Massive scale (100k+ members) but still helpful
- Strong moderation culture
- Beginner-friendly #help channels

**Tauri Discord:**
- Effective GitHub integration
- Active showcase channel
- Regional community support

---

## Questions & Support

**Need help setting up Discord?**
- DM the maintainers with questions
- Refer to Discord's official guides: https://support.discord.com/hc/en-us/articles/360047132851-Enabling-Your-Community-Server

**Want to suggest improvements to this guide?**
- Open a PR to update this document
- Discuss in #feedback channel once Discord is live

---

**Let's build an amazing community together.** 🚀

This Discord server is the heart of the cmdai community. With thoughtful structure, active moderation, and genuine engagement, we'll create a space where users, contributors, and team members thrive.

**Ready to launch? Follow the checklist above and let's go!**

---

*Last updated: 2025-11-19*
*Maintained by: cmdai Core Team*
*Feedback: Open a PR or post in #feedback*
