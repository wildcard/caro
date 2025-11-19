# cmdai Interactive Brand Assets
## Visual, Interactive, Developer-Friendly Reference

> **Open `brand-guide.html` in your browser for the full interactive experience!**

---

## 🎨 What's Here

This directory contains **visual, interactive brand assets** that developers can actually use:

### 1. **brand-guide.html** ⭐ **START HERE**
Interactive HTML brand guidebook with:
- ✅ **Click-to-copy color swatches** (hex, RGB, ANSI codes)
- ✅ **Live terminal output previews** with proper coloring
- ✅ **Copy-paste logos** in multiple formats
- ✅ **Code snippets** for CSS, Bash, Rust, Python
- ✅ **Typography examples** with actual fonts
- ✅ **Safety level visualizations** (Green/Yellow/Orange/Red)
- ✅ **Responsive design** (works on mobile too!)

**How to use:**
```bash
# Open in your default browser
open brand-guide.html

# Or on Linux
xdg-open brand-guide.html

# Or just double-click the file!
```

### 2. **SVG Logo Files**
Web-friendly vector logos that scale perfectly:

#### **logo-minimal.svg** (200x60px)
- Horizontal logo: ⚡🛡️ cmdai
- Perfect for: Headers, navigation bars, README badges
- Transparent background
- Includes glow effect

#### **logo-with-tagline.svg** (500x80px)
- Full logo with tagline
- Perfect for: Landing pages, hero sections, presentations
- Transparent background
- Complete brand message

#### **icon-square.svg** (128x128px)
- Square icon format
- Perfect for: Favicons, avatars, social media profiles, app icons
- Dark background with border
- Centered symbols

**How to use:**
```html
<!-- In HTML -->
<img src="logo-minimal.svg" alt="cmdai" />

<!-- In Markdown -->
![cmdai](logo-minimal.svg)

<!-- As favicon -->
<link rel="icon" href="icon-square.svg" type="image/svg+xml" />
```

---

## 🚀 Quick Start

### For Web Developers

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        /* cmdai brand colors */
        :root {
            --terminal-green: #00FF41;
            --cyber-cyan: #00D9FF;
            --deep-space: #0A0E27;
        }

        body {
            background: var(--deep-space);
            color: var(--terminal-green);
            font-family: 'Monaco', monospace;
        }
    </style>
</head>
<body>
    <img src="logo-minimal.svg" alt="cmdai" />
    <h1 style="color: var(--cyber-cyan);">
        AI-Powered Commands. Human-Level Safety.
    </h1>
</body>
</html>
```

### For Terminal Developers

```bash
# ANSI color codes
GREEN='\033[92m'    # Terminal Green (#00FF41)
CYAN='\033[96m'     # Cyber Cyan (#00D9FF)
YELLOW='\033[93m'   # Warning Amber (#FFB800)
RED='\033[91m'      # Critical Red (#FF0055)
RESET='\033[0m'

# Usage
echo -e "${GREEN}⚡🛡️ cmdai${RESET}"
echo -e "${CYAN}AI-Powered Commands. Human-Level Safety.${RESET}"
```

### For Rust Developers

```rust
use colored::*;

fn main() {
    println!("{}", "⚡🛡️ cmdai".green().bold());
    println!("{}", "AI-Powered Commands. Human-Level Safety.".cyan());

    // Safety levels
    println!("{}", "✓ SAFE".green());
    println!("{}", "⚠ MODERATE".yellow());
    println!("{}", "✗ CRITICAL".red());
}
```

### For Python Developers

```python
from colorama import Fore, Style

# cmdai brand colors
TERMINAL_GREEN = Fore.GREEN
CYBER_CYAN = Fore.CYAN

