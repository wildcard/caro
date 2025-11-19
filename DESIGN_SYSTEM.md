# cmdai Brand Design System
## Terminal-Inspired Aesthetic for All Platforms

> **Created by**: Claude (TUI Showcase Architect)
> **Version**: 1.0.0
> **Last Updated**: 2025-01-19
> **Purpose**: Comprehensive design system extending the TUI aesthetic to web, documentation, and all brand touchpoints

---

## 🎨 Vision Statement

**cmdai's visual identity is rooted in the golden age of computing** - when terminals were the primary interface and every character mattered. We celebrate this heritage while creating a modern, accessible, and delightful user experience.

**Core Aesthetic**: Retro-futuristic terminal UI with warm, approachable 8-bit influences

**Brand Personality**: Trustworthy • Nostalgic • Precise • Empowering • Playful

---

## 🎯 Design Principles

### 1. **Terminal-First Thinking**
Everything starts from how it would work in a terminal, then adapts to other mediums. The terminal is not a constraint - it's our superpower.

### 2. **Information Density with Clarity**
Pack information efficiently like a well-crafted TUI, but never sacrifice readability. Every element earns its space.

### 3. **Monospace is Beautiful**
Monospace fonts aren't just functional - they're aesthetic. Embrace the grid, celebrate alignment, make characters dance.

### 4. **Color as Semantic Signal**
Colors have meaning inherited from terminal conventions:
- Green = success, safe, go
- Red = error, danger, stop
- Yellow = warning, caution, attention needed
- Cyan = information, navigation, headers
- Magenta = special, keywords, emphasis

### 5. **ASCII Art Pride**
Box-drawing characters, Unicode symbols, and ASCII art aren't limitations - they're features. Use them with pride.

### 6. **Progressive Enhancement**
Core experience works everywhere (even in a basic terminal). Enhanced experiences add delight without excluding anyone.

---

## 🎨 Color Palette

### Primary Colors

#### Terminal Black
- **Hex**: `#1E1E1E` (background)
- **Hex**: `#2D2D2D` (elevated surfaces)
- **Hex**: `#3D3D3D` (borders, dividers)
- **Usage**: Backgrounds, dark mode primary

#### Terminal White
- **Hex**: `#E0E0E0` (primary text)
- **Hex**: `#CCCCCC` (secondary text)
- **Hex**: `#999999` (tertiary text/disabled)
- **Usage**: Text, light mode backgrounds

### Semantic Colors

#### Success Green
- **Hex**: `#50FA7B` (bright green - primary)
- **Hex**: `#5AF78E` (hover state)
- **Hex**: `#45E070` (active state)
- **Usage**: Success states, safe commands, checkmarks, confirmation buttons

#### Warning Yellow
- **Hex**: `#F1FA8C` (bright yellow - primary)
- **Hex**: `#FFE66D` (hover state)
- **Hex**: `#E5D352` (active state)
- **Usage**: Warnings, moderate risk, caution indicators

#### Error Red
- **Hex**: `#FF5555` (bright red - primary)
- **Hex**: `#FF6E67` (hover state)
- **Hex**: `#E63946` (active state)
- **Usage**: Errors, dangerous commands, critical warnings, blocked actions

#### Info Cyan
- **Hex**: `#8BE9FD` (bright cyan - primary)
- **Hex**: `#9AEDFE` (hover state)
- **Hex**: `#70D4ED` (active state)
- **Usage**: Information, navigation, headers, links, highlights

#### Special Magenta
- **Hex**: `#FF79C6` (bright magenta - primary)
- **Hex**: `#FF8BD0` (hover state)
- **Hex**: `#E568B3` (active state)
- **Usage**: Special states, keywords, model attribution, premium features

#### Neutral Purple
- **Hex**: `#BD93F9` (bright purple - primary)
- **Hex**: `#C9A5FA` (hover state)
- **Hex**: `#A67FE5` (active state)
- **Usage**: Secondary actions, alternative states

### Accent Colors

#### Orange (New!)
- **Hex**: `#FFB86C` (bright orange)
- **Usage**: High-priority warnings (between yellow and red)

