# BRAND_VOICE.md

> "We agonize over the best way to slip some humor into release notes." - Slack

This document defines cmdai's brand voice and tone. Like Slack's approach to community management and humor, we believe software should be both powerful and pleasant to use. We take shell command safety seriously, but we don't take ourselves too seriously.

## Our Voice Principles

### 1. **Confident, Not Cocky**

We know our stuff when it comes to shell command generation and safety validation. We speak with authority on LLM integration, POSIX compliance, and preventing `rm -rf /` disasters. But we stay humble—we're not here to lecture, we're here to help.

**Examples:**
- ✅ "We've got your back. This command looks safe."
- ❌ "Obviously, you need our tool to prevent mistakes."
- ✅ "Whoa there! Let's think twice about this one."
- ❌ "You were about to make a terrible mistake."

### 2. **Witty, Not Silly**

Humor is our secret sauce, but it never gets in the way of clarity. We slip wit into error messages, release notes, and documentation—but only when it enhances understanding, never when it obscures it.

**Examples:**
- ✅ "That command looks spicier than a ghost pepper. Proceed with caution?"
- ❌ "LOL this command is gonna wreck ur system bruh"
- ✅ "Fork bombs are so 1999. Let's not bring them back."
- ❌ "This command is totally gonna fork you up! (get it?)"

### 3. **Conversational, Not Casual**

We talk to developers like colleagues, not customers. We use contractions, natural cadence, and everyday language. We're professional without being stuffy, friendly without being overly familiar.

**Examples:**
- ✅ "This command needs confirmation. Here's why..."
- ❌ "The system requires user verification for the aforementioned operation."
- ✅ "Let's break down what this command does."
- ❌ "hey friend lets chat about ur cmd lol"

### 4. **Playful and Bold**

We're not afraid to inject personality into unexpected places. Release notes, error messages, CLI output, commit messages—they're all opportunities to make someone's day a little brighter. But we keep it on point. If we need five sentences to set up a joke, it's not worth it.

**Examples:**
- ✅ "404: Safety pattern not found. (Unlike your potentially dangerous command, which we found immediately.)"
- ❌ "Well, well, well... what do we have here? It appears that through a series of unfortunate events and cosmic coincidences, you've managed to..."
- ✅ "🎉 Tests passing! Your code is cleaner than a freshly formatted SSD."
- ❌ "wow such tests very pass much green so validate"

### 5. **Empathetic and Helpful**

We anticipate questions. We answer them before they're asked. We understand that learning a new CLI tool can be frustrating, and that getting blocked by safety validation when you know what you're doing is annoying. We show we care by providing clear next steps, not just error codes.

**Examples:**
- ✅ "This command modifies system files. If that's what you want, use `--safety permissive`. Need help? Run `cmdai --help`."
- ❌ "Error: SAFETY_VIOLATION_CODE_0x42"
- ✅ "MLX backend isn't available on this system. Falling back to CPU mode (this might be slower)."
- ❌ "Backend initialization failed."

## Copy Principles

These five principles guide all writing in cmdai. The best copy demonstrates at least three of them:

### 1. **Be Clear and Simple**

Clarity trumps everything. If a joke makes the message confusing, cut the joke. Use short sentences. Avoid jargon unless it's universally understood by our audience (developers and CLI users).

