# Welcome Message Templates

Ready-to-use welcome messages for different community touchpoints. Warm, helpful, on-brand.

---

## New GitHub Contributor (First PR Merged)

### Message
```
🎉 Congratulations on your first merged PR, @username!

You're now officially a cmdai contributor. Thank you for making the project better!

A few things to know:

🏆 **You're now listed in CONTRIBUTORS.md**
Check out your entry: [link to file]

🎁 **Claim your swag**
Fill out this form to get contributor stickers shipped to you: [link]

💬 **Join the community**
- Discord: [link]
- GitHub Discussions: [link]

📚 **What's next?**
- Check out issues labeled `good-next-issue` for your next contribution
- Help review other PRs
- Join our monthly community call

We're lucky to have you! Keep building with us.

⚡🛡️ Think Fast. Stay Safe. Contribute.

— The cmdai team
```

### When to Send
- Automatically via GitHub Action when PR is merged
- Manual comment from maintainer on the PR

---

## New Discord Member (DM)

### Message
```
👋 Welcome to the cmdai Community, @username!

I'm the welcome bot. Let me help you get started:

**1. Read the Rules** 📜
Check out #rules to understand our community guidelines.
TL;DR: Be kind, be helpful, have fun!

**2. Introduce Yourself** 🙋
Head to #introductions and tell us:
• Who you are
• What you do
• How you found cmdai
• What you're excited about

**3. Get Help** 🆘
Have questions? Ask in #help
- Include cmdai version: `cmdai --version`
- Share what you tried and what happened
- Our community loves helping!

**4. Explore**
- #general - Chat about cmdai and related topics
- #show-and-tell - Share what you've built
- #safety-patterns - Discuss dangerous commands
- #off-topic - Anything goes (within rules)

**5. Contribute** 🛠️
Interested in contributing to cmdai?
- #development - Technical discussions
- GitHub: https://github.com/wildcard/cmdai

**Quick Links:**
📖 Docs: https://cmdai.dev/docs
💻 GitHub: https://github.com/wildcard/cmdai
🌐 Website: https://cmdai.dev

**Need help with Discord?**
Just ask in #meta or DM a moderator (gold names).

We're glad you're here! ⚡🛡️

Think Fast. Stay Safe.
```

### When to Send
- Automatically when user joins Discord
- Via welcome bot

---

## New Email Subscriber

### Subject
```
Welcome to cmdai! Here's how to get started ⚡🛡️
```

### Body
```
Hi there!

Thanks for subscribing to the cmdai newsletter. You're now part of a community building safer, faster terminal automation.

**What to expect:**
📬 One email per week (Mondays)
📰 Project updates, new features, community highlights
🎓 Tutorials and tips
🙏 No spam, ever

**Get Started:**

1. **Install cmdai**
   ```bash
   # macOS
   brew install cmdai

   # Cargo
   cargo install cmdai

   # Download binary
   https://github.com/wildcard/cmdai/releases
   ```

2. **Try your first command**
   ```bash
   cmdai "list all log files modified in the last 7 days"
   ```

3. **Join the community**
   - Discord: [link]
   - GitHub Discussions: [link]
   - Twitter: @cmdai

**Resources:**
📖 Documentation: https://cmdai.dev/docs
💡 Examples: https://cmdai.dev/examples
🛡️ Safety Guide: https://cmdai.dev/safety

**Questions?**
Reply to this email - we read every response!

Looking forward to seeing what you build!

⚡🛡️ Think Fast. Stay Safe.

— The cmdai team

P.S. Want to contribute? Check out our CONTRIBUTING.md guide: [link]

---
Unsubscribe | Update preferences | Forward to a friend
```

### When to Send
- Immediately upon email subscription confirmation
- Welcome sequence: Day 1 (this), Day 3 (safety guide), Day 7 (contribution guide)

---

## New GitHub Discussion Participant

### Message
```
👋 Welcome to cmdai Discussions, @username!

Thanks for posting your first question/idea!

**A few tips to get the most out of Discussions:**

**For Questions:**
✅ Include cmdai version: `cmdai --version`
✅ Share what you tried
✅ Post error messages in code blocks
✅ Mark the best answer when your question is resolved

**For Feature Requests:**
✅ Describe the problem you're solving
✅ Explain your use case
✅ Consider safety implications
✅ Be open to alternative approaches

**For Showcases:**
✅ Share screenshots or code
✅ Explain what you built and why
✅ Inspire others!

**Response Times:**
Most questions get answered within 24 hours (community-driven, so patience appreciated!).

**Community Guidelines:**
Please read our Code of Conduct: [link]

We're glad you're here. Thanks for being part of the community!

⚡🛡️

— cmdai maintainers
```

