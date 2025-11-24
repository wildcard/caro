# cmdai Video Scripts

> Production-ready scripts for launch videos
>
> Style: Fast-paced, technical, engaging (Fireship/ThePrimeagen vibes)
>
> Created: 2025-11-19

---

## Video 1: "cmdai Demo - GitHub Copilot for Your Terminal"

**Duration:** 2:00
**Purpose:** Show what cmdai does, get people excited, drive GitHub stars
**Target audience:** Developers, DevOps engineers, CLI power users

### Full Script

#### [0:00 - 0:10] HOOK

**SCREEN:**
- Black screen
- Terminal cursor blinking

**NARRATION:**
```
You know that feeling when you're staring at a blank terminal,
trying to remember the exact syntax for find? Or grep? Or tar?
```

**TEXT OVERLAY:**
- "5 minutes wasted" (appears at 0:05)
- "Stack Overflow tab #47" (appears at 0:08)

**EDITING NOTE:** Quick cuts, slightly chaotic music builds tension

---

#### [0:10 - 0:15] THE SOLUTION

**SCREEN:**
- Cut to: `$ cmdai "find all PDFs larger than 10MB"`
- Command appears instantly

**NARRATION:**
```
What if you just... asked for what you want?
```

**TEXT OVERLAY:**
- "cmdai" (logo appears)

**MUSIC:** Shift to upbeat, confident

---

#### [0:15 - 1:15] DEMO (Core of the video)

**SCREEN RECORDING SEQUENCE:**

**Demo 1: Simple [0:15 - 0:25]**
```bash
$ cmdai "list all PDF files in Downloads"

Generated command:
  find ~/Downloads -name "*.pdf" -ls

Execute this command? (y/N) y
[Shows actual file listing]
```

**NARRATION:**
```
Natural language in. Safe shell command out. That's cmdai.
Built in Rust, runs locally - no API keys, no cloud, works offline.
```

**TEXT OVERLAY:**
- "No ChatGPT required" (0:18)
- "Works offline" (0:22)

---

**Demo 2: Moderate Complexity [0:25 - 0:40]**
```bash
$ cmdai "find large files taking up space"

Generated command:
  find . -type f -size +100M -exec ls -lh {} \; | sort -k5 -hr

Execute this command? (y/N) y
[Shows results with file sizes]
```

**NARRATION:**
```
It handles complex pipes and chains. The stuff you'd normally
copy-paste from Stack Overflow and pray it works.
```

**TEXT OVERLAY:**
- "Pipes? No problem." (0:28)
- "Stack Overflow: 0, cmdai: 1" (0:35)

---

**Demo 3: Real DevOps Task [0:40 - 0:55]**
```bash
$ cmdai "show me docker containers using more than 1GB of memory"

Generated command:
  docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}" | \
  awk 'NR>1 && $2+0 > 1000'

Execute this command? (y/N) y
[Shows container stats]
```

**NARRATION:**
```
DevOps tasks? Docker, Kubernetes, AWS CLI - it knows them all.
Because it's trained on actual operations commands, not generic code.
```

**TEXT OVERLAY:**
- "DevOps-native" (0:45)
- "Kubernetes ready" (0:50)

---

**Demo 4: SAFETY [0:55 - 1:10]**
```bash
$ cmdai "delete everything in root directory"

⚠️  DANGEROUS COMMAND DETECTED
Generated command:
  rm -rf /

Risk Level: CRITICAL
This command would destroy your system.

Blocked for your safety.
```

**NARRATION:**
```
But here's the thing - it won't let you shoot yourself in the foot.
Safety validation catches dangerous commands. No more accidentally
nuking your production server.
```

**TEXT OVERLAY:**
- "SAFETY FIRST" (red, bold, 0:58)
- "rm -rf / = BLOCKED" (1:05)

**EDITING NOTE:** Screen shake or red flash when "BLOCKED" appears

---

#### [1:10 - 1:40] HOW IT WORKS

**SCREEN:**
- Split screen / animated diagram

**LEFT SIDE:**
```
You type → "deploy to staging"
```

**RIGHT SIDE (animated flow):**
```
1. Local AI model (MLX on Apple Silicon)
   ↓
2. Generates: kubectl apply -f staging.yaml
   ↓
3. Safety validator checks
   ↓
4. Shows you the command
```

**NARRATION:**
```
Under the hood? Rust for blazing fast startup. Local AI models -
MLX optimized for Apple Silicon, CPU fallback for everyone else.
And a comprehensive safety layer that knows every footgun in the POSIX spec.

Open source. AGPL-3.0. Single binary under 50MB. Startup time under
100 milliseconds. First inference under 2 seconds.
```