#### Blue (Rare Use)
- **Hex**: `#6272A4` (muted blue)
- **Usage**: Paths, file references, secondary information

---

## 🔤 Typography

### Primary Font: JetBrains Mono

**Why**: Excellent readability, beautiful ligatures, specifically designed for code

**Web Fallback Stack**:
```css
font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Cascadia Code',
             'Consolas', 'Monaco', monospace;
```

**Usage**:
- All code displays
- Primary body text on developer-facing pages
- Terminal simulation elements

### Secondary Font: IBM Plex Mono

**Why**: Slightly warmer, excellent for longer reading, IBM heritage aligns with our retro aesthetic

**Web Fallback Stack**:
```css
font-family: 'IBM Plex Mono', 'Roboto Mono', 'Courier New', monospace;
```

**Usage**:
- Marketing copy
- Documentation prose
- Blog posts

### Display Font: Press Start 2P (8-bit Style)

**Why**: Pure nostalgia, perfect for headers and branding moments

**Usage** (SPARINGLY):
- Hero headlines
- Section headers on marketing pages
- Logo lockups
- Easter eggs

```css
font-family: 'Press Start 2P', 'VT323', cursive;
```

### Font Sizes (Web)

```css
--text-xs: 0.75rem;    /* 12px - small labels */
--text-sm: 0.875rem;   /* 14px - secondary text */
--text-base: 1rem;     /* 16px - body text */
--text-lg: 1.125rem;   /* 18px - emphasized text */
--text-xl: 1.25rem;    /* 20px - small headings */
--text-2xl: 1.5rem;    /* 24px - section headings */
--text-3xl: 1.875rem;  /* 30px - page headings */
--text-4xl: 2.25rem;   /* 36px - hero text */
--text-5xl: 3rem;      /* 48px - large hero (8-bit font) */
```

---

## 📐 Layout & Spacing

### Grid System

Based on **8px base unit** (matching terminal character spacing):

```css
--space-1: 0.5rem;   /* 8px */
--space-2: 1rem;     /* 16px */
--space-3: 1.5rem;   /* 24px */
--space-4: 2rem;     /* 32px */
--space-6: 3rem;     /* 48px */
--space-8: 4rem;     /* 64px */
--space-12: 6rem;    /* 96px */
--space-16: 8rem;    /* 128px */
```

### Terminal-Inspired Containers

**Code Block / Terminal Window**:
```css
.terminal-window {
  background: #1E1E1E;
  border: 2px solid #3D3D3D;
  border-radius: 8px; /* Subtle, not too modern */
  padding: var(--space-4);
  font-family: 'JetBrains Mono', monospace;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4);
}
```

**Card / Panel**:
```css
.panel {
  background: #2D2D2D;
  border: 1px solid #3D3D3D;
  border-radius: 4px;
  padding: var(--space-3);
}
```

**Focus States** (Accessibility!):
```css
:focus-visible {
  outline: 2px solid #8BE9FD;
  outline-offset: 2px;
  border-radius: 2px;
}
```

---

## 🎭 Component Patterns

### Buttons

#### Primary Button (Action)
```css
.btn-primary {
  background: #50FA7B;
  color: #1E1E1E;
  border: 2px solid #50FA7B;
  padding: 0.75rem 1.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary:hover {
  background: #5AF78E;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(80, 250, 123, 0.3);
}

.btn-primary:active {
  transform: translateY(0);
}
```

#### Secondary Button
```css
.btn-secondary {
  background: transparent;
  color: #8BE9FD;
  border: 2px solid #8BE9FD;
  /* ... similar structure */
}
```

#### Danger Button
```css
.btn-danger {
  background: #FF5555;
  color: #1E1E1E;
  border: 2px solid #FF5555;
  /* ... */
}
```

### Code Blocks

```css
.code-block {
  background: #1E1E1E;
  border-left: 4px solid #50FA7B; /* Accent color based on context */
  padding: var(--space-3);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.875rem;
  line-height: 1.6;
  overflow-x: auto;
}

.code-block--error {
  border-left-color: #FF5555;
}

.code-block--warning {
  border-left-color: #F1FA8C;
}
```

