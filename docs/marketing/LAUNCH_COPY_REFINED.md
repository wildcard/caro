# Caro Launch Copy - Platform-Specific Messaging

*Based on refined positioning: Local-first shell companion for DevOps/SREs*

---

## Product Hunt Launch Copy

### Tagline Options (Pick ONE focus)

**XKCD Energy (Recommended):**
> "For when you need tar flags but can't remember if it's -xvzf or -zxvf"

**Privacy Focus:**
> "The shell companion that never phones home"

**Safety Focus:**
> "Shell commands with guardrails—offline, every time"

**Specificity Focus:**
> "Forgot the find flags? Your SRE memory, locally"

**Humor + Value:**
> "Because nobody extracts a tar archive on the first try"

**Recommended Tagline:**
> "For XKCD #1168 moments—shell command recall, offline, with guardrails"

*Why this works: Instant recognition, shared pain, humor + differentiation*

---

### Product Hunt Description

```markdown
## The Problem

You know [XKCD #1168](https://xkcd.com/1168/)? The one where someone
needs to extract a tar archive to defuse a bomb, but can't remember
the flags?

Everyone laughs because everyone's lived it. Maybe not with bombs,
but definitely at 2am during a production incident.

You need to find log files modified in the last hour, but your brain
blanks on the exact `find` syntax. Is it `-mtime` or `-mmin`? You open
a browser tab. Stack Overflow. Copy. Paste. Typo. Google again.

**The context switch just cost you 5 minutes and your flow state.**

Shell commands are optimized for speed and repetition—not for recall.
But AI tools that help come with tradeoffs: cloud latency, data collection,
or trying to do everything at once.

## What Caro Does Differently

Caro is a **local-first shell companion** built specifically for
DevOps engineers, SREs, and system administrators.

**How it works:**
```bash
$ caro "find log files modified in the last hour"

Suggested: find /var/log -type f -mmin -60

Execute? [y/N] _
```

That's it. Describe what you need. Get the command. Confirm before running.

## What Makes Caro Different

🔒 **Local-First, No Exceptions**
- Runs entirely offline with local inference
- Zero telemetry. Zero data collection. Zero cloud calls.
- Your commands never leave your machine.

🛡️ **Rule-Based Safety, Not AI Promises**
- 52 pre-compiled patterns block dangerous operations
- `rm -rf /` gets blocked—not warned, blocked
- Explicit confirmation before ANY execution
- We validate independently; the model can't override safety

🎯 **Intentionally Focused**
- Built for shell command recall—not code generation
- A specialized sub-agent, not a full AI replacement
- Does one thing well: gets you back to work

⚡ **Built for Terminal Professionals**
- Written in Rust for native performance
- Works in your existing terminal (not a new emulator)
- <2s inference on Apple Silicon, <5s on CPU
- Understands BSD vs GNU differences automatically

## Current Status: Alpha

We're actively developing Caro and seeking feedback on:
- What safety rules matter most to you?
- What shell commands do you forget most often?
- What would make you trust an AI tool in your terminal?

**Open source (AGPL-3.0) | Built with Rust | No data collection**

Try it: `cargo install caro`

---

*We're not trying to replace your AI coding assistant. We're trying to
get you back to work when you forget whether it's `-mtime` or `-mmin`.*
```

---

### Product Hunt First Comment (From Maker)

```markdown
Hey Product Hunt! 👋

I'm Kobi, and I built Caro because I kept losing flow state to
forgotten shell commands.

**The story:** I was debugging a production issue, needed to grep
through rotated logs, and completely blanked on the zcat + grep
combination. Opened Chrome. Stack Overflow. Lost 10 minutes.
Happened again the next week with tar flags.

I wanted something that:
1. Works offline (production servers don't always have internet)
2. Confirms before executing (I don't trust AI in my terminal blindly)
3. Just does shell commands (not another "AI for everything" tool)

**What I'd love feedback on:**
- What safety rules would make you trust a tool like this?
- What commands do YOU forget most often?
- Is "local-only" a dealbreaker or a feature for you?

This is alpha—rough edges exist. But the core philosophy is set:
no cloud, explicit confirmation, focused on one job.

Happy to answer any questions! 🐕
```

---

### Prepared Comment Responses

**Q: "Why not just use ChatGPT/Copilot?"**
```markdown
Great question! Three reasons we went local-first:

1. **Latency**: When you're in flow, 3-5s of cloud latency + context
   switching to a browser matters. Local inference is <2s.

2. **Privacy**: For SREs working on production systems, commands often
   contain sensitive paths, server names, or patterns. We wanted zero
   data leaving the machine.

3. **Offline**: Production servers, air-gapped environments, airplane
   debugging—cloud tools aren't always available.

The tradeoff: our local model is smaller, so it won't write your entire
deployment script. But for "what's the find flag for modification time?"—
it's faster and private.
```