**TEXT OVERLAY:**
- "Rust 🦀" (1:12)
- "Apple Silicon optimized" (1:18)
- "<50MB binary" (1:25)
- "<100ms startup" (1:28)
- "Open source (AGPL-3.0)" (1:35)

**EDITING NOTE:** Fast cuts matching the narration beats

---

#### [1:40 - 1:50] THE VISION

**SCREEN:**
- Show ROADMAP.md or animated timeline

**VISUALS:**
```
Today: CLI tool
↓
Q1 2025: Cloud + team collaboration
↓
Q2 2025: Enterprise (audit logs, SSO)
↓
Future: AI-native ops platform
```

**NARRATION:**
```
This is just the beginning. We're following the PostHog model -
open source core, cloud for teams, enterprise for compliance.
The vision? AI-native operations platform for ten thousand teams.
```

**TEXT OVERLAY:**
- "PostHog playbook" (1:42)
- "Open source → Cloud → Enterprise" (1:46)

---

#### [1:50 - 2:00] CALL TO ACTION

**SCREEN:**
- GitHub page (github.com/wildcard/cmdai)
- Star count animating upward

**NARRATION:**
```
Try it now. Star it on GitHub. Or better yet - contribute.
We're building this in public, and we'd love your help.

Link in the description. Let's make the terminal safer, faster,
and actually enjoyable.
```

**TEXT OVERLAY (big, center screen):**
- "github.com/wildcard/cmdai" (1:52)
- "⭐ Star | 🔧 Contribute | 🚀 Try it" (1:56)

**MUSIC:** Triumphant finish

**EDITING NOTE:** End with terminal cursor blinking, then fade to black

---

### B-Roll Suggestions

- Terminal close-ups (typing, cursor)
- Code scrolling (Rust source)
- GitHub contribution graph
- Apple Silicon Mac (M1/M2 chip logo)
- Developer at desk (hands on keyboard)

### Music Suggestions

- Intro: Tense, suspenseful (problem)
- Main demo: Upbeat, electronic, technical
- Safety section: Brief dramatic pause
- Vision: Building, inspiring
- Outro: Energetic, call to action

**Reference tracks:**
- Fireship.io style: High-energy electronic/chiptune
- ThePrimeagen style: Tech house/lo-fi beats

---

## Video 2: "How to Contribute to cmdai (First-Time Contributor Guide)"

**Duration:** 5:00
**Purpose:** Lower barrier to entry, get first-time contributors onboarded
**Target audience:** Developers who want to contribute but don't know where to start

### Full Script

#### [0:00 - 0:30] WELCOME & WHY CONTRIBUTE

**SCREEN:**
- GitHub Issues page (cmdai repository)

**NARRATION:**
```
Hey! So you want to contribute to cmdai, but you're not sure where to start?
Maybe you've never contributed to open source before, or maybe you have,
but diving into a new Rust project feels intimidating.

Good news - I'm going to walk you through the entire process, from zero
to merged PR, in under five minutes. And I mean the *entire* process.

By the end of this video, you'll have everything you need to make your
first contribution today.
```

**TEXT OVERLAY:**
- "First time? Perfect." (0:05)
- "Already contributed? Even better." (0:10)
- "5 minutes to first PR" (0:25)

**EDITING NOTE:** Friendly, encouraging tone. Show your face in a corner if comfortable.

---

#### [0:30 - 2:00] SETUP (90 seconds)

**SCREEN RECORDING (real terminal, in real time):**

**Step 1: Fork and Clone [0:30 - 0:50]**

**SCREEN:**
```bash
# On GitHub: Click "Fork" button
# (show cursor clicking fork)

# In terminal:
$ git clone https://github.com/YOUR_USERNAME/cmdai.git
$ cd cmdai
```

**NARRATION:**
```
First, fork the repo on GitHub. Just click that fork button.
Then clone your fork - not the original, YOUR fork.
This is important because you can't push to the original repo.
```

**TEXT OVERLAY:**
- "Fork = your copy" (0:35)
- "Clone YOUR fork" (0:42)

---

**Step 2: Install Rust [0:50 - 1:10]**

**SCREEN:**
```bash
# If you don't have Rust installed:
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify:
$ rustc --version
rustc 1.75.0 (stable)

$ cargo --version
cargo 1.75.0
```

**NARRATION:**
```
Need Rust? Run this one-liner. It'll install Rust, Cargo, and everything
you need. Takes about two minutes.

Already have Rust? Make sure you're on 1.75 or newer.
```

**TEXT OVERLAY:**
- "Rust 1.75+ required" (0:55)
- "Takes ~2 minutes" (1:05)

**COMMON MISTAKE CALLOUT:**
```
⚠️ Don't forget to restart your terminal after installing Rust!
Or run: source $HOME/.cargo/env
```

---

**Step 3: Build and Test [1:10 - 1:40]**