### Tables (Data Display)

```css
.data-table {
  width: 100%;
  border-collapse: collapse;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.875rem;
}

.data-table thead {
  background: #2D2D2D;
  color: #8BE9FD;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.data-table th,
.data-table td {
  padding: var(--space-2);
  text-align: left;
  border-bottom: 1px solid #3D3D3D;
}

.data-table tbody tr:hover {
  background: #2D2D2D;
}

.data-table tbody tr.selected {
  background: #3D3D3D;
  border-left: 4px solid #8BE9FD;
}
```

### Input Fields

```css
.input {
  background: #2D2D2D;
  border: 2px solid #3D3D3D;
  color: #E0E0E0;
  padding: 0.75rem 1rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 1rem;
  border-radius: 4px;
  transition: border-color 0.15s ease;
}

.input:focus {
  border-color: #8BE9FD;
  outline: none;
}

.input::placeholder {
  color: #999999;
}

.input--error {
  border-color: #FF5555;
}
```

### Badges / Status Indicators

```css
.badge {
  display: inline-block;
  padding: 0.25rem 0.75rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-radius: 4px;
}

.badge--success {
  background: #50FA7B;
  color: #1E1E1E;
}

.badge--warning {
  background: #F1FA8C;
  color: #1E1E1E;
}

.badge--error {
  background: #FF5555;
  color: #FFFFFF;
}

.badge--info {
  background: #8BE9FD;
  color: #1E1E1E;
}
```

---

## 🎨 ASCII Art & Unicode Elements

### Box Drawing Characters

Use these for borders, separators, and visual structure:

```
┌─┬─┐  ╔═╦═╗  ╭─┬─╮
│ │ │  ║ ║ ║  │ │ │
├─┼─┤  ╠═╬═╣  ├─┼─┤
│ │ │  ║ ║ ║  │ │ │
└─┴─┘  ╚═╩═╝  ╰─┴─╯
```

**Usage**: Section dividers, table borders, terminal window frames

### Icons & Symbols

```
✓  Checkmark (success)
✗  X-mark (error/blocked)
⚠  Warning triangle
ℹ  Information
▶  Play/select/next
◀  Previous
▲  Upvote
▼  Downvote
⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏  Braille spinner frames
→  Arrow right
←  Arrow left
│  Vertical bar (separator)
─  Horizontal bar
```

### Progress Indicators

```
[████████░░] 80%
[=========>] Loading...
⠋ Processing...
```

---

## 🌐 Web Component Examples

### Hero Section (Homepage)

```html
<section class="hero">
  <div class="hero__terminal">
    <div class="terminal-header">
      <span class="terminal-title">cmdai</span>
      <div class="terminal-controls">
        <span class="control control--close"></span>
        <span class="control control--minimize"></span>
        <span class="control control--maximize"></span>
      </div>
    </div>
    <div class="terminal-body">
      <div class="terminal-line">
        <span class="prompt">$</span>
        <span class="command">cmdai "list all PDF files larger than 10MB"</span>
      </div>
      <div class="terminal-line">
        <span class="output">Generated command:</span>
      </div>
      <div class="terminal-line terminal-line--highlight">
        <span class="prompt">$</span>
        <span class="command-output">find . -name '*.pdf' -size +10M -ls</span>
      </div>
      <div class="terminal-line">
        <span class="success">✓ SAFE</span>
        <span class="output"> - This command is safe to execute</span>
      </div>
    </div>
  </div>
  <h1 class="hero__title">
    Natural Language<br>
    to Shell Commands
  </h1>
  <p class="hero__subtitle">
    Powered by local LLMs • Safety-first • Open source
  </p>
</section>
```

### Feature Card

```html
<div class="feature-card">
  <div class="feature-card__icon">
    <span class="icon-terminal">$_</span>
  </div>
  <h3 class="feature-card__title">Terminal Native</h3>
  <p class="feature-card__description">
    Built for the command line, respecting POSIX standards
    and terminal conventions.
  </p>
  <div class="feature-card__stats">
    <div class="stat">
      <span class="stat__value">&lt;100ms</span>
      <span class="stat__label">startup time</span>
    </div>
  </div>
</div>
```

