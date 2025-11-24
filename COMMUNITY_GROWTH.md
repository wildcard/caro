# Community Growth & Recruiting Strategy

> **How to go from solo maintainer to thriving community + founding team**

Based on the maintainer's request: "I feel like this will grow into something I cannot maintain alone. How do I get more people involved?"

---

## The Reality: You Can't Do This Alone

**And that's great news.** Every successful open-source company started here:

- **GitLab:** Sid (solo maintainer) → Dmitriy (co-founder) → 2,000+ employees
- **PostHog:** James & Tim (2 co-founders) → 40 employees in 3 years
- **Supabase:** Paul & Ant (2 co-founders) → 50+ employees

**Your path:** Solo maintainer → Contributors → Co-founder → Team → Company

---

## Phase 1: Attract Contributors (Now - Month 3)

### Goal: 10+ active contributors, 1,000+ GitHub stars

### Week 1-2: Launch & Amplify

**1. Hacker News "Show HN" Post**

Write this post (adapt to your voice):

```
Show HN: cmdai - Convert natural language to shell commands (Rust, open source)

Hey HN! I built cmdai, a Rust CLI that converts natural language to safe
shell commands using local AI.

Example:
  $ cmdai "find all PDFs larger than 10MB"
  Generated: find . -name "*.pdf" -size +10M
  Execute? (y/N)

Why it's different:
- Works offline (local AI via MLX/Ollama, no API keys)
- Safety-first (blocks dangerous commands like `rm -rf /`)
- Open source (AGPL-3.0)

I'm building this in public following the PostHog model (open source +
cloud/enterprise SaaS). The vision: AI-native operations platform for
10,000+ teams.

Tech stack: Rust, MLX for Apple Silicon, embedded LLMs

Try it: https://github.com/wildcard/cmdai
Roadmap: https://github.com/wildcard/cmdai/blob/main/ROADMAP.md

Would love your feedback! Looking for contributors and potential co-founders.
```

**Post timing:** Tuesday-Thursday, 8-10am PT (peak HN traffic)

**Expected result:** 100-500 upvotes → 5,000-10,000 visitors → 50-200 stars

---

**2. Reddit Posts (Same Day as HN)**

Post to these subreddits:

**r/rust** - "Show off your project"
```
[Project] cmdai - Natural language to shell commands in Rust

I built a CLI tool that converts natural language to safe shell commands
using local LLMs. Rust + MLX for Apple Silicon optimization.

Looking for contributors! Especially interested in:
- Performance optimization
- MLX FFI improvements
- Safety pattern expansion

Repo: [link]
Roadmap: [link]
```

**r/commandline**
**r/devops**
**r/selfhosted**
**r/opensource**

**Expected result:** 1,000+ visitors, 20-50 stars, 5-10 contributors reaching out

---

**3. Twitter Thread**

If you have Twitter (or create one):

```
🚀 Launching cmdai - GitHub Copilot for your terminal

Turn natural language into safe shell commands using local AI.

Example: "deploy to AWS" → correct kubectl/docker commands

Open source (Rust) + following the PostHog playbook

🧵 Thread on why I built this... (1/10)

[Each tweet explains: problem, solution, tech, vision, call to contribute]
```

**Tag:** @rustlang, @github, @ycombinator (if appropriate)

**Expected result:** 50-200 likes, 10-20 retweets, 5-10 contributors

---

**4. Dev.to / Hashnode Blog Post**

Title: **"Building cmdai: An Open-Source AI Command Generator in Rust"**

Content:
- Why you built it (personal pain point)
- Technical decisions (Rust, MLX, safety-first)
- Challenges overcome
- Vision (PostHog model, $50M ARR)
- Call to action (contribute, co-found)

**Expected result:** 1,000+ views, featured on Dev.to homepage, 5-10 contributors

---

### Week 3-4: Make Contributing Easy