**SCREEN:**
```bash
# Source Cargo environment (if needed)
$ . "$HOME/.cargo/env"

# Build the project
$ cargo build
   Compiling cmdai v0.1.0
   ...
   Finished dev [unoptimized + debuginfo] target(s) in 45.3s

# Run tests to make sure everything works
$ cargo test
   Running tests...
   test result: ok. 44 passed; 0 failed
```

**NARRATION:**
```
Now build the project. First time takes a minute or two - Cargo's
downloading and compiling dependencies.

Then run the tests. All 44 should pass. If they don't, something's wrong
with your setup - check the README or ask in Discord.
```

**TEXT OVERLAY:**
- "First build: ~2 min" (1:15)
- "44 tests must pass" (1:30)

**COMMON MISTAKE CALLOUT:**
```
⚠️ If cargo isn't found, run: . "$HOME/.cargo/env"
Check CLAUDE.md for details.
```

---

**Step 4: Try it Out [1:40 - 2:00]**

**SCREEN:**
```bash
# Run cmdai locally
$ cargo run -- "list files in current directory"

Generated command:
  ls -la

Execute this command? (y/N) y
[Shows directory listing]
```

**NARRATION:**
```
Finally, try it out. Use cargo run, then two dashes, then your prompt.
This builds and runs cmdai in one command.

If you see this, you're ready to contribute.
```

**TEXT OVERLAY:**
- "cargo run -- 'your prompt'" (1:45)
- "You're ready! 🎉" (1:58)

---

#### [2:00 - 3:00] FIND WORK (60 seconds)

**SCREEN:**
- GitHub Issues page filtered by `good-first-issue` label

**NARRATION:**
```
Alright, setup done. Now let's find something to work on.

Go to the Issues tab on GitHub. Filter by the 'good-first-issue' label.
These are bugs or features specifically chosen for newcomers - clear
scope, good documentation, not too complex.
```

**SCREEN (show actual issues):**
```
Issues labeled 'good-first-issue':
- #42: Add bash completion support
- #57: Improve error message for network timeouts
- #63: Add test for file path with spaces
- #71: Document safety validation patterns
```

**NARRATION:**
```
Pick one that interests you. Maybe it's adding a feature, fixing a bug,
or improving documentation. All contributions matter.

Once you pick an issue, leave a comment saying you want to work on it.
This prevents duplicate work and lets maintainers give you guidance.
```

**TEXT OVERLAY:**
- "good-first-issue = beginner friendly" (2:15)
- "Comment to claim it" (2:40)

**SCREEN (show comment example):**
```
Your comment:
"Hey! I'd like to work on this. First time contributing to cmdai.
Any tips before I start?"
```

**NARRATION:**
```
Something like this is perfect. Maintainers will usually respond within
a few hours with tips or context.
```

---

#### [3:00 - 4:30] MAKE YOUR CONTRIBUTION (90 seconds)

**SCREEN RECORDING (real workflow):**

**Step 1: Create a Branch [3:00 - 3:15]**

**SCREEN:**
```bash
$ git checkout -b fix/improve-error-messages
Switched to a new branch 'fix/improve-error-messages'
```

**NARRATION:**
```
Create a new branch. Name it something descriptive - like fix/issue-57
or feat/bash-completion. This keeps your work organized.
```

---

**Step 2: Write a Test First (TDD) [3:15 - 3:45]**

**SCREEN:**
```bash
# Open tests/integration/safety_tests.rs
$ vim tests/integration/safety_tests.rs
```

**Show file with new test:**
```rust
#[tokio::test]
async fn test_file_path_with_spaces() {
    let validator = SafetyValidator::new();
    let cmd = "cat 'my file.txt'";

    let result = validator.validate(cmd).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().risk_level, RiskLevel::Safe);
}
```

**NARRATION:**
```
This project uses test-driven development. Write your test first.
This forces you to think about what success looks like before you
start coding.

Don't worry if the test fails right now - that's the point.
```

**TEXT OVERLAY:**
- "TDD: Test first, code second" (3:20)
- "Red → Green → Refactor" (3:35)

---

**Step 3: Implement the Fix [3:45 - 4:10]**

**SCREEN:**
```bash
# Open the relevant source file
$ vim src/safety/mod.rs
```

**Show code changes (diff view):**
```rust
// Before:
if path.contains(' ') {
    return Err("Unquoted space in path");
}

// After:
if path.contains(' ') && !is_quoted(path) {
    return Err("Unquoted space in path");
}
```

**NARRATION:**
```
Now write the code to make the test pass. Keep it simple - don't
try to fix ten things at once. Just solve this one issue.
```

**TEXT OVERLAY:**
- "Keep changes focused" (3:50)
- "One issue per PR" (4:00)

---

**Step 4: Run Tests [4:10 - 4:30]**

