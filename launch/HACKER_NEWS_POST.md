# Hacker News "Show HN" Launch Guide

> Complete guide for launching cmdai on Hacker News

Last updated: 2025-11-19

---

## Pre-Launch Checklist

Before posting, ensure:

- [ ] README.md is polished and clear
- [ ] All tests pass (run `cargo test`)
- [ ] GitHub Issues are organized (good-first-issue labels added)
- [ ] CONTRIBUTING.md is up to date
- [ ] You have 2-3 hours to monitor and respond to comments
- [ ] Demo video or GIF is ready (optional but helpful)
- [ ] You've slept well (you'll need energy for engagement!)

---

## Version A: Conservative (Technical Focus)

**Recommended for:** First launch, testing waters, want to avoid seeming "too salesy"

### Title

```
Show HN: cmdai - Convert natural language to shell commands (Rust, local AI)
```

**Character count:** 75 (HN limit is 80)

**Why this works:**
- Clear value proposition
- Tech stack signals (Rust, local AI = HN catnip)
- No hype, just facts

### Post Body

```
Hey HN! I built cmdai, a Rust CLI that converts natural language to safe shell
commands using local AI.

Example:
  $ cmdai "find all PDFs in Downloads larger than 10MB"

  Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

  Execute this command? (y/N)

Why I built this:
I kept finding myself googling "find files by size" for the 100th time. ChatGPT
works but requires copy-paste, breaks my flow, and doesn't validate safety. I
wanted something that:

1. Works offline (local AI, no API keys)
2. Actually prevents foot-guns (blocks `rm -rf /` and friends)
3. Starts fast (<100ms target) and stays out of the way
4. Integrates with my terminal workflow

Technical details:
- Rust for performance and safety
- MLX backend for Apple Silicon (M1/M2/M3) with CPU fallback
- Remote backend support (Ollama, vLLM) with automatic fallback
- Embedded Qwen2.5-Coder-1.5B-Instruct model (quantized)
- Comprehensive safety validation (pattern matching, risk assessment)
- 44 tests passing, contract-based architecture

Current status:
Working MVP with all core features implemented. I'm optimizing performance
(targeting <100ms startup, <2s inference) and preparing binary distribution
(Homebrew, apt, cargo).

What's different from similar tools:
- GitHub Copilot CLI: They're broad (general coding), I'm deep (ops-specific)
- ChatGPT: Requires cloud, no safety validation, manual copy-paste
- Shell aliases/scripts: Static, not adaptive to context

Open source (AGPL-3.0):
I'm building this in public. The core CLI will always be free and open source.

Try it:
https://github.com/wildcard/cmdai

I'd love feedback on:
- Safety patterns I'm missing (what dangerous commands should I block?)
- Performance optimization ideas (Rust experts, I need you!)
- UX improvements (is the confirmation flow annoying?)
- Backend architecture (thoughts on the trait-based design?)

Looking for contributors! Especially interested in:
- Rust performance optimization
- MLX/Apple Silicon experts
- Safety pattern expansion
- Documentation improvements

Happy to answer questions about the architecture, design decisions, or anything
else. Fire away!
```

