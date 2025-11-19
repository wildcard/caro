# cmdai Terminal Output Specification

> Comprehensive specification for branded terminal output following cmdai's visual identity

## Table of Contents

1. [Overview](#overview)
2. [Color System](#color-system)
3. [Box Drawing Patterns](#box-drawing-patterns)
4. [Safety Level Indicators](#safety-level-indicators)
5. [ASCII Logo Usage](#ascii-logo-usage)
6. [Message Templates](#message-templates)
7. [Error Formatting](#error-formatting)
8. [Success Formatting](#success-formatting)
9. [Progress Indicators](#progress-indicators)
10. [Accessibility Considerations](#accessibility-considerations)

---

## Overview

cmdai's terminal output should be:
- **Visually distinctive** - Retro-futuristic aesthetic with modern clarity
- **Safety-focused** - Color-coded risk levels immediately visible
- **Performant** - Minimal overhead for rendering
- **Accessible** - Works in light/dark terminals, color-blind friendly
- **Consistent** - All output follows the same patterns

### Design Principles

1. **Safety First** - Risk levels always clearly indicated
2. **Information Density** - Show what matters, hide what doesn't
3. **Progressive Disclosure** - More detail with `--verbose`
4. **Human-Readable** - Technical but approachable tone
5. **Terminal-Native** - Use ANSI colors, box-drawing characters

---

## Color System

### ANSI Color Constants

```rust
// Primary colors
pub const TERMINAL_GREEN: &str = "\x1b[92m";    // #00FF41 - Safe operations
pub const CYBER_CYAN: &str = "\x1b[96m";        // #00D9FF - Info/commands
pub const WARNING_AMBER: &str = "\x1b[93m";     // #FFB800 - Moderate risk
pub const ALERT_ORANGE: &str = "\x1b[38;5;208m"; // #FF6B00 - High risk
pub const CRITICAL_RED: &str = "\x1b[91m";      // #FF0055 - Critical/blocked
pub const RESET: &str = "\x1b[0m";

// Secondary colors
pub const DIM: &str = "\x1b[2m";                // Dimmed text
pub const BOLD: &str = "\x1b[1m";               // Bold text
pub const UNDERLINE: &str = "\x1b[4m";          // Underlined text
```

### Color Usage by Context

| Context | Color | Code | Hex |
|---------|-------|------|-----|
| Safe commands | Green | `\x1b[92m` | #00FF41 |
| Command text | Cyan | `\x1b[96m` | #00D9FF |
| Moderate warnings | Yellow | `\x1b[93m` | #FFB800 |
| High risk warnings | Orange | `\x1b[38;5;208m` | #FF6B00 |
| Critical/blocked | Red | `\x1b[91m` | #FF0055 |
| Metadata/timing | Dim | `\x1b[2m` | Dimmed |
| Headers | Bold | `\x1b[1m` | Bold |

### Safety Level Colors

```
SAFE:      ✓ [SAFE]      - Green  (#00FF41)
MODERATE:  ⚠ [MODERATE]  - Yellow (#FFB800)
HIGH:      ⚠ [HIGH]      - Orange (#FF6B00)
CRITICAL:  ✗ [CRITICAL]  - Red    (#FF0055)
```

---

## Box Drawing Patterns

### Character Sets

**Single-line borders** (for normal/safe output):
```
─ ═ │ ║
┌ ┐ └ ┘
├ ┤ ┬ ┴ ┼
```

**Double-line borders** (for critical/blocked output):
```
╔ ╗ ╚ ╝
╠ ╣ ╦ ╩ ╬
```

**Progress/fill characters**:
```
▓ (filled)
░ (empty)
```

### Standard Box Patterns

#### Safe Command Box (Single-line)
```
┌─ cmdai ──────────────────────────────────────────┐
│  Your request: "list all files"                  │
│                                                   │
│  Command: ls -la                                  │
│                                                   │
│  Risk Level: ✓ SAFE                              │
│  ⚡ Execute? [Y/n]                                │
└───────────────────────────────────────────────────┘
```

#### Warning Box (Single-line with color)
```
┌─ cmdai ──────────────────────────────────────────┐
│  ⚠ CAUTION REQUIRED                    [MODERATE] │
│                                                   │
│  Command: rm -rf /tmp/*                           │
│                                                   │
│  This will delete multiple files                 │
│  ⚠  Type "yes" to confirm: _                      │
└───────────────────────────────────────────────────┘
```

#### Blocked Command Box (Double-line)
```
╔═ cmdai ══════════════════════════════════════════╗
║  ✗ COMMAND BLOCKED                    [CRITICAL] ║
║                                                   ║
║  Command: sudo rm -rf /                           ║
║                                                   ║
║  Reason: System destruction pattern detected     ║
║                                                   ║
║  🛡️  This operation would destroy your system.   ║
║                                                   ║
╚═══════════════════════════════════════════════════╝
```

---

## Safety Level Indicators

### Visual Representation

Each safety level has:
1. Symbol (✓, ⚠, ✗)
2. Label ([SAFE], [MODERATE], [HIGH], [CRITICAL])
3. Color (Green, Yellow, Orange, Red)
4. Progress bar representation

### Risk Level Bars

```rust
// Safe (100%)
"▓▓▓▓▓▓▓▓▓▓ 100%"  // Green

// Moderate (60%)
"▓▓▓▓▓▓░░░░  60%"   // Yellow

// High (40%)
"▓▓▓▓░░░░░░  40%"   // Orange

// Critical (10%)
"▓░░░░░░░░░  10%"   // Red
```

### Full Safety Analysis Section

```
├─ Safety Analysis ────────────────────────────────┤
│  ✓ No dangerous patterns detected                │
│  ✓ POSIX compliant                                │
│  ✓ Read-only operation                           │
│  ✓ File paths properly quoted                    │
│                                                   │
│  Risk Level:  ▓▓▓▓▓▓▓▓▓▓ 100%         [SAFE] ✓   │
└───────────────────────────────────────────────────┘
```

---

## ASCII Logo Usage

### Minimal Logo (Most Common)
```rust
"⚡🛡️ cmdai"
```
Use: Inline references, short headers, command prompts

### Compact Header
```
┌─ cmdai ─────────────────┐
```
Use: Box headers, section titles

### Full Logo (Startup/Version)
```
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
╚═══════════════════════════════════════════════════════════════════╝
```
Use: `--version`, startup banner, help screens

---

## Message Templates

### Template 1: Safe Command Generation

```
┌─ cmdai ──────────────────────────────────────────────────┐
│                                                           │
│  ▸ Your request:                                          │
│    "find all PDF files larger than 10MB"                 │
│                                                           │
├─ Generated Command ──────────────────────────────────────┤
│                                                           │
│  find ~/Downloads -type f -name "*.pdf" -size +10M       │
│                                                           │
├─ Safety Analysis ────────────────────────────────────────┤
│  ✓ No dangerous patterns detected                        │
│  ✓ POSIX compliant                                        │
│  ✓ Read-only operation                                   │
│                                                           │
│  Risk Level:  ▓▓▓▓▓▓▓▓▓▓ 100%              [SAFE] ✓      │
│                                                           │
└───────────────────────────────────────────────────────────┘

Performance: 47ms (validation: 12ms, generation: 35ms)
```

### Template 2: Moderate Risk with Confirmation

```
┌─ cmdai ──────────────────────────────────────────────────┐
│                                                           │
│  ▸ Your request:                                          │
│    "delete old log files"                                │
│                                                           │
├─ Generated Command ──────────────────────────────────────┤
│                                                           │
│  find /var/log -name "*.log" -mtime +30 -delete          │
│                                                           │
├─ Safety Analysis ────────────────────────────────────────┤
│  ⚠ File deletion operation (irreversible)                │
│  ✓ Limited scope (/var/log directory)                    │
│  ✓ Time-based filtering (>30 days)                       │
│  ⚠ Estimated files affected: ~127                        │
│                                                           │
│  Risk Level:  ▓▓▓▓▓▓░░░░  60%         [MODERATE] ⚠       │
│                                                           │
├─ Recommendation ─────────────────────────────────────────┤
│  💡 Preview files first with:                            │
│     find /var/log -name "*.log" -mtime +30 -ls           │
│                                                           │
├─ Execute ────────────────────────────────────────────────┤
│  ⚠  Type "yes" to confirm: _                             │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

### Template 3: Blocked Critical Command

```
╔═ cmdai ══════════════════════════════════════════════════╗
║                                                           ║
║  ▸ Your request:                                          ║
║    "wipe the system clean"                               ║
║                                                           ║
╠═ Generated Command ══════════════════════════════════════╣
║                                                           ║
║  sudo rm -rf / --no-preserve-root                         ║
║                                                           ║
╠═ Safety Analysis ════════════════════════════════════════╣
║  ✗ CRITICAL: System destruction pattern                  ║
║  ✗ CRITICAL: Root directory deletion                     ║
║  ✗ CRITICAL: Requires elevated privileges                ║
║  ✗ CRITICAL: Recursive forced removal                    ║
║                                                           ║
║  Risk Level:  ▓░░░░░░░░░  10%         [CRITICAL] ✗       ║
║                                                           ║
╠═ ACTION BLOCKED ═════════════════════════════════════════╣
║                                                           ║
║  🛡️  cmdai has BLOCKED this command for your safety.     ║
║                                                           ║
║  This would permanently destroy your entire system.      ║
║                                                           ║
║  💡 Perhaps you meant to:                                ║
║    • Clean temporary files: "remove temp files"          ║
║    • Free disk space: "show disk usage"                  ║
║    • Clear cache: "clear package cache"                  ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝

Safety validator: ACTIVE • Override: --allow-dangerous (NOT RECOMMENDED)
```

---

## Error Formatting

### General Error Format

```
┌─ cmdai ──────────────────────────────────────────┐
│  ✗ Error: <error_type>                           │
│                                                   │
│  <error_message>                                  │
│                                                   │
│  💡 <helpful_suggestion>                         │
│                                                   │
│  Need help? Run: cmdai --help                    │
└───────────────────────────────────────────────────┘
```

### Specific Error Examples

#### Invalid Input
```
┌─ cmdai ──────────────────────────────────────────┐
│  ✗ Invalid request                                │
│                                                   │
│  Your request: "xyzabc123"                        │
│                                                   │
│  This doesn't look like a command request.       │
│                                                   │
│  💡 Try being more specific:                     │
│    • "list all PDF files"                        │
│    • "find files larger than 100MB"              │
│    • "show disk usage"                           │
└───────────────────────────────────────────────────┘
```

#### Backend Error
```
┌─ cmdai ──────────────────────────────────────────┐
│  ✗ Backend unavailable                            │
│                                                   │
│  Could not connect to Ollama backend at          │
│  http://localhost:11434                           │
│                                                   │
│  💡 Check that Ollama is running:                │
│     ollama serve                                  │
│                                                   │
│  Or configure a different backend:               │
│     cmdai --show-config                          │
└───────────────────────────────────────────────────┘
```

#### Configuration Error
```
┌─ cmdai ──────────────────────────────────────────┐
│  ✗ Configuration error                            │
│                                                   │
│  Invalid safety level: "super-strict"            │
│                                                   │
│  Valid options:                                   │
│    • strict    - Blocks High and Critical        │
│    • moderate  - Blocks Critical only (default)  │
│    • permissive - Warns but allows all           │
│                                                   │
│  Update with: cmdai --safety moderate            │
└───────────────────────────────────────────────────┘
```

---

## Success Formatting

### Command Executed Successfully

```
✓ Command executed successfully

  Output:
  <command_output_here>

Performance: 47ms
```

### Operation Cancelled

```
⚠ Operation cancelled by user

Command was not executed.
```

### Configuration Updated

```
✓ Configuration updated successfully

  Safety level: moderate
  Default shell: bash
  Config file: ~/.config/cmdai/config.toml
```

---

## Progress Indicators

### Validation in Progress

```
⠋ Validating command safety...
```

Animation frames (braille spinner):
```
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

### Generation in Progress

```
⠋ Generating command...
```

### Shield Building Animation

```
Frame 1:  [░░░░░] Initializing safety validator...
Frame 2:  [▓░░░░] Loading patterns...
Frame 3:  [▓▓░░░] Compiling rules...
Frame 4:  [▓▓▓░░] Checking command...
Frame 5:  [▓▓▓▓░] Analyzing risk...
Frame 6:  [▓▓▓▓▓] ✓ Validation complete!
```

---

## Accessibility Considerations

### Color-Blind Friendly Design

The safety system uses multiple indicators:
1. **Symbols**: ✓, ⚠, ✗
2. **Words**: [SAFE], [MODERATE], [HIGH], [CRITICAL]
3. **Colors**: Green, Yellow, Orange, Red
4. **Position**: Risk level always in same location

Even without color support, the output is clear.

### Monochrome Terminal Support

Detect with `$TERM` variable or `NO_COLOR` environment variable.

Fallback rendering:
```
Risk Level: [SAFE] ✓
Risk Level: [MODERATE] ⚠
Risk Level: [HIGH] ⚠!
Risk Level: [CRITICAL] ✗
```

### Light vs Dark Terminal Compatibility

ANSI bright colors (90-97) work well on both:
- Dark backgrounds: Colors are vibrant
- Light backgrounds: Colors remain visible

Test with:
```bash
# Dark terminal
export TERM=xterm-256color

# Light terminal (same colors work)
# Colors auto-adjust based on terminal theme
```

### Screen Reader Support

- Use semantic symbols (✓, ✗, ⚠)
- Include text labels ([SAFE], [CRITICAL])
- Avoid ASCII art for critical information
- Ensure all information is in text form

---

## Performance Guidelines

### Minimal Overhead

- Pre-compile color strings as constants
- Use buffered output for large blocks
- Avoid unnecessary formatting in hot paths
- Lazy-render verbose output

### Efficient Box Drawing

```rust
// Pre-compute box dimensions
let width = 60;
let header = format!("┌─ cmdai {}┐", "─".repeat(width - 10));

// Reuse strings
const SAFE_INDICATOR: &str = "✓ [SAFE]";
const MODERATE_INDICATOR: &str = "⚠ [MODERATE]";
```

### Buffered Output

```rust
use std::io::{self, Write};

let mut stdout = io::stdout().lock();
writeln!(stdout, "┌─ cmdai ───────┐")?;
writeln!(stdout, "│  Output here  │")?;
writeln!(stdout, "└───────────────┘")?;
stdout.flush()?;
```

---

## Implementation Notes

### Rust Crates to Use

- `colored` (v2.1+) - ANSI color support
- `console` (v0.15+) - Advanced terminal features
- `indicatif` (v0.17+) - Progress bars/spinners
- `dialoguer` (v0.11+) - User confirmations (already in use)
- `textwrap` (v0.16+) - Text wrapping in boxes

### Feature Flags

```toml
[features]
fancy-output = ["colored", "indicatif"]
minimal = []  # Plain text only
```

### Environment Variables

```bash
NO_COLOR=1          # Disable all colors
CMDAI_PLAIN=1       # Disable box drawing, use plain text
CMDAI_VERBOSE=1     # Show debug info by default
```

---

## Testing Guidelines

### Manual Testing Checklist

- [ ] Test in iTerm2 (macOS)
- [ ] Test in Terminal.app (macOS)
- [ ] Test in GNOME Terminal (Linux)
- [ ] Test in Windows Terminal
- [ ] Test with dark theme
- [ ] Test with light theme
- [ ] Test with `NO_COLOR=1`
- [ ] Test with limited width (80 columns)
- [ ] Test with wide terminal (200+ columns)

### Unit Test Examples

```rust
#[test]
fn test_safe_command_box() {
    let output = format_safe_command("ls -la");
    assert!(output.contains("✓ SAFE"));
    assert!(output.contains("┌─ cmdai"));
}

#[test]
fn test_blocked_command_box() {
    let output = format_blocked_command("rm -rf /");
    assert!(output.contains("✗ CRITICAL"));
    assert!(output.contains("╔═ cmdai"));
}
```

---

## Examples in Context

### Full Workflow Example

```
$ cmdai "find all PDFs larger than 10MB"

┌─ cmdai ──────────────────────────────────────────────────┐
│                                                           │
│  ▸ Your request:                                          │
│    "find all PDFs larger than 10MB"                      │
│                                                           │
├─ Generated Command ──────────────────────────────────────┤
│                                                           │
│  find ~ -type f -name "*.pdf" -size +10M                 │
│                                                           │
├─ Safety Analysis ────────────────────────────────────────┤
│  ✓ No dangerous patterns detected                        │
│  ✓ POSIX compliant                                        │
│  ✓ Read-only operation                                   │
│                                                           │
│  Risk Level:  ▓▓▓▓▓▓▓▓▓▓ 100%              [SAFE] ✓      │
│                                                           │
└───────────────────────────────────────────────────────────┘

Performance: 47ms (validation: 12ms, generation: 35ms)
Backend: mlx (Apple Silicon)

⚡ Execute this command? [Y/n] █
```

### Version/Help Display

```
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
```

---

## Future Enhancements

### Phase 2 Features

1. **Rich Terminal Support**
   - Use `ratatui` for advanced TUI
   - Interactive command editing
   - Real-time safety updates

2. **Themes**
   - Classic (current spec)
   - Minimal (plain text)
   - Matrix (green phosphor)
   - Cyberpunk (neon colors)

3. **Animation**
   - Smooth progress bars
   - Typing effect for commands
   - Fade-in for recommendations

4. **Sound** (optional)
   - Success chime
   - Warning beep
   - Critical alert

---

## Conclusion

This specification provides a complete visual system for cmdai's terminal output. The design balances:

- **Safety** - Clear risk indicators at all times
- **Aesthetics** - Retro-futuristic terminal charm
- **Performance** - Fast rendering, minimal overhead
- **Accessibility** - Works for all users and terminals

Implementation should follow these patterns consistently to create a cohesive, branded experience that reinforces cmdai's core value: AI-powered commands with human-level safety.

---

**Version**: 1.0.0
**Last Updated**: 2025-11-19
**Author**: cmdai Core Team

⚡🛡️ Think Fast. Stay Safe.