**SCREEN:**
```bash
$ cargo test
   Running tests...
   test result: ok. 45 passed; 0 failed

# Format your code
$ cargo fmt

# Run the linter
$ cargo clippy
```

**NARRATION:**
```
Run the tests again. Everything should pass now - including your new test.

Then run cargo fmt to format your code and cargo clippy to catch
common mistakes. The CI will check these, so do it now.
```

**TEXT OVERLAY:**
- "All tests must pass" (4:12)
- "cargo fmt + cargo clippy" (4:22)

---

#### [4:30 - 4:50] SUBMIT YOUR PR (20 seconds)

**SCREEN:**
```bash
# Commit your changes
$ git add .
$ git commit -m "fix: handle file paths with spaces in safety validator

Adds validation for quoted paths containing spaces.

Fixes #63"

# Push to YOUR fork
$ git push origin fix/improve-error-messages
```

**NARRATION:**
```
Commit with a clear message. Mention the issue number - that links
everything together.

Push to your fork - not the main repo.
```

**TEXT OVERLAY:**
- "Reference issue in commit" (4:35)
- "Push to YOUR fork" (4:45)

---

**SCREEN (GitHub web interface):**
```
[Shows "Compare & pull request" button appearing]
```

**NARRATION:**
```
GitHub will show you a button to create a pull request. Click it.
```

---

**SCREEN (PR form):**
```
Title: Fix file path handling for paths with spaces

Description:
This PR fixes #63 by improving the safety validator to correctly
handle file paths containing spaces when they're properly quoted.

Changes:
- Added is_quoted() helper function
- Updated path validation logic
- Added test case for quoted paths with spaces

Testing:
All existing tests pass + new test added.
```

**NARRATION:**
```
Fill out the PR description. Explain what you changed and why.
Be clear - the maintainer might review ten PRs today.

Then hit submit.
```

---

#### [4:50 - 5:00] WHAT HAPPENS NEXT & ENCOURAGEMENT

**SCREEN:**
- Your submitted PR on GitHub

**NARRATION:**
```
That's it! A maintainer will review your PR - usually within a day or two.
They might request changes. That's normal. Make the changes, push again,
and the PR updates automatically.

Once it's approved, they'll merge it. And just like that, you're a cmdai
contributor. Your code is running on machines around the world.

Welcome to open source. Now go pick another issue and do it again.

Link to contribute: github.com/wildcard/cmdai/issues
Discord: [link in description]
Office hours: Every Friday, 2pm PT

See you in the repo.
```

**TEXT OVERLAY:**
- "Your code → production 🚀" (4:52)
- "github.com/wildcard/cmdai" (4:56)
- "Join Discord | Office hours Fri 2pm PT" (4:58)

**EDITING NOTE:** End with GitHub stars counter going up by 1 (your contribution)

---

### Common Mistakes to Call Out (Insert as text overlays during relevant sections)

1. **Cloning the wrong repo:**
   - ❌ `git clone github.com/wildcard/cmdai` (original)
   - ✅ `git clone github.com/YOUR_USERNAME/cmdai` (your fork)

2. **Forgetting to source Cargo environment:**
   - ❌ `cargo: command not found`
   - ✅ `. "$HOME/.cargo/env"` first

3. **Not running tests before pushing:**
   - ❌ Push broken code → CI fails → have to fix
   - ✅ `cargo test && cargo fmt && cargo clippy` first

4. **Working on main branch:**
   - ❌ All changes directly on main
   - ✅ Create a feature branch

5. **Vague commit messages:**
   - ❌ `git commit -m "fixed stuff"`
   - ✅ `git commit -m "fix: handle paths with spaces in validator (#63)"`

### Terminal Commands (Exact Text for Copy-Paste)

Include this list in the video description:

```bash
# Setup
git clone https://github.com/YOUR_USERNAME/cmdai.git
cd cmdai
. "$HOME/.cargo/env"  # If cargo not found
cargo build
cargo test

# Development workflow
git checkout -b fix/your-issue-name
# ... make changes ...
cargo test
cargo fmt
cargo clippy
git add .
git commit -m "fix: your clear message here (#issue-number)"
git push origin fix/your-issue-name
```

### Screen Recording Technical Setup

- **Resolution:** 1920x1080 (Full HD)
- **Font size:** 18pt+ (readable on mobile)
- **Terminal theme:** High contrast (dark bg, bright text)
- **Recording tool:** OBS Studio or ScreenFlow
- **Cursor:** Large, high visibility
- **Typing speed:** Slightly slower than normal (for clarity)

---

## Video 3: "cmdai Vision - Building the AI-Native Ops Platform"

**Duration:** 3:00
**Purpose:** Inspire people, explain the bigger vision, recruit co-founders
**Target audience:** Potential co-founders, investors, ambitious contributors

### Full Script

