# 🎮 cmdai Brand Style Guide
## The Terminal Safety Revolution

> **Version:** 1.0.0
> **Last Updated:** 2025-11-19
> **Status:** Official Brand Guidelines

---

## Table of Contents
1. [Brand Essence](#brand-essence)
2. [Visual Identity](#visual-identity)
3. [Color System](#color-system)
4. [Typography & Terminal Design](#typography--terminal-design)
5. [Voice & Tone](#voice--tone)
6. [Brand Messaging](#brand-messaging)
7. [8-Bit Design Language](#8-bit-design-language)
8. [Application Guidelines](#application-guidelines)
9. [Merchandise & Swag](#merchandise--swag)
10. [Web & Digital Presence](#web--digital-presence)

---

## Brand Essence

### Who We Are
**cmdai** is the guardrail for the AI terminal revolution. We're the responsible rebel who lets you run fast but keeps you safe. We're the security team's best friend and the developer's secret weapon.

### Brand Positioning
**"AI-Powered Commands. Human-Level Safety."**

We sit at the intersection of:
- 🚀 **Velocity** × 🛡️ **Security**
- 🤖 **Automation** × 🧠 **Intelligence**
- 🎮 **Fun** × 💼 **Professional**

### Brand Pillars

#### 1. Safety First, Always
- Every command is validated before execution
- Red/Yellow/Green safety system provides instant risk assessment
- We're the adult supervision for YOLO-mode AI agents
- **Tagline:** *"Guard Rails for the Fast Lane"*

#### 2. Lightning Performance
- <100ms startup time (target)
- Single binary, zero hassle
- Built with Rust - memory-safe, blazing-fast
- **Tagline:** *"Instant Safety. Zero Latency."*

#### 3. Radically Open
- AGPL-3.0 licensed
- Transparent validation logic
- Community-driven safety patterns
- Open architecture for custom backends
- **Tagline:** *"Open Source. Open Honest. Open for Business."*

#### 4. Retro-Modern Fusion
- 8-bit aesthetic meets modern CLI power
- Terminal-native, keyboard-first
- Nostalgia with a purpose
- **Tagline:** *"1985 Vibes. 2025 Brains."*

---

## Visual Identity

### Logo System

#### Primary Logo: ASCII Art (Terminal-Native)
```
   ╔═══════════════════════════════════════╗
   ║                                       ║
   ║     ██████╗███╗   ███╗██████╗        ║
   ║    ██╔════╝████╗ ████║██╔══██╗       ║
   ║    ██║     ██╔████╔██║██║  ██║       ║
   ║    ██║     ██║╚██╔╝██║██║  ██║       ║
   ║    ╚██████╗██║ ╚═╝ ██║██████╔╝       ║
   ║     ╚═════╝╚═╝     ╚═╝╚═════╝        ║
   ║                                       ║
   ║         ▲ AI                          ║
   ║                                       ║
   ╚═══════════════════════════════════════╝
          AI-Powered. Human-Safe.
```

#### Compact Logo (Single Line)
```
[cmdai] ▸ AI-Powered Commands. Human-Level Safety.
```

#### Icon/Avatar (8-bit Style)
```
    ▄▄▄▄▄▄▄▄▄▄▄▄▄
   █░░░░░░░░░░░░█
   █░█████░█████░█
   █░█▒▒▒█░█▒▒▒█░█   <-- Safety goggles/shield
   █░█████░█████░█
   █░░░░░░░░░░░░░█
   █░░██░███░██░░█   <-- "cmd" in retro display
   █░░░░░░░░░░░░░█
    ▀▀▀▀▀▀▀▀▀▀▀▀▀
      [cmdai]
```

#### Logo Variations

**Minimal Mark:**
```
⚡🛡️ cmdai
```

**Status Indicators:**
```
✓ cmdai     # Safe command executed
⚠ cmdai     # Warning issued
✗ cmdai     # Command blocked
```

---

## Color System

### Primary Palette

#### Brand Colors (Terminal-Optimized)

| Color Name | Hex Code | ANSI | RGB | Usage |
|------------|----------|------|-----|-------|
| **Terminal Green** | `#00FF41` | `\033[92m` | `0, 255, 65` | Safe commands, success, primary brand |
| **Cyber Cyan** | `#00D9FF` | `\033[96m` | `0, 217, 255` | Interactive elements, links, highlights |
| **Warning Amber** | `#FFB800` | `\033[93m` | `255, 184, 0` | Moderate risk, caution states |
| **Alert Orange** | `#FF6B00` | `\033[91m` | `255, 107, 0` | High risk, important warnings |
| **Critical Red** | `#FF0055` | `\033[91m` | `255, 0, 85` | Blocked commands, errors, danger |

#### Supporting Colors

| Color Name | Hex Code | RGB | Usage |
|------------|----------|-----|-------|
| **Deep Space** | `#0A0E27` | `10, 14, 39` | Background, dark mode base |
| **Midnight Blue** | `#1A1F3A` | `26, 31, 58` | Secondary background |
| **Silver Frost** | `#C0C5D0` | `192, 197, 208` | Primary text (dark bg) |
| **Ghost White** | `#F0F2F7` | `240, 242, 247` | High-contrast text |
| **Neon Purple** | `#B026FF` | `176, 38, 255` | Special features, premium elements |

### Safety Color System

This is our signature visual language:

```
┌─────────────────────────────────────────┐
│ SAFE       ▓▓▓▓▓▓▓▓▓▓ 100%   #00FF41   │  Execute freely
│ MODERATE   ▓▓▓▓▓▓░░░░  60%   #FFB800   │  Proceed with caution
│ HIGH       ▓▓▓▓░░░░░░  40%   #FF6B00   │  Think twice
│ CRITICAL   ▓░░░░░░░░░  10%   #FF0055   │  BLOCKED
└─────────────────────────────────────────┘
```

### Color Usage Rules

1. **Terminal Output:**
   - Use ANSI codes for colored text
   - Ensure 256-color terminal fallbacks
   - Provide plain text mode for accessibility

2. **Web/Marketing:**
   - Primary: Terminal Green + Cyber Cyan
   - Backgrounds: Deep Space + Midnight Blue
   - Accent: Neon Purple (sparingly)

3. **Accessibility:**
   - All color combinations must meet WCAG AA standards (4.5:1 contrast)
   - Never use color alone to convey information
   - Provide text labels + symbols + color

---

## Typography & Terminal Design

### Font Stack

#### Terminal/CLI
```
Primary:   "JetBrains Mono", "Fira Code", "SF Mono", monospace
Fallback:  "Courier New", "Consolas", monospace
```

#### Web/Marketing
```
Headers:   "Space Mono", "Press Start 2P", monospace  # 8-bit feel
Body:      "Inter", "SF Pro", system-ui, sans-serif   # Readable, modern
Code:      "JetBrains Mono", "Fira Code", monospace
```

### Terminal UI Patterns

#### Command Prompt Style
```bash
┌─ cmdai v1.0.0 ─────────────────────────────────┐
│
│ ▸ Your request:
│   "find all PDF files larger than 10MB"
│
│ ✓ Generated command:                    [SAFE]
│   find ~/Downloads -name "*.pdf" -size +10M
│
│ ⚡ Execute? [Y/n/explain]
│
└─────────────────────────────────────────────────┘
```

#### Progress Indicators
```
⠋ Analyzing request...
⠙ Generating command...
⠹ Validating safety...
⠸ Ready to execute!
```

#### Status Blocks
```
╔════════════════════════════════════════════╗
║  STATUS: READY                             ║
║  SAFETY: ✓ VALIDATED                       ║
║  BACKEND: mlx (Apple Silicon)              ║
║  LATENCY: 47ms                             ║
╚════════════════════════════════════════════╝
```

---

## Voice & Tone

### Brand Personality

**The Confident Guardian**
- Experienced but not condescending
- Witty but never reckless
- Technical but accessible
- Protective but empowering

**Think:**
- The senior engineer who teaches you to fish AND makes sure you don't blow up production
- The cool bouncer at the club - lets you have fun but keeps troublemakers out
- The copilot who's got your six

### Voice Attributes

| Attribute | We Are | We Are Not |
|-----------|--------|------------|
| **Humor** | Clever, dry wit, nerdy references | Sarcastic, mean, condescending |
| **Technical** | Precise, accurate, detailed | Jargon-heavy, gatekeeping, elitist |
| **Safety** | Protective, cautionary, responsible | Paranoid, preachy, fearful |
| **Speed** | Fast, efficient, direct | Rushed, careless, shallow |

### Tone by Context

#### CLI Output (Conversational, Helpful)
```bash
# GOOD
✓ Command is safe! Let's roll.

# BAD
✓ The command has been validated and determined to be safe for execution.
```

#### Warning Messages (Direct, Urgent, Clear)
```bash
# GOOD
⚠️  HOLD UP! This command will delete 1,247 files. Type "yes delete" to confirm.

# BAD
⚠️  Warning: The operation you are attempting may result in data loss.
```

#### Error Messages (Empathetic, Actionable)
```bash
# GOOD
✗ Blocked: This looks like a fork bomb. Not today, chaos demon.
  💡 Try: describe what you want to achieve instead

# BAD
✗ Error: Command rejected by safety validator (Code: E-FORKBOMB-001)
```

#### Documentation (Expert, Accessible)
```markdown
# GOOD
cmdai uses a three-tier safety system. Think of it like a security clearance:
- Green = Public access
- Yellow = Need to sign in
- Red = CEO approval required

# BAD
The cmdai safety validation system implements a tiered risk assessment model...
```

### Messaging Examples

#### For Developers
*"Ship faster. Sleep better. Your AI copilot with a safety net."*

#### For Security Teams
*"Let your developers use AI tools without giving you nightmares. Every command validated before execution."*

#### For DevOps/SRE
*"Infrastructure-as-code meets AI-as-assistant. With actual guardrails."*

#### For Executives/CSOs
*"AI acceleration without the risk. Open-source transparency. Enterprise-grade safety."*

---

## Brand Messaging

### Tagline System

#### Primary Tagline
**"AI-Powered Commands. Human-Level Safety."**

#### Alternative Taglines (Context-Dependent)

**Speed-Focused:**
- *"Your terminal. Now with a brain."*
- *"From thought to shell in milliseconds."*
- *"Think it. Type it. Trust it."*

**Safety-Focused:**
- *"The guardrails for YOLO-mode AI."*
- *"Fast and furious? Yes. Fast and dangerous? Never."*
- *"Run wild. Stay safe."*

**Technical/Dev-Focused:**
- *"Local LLM. Global protection."*
- *"Rust-powered safety. AI-driven productivity."*
- *"Single binary. Zero regrets."*

**Enterprise-Focused:**
- *"Compliance-ready AI for your terminal."*
- *"AI that passes your security audit."*
- *"Acceleration with accountability."*

**Community/Open-Source:**
- *"Open source. Open architecture. Open honest."*
- *"Built by the community. For the community."*
- *"AGPL: Our source is your source."*

### Signature Slogans

#### The Classics (Use Everywhere)
1. **"Guard Rails for the Fast Lane"** ← *The safety promise*
2. **"1985 Vibes. 2025 Brains."** ← *The retro-modern fusion*
3. **"Think Fast. Stay Safe."** ← *The core tension*

#### Situational Bangers

**GitHub/Social:**
- *"We put the 'safe' in 'unsafe { }'."* (Rust joke)
- *"Teaching AI agents not to `rm -rf /` since 2024."*
- *"Your terminal's new best friend. And bodyguard."*

**Conference Swag:**
- *"I run AI commands and I don't even yolo"*
- *"cmdai: Because `sudo rm -rf /` should require therapy first"*
- *"Keep Calm and Trust the Safety Validator"*

**Stickers:**
- *"Protected by cmdai"* (like a security badge)
- *"This terminal is under AI surveillance (the good kind)"*
- *"Warning: Guardrails Engaged"*

**T-Shirts:**
```
   [Front]
   ⚡🛡️ cmdai
   AI-Powered. Human-Safe.

   [Back]
   I DON'T ALWAYS RUN AI-GENERATED COMMANDS
   BUT WHEN I DO, THEY'RE VALIDATED FIRST
```

---

## 8-Bit Design Language

### Core Aesthetic Principles

1. **Pixel-Perfect Precision**
   - Everything snaps to 8px grid
   - Use blocky, geometric shapes
   - Embrace terminal character limitations

2. **Retro Gaming References**
   - Command execution = level progression
   - Safety checks = boss battles
   - Successfully validated command = achievement unlocked

3. **ASCII Art as Primary Medium**
   - Logos, icons, decorations all in ASCII
   - Works everywhere (terminal, email, docs)
   - Accessible and hackable

### 8-Bit UI Elements

#### Progress Bars (Game-Style)
```
COMMAND GENERATION
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░  80%  ████  HP

SAFETY VALIDATION
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 100%  ⚡⚡⚡  POWER
```

#### Achievement Badges
```
╔═══════════════════════════╗
║    🏆 ACHIEVEMENT GET!    ║
║                           ║
║   "First Safe Command"    ║
║                           ║
║  You trusted the process  ║
║      and it worked!       ║
╚═══════════════════════════╝
```

#### Dialog Boxes (RPG-Style)
```
┌─────────────────────────────────────────┐
│  CMDAI says:                            │
│                                         │
│  "This command wants to delete your     │
│   home directory. That's gonna be       │
│   a no from me, dawg."                  │
│                                         │
│         [OK]        [Explain]           │
└─────────────────────────────────────────┘
```

#### Stats Display
```
╔══════════════════════════════════════╗
║  CMDAI STATS                         ║
╠══════════════════════════════════════╣
║  Commands Generated:     1,247       ║
║  Disasters Prevented:       42       ║
║  Time Saved:          12.3 hrs       ║
║  Safety Score:            98.5%      ║
╚══════════════════════════════════════╝
```

### Character Set & Symbols

#### Box Drawing (Use Liberally)
```
─ ═ │ ║ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼
╔ ╗ ╚ ╝ ╠ ╣ ╦ ╩ ╬
▀ ▄ █ ▌ ▐ ░ ▒ ▓
```

#### Status Symbols
```
✓ Success
✗ Error
⚠ Warning
⚡ Power/Speed
🛡️ Safety
▸ Action
⟩ Navigate
◆ Info
★ Featured
```

---

## Application Guidelines

### Terminal Output

#### Color Coding
```bash
# Safe command
$ cmdai "list files"
✓ find . -type f -name "*" | head -20        [SAFE]
  ▸ Execute? [Y/n]

# Moderate risk
$ cmdai "delete temp files"
⚠ rm -rf /tmp/*                               [MODERATE]
  ▸ This will delete all temp files. Continue? [y/N]

# Blocked
$ cmdai "format drive"
✗ BLOCKED: mkfs.ext4 /dev/sda                 [CRITICAL]
  ⚡ This command would destroy your disk.
  💡 Try: describe what you actually need to do
```

#### Consistent Layout Pattern
```
┌─ Request ────────────────────────────┐
│ [user's natural language query]     │
├─ Generated ──────────────────────────┤
│ [shell command]               [RISK] │
├─ Safety Analysis ────────────────────┤
│ ✓ No dangerous patterns              │
│ ✓ POSIX compliant                    │
│ ⚠ File deletion (reversible)         │
├─ Action ─────────────────────────────┤
│ ▸ Execute? [Y/n/explain/modify]     │
└───────────────────────────────────────┘
```

### Configuration Files

```toml
# ~/.config/cmdai/config.toml
# cmdai configuration - https://cmdai.dev

[display]
color_scheme = "terminal-classic"  # terminal-classic, cyberpunk, monochrome
show_safety_badges = true
ascii_art = true
nerd_fonts = false  # Use powerline/nerd font symbols
```

---

## Merchandise & Swag

### T-Shirt Designs

#### Design 1: "The Guardian"
```
   [Front - Large centered]

        ⚡🛡️
      cmdai

   AI-Powered Commands
   Human-Level Safety


   [Back - Across shoulders]
   GUARD RAILS FOR THE FAST LANE
```

#### Design 2: "The Safety Matrix"
```
   [Front - Pocket area]
   [cmdai]

   [Back - Full back]

   ┌─ SAFETY LEVELS ────────────┐
   │                            │
   │  SAFE      ▓▓▓▓▓▓▓▓▓▓     │
   │  MODERATE  ▓▓▓▓▓▓░░░░     │
   │  HIGH      ▓▓▓▓░░░░░░     │
   │  CRITICAL  ▓░░░░░░░░░     │
   │                            │
   │  Which level are you?      │
   └────────────────────────────┘
```

#### Design 3: "The Meme"
```
   [Front]

   I DON'T ALWAYS RUN
   AI-GENERATED COMMANDS

   [Back]

   BUT WHEN I DO
   THEY'RE VALIDATED FIRST

   Stay safe, my friends.

   cmdai
```

### Sticker Pack

**Sticker 1: Logo Lockup**
```
Die-cut shape
Terminal Green on transparent
⚡🛡️ cmdai
```

**Sticker 2: Safety Badge**
```
Circular badge design
"PROTECTED BY cmdai"
Shield icon in center
```

**Sticker 3: Status Indicators**
```
Set of 4 small stickers:
✓ SAFE  (Green)
⚠ CAUTION (Yellow)
⚠ WARNING (Orange)
✗ BLOCKED (Red)
```

**Sticker 4: Slogan Series**
- "Guard Rails for the Fast Lane"
- "Think Fast. Stay Safe."
- "1985 Vibes. 2025 Brains."

### Coffee Mugs

**Design: Terminal Wrap**
```
[360° wrap design]

Left side:
$ cmdai "make coffee"

Front:
✓ brew --strength=extra --temperature=hot
  ▸ Execute? [Y/n]

Right side:
☕ COMMAND EXECUTED SUCCESSFULLY

[Interior rim]
"Powered by cmdai"
```

### Laptop Stickers

**Die-Cut Terminal Window**
```
┌─ cmdai ──────────────────┐
│                          │
│  ⚡ AI-Powered           │
│  🛡️ Human-Safe           │
│                          │
│  Think Fast. Stay Safe.  │
│                          │
└──────────────────────────┘
```

### Conference Swag Ideas

1. **"Safety Badge" Lanyards** - Green/Yellow/Orange/Red colored
2. **Pixel Art Enamel Pins** - cmdai logo in 8-bit style
3. **Terminal Keyboard Keycaps** - Custom ESC key with cmdai logo
4. **Microfiber Cloths** - ASCII art pattern
5. **Notebook/Journal** - Terminal-themed ruled paper
6. **Socks** - Pattern of safety indicators up the side

---

## Web & Digital Presence

### Website Design Principles

#### Homepage Hero
```
┌─────────────────────────────────────────────────────────┐
│  [Dark background - Deep Space #0A0E27]                 │
│                                                          │
│         ⚡🛡️ cmdai                                       │
│                                                          │
│    AI-Powered Commands. Human-Level Safety.             │
│                                                          │
│    [Try it now]  [Read docs]  [GitHub ↗]                │
│                                                          │
│                                                          │
│  ┌─ Terminal Demo ─────────────────────────────┐        │
│  │ $ cmdai "find all PDFs larger than 10MB"    │        │
│  │                                              │        │
│  │ ✓ Generated command:               [SAFE]   │        │
│  │   find ~ -name "*.pdf" -size +10M           │        │
│  │                                              │        │
│  │ ⚡ Execute? [Y/n]                            │        │
│  └──────────────────────────────────────────────┘        │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

#### Feature Grid (3-Column)
```
┌────────────┐  ┌────────────┐  ┌────────────┐
│  ⚡        │  │  🛡️        │  │  🎯        │
│  FAST      │  │  SAFE      │  │  SMART     │
│            │  │            │  │            │
│ <100ms     │  │ Every cmd  │  │ Local LLM  │
│ startup    │  │ validated  │  │ inference  │
└────────────┘  └────────────┘  └────────────┘
```

### Social Media Templates

#### Twitter/X Header
```
Dimensions: 1500x500px
Background: Deep Space gradient
Text: "AI-Powered Commands. Human-Level Safety."
Logo: Left side, large ⚡🛡️ cmdai
```

#### LinkedIn Banner
```
Dimensions: 1584x396px
Professional variant
Background: Midnight Blue
Tagline: "Enterprise-Grade AI Command Generation"
Use case icons across bottom
```

#### GitHub Social Preview
```
Dimensions: 1280x640px
Dark background
Terminal window mockup showing command generation
"Open Source • AGPL-3.0 • Community-Driven"
```

### Documentation Style

#### Code Examples
```bash
# Always show full context
$ cmdai "compress all images"

# Generated command:                    [SAFE]
find . -name "*.jpg" -o -name "*.png" | \
  xargs -I {} convert {} -quality 85 {}.compressed.jpg

# Explanation:
• Finds all JPG and PNG files
• Compresses to 85% quality
• Saves as .compressed.jpg
```

#### Callout Boxes
```markdown
> ⚡ **QUICK TIP**
> Use `--verbose` flag to see the full safety analysis.

> 🛡️ **SAFETY NOTE**
> This command is blocked in strict mode by default.

> 💡 **PRO TIP**
> Configure custom safety patterns in config.toml.
```

---

## Brand Don'ts

### Visual Don'ts
❌ Don't use rounded, soft shapes (stay angular, pixel-perfect)
❌ Don't use gradients in terminal output (ANSI limitations)
❌ Don't mix retro aesthetic with modern glossy effects
❌ Don't use the safety colors incorrectly (red = danger ALWAYS)

### Voice Don'ts
❌ Don't be condescending or gatekeep ("actually...")
❌ Don't oversell safety ("100% safe" - nothing is)
❌ Don't use corporate buzzword soup
❌ Don't make light of real security risks

### Messaging Don'ts
❌ Don't claim to replace human judgment
❌ Don't promise AI that "just works" without validation
❌ Don't position as "foolproof" (users aren't fools)
❌ Don't compete with AI assistants (we complement them)

---

## Brand Evolution

### Phase 1: Launch (Current)
- Establish core identity
- Build developer community
- Focus on safety + speed messaging

### Phase 2: Growth
- Expand use case messaging
- Enterprise positioning
- Integration partnerships

### Phase 3: Maturity
- Brand extensions (cmdai ecosystem)
- Platform partnerships
- Community leadership

---

## Quick Reference

### Color Codes (Copy-Paste Ready)
```
Terminal Green: #00FF41
Cyber Cyan:     #00D9FF
Warning Amber:  #FFB800
Alert Orange:   #FF6B00
Critical Red:   #FF0055
Deep Space:     #0A0E27
```

### ASCII Logo (Copy-Paste Ready)
```
⚡🛡️ cmdai
```

### ANSI Color Codes
```bash
GREEN='\033[92m'
CYAN='\033[96m'
YELLOW='\033[93m'
ORANGE='\033[91m'
RED='\033[91m'
RESET='\033[0m'
```

### Brand Hashtags
```
#cmdai
#SafeAI
#TerminalSafety
#AIGuardrails
#RustCLI
```

---

## Contact & Governance

**Brand Guidelines Maintained By:** cmdai Core Team
**Questions/Suggestions:** [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
**Brand Asset Requests:** Open an issue with "brand" label

**License:** This style guide is part of the cmdai project (AGPL-3.0)

---

**Built with Rust** | **Safety First** | **Open Source**

```
┌─────────────────────────────────────────────────┐
│                                                 │
│  Thanks for caring about our brand!             │
│  Now go make something awesome. Safely.         │
│                                                 │
│              ⚡🛡️ cmdai Team                     │
│                                                 │
└─────────────────────────────────────────────────┘
```