print(f"{TERMINAL_GREEN}⚡🛡️ cmdai{Style.RESET_ALL}")
print(f"{CYBER_CYAN}AI-Powered Commands. Human-Level Safety.{Style.RESET_ALL}")
```

---

## 🎨 Brand Colors (Quick Reference)

Copy-paste these into your projects:

### Hex Codes
```
Terminal Green:  #00FF41
Cyber Cyan:      #00D9FF
Deep Space:      #0A0E27
Midnight Blue:   #1A1F3A
Warning Amber:   #FFB800
Alert Orange:    #FF6B00
Critical Red:    #FF0055
Silver Frost:    #C0C5D0
Ghost White:     #F0F2F7
Neon Purple:     #B026FF
```

### RGB Values
```
Terminal Green:  rgb(0, 255, 65)
Cyber Cyan:      rgb(0, 217, 255)
Deep Space:      rgb(10, 14, 39)
Warning Amber:   rgb(255, 184, 0)
Alert Orange:    rgb(255, 107, 0)
Critical Red:    rgb(255, 0, 85)
```

### ANSI Codes (Terminal)
```bash
\033[92m  # Green (safe)
\033[96m  # Cyan (info)
\033[93m  # Yellow (moderate)
\033[91m  # Red (critical)
\033[0m   # Reset
```

---

## 💻 Terminal Output Patterns

### Safe Command Box
```
┌─ cmdai ──────────────────────────┐
│  ✓ Generated:           [SAFE]   │
│    find ~/Downloads -name "*.pdf"│
│  ⚡ Execute? [Y/n]                │
└───────────────────────────────────┘
```

### Warning Command Box
```
┌─ cmdai ──────────────────────────┐
│  ⚠ Caution required   [MODERATE] │
│    rm -rf /tmp/*                  │
│  ⚠ Confirm: [y/N]                │
└───────────────────────────────────┘
```

### Blocked Command Box
```
╔═ cmdai ══════════════════════════╗
║  ✗ BLOCKED            [CRITICAL] ║
║    sudo rm -rf /                 ║
║  🛡️  Safety validator: ACTIVE    ║
╚══════════════════════════════════╝
```

**Box Drawing Characters:**
```
─ ═ │ ║ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼
╔ ╗ ╚ ╝ ╠ ╣ ╦ ╩ ╬
```

---

## 🎯 Logos (Copy-Paste)

### Minimal (Inline)
```
⚡🛡️ cmdai
```

### One-Line
```
[cmdai] ▸ AI-Powered. Human-Safe.
```

### Terminal Prompt
```
[cmdai] ▸
```

### ASCII Art (Full)
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

---

## 📐 Safety Level System

Visual representation of risk levels:

```
SAFE      ▓▓▓▓▓▓▓▓▓▓ 100%   #00FF41  (Green)
MODERATE  ▓▓▓▓▓▓░░░░  60%   #FFB800  (Yellow)
HIGH      ▓▓▓▓░░░░░░  40%   #FF6B00  (Orange)
CRITICAL  ▓░░░░░░░░░  10%   #FF0055  (Red)
```

### Status Symbols
```
✓  Safe / Success
✗  Blocked / Error
⚠  Warning / Caution
⚡  Execute / Power
🛡️  Safety / Protection
▸  Prompt / Action
💡 Tip / Suggestion
```

---

## 🔧 Integration Examples

### React Component
```jsx
import React from 'react';

const CmdaiLogo = () => (
  <div style={{
    fontFamily: 'Monaco, monospace',
    color: '#00FF41',
    fontSize: '24px',
    fontWeight: 'bold'
  }}>
    ⚡🛡️ cmdai
  </div>
);

export default CmdaiLogo;
```

### Tailwind CSS
```html
<div class="font-mono text-2xl font-bold" style="color: #00FF41;">
  ⚡🛡️ cmdai
</div>
```

### GitHub README Badge
```markdown
![cmdai](brand-assets/interactive/logo-minimal.svg)

## ⚡🛡️ cmdai
**AI-Powered Commands. Human-Level Safety.**
```

---

## 📱 Responsive Usage

All assets are designed to work on:
- ✅ Desktop browsers
- ✅ Mobile devices
- ✅ Tablets
- ✅ Terminal emulators
- ✅ IDE integrated terminals
- ✅ Documentation sites

### Recommended Sizes

**Logo (minimal):**
- Small: 100px width
- Medium: 200px width (default)
- Large: 400px width

**Logo (with tagline):**
- Small: 250px width
- Medium: 500px width (default)
- Large: 800px width

**Icon (square):**
- Favicon: 32x32px or 64x64px
- Avatar: 128x128px (default)
- Large: 256x256px or 512x512px

---

## 🎨 Typography

### Font Families

**Monospace (code/terminal):**
```css
font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', monospace;
```

**Display (headers/logos):**
```css
font-family: 'Space Mono', monospace;
```

**Body (text):**
```css
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
```

### Google Fonts Import
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=Space+Mono:wght@400;700&display=swap" rel="stylesheet">
```

---

## 🚀 Using the Interactive Guide

### Opening the Guide
```bash
# Option 1: Direct open
open brand-guide.html

# Option 2: Local server (for development)
python3 -m http.server 8000
# Then visit: http://localhost:8000/brand-guide.html

# Option 3: VS Code Live Server extension
# Right-click brand-guide.html → "Open with Live Server"
```

### Features of the Interactive Guide

1. **Click-to-Copy Colors**
   - Click any color swatch to copy hex code
   - Includes hex, RGB, and ANSI values
   - Toast notification confirms copy

2. **Live Terminal Previews**
   - See exactly how terminal output looks
   - Proper ANSI color rendering
   - Examples for all safety levels

3. **Code Snippets**
   - Ready-to-use CSS, Bash, Rust, Python
   - Copy button on each code block
   - Syntax highlighted for readability

4. **Logo Variations**
   - Multiple format options
   - Copy-paste ready ASCII art
   - SVG files linked

5. **Smooth Navigation**
   - Sticky header with quick links
   - Smooth scroll to sections
   - Responsive mobile menu

---

## 📖 Additional Resources

**Related Files:**
- `../BRAND_STYLE_GUIDE.md` - Complete written guide
- `../ASCII_LOGOS.md` - Full ASCII art collection
- `../SLOGANS_AND_MESSAGING.md` - Taglines and messaging
- `../BRAND_APPLICATION_EXAMPLES.md` - Usage examples

**External Tools:**
- [Coolors](https://coolors.co) - Color palette generator
- [SVG Optimizer](https://jakearchibald.github.io/svgomg/) - Optimize SVG files
- [SVGR](https://react-svgr.com/) - Convert SVG to React components

---

## 🤝 Contributing

Found a bug in the interactive guide? Want to add more examples?

1. Open an issue with "brand" label
2. Describe the improvement
3. Submit a PR if you have code changes

**Common improvements:**
- Additional code language examples
- More terminal output patterns
- Better mobile responsiveness
- Accessibility enhancements

---

## 📜 License

All brand assets are part of the cmdai project (AGPL-3.0).

You're free to use these for:
- ✅ cmdai-related projects
- ✅ Documentation and tutorials
- ✅ Community content
- ✅ Merchandise (non-commercial)

---

## ⚡🛡️ Quick Start Checklist

**For your project:**
- [ ] Open `brand-guide.html` in browser
- [ ] Copy color variables from CSS section
- [ ] Download SVG logo files
- [ ] Copy terminal output patterns
- [ ] Implement safety color system
- [ ] Test on mobile devices
- [ ] Share your implementation!

---

**Think Fast. Stay Safe. Brand Consistently.**

*Last updated: 2025-11-19*
*Questions? Open an issue with "brand" label*