#### [0:00 - 0:45] THE PROBLEM

**SCREEN:**
- Screen recording of developer googling "docker ps filter by memory usage"
- Open 5+ Stack Overflow tabs
- Copy-paste command
- Command fails
- Google again

**NARRATION:**
```
This is Miguel. Senior DevOps engineer at a Series B startup.
He's trying to find Docker containers using more than 1GB of memory.

Watch what happens.

[Show the screen recordings described above]

He spends five minutes googling. Tries three different Stack Overflow
answers. One breaks his terminal. One shows the wrong format. The third
finally works.

Five minutes wasted. And Miguel does this twenty times per day.

That's an hour and forty minutes per day. Per engineer.
Across thirty million developers worldwide.
```

**TEXT OVERLAY:**
- "5 minutes wasted" (0:15)
- "20x per day" (0:20)
- "= 100 minutes lost" (0:25)
- "30M developers worldwide" (0:35)

**VISUALS:**
- Fast cuts between googling, tabs, frustration
- Timer counting up in corner
- Multiply: 5 min × 20 = 100 min animation

**MUSIC:** Tense, building frustration

---

#### [0:45 - 1:30] THE SOLUTION (TODAY)

**SCREEN:**
- Same task, but with cmdai

**SCREEN RECORDING:**
```bash
$ cmdai "show docker containers using more than 1GB memory"

Generated command:
  docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}" | \
  awk 'NR>1 && $2+0 > 1000'

Execute this command? (y/N) y
[Perfect results in 5 seconds]
```

**NARRATION:**
```
Now watch Miguel with cmdai.

[Show the 5-second solution]

Five seconds. First try. Perfect command.

That's what cmdai does today. Natural language to safe shell commands.
Local AI, works offline, comprehensive safety validation.

Open source. Rust. Single binary. Five seconds instead of five minutes.
Sixty times faster.
```

**TEXT OVERLAY:**
- "5 seconds" (big, bold, 0:50)
- "60x faster" (1:00)
- "First try ✓" (1:05)
- "Safe ✓ Local ✓ Open source ✓" (1:15)

**VISUALS:**
- Side-by-side comparison: 5 min vs 5 sec
- Checkmarks appearing
- Speed comparison animation

**MUSIC:** Shift to upbeat, problem solved

---

#### [1:30 - 2:30] THE VISION (WHERE WE'RE GOING)

**SCREEN:**
- Animated roadmap or product evolution

**NARRATION:**
```
But here's where it gets interesting.

Because cmdai isn't just a command generator. It's the foundation for
something much bigger.
```

**VISUAL TRANSITION:** Zoom out from terminal to show bigger picture

---

**Phase 1: Commands → Workflows [1:40 - 1:55]**

**SCREEN ANIMATION:**
```
Today:
  "deploy to staging" → single command

Tomorrow:
  "deploy to staging" →
    1. Run tests
    2. Build Docker image
    3. Push to registry
    4. Update k8s deployment
    5. Verify health checks
    [All automated, all safe]
```

**NARRATION:**
```
Imagine asking for 'deploy to staging' and cmdai orchestrates the entire
workflow. Tests, build, push, deploy, verify. End to end. Safely.
```

**TEXT OVERLAY:**
- "Multi-step workflows" (1:42)
- "End-to-end automation" (1:50)

---

**Phase 2: Local → Platform [1:55 - 2:15]**

**SCREEN ANIMATION:**
```
Open Source CLI (free forever)
          ↓
    Cloud (teams)
      - Better models (GPT-4, Claude)
      - Shared workflows
      - Team collaboration
          ↓
  Enterprise (compliance)
      - Audit logs (who ran what when)
      - SSO, RBAC
      - Self-hosted
      - SOC 2, compliance
```

**NARRATION:**
```
We're following the PostHog playbook. Open source core - free forever.
Cloud for teams who want collaboration and better models. Enterprise
for companies that need audit logs, SSO, compliance.

Think GitLab for operations. Terraform for AI-native workflows.
```

**TEXT OVERLAY:**
- "PostHog model" (2:00)
- "Open → Cloud → Enterprise" (2:05)
- "GitLab for Ops" (2:12)

---

**Phase 3: Integrations [2:15 - 2:30]**

**SCREEN ANIMATION:**
```
cmdai integrates with:
  [Logos appear one by one]
  GitHub, GitLab, AWS, GCP, Azure
  Kubernetes, Docker, Terraform
  Datadog, PagerDuty, Sentry
  Slack, Discord, Teams

  + 50 more tools
  + community marketplace (1,000+ workflows)
```

**NARRATION:**
```
And integrations. Fifty-plus tools out of the box. A community marketplace
with thousands of pre-built workflows. The ecosystem compounds.
```

**TEXT OVERLAY:**
- "50+ integrations" (2:18)
- "1,000+ workflows (community)" (2:25)

