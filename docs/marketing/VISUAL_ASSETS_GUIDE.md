# Caro Visual Assets Guide

## Overview

This document specifies the visual assets required for Caro's multi-channel product announcement, including detailed requirements for each platform and asset type.

---

## Existing Assets Inventory

### Available Now

| Asset | Location | Status | Notes |
|-------|----------|--------|-------|
| Pixel Art Mascot (Static) | `website/public/caro-pixel.png` | Ready | 8-bit style Shiba Inu |
| Mascot Animation | `presentation/public/mascot.gif` | Ready | Speech bubble animation |
| Mascot Loop | `presentation/public/mascot-loop.gif` | Ready | Continuous loop |
| Hero Animation | `website/public/caro-prompting.gif` | Ready | Website hero |
| Asciinema Recording | `demos/asciinema/*.cast` | Ready | Multiple demos |
| VHS Demo Scripts | `demos/vhs/*.tape` | Ready | For GIF generation |

### Need to Create

| Asset | Priority | Purpose |
|-------|----------|---------|
| Product Hunt Hero Image | P0 | PH launch |
| Social Media Cards | P0 | Twitter/LinkedIn |
| Demo Video (30s) | P0 | PH, social |
| Feature Screenshots | P1 | PH gallery |
| Comparison Charts | P1 | Marketing |
| YouTube Tutorial | P2 | Long-form content |

---

## Platform-Specific Requirements

### Product Hunt

#### Hero Image
```
Dimensions: 1200 x 630 pixels
Format: PNG or JPG
File size: < 2MB
Content Requirements:
  - Caro mascot prominently featured
  - Terminal mockup showing command generation
  - Tagline: "Your loyal shell companion"
  - Clean, professional design
  - Brand color: #ff8c42 (orange)
```

**Design Concept:**
```
+--------------------------------------------------+
|                                                  |
|    [CARO MASCOT]      [TERMINAL MOCKUP]          |
|        🐕                                         |
|                   $ caro "find large files"      |
|                   Generated: find . -size +100M  |
|                                                  |
|         "Your loyal shell companion"              |
|                                                  |
+--------------------------------------------------+
```

#### Gallery Images (5-7 required)

**Image 1: Basic Command Generation**
```
Screenshot of terminal showing:
$ caro "list all PDF files in Downloads"

Generated command:
  find ~/Downloads -name "*.pdf" -type f

Execute this command? (y/N)
```

**Image 2: Safety Blocking**
```
Screenshot showing:
$ caro "delete everything in /bin"

🚨 CRITICAL RISK DETECTED

Pattern matched: rm -rf /bin
Risk: Filesystem destruction
Status: BLOCKED

Alternative: Please specify exact files...
```

**Image 3: Platform Detection**
```
Screenshot showing platform awareness:
$ caro --verbose "show CPU info"

Platform detected: macOS 14.2 (arm64)
Shell: zsh
Using macOS-specific commands...

Generated: sysctl -n machdep.cpu.brand_string
```

**Image 4: Multiple Use Cases**
```
Grid showing multiple examples:
├── "find modified files" → find . -mtime -1
├── "disk usage" → du -sh *
├── "git recent" → git log --oneline -10
└── "count lines" → wc -l *.rs
```

**Image 5: Configuration Options**
```
Screenshot of config file:
# ~/.config/caro/config.toml

[backend]
primary = "embedded"
enable_fallback = true

[safety]
enabled = true
level = "moderate"
```

**Image 6: Before/After Comparison**
```
Side-by-side:
BEFORE (Manual):           AFTER (Caro):
$ man find                 $ caro "find files > 100MB"
$ google search...         Generated: find . -size +100M
$ copy-paste...            Execute? (y/N)
$ fix typos...             [Done in 2 seconds]
[15 minutes later]
```

#### Demo Video (30-60 seconds)

**Script/Storyboard:**

