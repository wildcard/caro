# Product Hunt Launch Assets

Ready-to-use copy for the Product Hunt submission. Review and customize before posting.

---

## Tagline (60 char max)

> Natural language to safe shell commands, 100% local

(51 characters)

**Alternatives:**
- "Type what you want your terminal to do. No cloud, no risk." (59 chars)
- "AI shell commands without the rm -rf disasters" (47 chars)

---

## First Comment (Maker Comment)

Hi Product Hunt!

I built caro after watching a friend paste an AI-generated command into their terminal and wipe their Downloads folder. The command looked right. It wasn't.

**The problem:** Every AI assistant — ChatGPT, Claude, Copilot — can generate shell commands that look correct but are subtly destructive. There's no safety net between "looks good" and `Enter`.

**What caro does:** You type what you want in plain English. Caro generates the POSIX command, runs it through 52+ safety patterns (fork bombs, `rm -rf /`, privilege escalation, disk wipes), and only then shows it to you. If something's dangerous, you'll know before it's too late.

**What makes it different:**
- **100% local** — Uses embedded LLMs (MLX on Apple Silicon, CPU everywhere else). Zero data leaves your machine.
- **Single binary** — `curl -fsSL https://setup.caro.sh | bash` and you're running. No Python, no Node, no Docker.
- **Platform-aware** — Knows the difference between BSD and GNU flags, detects your OS/shell/architecture automatically.
- **Open source** — AGPL-3.0, built in Rust, every line auditable.

Install and try it in 30 seconds:
```
brew install wildcard/tap/caro
caro "find all files larger than 100MB"
```

I'd love to hear: **What's the most dangerous command you've accidentally run?**

GitHub: https://github.com/wildcard/caro
Website: https://caro.sh

---

## Maker Bio (260 char max)

> Building caro because I watched a friend wipe their hard drive with an AI-generated command. Local LLM + 52 safety patterns = an AI terminal you can actually trust. Open source, built in Rust.

(192 characters)

---

## Gallery Image Descriptions

**Image 1: Hero — CLI in action**
- Source: Render `demos/vhs/caro-social.tape` with VHS
- Shows: Natural language prompt → generated command → safety check → execution
- Caption: "Type what you want. Get a safe command."

**Image 2: Safety validation**
- Source: Screenshot of caro blocking a dangerous command (e.g., `rm -rf /`)
- Caption: "52+ patterns catch dangerous commands before they execute"

**Image 3: Website landing page**
- Source: Screenshot of caro.sh
- Caption: "Full documentation at caro.sh"

**Image 4: Comparison table**
- Source: Screenshot of the README comparison table
- Caption: "The only CLI that validates AI-generated commands for safety"

**Image 5: Kyaro mascot**
- Source: `assets/kyaro/` animation frames or `presentation/public/mascot-loop.gif`
- Caption: "Meet Kyaro — your loyal shell companion"

---

## Topics / Categories

Primary: **Developer Tools**
Secondary: **Artificial Intelligence**, **Open Source**, **Command Line**, **Productivity**

---

## Social Media Launch Posts

### Twitter/X

```
Launching on Product Hunt today 🚀

caro: Natural language → safe shell commands, 100% local

Every AI can hallucinate `rm -rf /`. Caro blocks 52+ dangerous patterns before they reach your terminal.

No cloud. No API keys. Single binary.

👉 [PH link]
```

### Hacker News (Show HN)

```
Show HN: Caro – Local LLM CLI that validates commands before execution

I built an open-source Rust CLI that converts natural language to shell commands using local inference (MLX on Apple Silicon, CPU fallback). 

What makes it different from piping ChatGPT output to bash: every generated command passes through 52+ pre-compiled safety patterns for things like rm -rf /, fork bombs, privilege escalation. Sub-2s inference on M1. Single binary, no dependencies.

GitHub: https://github.com/wildcard/caro
Install: brew install wildcard/tap/caro
```

---

## Pre-Launch Checklist

### Must-have before PH submission
- [ ] README updated with hero, GIF, comparison table
- [ ] Gallery images rendered from VHS tapes
- [ ] PH page created with tagline + description
- [ ] First comment drafted and ready to post
- [ ] Homebrew formula updated to v1.2.0
- [ ] caro.sh website live with working demo section
- [ ] Waitlist form tested and functional

### Nice-to-have
- [ ] Discord server live with invite link in README
- [ ] Dev.to launch article queued
- [ ] 3+ real testimonials on website
- [ ] Show HN post coordinated for same day
- [ ] Social media posts scheduled

---

*Generated: 2026-04-05*