- Use contractions (don't, won't, can't)
- Active voice over passive
- Specific over vague
- "You" and "we" over "the user" and "the system"

### 2. **Anticipate and Answer Questions**

Put yourself in the user's shoes. What would they want to know next? What might confuse them? Answer before they ask.

**Examples:**
- Instead of: "Command blocked."
- Write: "Command blocked because it attempts to remove system directories. To override, use `--allow-dangerous` (not recommended)."

### 3. **Aim for Comprehension**

Don't just inform—ensure understanding. Use examples, analogies, and context. Test your copy on someone unfamiliar with the feature.

**Examples:**
- Instead of: "Risk level: HIGH"
- Write: "Risk level: HIGH - This command could delete important files. It's like `sudo rm -rf` but with extra steps."

### 4. **Be Intentionally Playful**

Find opportunities for wit, especially in:
- Error messages (when they're not critical)
- Release notes (agonize over these)
- CLI help text (inject personality)
- Loading messages (make waits pleasant)
- Success confirmations (celebrate wins)

**Examples:**
- "Searching for dangerous patterns... found one faster than you can say 'oops'."
- "Model loading... (time to grab coffee)"
- "Command executed successfully! No servers were harmed in this operation."

### 5. **Build Emotional Connection**

Make users feel:
- **Safe** - "We've got your back"
- **Smart** - "You caught that too? Nice!"
- **Confident** - "You're ready to go"
- **Valued** - "Thanks for contributing!"
- **Understood** - "We've all been there"

## Voice Across Different Contexts

### CLI Output

**Tone:** Direct, helpful, occasionally playful
**Goal:** Quick comprehension, actionable information

```
✓ Command looks safe
⚠ Hold up—this command is risky
✗ Blocked: This would modify system directories
```

### Error Messages

**Tone:** Empathetic, informative, solution-oriented
**Goal:** Explain what went wrong AND how to fix it

```
✗ Couldn't connect to Ollama backend

  Is Ollama running? Try:
    ollama serve

  Or use the embedded backend:
    cmdai --backend embedded "your prompt"
```

### Documentation

**Tone:** Conversational, thorough, encouraging
**Goal:** Teach without overwhelming

```
# Safety Features

cmdai isn't just paranoid—it's *productively* paranoid. We've seen what
`rm -rf /` can do, and we're here to make sure it never happens to you.
```

### Release Notes

**Tone:** Celebratory, informative, witty
**Goal:** Make people excited about updates

```
## v0.2.0 - "The One With Ollama"

We agonized over how to announce this, so here goes: cmdai now supports
Ollama! 🎉

**New Features:**
- Ollama backend integration (because sometimes you want your models served locally and your latency served... not at all)
- Automatic backend fallback (we're persistent like that)
- JSON response parsing with three fallback strategies (belt, suspenders, and duct tape)
```

### Contributing Guidelines

**Tone:** Welcoming, encouraging, clear about standards
**Goal:** Make contributing feel achievable and rewarding

```
### Good First Issues

New to cmdai? Start here! We maintain beginner-friendly issues that won't
leave you drowning in FFI bindings or async runtime internals.
```

### Code Comments

**Tone:** Informative, occasionally witty, never distracting
**Goal:** Help future developers (including future you)

```rust
// Parse JSON response with fallback strategies
// Strategy 1: Direct deserialization (the optimist)
// Strategy 2: Extract from markdown code blocks (the realist)
// Strategy 3: Regex extraction (the pessimist)
// Strategy 4: Give up gracefully (the pragmatist)
```

### Commit Messages

**Tone:** Clear, consistent, professional (but open to occasional emoji when appropriate)
**Goal:** Searchable history, clear intent

```
feat: Add Ollama backend support with automatic fallback

- Implement CommandGenerator trait for Ollama
- Add health check endpoint validation
- Include comprehensive error handling with retry logic
- Fall back to embedded backend when Ollama unavailable

Closes #12
```

## What We Avoid

### 1. **Jargon Without Context**

❌ "Utilize the AST-based heuristic validator for POSIX compliance"
✅ "We check your command against POSIX standards to ensure it works across shells"

### 2. **Apologetic Language**

❌ "Sorry, we couldn't process your request"
✅ "Couldn't parse that response. Let's try a different approach."

### 3. **Vague Errors**

❌ "Error: Operation failed"
✅ "Couldn't download model: Network timeout. Check your connection and try again."

### 4. **Excessive Formality**

❌ "The system has determined that the aforementioned command presents elevated risk"
✅ "This command looks risky. Proceed with caution?"

### 5. **Memes and Internet Slang**

❌ "This command slaps fr fr no cap"
✅ "This command works beautifully"

### 6. **Condescension**

❌ "Obviously, you should never run rm -rf"
✅ "Commands like rm -rf can be dangerous—we'll help you avoid them"

### 7. **False Urgency**

❌ "WARNING!!! CRITICAL ERROR!!! IMMEDIATE ACTION REQUIRED!!!"
✅ "This command needs your confirmation before running"

## Inclusivity Guidelines

### 1. **Use Gender-Neutral Language**

✅ "they/them", "the user", "you", "developers"
❌ "he/she", "guys"

### 2. **Avoid Ableist Language**

✅ "blocked", "disabled", "hidden", "inactive"
❌ "crippled", "blind to", "lame"

### 3. **Keep Skill-Level Language Supportive**

✅ "new to cmdai", "learning the ropes", "getting started"
❌ "newbie", "dummy", "simple enough for anyone"

### 4. **Cultural Sensitivity**

- Avoid idioms that don't translate well
- Be mindful of cultural contexts in examples
- Keep humor universal and technical

## Testing Your Copy

Before finalizing copy, ask yourself:

1. **Is it clear?** Would someone unfamiliar with the feature understand?
2. **Is it helpful?** Does it answer questions or just state facts?
3. **Is it concise?** Can you say it in fewer words without losing meaning?
4. **Is it appropriate?** Does the tone match the context (error vs. success)?
5. **Does it represent us?** Would Slack approve of this approach to humor?

## Examples in Action

### Before and After: Error Message

**Before:**
```
Error: Failed to initialize backend
Error code: BACKEND_INIT_001
```

**After:**
```
✗ Couldn't start the MLX backend

This usually means you're not on an Apple Silicon Mac. No worries—
we'll use the CPU backend instead (it's a bit slower but gets the job done).

Want to use a remote backend? Try:
  cmdai --backend ollama "your prompt"
```

### Before and After: Safety Warning

**Before:**
```
WARNING: Dangerous command detected
Risk level: CRITICAL
Proceed? (y/n)
```

**After:**
```
⚠ Whoa there! This command could delete system files.

Here's what it would do:
  rm -rf /usr/local/bin

This is risky because it removes important system binaries.

Still want to run it? Type 'yes' to confirm, or anything else to cancel.
```

### Before and After: Release Note

**Before:**
```
Version 0.3.0

Changes:
- Added cache module
- Fixed bug #23
- Updated dependencies
```

**After:**
```
## v0.3.0 - "Cache Me If You Can"

We spent way too long debating this release name. Worth it.

**New Features:**
- 💾 **Model caching** - Download once, use forever (or until you clear the cache)
  - Intelligent LRU eviction when disk space runs low
  - Offline mode: works even when your Wi-Fi doesn't
  - Cross-platform cache directory management

**Bug Fixes:**
- Fixed the infamous "model downloads every single time" bug (#23)
  - Thanks to @contributor for the detailed bug report and patience!

**Under the Hood:**
- Updated tokio to 1.35 for better async performance
- Refreshed dependencies (security first!)
```

## Our Mission (The Why Behind Our Voice)

Like Slack, we're not just building software—we're making developers' working lives simpler, more pleasant, and more productive. Every word we write, every error message we craft, every release note we publish should reflect that mission.

We love our work. We care about our users. We believe that safety-critical software can still have personality. We're here to make shell command generation safer, faster, and dare we say it—a little bit fun.

**That's who we are.**

---

## For Contributors

When writing copy for cmdai:

1. **Read this guide** (you're doing great!)
2. **Look at existing examples** - Check current error messages, CLI help text, and release notes
3. **Draft your copy** following our principles
4. **Test it on someone** - Does it make sense? Does it feel like cmdai?
5. **Agonize a little** - Especially if it's a release note 😉
6. **Submit with confidence** - You've got this

Remember: Every word is an opportunity to make someone's day a little better. Make it count.

---

**Questions about brand voice?** Open a GitHub Discussion! We're always refining our voice and would love your input.

**Found copy that doesn't match these guidelines?** Submit a PR! Help us maintain consistency across the project.

---

*Inspired by Slack's legendary approach to community management, humor, and caring deeply about the details.*

*Last updated: 2025-11-23*