**Q: "How is this different from [X tool]?"**
```markdown
We're intentionally narrower than most tools in this space:

- **OpenCode/Copilot**: General-purpose AI coding. Great for that!
  We're specifically for shell command recall.

- **Raycast**: Awesome launcher with broad features. We're terminal-only,
  deeper on shell safety.

- **Warp AI**: Requires their terminal. We work in your existing terminal.

Our lane: you forgot a command, you need it now, you want to confirm
before running. That's it.
```

**Q: "What's the safety model?"**
```markdown
We don't trust the AI's safety assessment—we validate independently:

1. **52 pre-compiled regex patterns** catch dangerous operations:
   - Filesystem destruction (`rm -rf /`, `rm -rf ~`)
   - Fork bombs
   - Disk operations (`mkfs`, `dd if=/dev/zero`)
   - Privilege escalation patterns

2. **Risk levels**: Safe (auto-ok) → Moderate (confirm) → Critical (blocked)

3. **Explicit confirmation**: Nothing executes without you pressing `y`

4. **Pattern matching happens AFTER generation**: The model can say
   "this is safe"—we check anyway.

We're actively seeking feedback on what rules matter most. What would
make YOU trust a tool like this?
```

**Q: "Alpha? Is this production-ready?"**
```markdown
Honest answer: not yet for critical workflows.

What works well:
- Basic command generation (find, grep, tar, etc.)
- Safety blocking for dangerous patterns
- Offline inference

What we're still building:
- Extended pattern library
- Performance optimization
- More platform testing

We're calling it alpha because we want honest feedback before claiming
production-ready. Would rather under-promise.
```

---

## Hacker News Launch Copy

### Show HN Post

**Title:**
```
Show HN: Caro – Local-first shell command companion with rule-based safety (Rust)
```

**Post Body:**
```markdown
Everyone's seen XKCD #1168—the one where someone needs tar syntax to
defuse a bomb. It's funny because it's true.

I built Caro because I was tired of being that person, minus the bomb.

`find -mtime` vs `-mmin`? The exact tar flags for excluding directories?
BSD sed vs GNU sed? These micro-interruptions add up.

**What Caro does:**

```bash
$ caro "find log files changed in the last hour"
Suggested: find /var/log -type f -mmin -60
Execute? [y/N]
```

Describe what you need in plain English, get the shell command, confirm
before running.

**The philosophy:**

1. **Local-first**: Runs entirely offline with local inference (MLX on
   Apple Silicon, CPU fallback elsewhere). No cloud calls, no telemetry,
   no data collection.

2. **Rule-based safety**: I don't trust AI safety assessments in a
   terminal. Caro has 52 pre-compiled regex patterns that block
   dangerous operations (`rm -rf /`, fork bombs, etc.) independent of
   what the model outputs. The model can claim something is safe—we
   check anyway.

3. **Explicit confirmation**: Nothing executes without user confirmation.
   Ever. The `[y/N]` isn't optional.

4. **Intentionally focused**: This isn't an AI coding assistant. It's
   specifically for shell command recall. I wanted to do one thing
   well rather than compete with general-purpose tools.

**Technical details:**

- Written in Rust
- MLX backend for Apple Silicon (<2s inference)
- CPU backend via Candle for cross-platform
- Optional remote backends (Ollama, vLLM) if you prefer larger models
- Model: Qwen2.5-Coder-1.5B (small enough for local, specialized enough
  for shell)

**Current status: Alpha**

The core works. Safety patterns work. But I'm actively seeking feedback:

- What safety rules would you want that I'm missing?
- What shell commands do you forget most often?
- For SREs/DevOps folks: would you actually use this in your workflow?

I'm particularly interested in edge cases where the safety system
might fail—I'd rather hear about them now.

Source: https://github.com/wildcard/caro
Install: `cargo install caro`

Happy to discuss the architecture, safety model, or tradeoffs.
```

---

### HN Comment Response Templates

**On "Why not just alias/function your common commands?"**
```markdown
You're right that aliases handle the common cases well. I use dozens of them.

Caro is for the long tail: commands I use once a month, complex flag
combinations, or when I'm on a server without my dotfiles.

Example: I don't have an alias for "find files larger than 100MB modified
in the last week excluding node_modules"—that's too specific to alias
ahead of time, but exactly when I'd reach for Stack Overflow.

Fair point that power users might not need this. It's most useful when
you're outside your muscle-memory commands.
```