```
[0:00-0:03] HOOK
Visual: Terminal with blinking cursor
Audio: "Tired of Googling shell commands?"

[0:03-0:10] PROBLEM
Visual: Show Google search, Stack Overflow tabs
Audio: "We've all spent more time looking up syntax than actually working."

[0:10-0:20] SOLUTION
Visual: Type caro command, show generation
Audio: "Caro transforms natural language into safe shell commands."
Terminal: caro "find Python files modified last week"
         → find . -name "*.py" -mtime -7

[0:20-0:35] SAFETY
Visual: Attempt dangerous command
Audio: "And it keeps you safe."
Terminal: caro "delete everything"
         → 🚨 BLOCKED: rm -rf / detected

[0:35-0:45] KEY FEATURES
Visual: Text overlay with icons
Audio: "100% local. No cloud. Works offline."
Text: ⚡ <2s inference
      🔒 No data leaves your machine
      🛡️ 52 safety patterns

[0:45-0:55] INSTALL
Visual: Installation command
Audio: "Try Caro today."
Terminal: cargo install caro

[0:55-0:60] CTA
Visual: Logo + URL
Audio: "caro.sh - Your loyal shell companion"
```

---

### Twitter/X

#### Image Card
```
Dimensions: 1200 x 675 pixels
Format: PNG
Content:
  - Caro mascot (left side)
  - Terminal mockup (right side)
  - Brief text: "Natural language → Shell commands"
  - URL: caro.sh
```

#### Thread Images

**Image 1: Problem Statement**
```
"Stop Googling shell commands"

Before:
🔍 Google: "how to find files larger than 100MB"
📖 Read 5 Stack Overflow answers
⌨️ Copy, paste, modify
❌ Get syntax error
🔄 Repeat

After:
$ caro "find files larger than 100MB"
→ find . -size +100M
✅ Done
```

**Image 2: Demo GIF**
```
Animated GIF showing 3-4 commands in sequence
Dimensions: 600 x 400 pixels
Duration: 10-15 seconds
Loop: Yes
```

**Image 3: Safety Feature**
```
"Caro keeps you safe"

$ caro "remove all system files"

🚨 BLOCKED
Pattern: rm -rf /
Risk: Critical
Status: Command rejected

"We validate independently. AI can't override safety."
```

**Image 4: Performance Comparison**
```
Inference Speed Comparison:

Caro (local):      ████████░░ ~2s
ChatGPT API:       ████████████████░░ ~5s
Copilot CLI:       ████████████████████░░ ~7s

Plus: Caro works offline! ✈️
```

---

### LinkedIn

#### Post Image
```
Dimensions: 1200 x 627 pixels
Format: PNG
Style: Professional, clean
Content:
  - Title: "Introducing Caro"
  - Subtitle: "Local AI for shell commands"
  - 3-4 key features with icons
  - GitHub/website URL
  - Caro mascot (smaller, corner)
```

---

### GitHub

#### Repository Social Preview
```
Dimensions: 1280 x 640 pixels
Format: PNG
Content:
  - Large "Caro" title
  - Tagline: "Your loyal shell companion"
  - Mascot illustration
  - Key badges: Rust, Open Source, Safety-First
```

#### README Header
```
Format: SVG or high-res PNG
Content:
  - Caro logo/mascot
  - Project name
  - Brief description
  - Badge row (CI, version, license)
```

---

## Demo Content Specifications

### Asciinema Recordings

**Standard Recording Settings:**
```bash
# Terminal size for recordings
stty rows 25 cols 100

# Font size recommendation
# Terminal font: 16pt minimum for readability

# Recording command
asciinema rec demo.cast \
  --idle-time-limit 2 \
  --title "Caro Demo"
```

**Demo Scenarios to Record:**

1. **Quickstart Demo (30s)**
   - Install command
   - First caro command
   - Basic example

2. **Safety Demo (45s)**
   - Safe command
   - Dangerous command (blocked)
   - Alternative suggestion

3. **Multi-Command Demo (60s)**
   - Git commands
   - File operations
   - System information
   - Network commands

4. **Advanced Demo (90s)**
   - Configuration
   - Different backends
   - Platform awareness

### VHS (GIF Generation)

