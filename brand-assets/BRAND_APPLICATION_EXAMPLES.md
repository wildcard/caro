# cmdai Brand Application Examples

This document shows how the cmdai brand comes to life across different contexts and platforms.

---

## Table of Contents
1. [Terminal Output Examples](#terminal-output-examples)
2. [Web Design Mockups](#web-design-mockups)
3. [Merchandise Designs](#merchandise-designs)
4. [Documentation Examples](#documentation-examples)
5. [Social Media Templates](#social-media-templates)
6. [Email Templates](#email-templates)

---

## Terminal Output Examples

### Example 1: Safe Command Generation

```bash
$ cmdai "find all PDF files larger than 10MB in Downloads"

┌─ cmdai ──────────────────────────────────────────────────┐
│                                                           │
│  ▸ Your request:                                          │
│    "find all PDF files larger than 10MB in Downloads"    │
│                                                           │
├─ Generated Command ──────────────────────────────────────┤
│                                                           │
│  find ~/Downloads -type f -name "*.pdf" -size +10M       │
│                                                           │
├─ Safety Analysis ────────────────────────────────────────┤
│  ✓ No dangerous patterns detected                        │
│  ✓ POSIX compliant                                        │
│  ✓ Read-only operation                                   │
│  ✓ File paths properly quoted                            │
│                                                           │
│  Risk Level:  ▓▓▓▓▓▓▓▓▓▓ 100%              [SAFE] ✓      │
│                                                           │
├─ Execute ────────────────────────────────────────────────┤
│  ⚡ Run this command? [Y/n/explain/modify]                │
│                                                           │
└───────────────────────────────────────────────────────────┘

Performance: 47ms (validation: 12ms, generation: 35ms)
Backend: mlx (Apple Silicon)
```

### Example 2: Moderate Risk Command

```bash
$ cmdai "delete all temporary files from last month"

┌─ cmdai ──────────────────────────────────────────────────┐
│                                                           │
│  ▸ Your request:                                          │
│    "delete all temporary files from last month"          │
│                                                           │
├─ Generated Command ──────────────────────────────────────┤
│                                                           │
│  find /tmp -type f -mtime +30 -delete                    │
│                                                           │
├─ Safety Analysis ────────────────────────────────────────┤
│  ⚠ File deletion operation (irreversible)                │
│  ✓ Limited to /tmp directory                             │
│  ✓ Time-based filtering (>30 days)                       │
│  ⚠ Number of affected files: ~247                        │
│                                                           │
│  Risk Level:  ▓▓▓▓▓▓░░░░  60%         [MODERATE] ⚠       │
│                                                           │
├─ Recommendation ─────────────────────────────────────────┤
│  💡 Consider previewing files first:                     │
│     find /tmp -type f -mtime +30 -ls                     │
│                                                           │
├─ Execute ────────────────────────────────────────────────┤
│  ⚠  Type "yes delete" to confirm: _                      │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

### Example 3: Blocked Dangerous Command

```bash
$ cmdai "clean up everything on the system"

╔═ cmdai ══════════════════════════════════════════════════╗
║                                                           ║
║  ▸ Your request:                                          ║
║    "clean up everything on the system"                   ║
║                                                           ║
╠═ Generated Command ══════════════════════════════════════╣
║                                                           ║
║  sudo rm -rf /                                            ║
║                                                           ║
╠═ Safety Analysis ════════════════════════════════════════╣
║  ✗ CRITICAL: System destruction pattern                  ║
║  ✗ CRITICAL: Root directory deletion                     ║
║  ✗ CRITICAL: Requires sudo privileges                    ║
║  ✗ CRITICAL: Recursive forced removal                    ║
║                                                           ║
║  Risk Level:  ▓░░░░░░░░░  10%         [CRITICAL] ✗       ║
║                                                           ║
╠═ ACTION BLOCKED ═════════════════════════════════════════╣
║                                                           ║
║  🛡️  cmdai has BLOCKED this command for your safety.     ║
║                                                           ║
║  This operation would destroy your entire system.        ║
║  If you're trying to free up disk space, try:           ║
║                                                           ║
║  • "show disk usage by directory"                        ║
║  • "find large files in home directory"                  ║
║  • "clean up package manager cache"                      ║
║                                                           ║
║  💡 Describe what you actually want to achieve.          ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝

Safety validator: ACTIVE • Override: --allow-dangerous (NOT RECOMMENDED)
```

### Example 4: Startup Banner

```bash
$ cmdai --version

╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║         ██████╗███╗   ███╗██████╗      █████╗ ██╗               ║
║        ██╔════╝████╗ ████║██╔══██╗    ██╔══██╗██║               ║
║        ██║     ██╔████╔██║██║  ██║    ███████║██║               ║
║        ██║     ██║╚██╔╝██║██║  ██║    ██╔══██║██║               ║
║        ╚██████╗██║ ╚═╝ ██║██████╔╝    ██║  ██║██║               ║
║         ╚═════╝╚═╝     ╚═╝╚═════╝     ╚═╝  ╚═╝╚═╝               ║
║                                                                   ║
║                  ⚡ AI-Powered · Human-Safe 🛡️                    ║
║                                                                   ║
║                      Version 1.0.0-beta                           ║
║                  Built with Rust • AGPL-3.0                       ║
║                                                                   ║
╠═══════════════════════════════════════════════════════════════════╣
║  SYSTEM STATUS                                                    ║
║  ✓ Safety validator: ACTIVE                                       ║
║  ✓ Backend: mlx (Apple Silicon M1)                                ║
║  ✓ Model: Qwen2.5-Coder-1.5B-Instruct (quantized)                ║
║  ✓ Config: ~/.config/cmdai/config.toml                           ║
║                                                                   ║
║  ⚡ Ready to generate safe commands!                              ║
╚═══════════════════════════════════════════════════════════════════╝

Usage: cmdai [OPTIONS] <PROMPT>

Examples:
  cmdai "list all files"
  cmdai --verbose "find large files"
  cmdai --safety permissive "compress images"

Docs:  https://cmdai.dev/docs
Help:  cmdai --help
```

### Example 5: Error Message

```bash
$ cmdai "xyzabc123"

┌─ cmdai ──────────────────────────────────────────────────┐
│  ✗ Hmm, I couldn't generate a command for that.          │
│                                                           │
│  Your request: "xyzabc123"                                │
│                                                           │
│  This doesn't look like a valid command request.         │
│                                                           │
│  💡 Try being more specific:                             │
│    • "list all PDF files"                                │
│    • "find files larger than 100MB"                      │
│    • "show disk usage"                                   │
│                                                           │
│  Need help? Run: cmdai --help                            │
└───────────────────────────────────────────────────────────┘
```

---

## Web Design Mockups

### Homepage Hero Section

```
═══════════════════════════════════════════════════════════════════
                    [cmdai Logo - Terminal Green]

            ⚡🛡️ cmdai

     AI-Powered Commands. Human-Level Safety.

  Your terminal assistant that validates every command
  before execution. Fast automation without the fear.

     [Try cmdai Now]  [View on GitHub →]  [Read Docs]

═══════════════════════════════════════════════════════════════════

                  [Live Terminal Demo Window]

  ┌─ cmdai ─────────────────────────────────────────┐
  │ $ cmdai "find all PDF files larger than 10MB"   │
  │                                                  │
  │ ✓ Generated:                          [SAFE]    │
  │   find ~ -name "*.pdf" -size +10M                │
  │                                                  │
  │ ⚡ Execute? [Y/n]                                │
  └──────────────────────────────────────────────────┘

              ↓ Watch it in action ↓
           [Animated Demo Video/GIF]

═══════════════════════════════════════════════════════════════════
```

### Feature Grid

```
═══════════════════════════════════════════════════════════════════

                    Why Developers Love cmdai

┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│      ⚡        │  │      🛡️        │  │      🎯        │
│                │  │                │  │                │
│  BLAZING FAST  │  │  ULTRA SAFE    │  │  SMART AI      │
│                │  │                │  │                │
│  <100ms start  │  │  Every command │  │  Local LLM     │
│  <2s inference │  │  validated     │  │  No API keys   │
│  on M1 Mac     │  │  Red/Yellow/   │  │  Works offline │
│                │  │  Green system  │  │                │
└────────────────┘  └────────────────┘  └────────────────┘

┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│      📦        │  │      🔓        │  │      🦀        │
│                │  │                │  │                │
│  SINGLE BINARY │  │  FULLY OPEN    │  │  BUILT IN RUST │
│                │  │                │  │                │
│  No deps       │  │  AGPL-3.0      │  │  Memory safe   │
│  Just download │  │  Transparent   │  │  Zero-cost     │
│  and run       │  │  Community     │  │  abstractions  │
│                │  │  driven        │  │                │
└────────────────┘  └────────────────┘  └────────────────┘

═══════════════════════════════════════════════════════════════════
```

### Testimonial Section (Future)

```
═══════════════════════════════════════════════════════════════════

              What The Community Is Saying

  ┌──────────────────────────────────────────────────────┐
  │  "Finally, an AI tool my security team approves of." │
  │  — Senior DevOps Engineer, Fortune 500              │
  └──────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────┐
  │  "cmdai saved me from `rm -rf /` THREE times. It's   │
  │   like having a senior engineer watching over me."   │
  │  — Junior Developer, Startup                         │
  └──────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────┐
  │  "The safety validation is brilliant. I can let my   │
  │   team use AI tools without losing sleep."           │
  │  — CISO, Tech Company                                │
  └──────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════
```

### Call-to-Action Section

```
═══════════════════════════════════════════════════════════════════

                   Ready to Try cmdai?

           Open Source • Free Forever • No Signup

  ┌─────────────────────────────────────────────────┐
  │  # Install via Homebrew (coming soon)           │
  │  $ brew install cmdai                           │
  │                                                 │
  │  # Or download binary                           │
  │  $ curl -sL cmdai.dev/install.sh | bash        │
  │                                                 │
  │  # Or build from source                         │
  │  $ git clone https://github.com/wildcard/cmdai │
  │  $ cd cmdai && cargo build --release           │
  └─────────────────────────────────────────────────┘

              [Download Now]  [View Docs]

                    ⚡🛡️ cmdai
          Guard Rails for the Fast Lane

═══════════════════════════════════════════════════════════════════
```

---

## Merchandise Designs

### T-Shirt Design 1: "The Classic"

```
───────────────────────────────────────────────────────────────
FRONT (Centered, large):

           ⚡🛡️
         cmdai

  AI-Powered Commands
  Human-Level Safety

───────────────────────────────────────────────────────────────
BACK (Across shoulders):

  GUARD RAILS FOR THE FAST LANE

───────────────────────────────────────────────────────────────
```

### T-Shirt Design 2: "The Safety Matrix"

```
───────────────────────────────────────────────────────────────
FRONT (Left chest pocket area):

  [cmdai]

───────────────────────────────────────────────────────────────
BACK (Full back print):

  ┌─ SAFETY LEVELS ─────────────────────────┐
  │                                         │
  │  SAFE      ▓▓▓▓▓▓▓▓▓▓  [100%]          │
  │  MODERATE  ▓▓▓▓▓▓░░░░  [ 60%]          │
  │  HIGH      ▓▓▓▓░░░░░░  [ 40%]          │
  │  CRITICAL  ▓░░░░░░░░░  [ 10%]          │
  │                                         │
  │         Which level are you?            │
  │                                         │
  │            ⚡🛡️ cmdai                    │
  │                                         │
  └─────────────────────────────────────────┘

───────────────────────────────────────────────────────────────
```

### T-Shirt Design 3: "The Meme"

```
───────────────────────────────────────────────────────────────
FRONT:

  I DON'T ALWAYS RUN
  AI-GENERATED COMMANDS

───────────────────────────────────────────────────────────────
BACK:

  BUT WHEN I DO
  THEY'RE VALIDATED FIRST

  Stay safe, my friends.

         ⚡🛡️ cmdai

───────────────────────────────────────────────────────────────
```

### Sticker Pack (6 designs)

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│             │  │             │  │             │
│   ⚡🛡️      │  │ PROTECTED   │  │  ✓ SAFE     │
│   cmdai     │  │    BY       │  │             │
│             │  │   cmdai     │  │  cmdai      │
│             │  │             │  │             │
└─────────────┘  └─────────────┘  └─────────────┘
  (Logo)         (Badge Style)     (Status)

┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│             │  │             │  │             │
│ GUARD RAILS │  │ THINK FAST  │  │ I VALIDATE  │
│  FOR THE    │  │ STAY SAFE   │  │ MY COMMANDS │
│ FAST LANE   │  │             │  │             │
│             │  │  ⚡🛡️       │  │    ⚡🛡️     │
└─────────────┘  └─────────────┘  └─────────────┘
  (Slogan 1)      (Slogan 2)       (Action)
```

### Coffee Mug Design

```
─────────────────────────────────────────────────────────────
[360° Wrap Design]

LEFT SIDE:
  $ cmdai "make coffee"

FRONT:
  ┌─────────────────────┐
  │  ✓ Generated:       │
  │                     │
  │  brew --strength=   │
  │    extra --temp=hot │
  │                     │
  │  [SAFE] ⚡          │
  └─────────────────────┘

RIGHT SIDE:
  ☕ COMMAND EXECUTED
     SUCCESSFULLY

BOTTOM (visible when drinking):
  ⚡🛡️ Powered by cmdai

─────────────────────────────────────────────────────────────
COLOR OPTIONS:
- Black mug with Terminal Green text
- White mug with Deep Space text
- Terminal Green mug with black text
─────────────────────────────────────────────────────────────
```

### Laptop Sticker (Die-Cut Terminal Window)

```
┌─ cmdai ──────────────────────────┐
│                                  │
│  ⚡ AI-Powered                   │
│  🛡️ Human-Safe                   │
│                                  │
│  Think Fast. Stay Safe.          │
│                                  │
└──────────────────────────────────┘

Size: 3" x 2"
Material: Vinyl, weatherproof
Colors: Terminal Green on transparent
```

---

## Documentation Examples

### Getting Started Page

```markdown
# Getting Started with cmdai

⚡🛡️ Welcome to cmdai - AI-powered commands with human-level safety!

## Quick Start

### Installation

Choose your preferred method:

```bash
# Homebrew (macOS/Linux) - Coming soon
brew install cmdai

# From binary
curl -sL https://cmdai.dev/install.sh | bash

# From source (Rust required)
git clone https://github.com/wildcard/cmdai
cd cmdai
cargo build --release
```

### Your First Command

```bash
$ cmdai "list all files in current directory"

┌─ cmdai ──────────────────────────┐
│  ✓ Generated:           [SAFE]   │
│    ls -la                         │
│                                   │
│  ⚡ Execute? [Y/n]                │
└───────────────────────────────────┘
```

### Understanding Safety Levels

cmdai uses a color-coded safety system:

| Level | Color | Description | Action |
|-------|-------|-------------|--------|
| ✓ **SAFE** | 🟢 Green | No risk detected | Executes freely |
| ⚠ **MODERATE** | 🟡 Yellow | Minor risk (e.g., file deletion) | Asks for confirmation |
| ⚠ **HIGH** | 🟠 Orange | Significant risk | Requires explicit confirmation |
| ✗ **CRITICAL** | 🔴 Red | Dangerous operation | Blocked by default |

---

💡 **Pro Tip:** Use `cmdai --explain` to understand why a command received its safety rating.
```

### API Documentation Example

```markdown
# Backend Configuration API

## Overview

cmdai supports multiple LLM backends through a unified trait system.

```rust
#[async_trait]
pub trait CommandGenerator {
    async fn generate_command(
        &self,
        request: &CommandRequest
    ) -> Result<GeneratedCommand>;

    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

## Backends

### MLX Backend (Apple Silicon)

Optimized for M1/M2/M3 Macs using Metal Performance Shaders.

```toml
[backend]
primary = "mlx"

[backend.mlx]
model_path = "~/.cache/cmdai/models/qwen2.5-coder-1.5b"
quantization = "q4"  # q4, q8, or fp16
```

**Performance:**
- Startup: <100ms
- Inference: <2s
- Memory: ~1.5GB

---

🛡️ **Safety Note:** All backends use the same safety validation system.
```

---

## Social Media Templates

### Twitter/X Post Templates

#### Launch Announcement
```
🚀 Introducing cmdai: AI-powered shell commands with built-in safety.

✓ Validates EVERY command before execution
✓ Local LLM (your data stays private)
✓ <100ms startup time
✓ Open source (AGPL-3.0)

Think fast. Stay safe.

Try it: [link]

#cmdai #OpenSource #AI #CLI
```

#### Feature Highlight
```
⚡ cmdai safety validation in action:

User: "clean up the system"
AI generates: sudo rm -rf /
cmdai: ✗ BLOCKED

Why? System destruction pattern detected.

Your AI copilot with a safety net.

[link] #SafeAI #DevTools
```

#### Community Engagement
```
Poll: What's your biggest fear with AI coding assistants?

○ Deleting important files
○ Breaking production
○ Security vulnerabilities
○ I don't trust AI with my terminal

cmdai validates commands for safety. Sleep better.
```

### LinkedIn Post Template

```
🚀 Why we built cmdai: An AI terminal tool that security teams approve

After watching AI assistants suggest `rm -rf /` one too many times, we
realized the industry needed guardrails.

cmdai validates every AI-generated command before execution:

🛡️ Pattern matching for dangerous operations
⚡ <100ms validation time
🔍 POSIX compliance checking
📊 Risk-level assessment (Red/Yellow/Green)
🏠 Local LLM inference (privacy-first)

Built with Rust. Open source (AGPL-3.0). Free forever.

Because "YOLO" shouldn't be your deployment strategy.

Try it: [link]

#AI #DevOps #OpenSource #CyberSecurity #DeveloperTools

---

What's your experience with AI coding assistants?
Have you ever had a close call with a dangerous command?
```

### GitHub Social Preview Card

```
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║                    ⚡🛡️ cmdai                         ║
║                                                       ║
║       AI-Powered Commands. Human-Level Safety.        ║
║                                                       ║
║  ┌─────────────────────────────────────────────┐     ║
║  │ $ cmdai "find large files"                  │     ║
║  │                                             │     ║
║  │ ✓ Generated:                      [SAFE]   │     ║
║  │   find ~ -size +100M                        │     ║
║  └─────────────────────────────────────────────┘     ║
║                                                       ║
║     Open Source • AGPL-3.0 • Built with Rust         ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝

1280x640px • Deep Space background • Terminal Green accents
```

---

## Email Templates

### Welcome Email (For Newsletter Signups)

```
Subject: Welcome to cmdai - Your terminal just got safer ⚡🛡️

───────────────────────────────────────────────────────────

Hi there! 👋

Welcome to the cmdai community! You're now part of a movement to
make AI-powered terminals both fast AND safe.

🚀 GET STARTED

# Install cmdai
curl -sL https://cmdai.dev/install.sh | bash

# Try your first command
cmdai "list all PDF files"

───────────────────────────────────────────────────────────

🛡️ HOW IT WORKS

1. You describe what you want in plain English
2. cmdai generates a shell command using local AI
3. Every command is validated for safety
4. You approve before execution

No more `rm -rf /` accidents. Ever.

───────────────────────────────────────────────────────────

📚 RESOURCES

• Docs: https://cmdai.dev/docs
• GitHub: https://github.com/wildcard/cmdai
• Community: https://discord.gg/cmdai

───────────────────────────────────────────────────────────

⚡ Think Fast. Stay Safe.

The cmdai Team

P.S. Questions? Just reply to this email!

───────────────────────────────────────────────────────────
```

### Release Announcement Email

```
Subject: cmdai v1.0 is here: AI commands you can trust 🚀

───────────────────────────────────────────────────────────

After months of testing with the community, cmdai v1.0 is
officially here!

🎉 WHAT'S NEW

✓ MLX backend for Apple Silicon (2x faster)
✓ Enhanced safety patterns (42 new dangerous command blocks)
✓ Multi-backend support (Ollama, vLLM)
✓ Single binary under 30MB
✓ 500+ community-contributed test cases

───────────────────────────────────────────────────────────

📊 BY THE NUMBERS

• <100ms startup time ⚡
• <2s inference on M1 Mac
• 98.5% safety accuracy
• 0 false positives in production use

───────────────────────────────────────────────────────────

🔥 UPGRADE NOW

# Homebrew
brew upgrade cmdai

# From binary
curl -sL https://cmdai.dev/install.sh | bash

# From source
git pull && cargo build --release

───────────────────────────────────────────────────────────

💙 THANK YOU

This release wouldn't be possible without our amazing
community of contributors, testers, and early adopters.

You trusted us with your terminals. We took that seriously.

───────────────────────────────────────────────────────────

⚡🛡️ Think Fast. Stay Safe.

The cmdai Team

Full changelog: https://cmdai.dev/changelog

───────────────────────────────────────────────────────────
```

---

## Conference Booth Design

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║                  [CONFERENCE BOOTH LAYOUT]                    ║
║                                                               ║
║  ┌─────────────────────────────────────────────────────────┐ ║
║  │                    OVERHEAD BANNER                       │ ║
║  │                                                          │ ║
║  │           ⚡🛡️ cmdai                                     │ ║
║  │   Guard Rails for the Fast Lane                         │ ║
║  └─────────────────────────────────────────────────────────┘ ║
║                                                               ║
║  ┌──────────┐         ┌──────────┐         ┌──────────┐     ║
║  │ LIVE     │         │  TEAM    │         │  SWAG    │     ║
║  │ DEMO     │         │  AREA    │         │  TABLE   │     ║
║  │          │         │          │         │          │     ║
║  │ Terminal │         │ Talk to  │         │ Stickers │     ║
║  │ running  │         │ creators │         │ T-shirts │     ║
║  │ cmdai    │         │          │         │ Buttons  │     ║
║  └──────────┘         └──────────┘         └──────────┘     ║
║                                                               ║
║  ┌─────────────────────────────────────────────────────────┐ ║
║  │                    FRONT COUNTER                         │ ║
║  │                                                          │ ║
║  │  "Try cmdai - AI commands that won't destroy your OS"   │ ║
║  └─────────────────────────────────────────────────────────┘ ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

COLOR SCHEME:
- Background: Deep Space (#0A0E27)
- Text: Terminal Green (#00FF41)
- Accents: Cyber Cyan (#00D9FF)

INTERACTIVE ELEMENTS:
- Live terminal demo with challenge prompts
- Safety quiz game (win a t-shirt)
- "Dangerous Command Hall of Fame" display
```

---

## Consistent Application

### Brand Checklist

When creating new cmdai materials, ensure:

✓ **Logo:** Use ⚡🛡️ emoji combo or ASCII art version
✓ **Colors:** Stick to Terminal Green, Cyber Cyan, Deep Space
✓ **Safety Levels:** Always use Green/Yellow/Orange/Red system
✓ **Voice:** Confident, helpful, never condescending
✓ **Typography:** Monospace fonts for code/terminal
✓ **Tagline:** Include at least one signature slogan
✓ **CTA:** Clear next action (install, try, read, join)

---

**Remember:** Every interaction with cmdai should reinforce our core
message: AI-powered speed with human-level safety.

⚡🛡️ Think Fast. Stay Safe.
