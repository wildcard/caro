# Terminal UI Constraints & Guidelines

> **Technical requirements for designing cmdai's terminal interface**

---

## 🖥️ Overview

cmdai is **terminal-first**. Every design decision must respect the constraints and capabilities of terminal emulators while maximizing delight and usability.

**Philosophy:** "Make the terminal delightful" by embracing constraints, not fighting them.

---

## ✅ Must-Haves (Required)

### 1. Monospace Only

**All text must use monospace/fixed-width fonts.**

**Why:** Terminals only support monospace rendering. Any proportional font will break alignment.

**Allowed fonts:**
- System default (usually Courier/Consolas/Menlo)
- Custom bitmap fonts (if user configures terminal)
- Nerd Fonts / Powerline glyphs (if available)

### 2. ANSI Color Support

**Use ANSI escape codes for colors.**

**Standard 16-color palette:**
```
Black:   \e[30m    Bright Black:   \e[90m
Red:     \e[31m    Bright Red:     \e[91m
Green:   \e[32m    Bright Green:   \e[92m
Yellow:  \e[33m    Bright Yellow:  \e[93m
Blue:    \e[34m    Bright Blue:    \e[94m
Magenta: \e[35m    Bright Magenta: \e[95m
Cyan:    \e[36m    Bright Cyan:    \e[96m
White:   \e[37m    Bright White:   \e[97m
```

**256-color support** (when available):
```
\e[38;5;<num>m  # Foreground
\e[48;5;<num>m  # Background
```

**True color (24-bit)** (when available):
```
\e[38;2;<r>;<g>;<b>m  # Foreground
\e[48;2;<r>;<g>;<b>m  # Background
```

**Fallback strategy:**
1. Try 24-bit color
2. Fall back to 256-color
3. Fall back to 16-color
4. Fall back to no color (monochrome)

### 3. Box Drawing Characters

**Use Unicode box-drawing for borders and UI elements.**

**Common characters:**
```
┌ ┐ └ ┘  # Corners
│ ─       # Lines
├ ┤ ┬ ┴  # T-junctions
┼         # Cross
═ ║       # Double lines
```

**Example usage:**
```
┌─────────────────┐
│ cmdai v1.0      │
├─────────────────┤
│ > Command here  │
└─────────────────┘
```

### 4. ASCII Fallback

**Every UI element must have an ASCII fallback.**

**For terminals that don't support:**
- Unicode box drawing
- ANSI colors
- Special characters

**Example:**
```
Unicode:  ┌───┐
          │   │
          └───┘

ASCII:    +---+
          |   |
          +---+
```

### 5. Powerline Glyphs (Optional Enhancement)

**If Nerd Font is available:**
- Use Powerline symbols for enhanced UI
-  (solid arrow)
-  (line arrow)
-  (flame)
- 類 (other symbols)

**Always provide fallback:**
```
Powerline:  >
Fallback:   > >
```

---

## ❌ No-Go Rules (Forbidden)

### 1. No Gradients ❌

**Gradients don't work in terminals.**

**Why:** ANSI colors are discrete, not continuous.

**Workaround:** Use dithering or solid color blocks.

### 2. No Photos/Raster Images ❌

**Terminals can't display images** (except in very specific terminals like iTerm2 with special protocols).

**Why:** Text-only rendering.

**Workaround:** Use ASCII art, ANSI art, or Unicode symbols.

### 3. No Non-Monospace Fonts ❌

**Proportional fonts break layout.**

**Why:** Terminals assume every character is the same width.

**Result:** Misaligned columns, broken tables, ugly UI.

### 4. No Shadows/Blur/Translucency ❌

**Effects don't render in terminals.**

**Why:** ANSI doesn't support opacity or blur.

**Workaround:** Use solid colors and clear outlines.

### 5. No Complex Pixel Art ❌

**Intricate pixel art doesn't translate well.**

**Why:** Limited resolution and color.

**Workaround:** Keep pixel art simple, high-contrast, and recognizable at small sizes.

### 6. No Diagonal Antialiased Shapes ❌

**Smooth diagonals don't work.**