**VISUAL:** Logos appearing in orbit around cmdai logo

---

#### [2:30 - 2:50] THE MODEL & TRACTION

**SCREEN:**
- Business model slide

**VISUAL:**
```
┌─────────────────────────────────────────┐
│  Free (Open Source)                    │
│  → Growth engine                       │
│  → Trust, adoption, community          │
└─────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────┐
│  Pro: $10/user/mo                      │
│  Team: $20/user/mo                     │
│  Enterprise: $50-75/user/mo            │
│  → Revenue engine                      │
└─────────────────────────────────────────┘
```

**NARRATION:**
```
The business model? Open source is the growth engine - builds trust,
drives adoption. Cloud and enterprise are the revenue engine.

Gross margins north of eighty-five percent. LTV to CAC ratios between
two and eight X. This is a proven model. PostHog hit forty million ARR
in three years. GitLab went public at eleven billion. Supabase,
one hundred million ARR.
```

**TEXT OVERLAY:**
- "85%+ gross margins" (2:35)
- "PostHog: $40M ARR (3 yrs)" (2:42)
- "GitLab: $11B IPO" (2:45)
- "Proven model ✓" (2:48)

---

**SCREEN:**
- Current traction metrics

**VISUAL:**
```
Today (MVP):
  ✓ Working CLI (Rust)
  ✓ Apple Silicon optimized (MLX)
  ✓ Safety validation
  ✓ 44 tests passing
  ✓ Open source (AGPL-3.0)
```

**NARRATION:**
```
We've built the MVP. It works. It's fast. It's safe. It's open source.
```

---

#### [2:50 - 3:05] THE OPPORTUNITY

**SCREEN:**
- Split screen: TAM numbers + Roadmap

**LEFT: Market**
```
TAM:
  30M developers worldwide
  10M CLI power users
  1M at compliance companies

  Target: 10,000 teams
  × $60K/year average
  = $600M opportunity
```

**RIGHT: Roadmap**
```
Q1 2025: Cloud launch ($2K MRR)
Q2 2025: Enterprise ($150K ARR)
Q3 2025: Platform ($500K ARR)
Q4 2025: Series A ($100K MRR)

2028: $50M+ ARR
```

**NARRATION:**
```
The market? Six hundred million dollar opportunity. We're targeting
ten thousand teams by twenty twenty-eight.

The roadmap? Cloud launch Q1. Enterprise Q2. Platform Q3. Series A by
end of year. Fifty million ARR by twenty twenty-eight.

This is ambitious. But the path is clear. The model is proven. And the
market is ready.
```

**TEXT OVERLAY:**
- "$600M TAM" (2:52)
- "$50M ARR by 2028" (2:58)

---

#### [3:05 - 3:20] THE ASK

**SCREEN:**
- Split into three sections

**SECTION 1: Co-founders**
```
Looking for:
  → Technical co-founder/CTO
  → 7+ years backend/systems
  → Rust experience (or eager to learn)
  → Loves open source + startups

Offering:
  → 20-40% equity
  → Build from ground floor
  → Shape the future of DevOps
```

**SECTION 2: Contributors**
```
Contribute:
  → V1.0 completion
  → Cloud backend
  → Enterprise features
  → Documentation

  → Top contributors may be hired
```

**SECTION 3: Everyone**
```
Star on GitHub
Try it
Spread the word
```

**NARRATION:**
```
So here's the ask.

If you're a senior engineer looking for a co-founder opportunity -
let's talk. Twenty to forty percent equity. Build this from the ground up.

If you want to contribute - we have issues ready. Top contributors get
hired when we raise.

And everyone - star the repo. Try cmdai. Tell other developers.

We're building the AI-native operations platform for the next decade.
And we're doing it in public.
```

**TEXT OVERLAY:**
- "github.com/wildcard/cmdai" (3:12)
- "⭐ Star | 🔧 Contribute | 📧 Reach out" (3:18)

---

#### [3:20 - 3:30] CLOSING

**SCREEN:**
- Terminal with cmdai running
- GitHub star count going up
- Fade to cmdai logo

**NARRATION:**
```
Let's make the terminal safer, faster, and AI-native.

cmdai. GitHub Copilot for your terminal.

Link in description. Let's build.
```

**TEXT OVERLAY (centered):**
```
cmdai
Building the AI-native ops platform

github.com/wildcard/cmdai
```

**MUSIC:** Triumphant, inspiring finish

**EDITING NOTE:** Fade to black with terminal cursor blinking one last time

---

### B-Roll Suggestions

**Problem section:**
- Frustrated developer at desk
- Multiple browser tabs
- Terminal errors
- Clock ticking (time wasted)

**Solution section:**
- Clean terminal workflow
- Developer smiling
- Fast command execution
- Checkmarks appearing

