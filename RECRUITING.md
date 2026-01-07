# Recruiting Contributors: Outreach Guide

This document provides copy-paste templates for recruiting contributors to Caro based on specific personas identified during demos and community interactions.

---

## A. High-Engagement Leads from Demos

### 1. The "Guardrails" Advocate

**Signal:** Immediately worried about "blind execution" and unsafe commands during demos.

**Best fit:** Security Lane - Red Team Captain

**Outreach Template:**

```
Subject: Caro needs a Red Team Captain - interested?

Hi [Name],

During the Caro demo, you surfaced the #1 failure mode for shell agents: unsafe blind execution. Your instinct to worry about guardrails is exactly what we need.

We're looking for someone to lead our Security Lane and define the "policy engine" where the AI proposes actions but code enforces safety.

Concrete deliverables:
• Safety specification (what must never happen without confirmation)
• Red-team test suite (prompt injection, shell escapes, path traversal)
• Policy engine design (AI proposes, code enforces)
• Audit log format (what model said vs what ran)

First issue: Build a policy engine MVP that denies destructive commands unless explicit override + typed confirmation.

Interested? Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#security-lane

We'd love to have you lead this critical work.

[Your name]
```

### 2. The "Cloud vs Local" Analyst

**Signal:** Asked competitive differentiation questions (Caro vs Gemini/Cloud CLIs).

**Best fit:** Distribution Lane - Enterprise Integration

**Outreach Template:**

```
Subject: Help shape Caro's air-gapped/enterprise story?

Hi [Name],

Your question about differentiation vs cloud CLIs maps directly to our air-gapped/enterprise deployment story. We need someone who understands:

• Offline constraints and auditability requirements
• Reproducible builds for regulated environments
• Policy configs for enterprise adoption

Concrete deliverables:
• Positioning doc: local-first advantages, offline constraints, auditability
• Enterprise install story: reproducible builds, artifact signing, policy configs
• "Deployment modes" matrix (single binary, bundled models, BYO model, MCP)

First issue: Create Nix flake + offline bundle creator for air-gapped deployments.

Interested? Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#distribution-lane

Would love to collaborate on this.

[Your name]
```

---

## B. Explicit Gap Recruiting (Direct CTAs)

### 3. Rust Core Contributors

**Target:** Experienced Rust developers who care about architecture

**Outreach Template (Reddit /r/rust, Discord, etc.):**

```
**Caro: Local-first shell agent in Rust - Runtime Lane open**

We're building a local-first shell agent in Rust (think: "CLI that understands natural language, runs locally, safety-first").

Current state: Published on crates.io, embedded backends (MLX/Candle) working, safety validation in place.

**We need help with:**
• Streaming UX + cancellation (tokio plumbing)
• Backend abstraction + capability detection
• Command planning pipeline with structured outputs
• CLI ergonomics (shell completion, better errors)

**Tech stack:** tokio, clap, serde, candle/MLX for inference

**First issue:** Streaming response display (show tokens as they arrive)

If you like: async Rust, CLI tools, agent architectures
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#runtime-lane

Labels: `lane/runtime`, `good-first-issue`
```

### 4. Local Inference Engineers

**Target:** ML engineers focused on performance

**Outreach Template (Hugging Face forums, ML Discord servers):**

```
**Help optimize local inference for shell agent**

Caro is a Rust CLI that runs local LLMs for command generation. We need help making it blazingly fast.

**Current performance:**
• TTFT: ~1s on M1 (Candle + Qwen 1.5B Q4)
• Tokens/sec: ~15-20 on consumer hardware
• Model size: 1.5B params (quantized)

**We need help with:**
• Quantization profiling (Q4 vs Q5 vs Q8 vs FP16)
• CPU feature detection (AVX2/AVX512/NEON)
• Benchmark harness (TTFT, tokens/sec, task success)
• Prompt engineering for shell tasks

**Tech:** Candle, llama.cpp bindings, MLX (Apple Silicon)

**First issue:** Create benchmark harness for inference performance

If you care about: local inference, quantization, performance
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#inference-lane

Labels: `lane/inference`, `performance`
```

### 5. Security Advisors

**Target:** Security researchers, penetration testers

**Outreach Template (InfoSec Twitter, HN):**

```
**Red-team a local shell agent (prevent "rm -rf /" accidents)**

Caro generates shell commands from natural language. We need security experts to break it and design guardrails.

**Your mission:**
• Build adversarial test suite (prompt injection, shell escapes)
• Design policy engine (deny/allow rules, risk scoring)
• Define safe execution defaults (dry-run, explain, sandbox)
• Create audit log format (provenance tracking)

**Attack vectors we're worried about:**
• Prompt injection → destructive commands
• Path traversal outside allowed roots
• Privilege escalation via sudo tricks
• Data exfiltration patterns

**First issue:** Policy engine MVP (deny destructive commands unless explicit override)

If you like: breaking things, guardrails, abuse-case design
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#security-lane

Labels: `lane/security`, `critical`
```