**Example VHS Script:**
```tape
# demos/vhs/caro-quickstart.tape

Output caro-quickstart.gif

Set FontSize 18
Set Width 900
Set Height 500
Set Theme "Dracula"

Type "caro 'find Python files modified today'"
Sleep 500ms
Enter
Sleep 3s

Type "caro 'show disk usage sorted by size'"
Sleep 500ms
Enter
Sleep 3s

Type "caro 'list all listening ports'"
Sleep 500ms
Enter
Sleep 3s
```

**Running VHS:**
```bash
cd demos
make quickstart  # Generate GIF
```

---

## Brand Guidelines

### Colors

| Color | Hex | Usage |
|-------|-----|-------|
| Primary Orange | #ff8c42 | Main brand color, CTAs |
| Secondary Orange | #ff6b35 | Gradients, accents |
| Background Light | #fff8f0 | Light mode bg |
| Background Dark | #1a1a2e | Dark mode bg |
| Text Primary | #2d2d2d | Light mode text |
| Text Dark | #ffffff | Dark mode text |
| Success | #22c55e | Safe commands |
| Warning | #fbbf24 | Moderate risk |
| Danger | #ef4444 | Critical/blocked |

### Typography

| Element | Font | Weight | Size |
|---------|------|--------|------|
| Logo | System/Custom | Bold | 72px |
| Headings | Inter/System | SemiBold | 24-48px |
| Body | Inter/System | Regular | 16-18px |
| Terminal | JetBrains Mono | Regular | 14-16px |

### Logo Usage

```
DO:
✓ Use on white or very light backgrounds
✓ Maintain aspect ratio
✓ Keep clear space around logo
✓ Use provided color versions

DON'T:
✗ Stretch or distort
✗ Add effects (shadow, glow)
✗ Change colors
✗ Place on busy backgrounds
```

---

## Asset Creation Tools

### Recommended Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| Figma | Graphics, layouts | Primary design tool |
| Asciinema | Terminal recording | For demos |
| VHS (Charm) | GIF generation | Terminal GIFs |
| FFmpeg | Video processing | Conversion |
| ImageOptim | Compression | File size reduction |

### Figma Template Structure

```
Caro Marketing Assets/
├── Logos/
│   ├── Logo-Primary
│   ├── Logo-Dark
│   └── Logo-Mono
├── Product Hunt/
│   ├── Hero Image
│   └── Gallery Images (1-7)
├── Social Media/
│   ├── Twitter Cards
│   ├── LinkedIn
│   └── Reddit
├── GitHub/
│   ├── Social Preview
│   └── README Header
├── Screenshots/
│   └── Terminal Mockups
└── Components/
    ├── Mascot
    ├── Terminal Frame
    └── Feature Icons
```

---

## File Naming Convention

```
caro-[platform]-[type]-[variant].[ext]

Examples:
- caro-ph-hero-v1.png
- caro-twitter-card-safety.png
- caro-demo-quickstart.gif
- caro-github-social-preview.png
- caro-screenshot-generation.png
```

---

## Delivery Checklist

### Product Hunt Launch (P0)

- [ ] Hero image (1200x630)
- [ ] Gallery images (5-7 × 1200x900)
- [ ] Demo video (30-60s MP4)
- [ ] Logo (512x512 PNG)
- [ ] Animated GIF demo

### Social Media (P0)

- [ ] Twitter card image
- [ ] Twitter thread images (4-5)
- [ ] Demo GIF for Twitter
- [ ] LinkedIn post image
- [ ] Reddit-ready images

### GitHub (P1)

- [ ] Repository social preview
- [ ] README header graphic
- [ ] Feature screenshots for docs

### Marketing Website (P1)

- [ ] Updated hero animation
- [ ] Feature section icons
- [ ] Comparison graphics
- [ ] Press kit download

---

## Quality Checklist

Before publishing any asset, verify:

- [ ] Correct dimensions for platform
- [ ] File size within limits
- [ ] Text is readable at all sizes
- [ ] Colors match brand guidelines
- [ ] No spelling/grammar errors
- [ ] Links/URLs are correct
- [ ] Tested on light/dark backgrounds
- [ ] Accessible (contrast, alt text)
- [ ] Approved by team

---

*Document Version: 1.0*
*Last Updated: December 2025*