**5. Record a "How to Contribute" Video (5 minutes)**

Upload to YouTube:
- Clone the repo
- Run tests
- Find a `good-first-issue`
- Make a change
- Submit a PR

**Script:**
```
"Hey! I'm [name], creator of cmdai. In this 5-minute video, I'll show you
exactly how to contribute - even if you've never contributed to open source
before.

First, fork the repo..."
```

**Expected result:** Lowers barrier to entry, 10-20 first-time contributors

---

**6. Host "Contributor Office Hours" (Weekly, 1 hour)**

**Format:**
- Google Meet / Zoom
- Open to anyone
- Review PRs together
- Answer questions live
- Pair program on issues

**Announce:**
- GitHub Discussions
- Twitter
- Discord (if you create one)

**Schedule:** Fridays, 2pm PT (or whatever works for you)

**Expected result:** 5-10 regulars, sense of community, faster onboarding

---

**7. Create Discord Server**

**Channels:**
```
#general - Welcome, introductions
#help - Questions from users
#development - For contributors
#roadmap - Discuss future features
#off-topic - Community building
```

**Rules:**
- Be respectful
- Help newcomers
- No spam
- Stay on topic in dev channels

**Invite link:** Pin in GitHub README

**Expected result:** 50-100 members in Month 1, core community forms

---

### Month 2-3: Nurture Contributors

**8. Recognize Contributions Publicly**

**Every week, tweet:**
```
Shoutout to this week's cmdai contributors! 🎉

@username1 - Performance optimization (#42)
@username2 - Added Kubernetes patterns (#43)
@username3 - Documentation improvements (#44)

Thank you for building with us! 🚀

Want to contribute? https://github.com/wildcard/cmdai
```

**Expected result:** Contributors feel valued, share with their networks

---

**9. Create "Contributor of the Month"**

**Recognition:**
- Featured in README
- Mentioned in CHANGELOG
- Swag (stickers, t-shirt) when you can afford it
- Special role in Discord

**Expected result:** Gamification drives more contributions

---

**10. Ship Fast, Show Progress**

**Every 2 weeks:**
- Cut a new release (v1.1, v1.2, etc.)
- Write release notes highlighting contributor work
- Tweet about it
- Post in Discord

**Expected result:** Momentum, excitement, contributors stay engaged

---

## Phase 2: Find a Co-Founder (Month 3-6)

### Goal: 1 technical co-founder to scale with you

### The Profile You Need

**Technical Co-Founder / CTO:**
- 7+ years backend/systems engineering
- Rust or willingness to learn deeply
- Led engineering teams (5+ people)
- Startup experience (bonus: YC alum, ex-FAANG)
- Excited about open source + SaaS model