---

## C. "Augmented Knowledge" Archetypes

### 6. MCP Implementers

**Target:** Developers building MCP skills/clients

**Outreach Template (Anthropic Discord, MCP repos):**

```
**Turn Caro into an MCP skill (bring shell agent to Claude Desktop)**

We're building MCP integration for Caro so it can be a first-class "skill" in Claude Desktop, VS Code, etc.

**What we're building:**
• MCP server exposing command generation + validation tools
• Claude Desktop workflows (offline-friendly)
• VS Code extension with inline command suggestions

**MCP tools to implement:**
• generate_command(prompt, shell, safety_level)
• validate_command(command) → safety result
• explain_safety(command) → why it's risky
• show_decision_tree(prompt) → AI reasoning

**First issue:** MCP server MVP with basic command generation

If you're into: MCP, IDE integrations, agent workflows
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#ecosystem-lane

We have a detailed implementation guide in: .github/first-time-issues/06-mcp-claude-code-integration.md

Labels: `lane/ecosystem`, `mcp`, `integration`
```

### 7. NixOS / Packaging Maintainers

**Target:** Package maintainers in Nix/Homebrew/AUR communities

**Outreach Template (Nix Discourse, Homebrew discussions):**

```
**Package Caro for [Nix/AUR/etc.] - reproducible shell agent**

Caro is a local-first shell agent (Rust CLI that generates commands from natural language).

**Why package this:**
• Safety-first design (prevents dangerous commands)
• Fully offline-capable (no cloud dependencies)
• Single binary + optional bundled model
• Growing adoption (published on crates.io)

**Packaging needs:**
• Nix flake with reproducible builds
• Signed artifacts + checksum verification
• Offline bundles (binary + model + rules)
• SBOM and provenance notes

**First issue:** Create Nix flake for reproducible installation

**Current:** Homebrew formula exists, needs Nix/AUR/APT

If you maintain: Nix packages, care about reproducibility
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#distribution-lane

Labels: `lane/distribution`, `packaging`, `nix`
```

### 8. Shell Prompt/Agent Engineers

**Target:** Developers building AI agents for CLI/shell tasks

**Outreach Template (Agent forums, HN):**

```
**Fix "CWD correctness" for shell agents (hard problem)**

We're solving a core challenge in shell agents: preventing "search / instead of ." mistakes.

**The problem:**
LLMs generating commands often:
• Search entire filesystem instead of current directory
• Use wrong paths (/ instead of ./project)
• Ignore working directory context

**Our approach:**
• System prompt that hard-binds cwd and allowed roots
• Environment snapshot (pwd, git status, file tree caps)
• Guarded file ops requiring explicit target paths

**First issue:** Design tool schema + runtime checks preventing traversal outside allowed roots

If you're into: prompt engineering, agent safety, shell semantics
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#security-lane

This is a hard problem. We'd love your expertise.

Labels: `lane/security`, `lane/inference`, `agent-design`
```

### 9. TUI Designers (Ratatui)

**Target:** Terminal UI developers

**Outreach Template (Ratatui Discord, /r/rust):**

```
**Design safety UX for shell agent (Ratatui)**

Good UI is a safety feature. We need TUI designers to build confirmation flows that prevent accidents.

**What we're building:**
• Interactive confirm UI (plan/review/apply flow)
• Risk indicators with clear "why" explanations
• Diff preview for file modifications
• Command history browser with filtering

**Example confirm flow:**
┌─ Command Preview ──────────────────┐
│ 🔴 RISKY - Confirmation Required   │
│ $ rm -rf ./old-project/            │
│ Risk: Permanent deletion           │
│ [ ] I understand this will delete  │
│ Type "delete old-project": _       │
└────────────────────────────────────┘

**First issue:** Ratatui confirm UI with risk indicators + typed confirmation

If you like: TUI design, Ratatui, making safety beautiful
Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md#ux-lane

Labels: `lane/ux`, `ratatui`, `design`
```

---

## General Recruiting Call (Broad Audience)

**For: HN, Reddit, Twitter, conferences**

### Short Version (Twitter)

```
We're building Caro - a local-first shell agent in Rust.

If you like:
• Tokio + async Rust
• Local inference tuning
• Guardrails that prevent "rm -rf /"

We have scoped issues + fast onboarding: https://github.com/wildcard/caro

Labels: `good-first-issue` for newcomers
Lanes: security, runtime, inference, ux, ecosystem, distribution
```

### Medium Version (Reddit/HN)