**On "Local models are too dumb for this"**
```markdown
For general coding tasks, I'd agree—local models have real limitations.

For shell command syntax specifically, smaller models work surprisingly
well. The domain is constrained: POSIX utilities, common flags,
predictable patterns. Qwen2.5-Coder-1.5B handles "find files by date"
reliably.

Where it breaks down: complex multi-step pipelines, obscure tools,
anything that needs broader context. For those, the optional Ollama
backend lets you use larger models.

The local-first default is a values choice (privacy/offline) more than
a capability claim.
```

**On "What stops rm -rf from sneaking through?"**
```markdown
Good question—this is the core challenge.

The safety system works in layers:

1. **Pattern matching (regex)**: 52 patterns like `rm\s+-rf\s+/` catch
   obvious cases. These run post-generation, independent of what the
   model claims about safety.

2. **Path analysis**: Operations on `/bin`, `/usr`, `/etc`, `~` trigger
   higher risk levels.

3. **Confirmation requirement**: Even if something passes patterns,
   you still see the command and confirm.

What could still sneak through:
- Obfuscated commands (base64-encoded, etc.)
- Indirect destruction (program that does bad things)
- Novel patterns we haven't seen

I'm not claiming it's bulletproof—that's why confirmation is mandatory
and why I want feedback on edge cases. What patterns should I add?
```

**On "AGPL is a problem for enterprise"**
```markdown
Fair concern. The AGPL choice was intentional: if someone builds a
service around Caro, modifications should flow back.

For local CLI usage (which is the intended use case), AGPL shouldn't
affect you—you're not distributing or providing network services.

If enterprise licensing becomes a real blocker for adoption, I'm open
to discussing alternatives. Right now I'd rather err toward ensuring
contributions return to the community.
```

---

## Emerging Platform Copy Templates

### DevTool.io / Specialized Developer Communities

**Tagline:**
> "Shell command recall for SREs—offline, with guardrails"

**Short Description:**
```markdown
Caro is a local-first CLI that suggests the shell command you forgot.
Built in Rust, runs offline, requires confirmation before execution.
Specifically for DevOps/SRE workflows where privacy matters and
context-switching kills productivity.

- 52 safety patterns block dangerous operations
- No cloud, no telemetry, no data collection
- <2s inference on Apple Silicon

Alpha release. Seeking feedback on safety rules.
```

### SRE/DevOps Specific Communities

**Tagline:**
> "Never Google 'tar exclude directory' again"

**Angle:**
```markdown
Built for terminal professionals who:
- Debug production at 2am and blank on find syntax
- Work on servers where cloud tools aren't available
- Don't want AI training on their command history

Caro runs locally, confirms before executing, and blocks obviously
dangerous operations. It's a focused tool for a specific problem—
not trying to be your AI coding assistant.
```

### Privacy-Focused Communities

**Tagline:**
> "AI shell assistance that stays on your machine"

**Angle:**
```markdown
Every command you type could reveal server names, file paths, or
operational patterns. Caro processes everything locally:

- No API calls for inference
- No telemetry collection
- No command history uploaded
- Works fully offline

The tradeoff: smaller local model. The benefit: your commands are yours.
```

---

## Messaging Checklist Verification

For each piece of copy above:

| Requirement | Product Hunt | Hacker News | Emerging |
|-------------|--------------|-------------|----------|
| Pain point (command recall) | ✅ | ✅ | ✅ |
| Local-first/offline explicit | ✅ | ✅ | ✅ |
| Safety approach (rule-based) | ✅ | ✅ | ✅ |
| Specific profession target | ✅ | ✅ | ✅ |
| Sub-agent positioning | ✅ | ✅ | ✅ |
| No hype language | ✅ | ✅ | ✅ |
| Alpha acknowledgment | ✅ | ✅ | ✅ |
| Technical credibility (Rust) | ✅ | ✅ | ✅ |

---

## Voice/Tone Reference

**Do:**
- "We don't trust AI safety claims—we validate independently"
- "This is alpha. Rough edges exist."
- "What would make YOU trust this?"
- "Intentionally focused on one thing"

**Don't:**
- "Revolutionary AI assistant"
- "Finally, the solution to..."
- "Better than [competitor]"
- "Works perfectly every time"

---

*Document Version: 1.0*
*Positioning: Local-first shell companion for DevOps/SREs*
*Last Updated: December 2025*