### Command Comparison Table

```html
<table class="command-comparison">
  <thead>
    <tr>
      <th>Natural Language</th>
      <th>Generated Command</th>
      <th>Safety</th>
      <th>Rating</th>
    </tr>
  </thead>
  <tbody>
    <tr class="selected">
      <td>find large files</td>
      <td><code>find . -type f -size +100M</code></td>
      <td><span class="badge badge--success">SAFE</span></td>
      <td>
        <span class="vote">▲ 47</span>
        <span class="vote vote--down">▼ 3</span>
      </td>
    </tr>
  </tbody>
</table>
```

---

## 🎯 Animation Guidelines

### Timing

```css
--transition-fast: 0.15s;
--transition-base: 0.3s;
--transition-slow: 0.5s;
```

### Easing

```css
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
```

### Hover Effects

- **Buttons**: Slight lift (2-4px) + glow shadow
- **Cards**: Subtle scale (1.02) + border highlight
- **Links**: Underline animation left-to-right

### Loading States

- Use Braille spinner: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
- Subtle pulse for skeleton screens
- Terminal-style typing effect for dramatic reveals

---

## 📱 Responsive Breakpoints

```css
--breakpoint-sm: 640px;   /* Mobile */
--breakpoint-md: 768px;   /* Tablet */
--breakpoint-lg: 1024px;  /* Laptop */
--breakpoint-xl: 1280px;  /* Desktop */
--breakpoint-2xl: 1536px; /* Large Desktop */
```

### Mobile Adaptations

- Reduce spacing by 25-50%
- Stack terminal windows vertically
- Simplify ASCII art (use lighter borders)
- Ensure monospace remains readable (min 14px)
- Touch targets minimum 44x44px

---

## ♿ Accessibility

### Contrast Ratios

All text must meet WCAG AA standards:
- Normal text: 4.5:1 minimum
- Large text: 3:1 minimum
- UI components: 3:1 minimum

### Screen Reader Support

```html
<!-- Example: Icon with proper label -->
<button aria-label="Execute command">
  <span aria-hidden="true">▶</span>
</button>

<!-- Example: Status indicator -->
<div role="status" aria-live="polite">
  <span class="badge badge--success" aria-label="Safe command">
    <span aria-hidden="true">✓</span> SAFE
  </span>
</div>
```

### Keyboard Navigation

- All interactive elements must be keyboard accessible
- Clear focus indicators (cyan outline)
- Logical tab order
- Skip links for navigation

---

## 🎨 Logo & Branding

### Logo Concepts

**Option 1: Terminal Prompt**
```
cmdai$_
```
- Simple, direct, terminal-native
- Works in any context
- Easy to reproduce

**Option 2: ASCII Box**
```
┌─────────┐
│  cmdai  │
│   $     │
└─────────┘
```

**Option 3: Retro Badge**
```
╔═══════════╗
║  cmd ai   ║
║  ▶ SAFE   ║
╚═══════════╝
```

### Logo Usage

- **Primary**: Full color on dark background
- **Reversed**: White/cyan on colored background
- **Monochrome**: Single color when needed
- **Minimum Size**: 32px height (web), 16pt (print)
- **Clear Space**: Minimum 8px on all sides

---

## 📄 Documentation Style

### Code Examples

Always show:
1. Natural language query
2. Generated command
3. Safety indicator
4. Expected output (when relevant)

```markdown
**Query**: "find all PDF files larger than 10MB"

**Generated Command**:
```bash
$ find . -name '*.pdf' -size +10M -ls
```

**Safety**: ✓ SAFE

**Output**:
```
/home/user/documents/manual.pdf
/home/user/downloads/presentation.pdf
```
```

### Callouts

```markdown
> 💡 **TIP**: Use descriptive queries for better results

> ⚠ **WARNING**: This command modifies system files

> ✓ **SUCCESS**: Configuration updated

> ✗ **ERROR**: Connection failed
```

---

## 🌟 Brand Voice & Tone

### Voice Characteristics