**Vision section:**
- Whiteboard animations (workflows)
- Architecture diagrams
- Integration logos
- Growing graphs (revenue, users)

**Opportunity section:**
- Team collaboration shots
- Office/remote work scenes
- Code being written
- Handshake (co-founder recruiting)

### Visual Diagrams to Create

**Diagram 1: Workflow Evolution**
```
[Simple terminal command]
     ↓
[Multi-step workflow]
     ↓
[Full platform orchestration]
```

**Diagram 2: Business Model Funnel**
```
Open Source (wide top)
    ↓
Cloud (medium)
    ↓
Enterprise (narrow, but high value)
```

**Diagram 3: Integration Ecosystem**
```
[cmdai logo in center]
[Concentric circles of integrations]
  - Inner: Core (Docker, K8s)
  - Middle: Cloud (AWS, GCP)
  - Outer: Community (marketplace)
```

### Music Recommendations

- **Intro (Problem):** Tense, minor key, building tension
  - Reference: "Stranger Things" theme vibes

- **Solution:** Resolution, major key, upbeat
  - Reference: Fireship.io style electronic

- **Vision:** Inspiring, epic, building
  - Reference: Startup/tech keynote music

- **Outro:** Triumphant, call to action
  - Reference: Sports highlight reel energy

### Color Grading Notes

- **Problem section:** Slightly desaturated, cooler tones (blue/gray)
- **Solution section:** Vibrant, warmer tones (success green)
- **Vision section:** Bright, saturated, optimistic
- **Outro:** High contrast, punchy

---

## Production Notes (All Videos)

### Equipment Recommendations

**Minimum (DIY):**
- Screen recording: OBS Studio (free)
- Audio: USB microphone ($50-100)
- Editing: DaVinci Resolve (free) or iMovie
- Terminal: iTerm2 with high contrast theme

**Professional:**
- Screen: ScreenFlow or Camtasia
- Audio: Shure SM7B + audio interface
- Editing: Final Cut Pro or Premiere Pro
- Camera: DSLR/mirrorless for face shots (if used)

### Terminal Setup for Recording

```bash
# Increase font size for readability
# iTerm2: Preferences → Profiles → Text → Font Size: 18

# High contrast theme
# Recommend: "Dracula" or "Monokai"

# Show commands clearly
# Type slightly slower than normal
# Pause 1-2 seconds after each command output
```

### Narration Tips

**Voice:**
- Speak clearly, but conversationally (use contractions)
- Vary your pace (slow for complex, fast for simple)
- Use pauses for emphasis
- Smile while talking (it comes through in audio)

**Recording:**
- Record in a quiet room (closets work great)
- Use a pop filter (even a cheap one)
- Record in the morning (voice is clearer)
- Do multiple takes, pick the best

**Script adherence:**
- These scripts are guides, not strict word-for-word
- Add your personality
- If a better phrase comes to mind while recording, use it
- But keep the timing roughly the same

### Editing Workflow

**Video 1 (Demo):**
1. Record all screen captures first
2. Record narration to match the video
3. Add music underneath (20-30% volume)
4. Add text overlays at specified timestamps
5. Color grade for consistency
6. Export at 1080p, 60fps

**Video 2 (Contribute):**
1. Record entire workflow in real-time
2. Speed up slow parts (cargo build, downloads) to 2-4x
3. Keep typing at normal speed (for clarity)
4. Add callout boxes for common mistakes
5. Picture-in-picture for your face (optional, but more personal)

**Video 3 (Vision):**
1. Create diagrams/animations first (tools: Figma, After Effects, or Keynote)
2. Record narration
3. Edit visuals to match narration beats
4. Add b-roll where specified
5. Dynamic text overlays (animate in/out)

### File Organization

```
video-production/
├── video-1-demo/
│   ├── footage/
│   │   ├── screen-recordings/
│   │   └── b-roll/
│   ├── audio/
│   │   ├── narration-v1.wav
│   │   └── music.mp3
│   ├── graphics/
│   │   └── text-overlays/
│   └── final-export/
├── video-2-contribute/
│   └── [same structure]
└── video-3-vision/
    └── [same structure]
```

### YouTube Upload Checklist

**Each video:**
- [ ] Title (SEO optimized)
  - Video 1: "cmdai Demo: GitHub Copilot for Your Terminal (Rust, Open Source)"
  - Video 2: "How to Contribute to cmdai - First-Time Contributor Guide"
  - Video 3: "cmdai Vision: Building the AI-Native Ops Platform"

- [ ] Description (include):
  - What cmdai is (1-2 sentences)
  - Timestamps for major sections
  - Links (GitHub, Discord, docs)
  - Terminal commands (for Video 2)
  - Call to action

- [ ] Tags:
  - rust, cli, devops, ai, open-source, terminal, shell, command-line