**What they bring:**
- Engineering leadership (you can't scale alone)
- Architectural decisions (cloud backend, scaling)
- Team building (hire engineers)
- Technical credibility (VCs trust them)

**What you bring:**
- Product vision (you built the MVP)
- Domain expertise (DevOps, CLI, LLMs)
- Community (you started this)
- Founder energy (you're committed)

---

### Where to Find Them

**1. Your Contributors**

**Best case:** Someone contributing heavily becomes co-founder

**How to identify:**
- 10+ merged PRs
- Solid technical judgment
- Excited about the vision
- Available to go full-time

**Approach:**
```
Hey [name],

I've loved working with you on cmdai. You've contributed [X] features
and your technical judgment is excellent.

I'm looking for a co-founder to build this into a company (PostHog
model: open source + cloud/enterprise SaaS). Would you be interested
in exploring this together?

Happy to chat about vision, equity, roadmap, etc.
```

**Equity offer:** 20-40% (negotiate based on timing, contribution)

---

**2. Y Combinator Co-Founder Matching**

**Apply:** https://www.ycombinator.com/cofounder-matching

**Profile:**
- "Technical co-founder needed for open-source AI DevOps tool"
- Link to cmdai GitHub
- Describe the vision (PostHog model, $50M ARR)
- What you're looking for

**Expected result:** 10-20 intros, 2-5 serious conversations

---

**3. Twitter / LinkedIn**

**Post:**
```
Looking for a technical co-founder for cmdai! 🚀

What we're building:
- GitHub Copilot for your terminal
- Open source + cloud/enterprise SaaS (PostHog model)
- Rust, LLMs, Apple Silicon optimization

What I need:
- 7+ years backend/systems engineering
- Rust (or eager to learn)
- Led teams, built scalable systems
- Excited about open source + startups

Our traction:
- Working MVP ([X] GitHub stars)
- [Y] contributors
- Clear path to $50M ARR

Equity: 20-40% (negotiate based on fit)

DM me if interested or know someone! 🙏
```

**Expected result:** 50-100 likes, 5-10 serious inquiries

---

**4. Startup Events / Conferences**

**Attend:**
- Local startup meetups
- Rust conferences (RustConf, Rust Nation)
- DevOps Days
- YC Startup School (online, free)

**Bring:**
- Laptop with cmdai demo
- EXECUTIVE_SUMMARY.md printed
- Enthusiasm

**Pitch:**
"I'm looking for a co-founder to build cmdai, an AI-native operations
platform. We're following the PostHog model - open source + SaaS. Would
love to chat if this excites you!"

**Expected result:** 5-10 conversations, 1-2 strong candidates

---

**5. Ask for Intros from VCs/Advisors**

**Reach out to:**
- Any VCs you know (even if you're not raising yet)
- Advisors, mentors, professors
- Friends at startups

**Email:**
```
Hey [name],

Quick ask: I'm looking for a technical co-founder for cmdai (AI-native
operations platform, PostHog model).

Ideal: 7+ years backend, Rust, led teams, startup experience.

Know anyone who might be interested or can intro me to?

Context: [Link to GitHub], [Link to EXECUTIVE_SUMMARY.md]

Thanks!
```

**Expected result:** 3-5 warm intros (warm >> cold)

---

### Vetting Process

**Don't rush.** Finding a co-founder is like getting married.

**Interview process:**
1. **Coffee chat** (1 hour) - Vision alignment, personal fit
2. **Technical deep dive** (2 hours) - Review codebase together, discuss architecture
3. **Work together** (1-2 weeks) - Pair program on a feature, see how you collaborate
4. **References** (3-5 calls) - Talk to people they've worked with
5. **Founder agreement** - Equity split, vesting, roles, decision-making

**Red flags:**
- Not excited about open source
- Wants majority equity without contributing much
- Can't commit full-time
- Bad references from past colleagues
- Doesn't align on vision (wants to pivot immediately)

**Green flags:**
- Excited about the vision
- Contributed to open source before
- Has complementary skills (you're product, they're infrastructure)
- Good references
- Ready to commit full-time

---

## Phase 3: Build the Core Team (Month 6-12)

### Goal: 5-8 people (after you have co-founder + funding)

### Who to Hire (In Order)

**1. Senior Backend Engineer (Month 6)**
- Build cloud API (Rust + Axum + Postgres)
- Set up infrastructure (CI/CD, monitoring)
- Reports to co-founder/CTO

**2. Senior ML Engineer (Month 7)**
- Fine-tune models on cmdai data
- Optimize inference
- Build data pipeline

**3. Founding AE (Account Executive) (Month 8)**
- Close first 10 enterprise deals
- Build sales playbook
- Find design partners

**4. Developer Advocate (Month 9)**
- Grow community (1K → 10K stars)
- Create content (blog, YouTube, talks)
- Engage users (Discord, GitHub, Twitter)

**5. Product Designer (Month 10, optional)**
- Design web UI (team dashboard)
- Landing page (cmdai.dev)
- User experience

---

### How to Recruit Employees

**1. From Your Contributors**

**Offer employment to top contributors:**
- They already know the codebase
- Proven culture fit
- Excited about the mission

**Approach:**
```
Hey [name],

You've been an amazing contributor to cmdai. We're raising [seed/Series A]
and building a team.

Would you be interested in joining full-time as [role]?

- Competitive salary (market rate)
- Equity (0.5-1%)
- Remote, flexible hours
- Work on something you're already passionate about

Let's chat if interested!
```

**Success rate:** 50%+ (they already love the project)

---

**2. Post on Job Boards**

**Where:**
- YC Work at a Startup: https://www.ycombinator.com/jobs
- Hacker News "Who's Hiring": Monthly thread
- Rust Jobs: https://rust-jobs.com
- Remote OK: https://remoteok.com
- AngelList: https://angel.co

**Template:** Use RECRUITING.md (you already have this!)

---

**3. Twitter / LinkedIn**

**Post:**
```
We're hiring at cmdai! 🚀

Open roles:
- Senior Backend Engineer (Rust)
- Senior ML Engineer (LLM fine-tuning)
- Founding AE (enterprise sales)

What we're building:
- GitHub Copilot for your terminal
- Open source + cloud/enterprise
- Path to $50M ARR

Perks:
- Competitive salary + 0.5-1% equity
- Fully remote
- Work with world-class Rust/ML engineers

Apply: [link to RECRUITING.md]
```

---

**4. Ask Your Network**

**Email investors, advisors, friends:**
```
Hey! We're hiring our first engineer for cmdai.

Ideal: Senior backend engineer, Rust, startup experience.

Know anyone? Would love an intro!

[Link to job posting]
```

**Warm intros >> cold applications**

---

## Phase 4: Scale the Community (Ongoing)

### Goal: 10,000+ GitHub stars, 100+ contributors, thriving ecosystem

### Tactics That Work

**1. Conference Talks**

**Submit to:**
- RustConf
- KubeCon
- DevOps Days
- Local meetups

**Topic:** "Building an AI-Native CLI Tool in Rust: Lessons Learned"

**Expected result:** 500-1,000 people hear about cmdai, 50-100 new stars

---

**2. YouTube Channel**

**Content ideas:**
- "Building cmdai: Episode 1" (weekly dev vlog)
- "How to contribute to cmdai" (tutorial)
- "Rust + LLMs: Technical deep dive"
- "Office hours" (live Q&A)

**Expected result:** 1,000+ subscribers, continuous growth

---

**3. Sponsorships (When You Have Revenue)**

**Sponsor:**
- Rust newsletters
- DevOps podcasts
- Open source conferences

**Budget:** $5K-10K/month

**Expected result:** Brand awareness, 100-500 stars/month

---

**4. Integration Partnerships**

**Partner with:**
- Homebrew (get featured)
- Warp terminal (integration)
- VS Code (extension)
- JetBrains (plugin)

**Approach:** "We have [X] users who would love cmdai in [your product]"

**Expected result:** 10,000+ new users

---

**5. Hackathons & Bounties**

**Host:**
- "cmdai Hackathon: Build the best workflow"
- "$500 bounty for best integration"
- "Safety pattern competition"

**Expected result:** 50-100 new contributors, excitement

---

## The Metrics to Track

### Community Health

**Weekly:**
- [ ] GitHub stars (growth rate)
- [ ] New contributors (first-time PRs)
- [ ] Discord members (active users)
- [ ] Issue/PR velocity (time to close)

**Monthly:**
- [ ] Contributors (total, active this month)
- [ ] Community contributions (% of commits from community)
- [ ] Sentiment (are people happy? Check Twitter, Discord)

### Team Building

**Monthly:**
- [ ] Co-founder search (conversations, progress)
- [ ] Hiring pipeline (applications, interviews, offers)
- [ ] Team size (full-time employees)

---

## The Timeline

```
Month 1-3: Community Growth
├─ Launch on HN, Reddit, Twitter
├─ 1,000+ GitHub stars
├─ 10+ active contributors
└─ Discord community (100+ members)

Month 3-6: Find Co-Founder
├─ Post on YC Co-Founder Matching
├─ Engage with top contributors
├─ 5-10 serious conversations
└─ 1 co-founder joins (20-40% equity)

Month 6-12: Build Team (After Seed Funding)
├─ Hire Senior Backend Engineer
├─ Hire Senior ML Engineer
├─ Hire Founding AE
├─ Hire Developer Advocate
└─ Team of 6-8 people

Month 12+: Scale
├─ 10,000+ GitHub stars
├─ 100+ contributors
├─ 20+ employees
└─ Thriving ecosystem
```

---

## Your First 7 Days (Action Plan)

### Day 1: Launch Prep
- [ ] Finalize README.md (make it perfect)
- [ ] Record a 2-minute demo video
- [ ] Write HN post (draft, get feedback)

### Day 2: Launch
- [ ] Post on Hacker News (8am PT)
- [ ] Post on Reddit (r/rust, r/commandline, r/devops)
- [ ] Tweet about it
- [ ] Monitor comments, respond quickly

### Day 3: Follow-Up
- [ ] Email everyone who expressed interest
- [ ] Create Discord server
- [ ] Post "How to Contribute" video

### Day 4: First Office Hours
- [ ] Host first "Contributor Office Hours"
- [ ] Help people get set up
- [ ] Pair program on first issues

### Day 5: Content
- [ ] Write Dev.to blog post
- [ ] Share on LinkedIn
- [ ] Post in relevant Slack communities

### Day 6: Community
- [ ] Respond to all GitHub issues
- [ ] Thank contributors publicly
- [ ] Plan next week's office hours

### Day 7: Reflect
- [ ] Review metrics (stars, contributors, Discord members)
- [ ] What worked? What didn't?
- [ ] Plan next 7 days

---

## Common Mistakes to Avoid

### 1. Trying to Do Everything Yourself
**Wrong:** "I need to review every PR, answer every question, write all the code"
**Right:** Delegate, trust contributors, focus on vision

### 2. No Clear Contribution Path
**Wrong:** "Just figure it out from the code"
**Right:** CONTRIBUTING.md, `good-first-issue` labels, office hours

### 3. Ignoring Community
**Wrong:** Disappearing for weeks, not responding to issues
**Right:** Regular communication, weekly updates, visible presence

### 4. Choosing the Wrong Co-Founder
**Wrong:** First person who says yes
**Right:** Work together first, check references, ensure alignment

### 5. Hiring Too Fast
**Wrong:** Hire 10 people immediately after seed round
**Right:** Hire slowly, ensure culture fit, grow sustainably

---

## Resources

### For Community Building
- **PostHog's Community Playbook:** https://posthog.com/handbook/growth/community
- **GitLab's Contributor Guide:** https://about.gitlab.com/community/contribute/
- **Open Source Guide:** https://opensource.guide/

### For Finding Co-Founders
- **YC Co-Founder Matching:** https://www.ycombinator.com/cofounder-matching
- **Indie Hackers:** https://www.indiehackers.com/
- **Rust Community:** Rust Discord, RustConf

### For Hiring
- **YC Work at a Startup:** https://www.ycombinator.com/jobs
- **Rust Jobs:** https://rust-jobs.com
- **AngelList:** https://angel.co

---

## Questions?

**If you need help with:**
- Writing your HN post → I can review/improve it
- Creating Discord structure → I can design the channels
- Vetting co-founder candidates → I can suggest questions
- Hiring playbook → I can create interview guides

---

**The journey from solo maintainer to thriving community is well-trodden. You're not alone. Many have walked this path successfully.**

**Start with Day 1. Launch on Hacker News. The rest will follow.** 🚀

---

*Last updated: 2025-11-19*