- **Knowledgeable but not condescending**
- **Precise but not cold**
- **Nostalgic but not dated**
- **Empowering but not overwhelming**

### Writing Style

**DO**:
- Use active voice
- Explain why, not just what
- Provide examples
- Acknowledge user expertise
- Celebrate terminal culture

**DON'T**:
- Use unnecessary jargon
- Assume beginner OR expert exclusively
- Oversimplify complex topics
- Mock other tools/approaches

### Example Copy

**Homepage Hero**:
```
Transform natural language into safe shell commands.
Powered by local LLMs. Built for developers who live in the terminal.
```

**Feature Description**:
```
Safety-First Validation
Every generated command passes through comprehensive safety checks.
Dangerous patterns are blocked. You stay in control.
```

**Error Message**:
```
✗ Command Blocked: This command could delete system files

Alternative: find /tmp -name '*.tmp' -mtime +7 -delete

This safer version targets only temporary files older than 7 days.
```

---

## 🚀 Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Set up design tokens (CSS variables)
- [ ] Create base component library
- [ ] Implement typography system
- [ ] Build color palette with dark/light modes

### Phase 2: Core Components (Week 3-4)
- [ ] Terminal window component
- [ ] Code block with syntax highlighting
- [ ] Button variants
- [ ] Input fields and forms
- [ ] Table components

### Phase 3: Advanced Patterns (Week 5-6)
- [ ] Command comparison views
- [ ] History timeline
- [ ] Voting/rating system
- [ ] Output viewer
- [ ] Interactive demos

### Phase 4: Polish & Launch (Week 7-8)
- [ ] Animations and micro-interactions
- [ ] Accessibility audit
- [ ] Performance optimization
- [ ] Documentation site
- [ ] Style guide showcase

---

## 📚 Resources & Tools

### Recommended Tools

- **Design**: Figma (design system components)
- **Prototyping**: CodePen/CodeSandbox (quick demos)
- **Color Tools**: Coolors.co (palette generation)
- **Accessibility**: WAVE, axe DevTools
- **Testing**: BrowserStack (cross-browser)

### Font Resources

- JetBrains Mono: https://www.jetbrains.com/lp/mono/
- IBM Plex Mono: https://www.ibm.com/plex/
- Press Start 2P: https://fonts.google.com/specimen/Press+Start+2P

### Inspiration Sites

- Terminal.sexy (color schemes)
- Dracula Theme (color palette inspiration)
- Nord Theme (alternative color scheme)
- Solarized (classic terminal colors)

---

## 🤝 Contributing to the Design System

### Adding New Components

1. Start with the TUI version (if applicable)
2. Extract core visual patterns
3. Create web-friendly adaptation
4. Document usage and variants
5. Add to component library

### Proposing Changes

1. Open issue with "Design System" label
2. Provide visual examples
3. Explain rationale
4. Consider accessibility impact
5. Get community feedback

---

## 📊 Success Metrics

Track these to measure design system effectiveness:

- **Consistency**: % of components using design tokens
- **Accessibility**: WCAG compliance score
- **Performance**: Page load time, Core Web Vitals
- **Developer Satisfaction**: Survey scores, GitHub reactions
- **Adoption**: # of components in production use

---

## 🎯 Next Steps for Web Team

1. **Review this document** - Understand the aesthetic and principles
2. **Set up design tokens** - Create CSS variables matching this spec
3. **Build component library** - Start with buttons, inputs, code blocks
4. **Create homepage mockup** - Apply patterns to real content
5. **Iterate with community** - Share early, get feedback often

---

## 💝 Acknowledgments

This design system is built on the foundation of incredible open-source projects:

- **Ratatui** - The amazing Rust TUI framework
- **Dracula Theme** - Color palette inspiration
- **JetBrains Mono** - Perfect developer font
- **cmdai Community** - Your enthusiasm made this possible!

---

**Let's build something beautiful together!** 🚀✨

*For questions, suggestions, or collaboration: Open an issue with the "Design System" label*

---

**Document Version**: 1.0.0
**Last Updated**: 2025-01-19
**Maintained By**: cmdai Design Team
**License**: Same as cmdai project (AGPL-3.0)