- [ ] Thumbnail:
  - High contrast
  - Large text (readable on mobile)
  - Include cmdai logo
  - Video 1: Terminal with command
  - Video 2: "First PR" or checklist
  - Video 3: Roadmap/vision diagram

- [ ] End screen:
  - Link to GitHub
  - Link to other videos
  - Subscribe button

- [ ] Cards:
  - Add at key moments (GitHub link, Discord, etc.)

### Social Media Snippets

Create 15-30 second clips for:
- Twitter/X (1:1 square format)
- LinkedIn (landscape)
- Instagram Reels (9:16 vertical)

**Suggested clips:**
- Video 1: The dangerous command safety demo (hooks attention)
- Video 2: The 5-second "you did it!" moment
- Video 3: The PostHog model comparison (shareable insight)

### Cross-Promotion Strategy

**Launch sequence:**
1. Upload all 3 videos to YouTube (unlisted)
2. Publish Video 1 on launch day (Tuesday/Thursday)
3. Post to HN, Reddit, Twitter (link to Video 1)
4. Publish Video 2 two days later (mention in GitHub issues)
5. Publish Video 3 one week later (target investors/co-founders)
6. Link between videos in end screens and descriptions

---

## Budget Estimates

### DIY (Solo Creator)

**Equipment:**
- USB Microphone: $50-100
- Total: $100

**Software:**
- OBS Studio: Free
- DaVinci Resolve: Free
- Total: $0

**Time:**
- Recording: 4-6 hours per video
- Editing: 8-12 hours per video
- Total: 36-54 hours for all 3 videos

**Total cost: $100 + your time**

---

### Professional (Hire Video Editor)

**Production:**
- Video editor (Upwork/Fiverr): $500-1,500 per video
- Professional voiceover (optional): $200-500
- Stock music/graphics: $100-200
- Total: $2,100-5,100 for all 3 videos

**Timeline:**
- Pre-production (scripting, planning): 1 week (done!)
- Recording: 1-2 days
- Editing: 1-2 weeks
- Revisions: 3-5 days
- Total: 3-4 weeks from start to publish

---

## Success Metrics

**Track these after publishing:**

**Video 1 (Demo):**
- Target views: 5,000-10,000 (first month)
- Target CTR: 8-12%
- Conversion to GitHub: 5-10% of viewers
- Expected stars: 100-500

**Video 2 (Contribute):**
- Target views: 1,000-3,000
- Target completion rate: 60%+
- Conversion to contributors: 10-20 people
- Expected PRs: 5-10 within 2 weeks

**Video 3 (Vision):**
- Target views: 2,000-5,000
- Target audience: VCs, potential co-founders
- Expected outreach: 5-10 serious inquiries
- Expected co-founder conversations: 2-3

---

## Next Steps

**Immediate (This Week):**
1. [ ] Decide: DIY or hire editor
2. [ ] Set up recording environment (terminal, mic, quiet space)
3. [ ] Record Video 1 screen captures (do this first!)
4. [ ] Practice narration (read script out loud 3x)

**Week 2:**
1. [ ] Record all narration for Video 1
2. [ ] Edit Video 1 (or hand off to editor)
3. [ ] Create thumbnail for Video 1
4. [ ] Write YouTube description

**Week 3:**
1. [ ] Publish Video 1
2. [ ] Post to HN, Reddit, Twitter
3. [ ] Start recording Video 2
4. [ ] Monitor metrics and feedback

**Week 4:**
1. [ ] Publish Video 2
2. [ ] Link in GitHub CONTRIBUTING.md
3. [ ] Start recording Video 3
4. [ ] Iterate based on feedback

**Week 5:**
1. [ ] Publish Video 3
2. [ ] Share with potential investors/co-founders
3. [ ] Create social media snippets
4. [ ] Plan follow-up content based on what resonated

---

## Questions or Need Help?

**If you get stuck:**
- Script review: Happy to give feedback on your recording
- Editing tips: Can recommend specific cuts or transitions
- Technical setup: Terminal recording settings, audio cleanup
- Distribution: Where to post, how to maximize reach

**Remember:** Done is better than perfect. Video 1 doesn't need to be Fireship quality on day one. Ship it, get feedback, improve Video 2 and 3.

---

**These videos will:**
1. Drive GitHub stars (Video 1)
2. Convert stars → contributors (Video 2)
3. Attract co-founders/investors (Video 3)

**Combined impact:** Accelerate cmdai from solo project to thriving community in 30 days.

Now go make some videos. The command line needs you. 🎥🚀

---

*Scripts created: 2025-11-19*
*Ready for production: Yes*
*Estimated total runtime: 10 minutes*
*Estimated total production time: 36-54 hours (DIY) or 3-4 weeks (professional)*