**Why:** Monospace grid creates jagged edges.

**Workaround:** Use vertical/horizontal lines or accept jaggedness.

### 7. No Emoji in UI ❌

**Emoji are inconsistent across terminals.**

**Why:** Different rendering engines, different widths, different styles.

**Allowed:** Emoji in user content (messages, logs)
**Forbidden:** Emoji as UI elements (buttons, icons)

**Exception:** Kyaro's ASCII/Unicode representation uses simple characters.

---

## ⚠️ Allowed With Care

### 1. Simple Dithering ✅

**Basic dithering patterns for texture:**

```
░ ▒ ▓ █  # Shade characters
```

**Example:**
```
Light:  ░░░░
Medium: ▒▒▒▒
Dark:   ▓▓▓▓
Solid:  ████
```

### 2. Fancy TUI Components ✅

**Use libraries like Bubble Tea (Go) for rich TUI:**
- Tables
- Lists
- Input fields
- Progress bars
- Spinners

**But:** Must feel terminal-native, not like a GUI app.

### 3. Longer Animations ✅

**ANSI-based animations are allowed:**
- Spinner: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
- Progress bar: [▓▓▓░░░░░░░]
- Kyaro state transitions

**Keep it:**
- Smooth (60fps if possible)
- Interruptible (Ctrl+C should work)
- Performance-conscious (low CPU)

### 4. Unicode Symbols ✅

**Safe symbols that work in most terminals:**
- ✓ ✗ ⚠ ⚡ 🔒 🔓
- ▶ ◀ ▲ ▼
- ● ○ ◆ ◇
- ★ ☆

**Test across terminals:**
- macOS Terminal
- iTerm2
- Windows Terminal
- Linux terminal emulators

---

## 🎨 Color Palette Strategy

### Terminal-Safe Palette

**Base colors (always work):**
```
Background: ANSI Black or Default
Foreground: ANSI White or Default
Primary:    ANSI Bright Green (92)
Secondary:  ANSI Bright Blue (94)
Warning:    ANSI Bright Yellow (93)
Error:      ANSI Bright Red (91)
```

### Enhanced Palette (256-color)

**cmdai extended palette:**
```
--terminal-bg-dark:      #0f0f23 (ANSI 233)
--terminal-fg-light:     #9bbc0f (ANSI 148)
--terminal-accent-1:     #00ff41 (ANSI 47)
--terminal-accent-2:     #00f0ff (ANSI 51)
--terminal-warning:      #ffb000 (ANSI 214)
--terminal-error:        #ff3b3b (ANSI 203)
```

### Game Boy Theme

**Authentic GB colors:**
```
--gb-dark:    #0f380f (ANSI 22)
--gb-medium:  #306230 (ANSI 65)
--gb-light:   #8bac0f (ANSI 106)
--gb-white:   #9bbc0f (ANSI 148)
```

### User Themes

**Configurable via:**
- Environment variables
- Config file (`~/.config/cmdai/theme.toml`)
- Command-line flags

**Example:**
```toml
[theme]
name = "retro-green"
primary = "green"
secondary = "cyan"
style = "gameboy"
```

---

## 📐 Layout Guidelines

### Grid System

**Respect character grid:**
- Width: 80 columns (min), 120 columns (ideal)
- Height: 24 rows (min), 40 rows (comfortable)

**Box sizing:**
```
┌────────────────────────┐  # 24 chars wide
│ Title                  │
├────────────────────────┤
│ Content here           │
│ More content           │
└────────────────────────┘
```

### Alignment

**Everything aligns to character grid:**
- Left-align most content
- Right-align numbers/timestamps
- Center-align titles (sparingly)

**Example:**
```
Command:     find . -name "*.rs"
Status:      ✓ Success
Time:        1.2s
Lines:       42
```

### Spacing

**Use consistent spacing:**
- 1 line between sections
- 2 lines between major blocks
- Indent: 2 or 4 spaces (configurable)

---

## 🎭 Kyaro in Terminal

### ANSI Art Kyaro