### When to Send
- Manual comment on first Discussion post
- Could be automated via GitHub Actions

---

## New Twitter/X Follower (Auto-DM)

### Message
```
👋 Thanks for following cmdai!

We share:
• New features & releases
• Safety tips & validation examples
• Community highlights
• Terminal tricks

Want to dive deeper?
📖 Docs: cmdai.dev/docs
💻 GitHub: github.com/wildcard/cmdai
💬 Discord: [link]

Built something cool with cmdai? Tag us - we love sharing community work!

⚡🛡️ Think Fast. Stay Safe.
```

### When to Send
- Optional auto-DM for new followers
- Not recommended unless Twitter API allows (often seen as spam)
- Better: Welcome tweet when they @ mention you

---

## First-Time Issue Reporter

### Message
```
Thanks for opening your first issue, @username! 🙏

A maintainer will review this soon. In the meantime:

**For Bug Reports:**
- Make sure you're on the latest version: `cmdai --version`
- Check if there's a duplicate issue
- Add any additional context that might help

**For Feature Requests:**
- Community discussion helps refine ideas: [link to Discussions]
- Upvote with 👍 if you see others request this
- Consider contributing if you'd like to implement it!

**For Security Issues:**
If this is a security vulnerability, please:
1. Delete this issue
2. Email security@cmdai.dev
3. See SECURITY.md for our disclosure process

**Response Time:**
- Security issues: 24 hours
- Bugs: 48 hours acknowledgment, 1 week triage
- Features: 1 week triage

We appreciate you taking the time to make cmdai better!

⚡🛡️ Think Fast. Stay Safe.

— cmdai maintainers
```

### When to Send
- Automatically via GitHub Action on first issue
- Template appears when issue is opened

---

## First-Time Conference/Meetup Attendee

### Message (In-Person)
```
👋 Hi! Welcome to the cmdai [booth/talk/meetup]!

**Try cmdai live:**
1. Scan this QR code to visit our demo page
2. Or try on your laptop: github.com/wildcard/cmdai

**What cmdai does:**
Generates shell commands from plain English + validates them for safety

Example:
You: "delete log files older than 30 days"
cmdai: `find /var/log -type f -name "*.log" -mtime +30 -delete`
Safety: ⚠️ MODERATE - Review before executing

**Challenge:**
Try to get cmdai to approve a dangerous command. We've blocked:
- rm -rf /
- chmod 777 /etc
- fork bombs
- and more!

**Take With You:**
- Sticker pack ✓
- Quick start guide ✓
- QR code for GitHub ✓

**Stay Connected:**
Discord: [QR code]
Twitter: @cmdai
GitHub: github.com/wildcard/cmdai

**Want to contribute?**
Talk to us! We love new contributors.

⚡🛡️ Think Fast. Stay Safe.
```

### When to Use
- Printed handout at conferences
- Booth display
- Meetup welcome packet

---

## Welcome to Beta/Early Access

### Subject
```
You're in! Welcome to cmdai early access ⚡🛡️
```

### Body
```
Hi @username,

You're one of the first people testing cmdai. Thank you for helping us build something safer and faster!

**What's Beta:**
cmdai is functional but not feature-complete. You might encounter:
- Rough edges in UX
- Performance issues
- Bugs (please report them!)

That's why you're here - to help us find and fix these before launch.

**How to Get Started:**

1. **Install:**
   ```bash
   cargo install --git https://github.com/wildcard/cmdai
   ```

2. **Configure:**
   ```bash
   cmdai --setup
   ```

3. **Try it:**
   ```bash
   cmdai "your command description here"
   ```

**We Need Your Feedback:**

🐛 **Found a bug?**
Open an issue: https://github.com/wildcard/cmdai/issues

💡 **Have an idea?**
Start a discussion: https://github.com/wildcard/cmdai/discussions

⚠️ **Safety issue?**
Email security@cmdai.dev immediately

📊 **General feedback?**
Reply to this email or join Discord: [link]

**Beta Perks:**

✅ Early access to new features
✅ Direct line to maintainers (Discord #beta channel)
✅ Listed as beta tester in CONTRIBUTORS.md (if you want)
✅ Exclusive beta tester sticker pack (we'll ship it!)

**Timeline:**

- Beta phase: 4-6 weeks
- Feature freeze: [date]
- Public launch: [date]

**Privacy:**

Beta telemetry is disabled by default. Your commands stay on your machine.

If you opt-in to anonymous usage stats (helps us improve), you can toggle with:
```bash
cmdai --telemetry [on|off]
```

**Questions?**

Email beta@cmdai.dev or ask in the Discord #beta channel.

Thanks for being here early. Your feedback will shape cmdai for thousands of future users.

⚡🛡️ Think Fast. Stay Safe. Build Better.

— The cmdai team

P.S. Know someone who'd be a great beta tester? Send them to: [link]
```