```
**Caro: Local-first shell agent in Rust [Help Wanted]**

Caro converts natural language to safe shell commands using local LLMs. Think: "list all PDF files > 10MB" → find command with safety validation.

**Current state:**
• Published on crates.io
• Embedded backends (MLX for Apple Silicon, Candle for CPU)
• Safety validation (52 dangerous patterns pre-compiled)
• Multi-platform (macOS, Linux, Windows)

**We need help with:**
• Security: Policy engine, red-team tests, guardrails
• Runtime: Streaming, cancellation, backend orchestration
• Inference: Quantization, performance, benchmarking
• UX: Ratatui confirmations, plan/review/apply flows
• Ecosystem: MCP integration, IDE plugins
• Distribution: Nix packages, offline bundles, signing

**Tech stack:** Rust, tokio, Candle/MLX, Ratatui, MCP

**First-time friendly:** We have 10+ beginner issues + detailed guides

Check out contribution lanes: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md

If you care about local-first AI, safety-critical software, or great CLI UX - we'd love your help.
```

### Long Version (Blog post / Conference talk follow-up)

```
**Join the Caro Community: Build the Safest Shell Agent**

[Include after demo or blog post]

Thank you for the interest in Caro! We're building this in the open and need contributors across six lanes:

**1. Security Lane** - Prevent catastrophic commands
• Lead role: Red Team Captain
• Deliverables: Policy engine, adversarial tests, audit logs
• First issue: Policy engine MVP

**2. Runtime Lane** - Fast, reliable async Rust
• Lead role: Rust Architect
• Deliverables: Streaming UX, backend abstraction, cancellation
• First issue: Streaming response display

**3. Inference Lane** - Optimize local model performance
• Lead role: Performance Engineer
• Deliverables: Benchmarks, quantization, CPU features
• First issue: Benchmark harness

**4. UX Lane** - Make safety beautiful
• Lead role: TUI Designer
• Deliverables: Confirm UI, plan/review/apply, diff preview
• First issue: Ratatui confirmation flow

**5. Ecosystem Lane** - Integrate everywhere
• Lead role: Integration Engineer
• Deliverables: MCP server, VS Code extension, IDE plugins
• First issue: MCP server MVP

**6. Distribution Lane** - Package for all platforms
• Lead role: Packaging Maintainer
• Deliverables: Nix/AUR/APT, signed artifacts, offline bundles
• First issue: Nix flake

**How to get started:**
1. Read HELP_WANTED.md to pick your lane
2. Browse issues with your lane label
3. Comment "I'd like to work on this" on an issue
4. Join GitHub Discussions for questions

**Lane leads wanted:** If you have relevant experience and want to own a lane's technical direction, we're looking for leads!

Links:
• Contribution lanes: [HELP_WANTED.md]
• First-time issues: [.github/first-time-issues/]
• Quick start: [CONTRIBUTING.md]

Let's build something great together.
```

---

## Follow-Up Templates

### After Demo Interest

```
Hi [Name],

Great to meet you at [event/demo]! You mentioned interest in [specific aspect].

That maps to our [Lane Name] Lane. We're looking for help with:
• [Specific deliverable 1]
• [Specific deliverable 2]

First issue to try: [Issue name + link]

Would love to have you contribute! Any questions, just ask in the issue or GitHub Discussions.

[Your name]
```

### After GitHub Star/Watch

```
Hi [Name],

Thanks for starring Caro! If you're interested in contributing, we have scoped issues across 6 lanes:

• Security - guardrails and red-team testing
• Runtime - tokio and async Rust
• Inference - performance optimization
• UX - Ratatui and TUI design
• Ecosystem - MCP and IDE integration
• Distribution - packaging and reproducible builds

First-time contributors: We have 10 beginner-friendly issues with detailed guides!

Check out: https://github.com/wildcard/caro/blob/main/HELP_WANTED.md

Let me know if any lane interests you.

[Your name]
```

---

## Metrics to Track

**Engagement funnel:**
1. Demo attendee / blog reader
2. GitHub star / discussion participant
3. Issue comment / question asked
4. PR submitted
5. Regular contributor
6. Lane lead

**Track:**
- Source of contributors (demo, HN, Reddit, etc.)
- Time from first contact to first PR
- Lane distribution (which lanes attract most contributors)
- First-time vs returning contributor ratio

**Success indicators:**
- 2+ active contributors per lane
- 1 lane lead per lane
- <7 day turnaround on issue claims
- >50% PR acceptance rate from first-timers

---

## Community Channels

**Where to recruit:**
- HN (Show HN, Ask HN)
- Reddit (/r/rust, /r/CommandLine, /r/linux)
- Rust Discord servers
- Anthropic Discord (for MCP lane)
- ML/AI forums (Hugging Face, Papers with Code)
- Conference talks + demos
- Twitter/Mastodon
- Blog posts

**Templates are in this doc - just customize and post!**

---

## Questions About Recruiting?

Open a discussion: https://github.com/wildcard/caro/discussions

We're learning as we go and welcome suggestions for better recruiting strategies!