**Small (7 lines):**
```
    /\_/\
   ( o.o )
    > ^ <
   /|   |\
  (_|   |_)
```

**With color (16-color):**
```
\e[93m    /\_/\    \e[0m
\e[93m   ( \e[97mo.o\e[93m )   \e[0m
\e[93m    > ^ <    \e[0m
\e[93m   /|\e[97m   \e[93m|\   \e[0m
\e[93m  (_|   |_)  \e[0m
```

### Kyaro States (Terminal)

**Idle:**
```
( o.o )
```

**Thinking:**
```
( -.- )
```

**Success:**
```
( ^.^ )
```

**Warning:**
```
( O.O )
```

**Error:**
```
( ;_; )
```

### Integration Example

```
┌─────────────────────────┐
│ cmdai v1.0              │
│                         │
│     ( ^.^ )  Ready!     │
│                         │
│ > _                     │
└─────────────────────────┘
```

---

## 🔧 Technical Implementation

### Color Detection

**Check terminal capabilities:**
```rust
use termcolor::{ColorChoice, StandardStream};

let stdout = StandardStream::stdout(ColorChoice::Auto);

// Auto-detect:
// - Always: 16-color ANSI
// - Sometimes: 256-color
// - Rarely: True color (24-bit)
```

### Resolution Detection

**Query terminal size:**
```rust
use terminal_size::{Width, Height, terminal_size};

if let Some((Width(w), Height(h))) = terminal_size() {
    println!("Width: {}, Height: {}", w, h);
}
```

### Unicode Support Detection

**Test if Unicode works:**
```rust
// Try to print a Unicode box character
// If it renders correctly, Unicode is supported
// Otherwise, fall back to ASCII
```

---

## 🧪 Testing Strategy

### Test Matrix

**Minimum test terminals:**
- macOS Terminal (default)
- iTerm2 (enhanced)
- Windows Terminal (modern)
- Windows CMD (legacy)
- Linux xterm (standard)
- Linux Alacritty (GPU-accelerated)

### Test Checklist

For each terminal:
- [ ] Colors render correctly
- [ ] Box drawing works or falls back to ASCII
- [ ] Kyaro ASCII art is recognizable
- [ ] Animations are smooth
- [ ] No visual glitches
- [ ] Keyboard shortcuts work
- [ ] Resizing handles gracefully

### Automated Testing

```rust
#[test]
fn test_ansi_fallback() {
    // Test that ANSI codes degrade gracefully
    let output = render_with_ansi_disabled();
    assert!(output.contains("+---+"));  // ASCII box
}

#[test]
fn test_kyaro_ascii() {
    let kyaro = render_kyaro_ascii();
    assert!(kyaro.contains("( o.o )"));
}
```

---

## 📚 Resources

### Tools

- **ANSI Escape Code Tester:** https://github.com/fidian/ansi
- **Box Drawing Reference:** https://en.wikipedia.org/wiki/Box-drawing_character
- **Nerd Fonts:** https://www.nerdfonts.com/
- **Powerline Glyphs:** https://github.com/powerline/powerline

### Libraries (Rust)

- `colored` - ANSI color support
- `termcolor` - Cross-platform colors
- `crossterm` - Terminal manipulation
- `tui-rs` / `ratatui` - Terminal UI framework
- `console` - Styled terminal output

### Inspiration

- **Bubble Tea (Go):** https://github.com/charmbracelet/bubbletea
- **Ink (Node.js):** https://github.com/vadimdemedes/ink
- **Charm CLI:** https://charm.sh/

---

## ✅ Checklist for Aci (Art Director)

When designing terminal UI:
- [ ] Only use monospace fonts
- [ ] Stick to ANSI color palette
- [ ] Provide ASCII fallback for all Unicode
- [ ] Test in at least 3 different terminals
- [ ] No gradients, shadows, or blur
- [ ] Keep Kyaro simple and recognizable
- [ ] Respect 80-column layout
- [ ] Box drawing or ASCII boxes only
- [ ] Emoji for content, not UI
- [ ] Document color codes (ANSI numbers)

---

**Terminal-first design = delight within constraints.** 🖥️✨