### When to Send
- When user is accepted to beta program
- Include instructions and special access

---

## First Swag Claim

### Subject
```
Your cmdai swag is on the way! 📦
```

### Body
```
Hi @username!

Your cmdai contributor swag pack has been shipped! 🎉

**What's in it:**
- cmdai sticker pack (5 designs)
- "Guard Rails for the Fast Lane" laptop sticker
- "Think Fast. Stay Safe" vinyl sticker
- Contributor thank-you card (handwritten!)

**Shipping:**
- Tracking: [number]
- Carrier: [carrier]
- Est. arrival: [date]

**Share Your Swag:**

When it arrives, we'd love to see photos!

Tag us:
- Twitter: @cmdai #cmdaiSwag
- Discord: #show-and-tell

**Keep Contributing:**

You earned this swag by contributing to cmdai. We hope you'll keep building with us!

Next PR: [Link to good-next-issue]
Community call: [Next date/time]
Discord: [Link]

**Questions?**

Reply to this email or DM us on Twitter.

Thanks for being part of the cmdai community! ⚡🛡️

— The cmdai team

P.S. Swag didn't arrive or something's wrong? Let us know and we'll make it right.
```

### When to Send
- After processing swag claim form
- Includes tracking information

---

## New Sponsor/Backer

### Subject
```
Thank you for supporting cmdai! 🙏
```

### Body
```
Hi @username,

Wow. Thank you for sponsoring cmdai!

Your support means the world to us. It helps with:
- Development time
- Infrastructure costs
- Community events
- Contributor swag
- Documentation and tutorials

**Sponsor Perks:**

✅ Listed on README (if $50+/month)
✅ Listed on website sponsors page
✅ Sponsor badge in Discord (if you join)
✅ Early access to roadmap discussions
✅ Our eternal gratitude 🙏

**Would you like:**
- [ ] To be listed publicly as a sponsor
- [ ] Your company logo on the README
- [ ] Access to sponsor-only Discord channel
- [ ] To remain anonymous

Reply to this email with your preferences!

**Stay Connected:**

Discord: [link]
GitHub Discussions: [link]
Twitter: @cmdai

We'll keep you updated on how your support is making cmdai better.

**Questions or feedback?**

Reply anytime. We read every email from sponsors.

Thank you for believing in cmdai. ⚡🛡️

— The cmdai team

P.S. Want to get more involved? We'd love to hear your ideas for where cmdai should go next.
```

### When to Send
- Immediately upon sponsorship confirmation
- Personalized thank you from maintainer

---

## Welcome Templates Summary

**Key Principles:**

1. **Warm and personal** - Not corporate, actually welcoming
2. **Action-oriented** - Clear next steps
3. **Brand-consistent** - Use ⚡🛡️ and "Think Fast. Stay Safe"
4. **Helpful** - Links to resources, clear expectations
5. **Gratitude** - Thank people for participation
6. **Community-focused** - Point to Discord, Discussions, etc.

**Tone Guidelines:**

✅ DO:
- Use emoji (but not excessively)
- Be enthusiastic and grateful
- Give clear, actionable next steps
- Set expectations (response times, etc.)
- Include links to key resources

❌ DON'T:
- Be overly formal or corporate
- Overwhelm with information
- Use jargon without explanation
- Forget to say thank you
- Leave people wondering what to do next

---

## Customization Variables

When implementing these templates, replace:

- `@username` - Actual username
- `[link]` - Actual URLs
- `[date]` - Actual dates
- `[number]` - Actual numbers
- `[carrier]` - Shipping carrier

**Personalization:**
- Add user's actual contribution (if welcoming contributor)
- Reference their specific question/issue (if applicable)
- Mention how they found cmdai (if known)

---

## A/B Testing Welcome Messages

Track these metrics for welcome messages:

**Email:**
- Open rate (target: >40%)
- Click-through rate (target: >10%)
- Unsubscribe rate (target: <1%)
- Reply rate (shows engagement)

**Discord:**
- Introduction post rate (% who post in #introductions)
- Message rate (% who post within 7 days)
- Retention rate (% still active after 30 days)

**GitHub:**
- Second contribution rate (% who contribute again)
- Discussion participation
- Issue/PR interaction

**Test variations:**
- Length (short vs detailed)
- Tone (casual vs professional)
- Call-to-action (single vs multiple)
- Timing (immediate vs delayed)

---

**Remember:** First impression matters. Make people feel welcomed, valued, and excited to be part of the community.

⚡🛡️ Think Fast. Stay Safe. Welcome Everyone.