**Character count:** ~1,850 (well within HN's limit)

**Why this works:**
- Leads with the problem (relatable)
- Shows, doesn't tell (concrete example)
- Technical depth (Rust, MLX, architecture)
- Humble tone ("I kept googling...")
- Clear call to action (feedback, contributors)
- No mention of fundraising or business plans

---

## Version B: Ambitious (Includes Business Vision)

**Recommended for:** After community validation, want to signal ambition, recruiting co-founder

### Title

```
Show HN: cmdai - Building GitHub Copilot for your terminal (open source + SaaS)
```

**Character count:** 79

**Why this works:**
- Ambitious comparison (GitHub Copilot)
- Signals business model (SaaS)
- Still tech-focused

### Post Body

```
Hey HN! I'm building cmdai - a Rust CLI that converts natural language to safe
shell commands using local AI. Think GitHub Copilot for your terminal.

Example:
  $ cmdai "find all PDFs in Downloads larger than 10MB"

  Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

  Execute this command? (y/N)

The vision:
Every DevOps engineer wastes hours googling command syntax, fears typos causing
production outages, and fumbles during incidents. I'm building the AI-native
operations platform to solve this.

What I've built (MVP):
- Rust CLI with local AI (works offline, no API keys)
- Safety-first (blocks `rm -rf /`, fork bombs, privilege escalation)
- Apple Silicon optimized (MLX backend) with CPU fallback
- Remote backend support (Ollama, vLLM)
- 44 tests passing, production-ready architecture

Tech stack:
- Rust for performance (<100ms startup target)
- MLX for Apple Silicon, Candle for CPU
- Embedded Qwen2.5-Coder-1.5B-Instruct (quantized)
- Trait-based backend system (extensible)
- Comprehensive safety validation

What's different:
- vs GitHub Copilot CLI: Deep on ops (they're broad on code)
- vs ChatGPT: Local-first, safety validation, terminal native
- vs shell scripts: AI-adaptive, learns from context

The business model (PostHog playbook):
- Open source core (free forever, trust + growth)
- Cloud SaaS (better models, team collaboration)
- Enterprise (audit logs, SSO, RBAC, compliance)

Target: $50M ARR by 2028 following the PostHog/GitLab model.

Current status:
- Working MVP, optimizing for V1.0 launch
- Planning cloud backend (Q1 2025)
- AGPL-3.0 licensed (core always open source)

Why I'm sharing this on HN:
1. Get feedback from the best technical community
2. Find contributors (especially Rust/MLX experts)
3. Connect with potential co-founders (looking for technical co-founder/CTO)
4. Validate the vision (does this resonate with you?)

Try it:
https://github.com/wildcard/cmdai

Full roadmap/business plan:
https://github.com/wildcard/cmdai/blob/main/EXECUTIVE_SUMMARY.md

I'd love to hear:
- Would you use this? What features do you need?
- Safety patterns I'm missing?
- Performance optimization ideas?
- Thoughts on the business model?

Also: If you're a senior engineer excited about building this into a company,
let's talk. Looking for a technical co-founder.

Happy to answer any questions - technical, business, or otherwise!
```

**Character count:** ~2,200

**Why this works:**
- Still leads with technical details
- Shows ambition without being salesy
- Transparent about business model
- Explicitly asks for co-founder (attracts serious people)
- Provides roadmap link for those interested
- Balances vision with execution

---

## Timing Recommendations

### Best Day to Post

**Optimal:**
- **Tuesday, Wednesday, or Thursday**
- Avoid Monday (people catching up from weekend)
- Avoid Friday (people checking out early)
- NEVER weekend (low traffic, high competition from weekend projects)

### Best Time to Post

**Peak HN traffic:**
- **8:00 AM - 10:00 AM Pacific Time (PT)**
- This hits:
  - West Coast morning (coffee + HN ritual)
  - East Coast lunch (11 AM - 1 PM ET)
  - Europe evening (6-8 PM GMT)

**Alternative:**
- **1:00 PM - 2:00 PM PT** (lunch hour traffic)
- Avoid: Late evening PT (only West Coast, misses Europe)

### How to Post

1. **Create account ahead of time** (age matters for credibility)
2. **Post URL:** https://github.com/wildcard/cmdai
3. **Title:** Copy exactly from Version A or B
4. **Text:** Paste the body (HN supports markdown-ish formatting)
5. **Submit and stay online** for 2-3 hours to respond

### What to Expect

**Good outcome:**
- 100-300 points
- 50-150 comments
- 5,000-20,000 visitors
- 100-500 GitHub stars
- 10-30 contributor inquiries

**Great outcome:**
- 300-600 points
- 150-300 comments
- 20,000-50,000 visitors
- 500-1,500 GitHub stars
- 30-100 contributor inquiries
- Front page for 6-12 hours

**Modest outcome:**
- 20-100 points
- 10-50 comments
- 1,000-5,000 visitors
- 20-100 GitHub stars
- 5-10 contributor inquiries

**All of these are valuable!** Even modest outcomes build momentum.

---

## Response Strategy

### General Guidelines

**DO:**
- ✅ Respond within 15-30 minutes (shows you're engaged)
- ✅ Be humble and thankful ("Great point!", "Thanks for this!")
- ✅ Admit limitations honestly ("You're right, I haven't thought about X")
- ✅ Ask follow-up questions ("How would you approach this?")
- ✅ Link to specific code/docs when relevant
- ✅ Use technical depth to earn credibility
- ✅ Thank people for trying it out

**DON'T:**
- ❌ Get defensive about criticism
- ❌ Argue or correct people aggressively
- ❌ Ignore tough questions
- ❌ Spam links to your project
- ❌ Sound like marketing/sales
- ❌ Take it personally (HN can be harsh)

---

## Common Questions & How to Respond

### 1. "How is this different from GitHub Copilot CLI?"

**Response:**
```
Great question! GitHub Copilot CLI is amazing for general command suggestions.
cmdai is more focused on:

1. Operations-specific workflows (deploying, monitoring, incident response)
2. Safety validation (we block dangerous operations, not just suggest safe ones)
3. Local-first architecture (works offline, no GitHub account needed)
4. Extensible backends (MLX for Apple Silicon, Ollama, vLLM, etc.)

Think of it as: Copilot CLI is broad (covers all CLI usage), cmdai is deep
(specialized for DevOps/SRE workflows).

That said, there's definitely overlap! I'm focused on being the best tool for
ops engineers specifically.
```

**Why this works:**
- Acknowledges Copilot CLI is good
- Differentiates without attacking
- Specific examples
- Humble tone

---

### 2. "Why not just use ChatGPT?"

**Response:**
```
Fair point! ChatGPT is great for generating commands. Key differences:

1. Workflow: ChatGPT requires copy-paste, cmdai runs in-terminal
2. Safety: ChatGPT might suggest `rm -rf /tmp/*` without context, cmdai blocks
   anything that matches dangerous patterns
3. Offline: cmdai works without internet (embedded model)
4. Speed: Local inference is faster for simple commands (targeting <2s)
5. Context: cmdai knows your shell, OS, current directory (future feature)

For complex reasoning, ChatGPT is better. For quick "what's that find command
again?" cmdai is faster.

Different tools for different use cases!
```

**Why this works:**
- Doesn't claim to be better at everything
- Specific use cases
- Acknowledges ChatGPT's strengths

---

### 3. "This seems dangerous. What if the AI generates a bad command?"

**Response:**
```
100% valid concern! This is why safety is the #1 design principle:

1. **User confirmation required** (default): Every command shows "Execute? (y/N)"
2. **Pattern matching**: Blocks known dangerous operations:
   - `rm -rf /`, `rm -rf ~`
   - `mkfs`, `dd if=/dev/zero`
   - Fork bombs: `:(){ :|:& };:`
   - Critical path operations: `/bin`, `/usr`, `/etc`
3. **Risk levels**: Safe (green), Moderate (yellow), High (orange), Critical (red)
4. **Strict mode** (default): Blocks critical operations entirely
5. **POSIX compliance validation**: Rejects invalid syntax

Code: https://github.com/wildcard/cmdai/tree/main/src/safety

I'm actively looking for more patterns to block. What dangerous commands am I
missing? (Seriously, I'd love input here!)
```

**Why this works:**
- Takes concern seriously
- Provides concrete details
- Links to code (proof)
- Asks for help (engages commenter)

---

### 4. "Why Rust? Seems overkill for a CLI tool."

**Response:**
```
Fair question! Reasons I chose Rust:

1. **Performance**: Startup time matters. Targeting <100ms cold start, <2s inference
2. **Memory safety**: Handling untrusted AI output, filesystem operations - no room for segfaults
3. **Single binary**: No Python dependencies, no runtime, just works
4. **Cross-compilation**: Easy to ship binaries for macOS/Linux/Windows
5. **MLX FFI**: cxx crate makes Apple Silicon integration smooth
6. **Learning**: Wanted to get better at Rust (honest reason!)

Could I have built this in Go/Python? Absolutely. But Rust gives me the
performance and safety guarantees I want for a tool that runs privileged commands.

Plus, the Rust community is amazing for getting feedback on architecture decisions!
```

**Why this works:**
- Acknowledges it's a choice, not mandatory
- Specific technical reasons
- Humble (learning motivation)
- Compliments HN community (Rust folks)

---

### 5. "How do you plan to make money from this?"

**Response:**
```
Good question! Following the PostHog/GitLab/Supabase playbook:

**Open source (free forever):**
- CLI tool for individuals
- Local AI backends
- Full safety validation
- Purpose: Growth engine, build trust

**Cloud + Enterprise (paid):**
- Better models (GPT-4, Claude)
- Team collaboration (shared prompts, workflows)
- Audit logs + compliance (SOC 2)
- SSO, RBAC, enterprise features
- Purpose: Revenue engine

Pricing (planned):
- Free: $0 (individuals)
- Pro: $10/user/mo (power users)
- Team: $20/user/mo (small teams)
- Enterprise: $50+/user/mo (compliance, audit, self-hosted)

The CLI will NEVER have features removed to force upgrades. Cloud/enterprise
features are additive (like PostHog, not like Docker's rug-pull).

Target: $50M ARR by 2028. Ambitious? Yes. Possible? PostHog did $40M in 3 years
with similar model.
```

**Why this works:**
- Transparent about business model
- Cites successful examples
- Clear promise (no rug-pull)
- Realistic but ambitious

---

### 6. "AGPL license? Why not MIT/Apache?"

**Response:**
```
Great question! AGPL was chosen deliberately:

**Why AGPL:**
- Prevents cloud providers from offering cmdai-as-a-service without contributing back
- If someone forks and runs a SaaS version, they must open-source their changes
- Protects against "embrace, extend, extinguish" by big tech
- Still allows free use, modification, distribution for self-hosting

**Trade-offs:**
- Some companies won't use AGPL in production (fair)
- More restrictive than MIT/Apache
- Might slow enterprise adoption

I'm open to dual-licensing (AGPL for self-hosted, commercial license for
enterprises that need it). This is the MongoDB/Elastic playbook.

Thoughts? Would MIT/Apache be better for this use case? I'm genuinely open to
feedback here.
```

**Why this works:**
- Shows you've thought about it
- Explains reasoning
- Open to feedback
- Asks for opinions

---

### 7. "What happens when OpenAI/Microsoft builds this?"

**Response:**
```
They might! Here's my thinking:

**Moats I'm building:**
1. **Data moat**: After 100K commands, I'll fine-tune models on real ops usage.
   That proprietary training data = better accuracy than GPT-4 for ops tasks
2. **Integration moat**: 50+ pre-built integrations (AWS, k8s, Datadog, etc.)
   with community marketplace
3. **Compliance moat**: SOC 2, audit logs, RBAC = 6-12 months for competitors
   to replicate
4. **Community moat**: Open source = trust. Enterprises prefer community-backed
   tools over single-vendor

**What they have:**
- Better models (for now)
- More capital
- Brand recognition

**What I have:**
- Focus (deep on ops, not broad on everything)
- Speed (small team, ship faster)
- Community trust (open source, transparent)
- Incentive alignment (I care about ops engineers, they care about revenue)

Could still be crushed, but I think there's a wedge here. See: PostHog vs Google
Analytics, GitLab vs GitHub, Supabase vs Firebase.

Time will tell!
```

**Why this works:**
- Honest about competition
- Concrete moat strategy
- Cites precedents
- Humble but confident

---

### 8. "I tried it and got an error with [X]"

**Response:**
```
Thanks for trying it out! Sorry you hit an error. Can you share:

1. What command you ran? (e.g., `cmdai "your prompt"`)
2. What error you got? (exact message if possible)
3. Your OS and Rust version? (`rustc --version`)

I'll fix this ASAP. Also, would you mind opening a GitHub issue so I can track it?
https://github.com/wildcard/cmdai/issues/new

If you want to dig in yourself, the relevant code is likely in:
- src/backends/ (if it's an inference error)
- src/safety/ (if it's a validation error)

Happy to pair debug on Discord if that's easier!
```

**Why this works:**
- Grateful they tried it
- Asks for specifics
- Offers multiple ways to help
- Invites contribution

---

### 9. "Why should I contribute? What's in it for me?"

**Response:**
```
Great question! Here's what contributors get:

**Short term:**
1. Learn Rust, LLMs, systems programming (real production codebase)
2. Your code runs on thousands of machines (portfolio piece)
3. Recognition (weekly shoutouts, CONTRIBUTORS.md, social media)
4. Community (work with talented devs, grow your network)

**Long term:**
1. Early equity if we raise (top contributors get offers to join as employees)
2. Job opportunities (contributing to OS projects = hiring signal)
3. Shape the product (your ideas influence the roadmap)

**What I need:**
- Rust optimization (performance improvements)
- MLX/Apple Silicon expertise (faster inference)
- Safety patterns (expand dangerous command detection)
- Documentation (tutorials, guides, examples)

No pressure! Even small PRs (fixing typos, adding tests) help. I'm just grateful
for any contributions.

Start here: https://github.com/wildcard/cmdai/labels/good-first-issue
```

**Why this works:**
- Specific benefits
- Long-term incentives (equity)
- Actionable next steps
- Humble tone

---

### 10. "This is just a wrapper around [LLM]. Why not use [tool] directly?"

**Response:**
```
You're right that at its core, cmdai uses LLMs for generation. But there's more
to it than just calling an API:

**Value-add:**
1. **Safety layer**: Pattern matching, validation, risk assessment (this is hard!)
2. **UX**: Terminal-native workflow, no copy-paste, confirmation flow
3. **Performance**: Optimized for <2s inference, <100ms startup
4. **Offline capability**: Embedded models, works without internet
5. **Backend flexibility**: MLX, Ollama, vLLM, cloud APIs (you choose)
6. **Context awareness**: Future: knows your shell, OS, environment

Could you build this with a bash script + curl? Sure! But:
- Safety validation is complex (regex, AST parsing, POSIX compliance)
- Multi-backend fallback logic is non-trivial
- Performance optimization (startup time, inference speed) takes work
- Distribution (Homebrew, apt, single binary) requires polish

Think of it as: The LLM is the engine, cmdai is the car. Both are needed.

Fair criticism though! I need to make the value-add clearer in the docs.
```

**Why this works:**
- Acknowledges the criticism
- Specific value-add examples
- Explains complexity
- Takes feedback (improve docs)

---

## Follow-Up Comment Template

**Post this 10-15 minutes after your initial post, as a top-level comment:**

```
OP here! A few things I forgot to mention in the post:

**For contributors:**
I've labeled several issues as "good-first-issue" to make it easy to get started:
https://github.com/wildcard/cmdai/labels/good-first-issue

Also hosting weekly "Contributor Office Hours" (Fridays, 2pm PT) to help people
get onboarded. Join here: [Discord/Zoom link]

**Performance benchmarks:**
Current numbers (M1 Mac):
- Cold start: ~150ms (targeting <100ms)
- First inference: ~2.5s (targeting <2s)
- Subsequent: ~1.2s

Open to optimization suggestions!

**Safety patterns I'm considering adding:**
- systemctl operations on critical services
- iptables/firewall modifications
- Docker/Kubernetes destructive operations
- Package manager --force flags

What else should I block/warn about?

**Tech questions I'm grappling with:**
1. Should I use streaming inference for real-time feedback?
2. Better approach to POSIX validation than regex?
3. Trade-offs of embedding model vs always using remote?

Would love input from folks with experience here!

**Looking for:**
- Technical co-founder (senior backend/systems engineer, Rust experience)
- Early design partners (DevOps teams willing to try alpha/beta)
- Advisors (especially in AI, developer tools, or open-source business models)

Thanks for all the feedback so far! I'll be monitoring this thread all day.
```

**Why post this:**
- Adds information you couldn't fit in main post
- Shows you're engaged and responsive
- Provides actionable next steps
- Asks specific questions (drives more comments)
- Signals you're serious (co-founder search, design partners)

---

## Engagement Tactics During Peak Hours

### First 30 Minutes (Critical!)

**Your goal:** Get to 10-20 upvotes quickly (this determines front-page placement)

**Actions:**
1. ✅ Share on Twitter immediately (your followers can upvote)
2. ✅ Share in relevant Slack/Discord communities (Rust, DevOps, etc.)
3. ✅ Email 5-10 friends: "Just posted on HN, would mean a lot if you checked it out"
4. ✅ Respond to EVERY comment within 5-10 minutes

**Don't:**
- ❌ Ask for upvotes directly (against HN rules, can get flagged)
- ❌ Create multiple accounts to upvote yourself (insta-ban)
- ❌ Spam the link everywhere

---

### Hours 1-3 (Build Momentum)

**Your goal:** Keep the thread active (active threads rank higher)

**Actions:**
1. ✅ Respond to every comment (even short ones: "Thanks!")
2. ✅ Ask follow-up questions to commenters
3. ✅ Post your follow-up comment (see template above)
4. ✅ Update GitHub README if people mention confusion
5. ✅ Create GitHub issues from feedback ("Great idea! Tracked here: [link]")

---

### Hours 3-6 (Sustain Engagement)

**Your goal:** Stay on front page as long as possible

**Actions:**
1. ✅ Continue responding (slower pace is fine)
2. ✅ Post updates as replies ("Just pushed a fix for [issue]!")
3. ✅ Thank people for trying it out
4. ✅ Share interesting feedback on Twitter (link back to HN thread)

---

### After 6 Hours (Long Tail)

**Your goal:** Convert engaged users to contributors

**Actions:**
1. ✅ Respond to all remaining comments (even if thread has died down)
2. ✅ Email people who expressed interest in contributing
3. ✅ Follow up with potential co-founder candidates
4. ✅ Write a summary post ("Key takeaways from HN feedback")

---

## Links to Include

### In Post

**Essential:**
- GitHub repo: `https://github.com/wildcard/cmdai`

**Optional (Version B):**
- Executive summary: `https://github.com/wildcard/cmdai/blob/main/EXECUTIVE_SUMMARY.md`
- Roadmap: `https://github.com/wildcard/cmdai/blob/main/ROADMAP.md`

### In Comments

**When relevant:**
- Safety code: `https://github.com/wildcard/cmdai/tree/main/src/safety`
- Backend architecture: `https://github.com/wildcard/cmdai/tree/main/src/backends`
- Contributing guide: `https://github.com/wildcard/cmdai/blob/main/CONTRIBUTING.md`
- Good first issues: `https://github.com/wildcard/cmdai/labels/good-first-issue`
- Tests: `https://github.com/wildcard/cmdai/tree/main/tests`

---

## Post-Launch Actions

### Immediately After

1. **Cross-post to Reddit** (same day):
   - r/rust
   - r/commandline
   - r/devops
   - r/selfhosted
   - r/opensource

2. **Tweet about it**:
   ```
   Just launched cmdai on Hacker News! 🚀

   A Rust CLI that converts natural language to safe shell commands using
   local AI.

   Would love your feedback: [HN link]

   #rust #devops #ai
   ```

3. **Share in communities**:
   - Rust Discord
   - DevOps Slack communities
   - MLOps Discord
   - Local tech meetup Slacks

---

### First 24 Hours

1. **Respond to all emails** from interested contributors/users
2. **Create GitHub issues** from all feedback
3. **Ship a small fix** based on feedback (shows momentum)
4. **Update README** if confusion emerged
5. **Thank contributors publicly** on Twitter

---

### First Week

1. **Write a "HN Launch Retrospective" blog post**
   - What worked
   - What didn't
   - Key feedback
   - What you're changing

2. **Host first Contributor Office Hours**
   - Invite everyone who expressed interest
   - Pair program on issues
   - Answer questions

3. **Reach out to co-founder candidates**
   - Schedule 1:1 calls
   - Share full vision
   - Assess fit

---

## Metrics to Track

### During Launch (Real-Time)

- [ ] HN points (check every 30 min)
- [ ] HN comments (respond to all)
- [ ] GitHub stars (watch it grow!)
- [ ] GitHub traffic (check Insights)
- [ ] Contributor inquiries (email, Discord, GitHub)

### After Launch (Week 1)

- [ ] Total upvotes (final count)
- [ ] Total comments (engagement)
- [ ] GitHub stars (growth)
- [ ] Forks (interest in contributing)
- [ ] Issues opened (engagement)
- [ ] PRs submitted (conversion to contributors)
- [ ] Email/Discord signups (community building)
- [ ] Co-founder candidates (quality conversations)

---

## Success Criteria

### Minimum Success

- 50+ upvotes
- 20+ comments
- 50+ GitHub stars
- 5+ contributor inquiries
- 1-2 co-founder conversations

**Action:** Consider it a win! Build on it.

---

### Good Success

- 150+ upvotes
- 50+ comments
- 200+ GitHub stars
- 15+ contributor inquiries
- 5+ co-founder conversations
- 1-2 press inquiries

**Action:** Capitalize with follow-up content (blog post, demo video)

---

### Exceptional Success

- 300+ upvotes
- 100+ comments
- 500+ GitHub stars
- 30+ contributor inquiries
- 10+ co-founder conversations
- 5+ press inquiries
- Y Combinator partner reaches out

**Action:** Strike while iron is hot (ship fast, build community, close co-founder)

---

## Red Flags (When to Pivot)

### If You Get Heavily Downvoted or Flagged

**Reasons:**
- Too salesy (Version B might be too much)
- Posted at wrong time
- Title was clickbait
- HN community just didn't like it

**Action:**
- Don't repost immediately (against HN rules)
- Wait 2-3 months
- Try Version A (more conservative)
- Focus on technical depth
- Avoid business model discussion

---

### If Comments Are Mostly Negative

**Reasons:**
- Product doesn't resonate
- Technical approach is flawed
- Safety concerns too high
- Comparison to existing tools falls flat

**Action:**
- Don't get defensive!
- Take feedback seriously
- Build the requested features
- Come back in 3-6 months with improvements
- Lead with "I launched this 3 months ago, here's what I changed based on HN feedback"

---

## Final Checklist (Morning of Launch)

**2 Hours Before:**
- [ ] Sleep well (you need energy!)
- [ ] Eat breakfast (you'll forget during launch)
- [ ] Clear your calendar (dedicate 3-4 hours)
- [ ] Test GitHub repo (clone, build, run)
- [ ] Verify all links work
- [ ] Read your post one more time
- [ ] Have responses ready for common questions

**30 Minutes Before:**
- [ ] Make coffee/tea
- [ ] Open HN in one tab
- [ ] Open GitHub in another tab
- [ ] Open Twitter in third tab
- [ ] Set phone to Do Not Disturb
- [ ] Tell family/roommates you'll be heads-down

**Launch Time:**
- [ ] Post on HN (8-10am PT)
- [ ] Copy HN link
- [ ] Tweet it immediately
- [ ] Share in Slack/Discord
- [ ] Email 5-10 friends
- [ ] Refresh HN every 5 minutes
- [ ] Respond to every comment

**You've got this!** 🚀

---

## Templates for Other Platforms

### Twitter Launch Thread

```
🚀 Launching cmdai on Hacker News today!

A Rust CLI that converts natural language to safe shell commands using local AI.

Example:
$ cmdai "find all PDFs larger than 10MB"
→ find . -name "*.pdf" -size +10M

No cloud, no API keys, just works offline.

Thread on why I built this 👇 (1/8)

---

1/ The problem:

Every DevOps engineer wastes hours googling command syntax.

"How do I find files by size again?"
"What's the tar flags for compressed archive?"
"How do I kill a process by name?"

We've all been there. (2/8)

---

2/ Existing solutions:

ChatGPT works but:
- Requires copy-paste (breaks flow)
- Needs internet
- No safety validation
- Not terminal-native

GitHub Copilot CLI is great but:
- Broad (code + CLI)
- Requires GitHub account
- Cloud-only

(3/8)

---

3/ What I built:

cmdai = GitHub Copilot for your terminal

- Works offline (embedded local AI)
- Safety-first (blocks dangerous commands)
- Fast (<2s inference target)
- Open source (AGPL-3.0)

(4/8)

---

4/ Tech stack:

- Rust (performance + safety)
- MLX (Apple Silicon optimization)
- Qwen2.5-Coder-1.5B (embedded model)
- Ollama/vLLM support (remote backends)
- 44 tests passing

Built for production from day 1.

(5/8)

---

5/ Safety is key:

Pattern matching blocks:
- rm -rf /
- Fork bombs
- Critical path operations
- Privilege escalation

User confirmation required.
Risk levels: Safe → Critical

Code: [link to safety/]

(6/8)

---

6/ The vision:

Following PostHog model:
- Open source core (free forever)
- Cloud SaaS (team collaboration)
- Enterprise (compliance, audit logs)

Target: $50M ARR by 2028

Ambitious? Yes.
Possible? PostHog did $40M in 3 yrs.

(7/8)

---

7/ How you can help:

⭐ Star the repo: [link]
💬 Give feedback on HN: [link]
🔨 Contribute: [link to good-first-issue]
🤝 Join as co-founder: DM me

Looking for technical co-founder (CTO) to build this together!

(8/8)

---

Thanks for reading! 🙏

Try cmdai:
https://github.com/wildcard/cmdai

HN discussion:
[HN post link]

Questions? Reply to this thread or DM me!
```

---

### Reddit r/rust Post

**Title:**
```
[Project] cmdai - Natural language to shell commands in Rust (with MLX Apple Silicon optimization)
```

**Body:**
```
Hey r/rust!

I built cmdai, a CLI tool that converts natural language to safe shell commands
using local LLMs.

**Example:**
```bash
$ cmdai "find all PDFs in Downloads larger than 10MB"

Generated command:
find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N)
```

**Tech stack:**
- Rust (obviously! 🦀)
- MLX for Apple Silicon (via FFI with cxx crate)
- Embedded Qwen2.5-Coder-1.5B-Instruct model
- Trait-based backend system (extensible)
- Tokio for async runtime

**Architecture:**
I went with a trait-based design for backends:

```rust
#[async_trait]
trait CommandGenerator {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

This lets me support multiple backends (MLX, Ollama, vLLM) with automatic fallback.

**Safety:**
Comprehensive pattern matching for dangerous operations:
- System destruction (`rm -rf /`)
- Fork bombs
- Critical path operations
- Privilege escalation

Code: https://github.com/wildcard/cmdai/tree/main/src/safety

**Performance targets:**
- <100ms cold start
- <2s first inference (on M1 Mac)
- Single binary <50MB

Currently at ~150ms startup, ~2.5s inference. Open to optimization ideas!

**Looking for:**
- Code reviews (especially on the FFI layer with MLX)
- Performance optimization suggestions
- Safety pattern expansion
- Test coverage improvements

**Repo:**
https://github.com/wildcard/cmdai

Also posted on HN today: [link]

Would love feedback from the Rust community!
```

---

### Dev.to Blog Post Outline

**Title:** Building cmdai: An Open-Source AI Command Generator in Rust

**Sections:**
1. **The Problem I Wanted to Solve**
   - Personal pain point
   - Why existing tools weren't enough

2. **Why Rust?**
   - Performance requirements
   - Safety guarantees
   - FFI for MLX integration

3. **Architecture Decisions**
   - Trait-based backend system
   - Safety validation layer
   - Async runtime (Tokio)

4. **The MLX Integration Challenge**
   - FFI with cxx crate
   - Apple Silicon optimization
   - CPU fallback strategy

5. **Safety: The Hardest Part**
   - Pattern matching approach
   - Risk assessment
   - POSIX compliance validation

6. **Testing Strategy**
   - Contract-based testing
   - Integration tests
   - Property-based testing for safety

7. **What I Learned**
   - Rust ownership challenges
   - FFI debugging
   - CLI UX design

8. **What's Next**
   - Performance optimization
   - Cloud backend
   - Community growth

9. **How You Can Help**
   - Contribute
   - Provide feedback
   - Join as co-founder

**CTA:** Star the repo, join Discord, contribute!

---

## Final Words of Advice

**From successful HN launches:**

1. **Be authentic** - HN can smell marketing BS from miles away
2. **Engage deeply** - The more you engage, the better the response
3. **Take criticism gracefully** - Some comments will be harsh, learn from them
4. **Don't optimize for upvotes** - Optimize for quality conversations
5. **Follow up** - The real value is in connections made, not points scored
6. **Build in public** - Share your journey, wins and failures
7. **Stay humble** - You're asking for feedback, not validation

**Remember:**
- Even "failed" launches teach valuable lessons
- One good connection > 1,000 upvotes
- The goal is to build community, not go viral
- HN is the start, not the finish line

**Most importantly:**
Have fun with it! You built something cool. Share it with the world. 🚀

---

## Questions?

If you need help:
- Reviewing your post before launch
- Crafting responses during launch
- Analyzing results after launch
- Planning next steps

Just ask! I'm here to help.

Good luck! You've got this. 💪

---

**Last updated:** 2025-11-19

**Version:** 1.0

**Author:** Technical Writing Specialist for cmdai

**License:** Use this however helps you launch successfully!
