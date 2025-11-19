# cmdai Design System - Web Edition
## From Terminal to Web: The 8-Bit Aesthetic

**Version:** 1.0.0
**Status:** Active
**Author:** Architecture Team
**Purpose:** Translate cmdai's beloved TUI aesthetic into a cohesive web design system

---

## 🎨 Design Philosophy

### Core Principles

1. **Terminal Authenticity** - The web should feel like a real terminal, not a cartoon of one
2. **8-Bit Nostalgia** - Monospace fonts, ASCII art, retro color palettes
3. **Functional Beauty** - Every visual element serves a purpose
4. **Responsive Clarity** - Information hierarchy that works at any size
5. **Keyboard-First** - Mouse is optional, keyboard is primary

### The cmdai Vibe

> "What if VS Code and a 1980s terminal had a baby, and that baby learned design from GitHub?"

- **Serious but Playful**: Professional tool with personality
- **Geeky but Accessible**: Appeals to power users and newcomers
- **Minimal but Rich**: Clean layouts with depth through subtle details
- **Fast but Smooth**: Instant feedback with delightful micro-interactions

---

## 🎨 Color System

### Brand Palette

```typescript
// cmdai color tokens
export const colors = {
  // Primary Colors
  primary: {
    DEFAULT: '#00FFFF',    // Cyan - main accent
    light: '#66FFFF',      // Lighter cyan for hover
    dark: '#00CCCC',       // Darker cyan for active
    glow: 'rgba(0, 255, 255, 0.3)', // Glow effect
  },

  // Semantic Colors
  success: {
    DEFAULT: '#00FF00',    // Green - safe, success
    light: '#66FF66',
    dark: '#00CC00',
  },

  warning: {
    DEFAULT: '#FFFF00',    // Yellow - moderate risk
    light: '#FFFF66',
    dark: '#CCCC00',
  },

  danger: {
    DEFAULT: '#FF0000',    // Red - high risk, errors
    light: '#FF6666',
    dark: '#CC0000',
  },

  info: {
    DEFAULT: '#0099FF',    // Blue - information
    light: '#66B3FF',
    dark: '#0077CC',
  },

  // Grayscale
  background: {
    primary: '#0A0A0A',    // Almost black - main background
    secondary: '#141414',  // Slightly lighter - panels
    tertiary: '#1E1E1E',   // Card backgrounds
  },

  foreground: {
    primary: '#FFFFFF',    // White - main text
    secondary: '#B0B0B0',  // Light gray - secondary text
    tertiary: '#808080',   // Medium gray - muted text
    disabled: '#4A4A4A',   // Dark gray - disabled
  },

  border: {
    DEFAULT: '#333333',    // Default borders
    light: '#4A4A4A',      // Lighter borders
    dark: '#1A1A1A',       // Darker borders
    accent: '#00FFFF',     // Accent borders (cyan)
  },
}
```

### Color Usage Guidelines

#### Background Layers
```
┌─────────────────────────────────┐
│ Page Background (#0A0A0A)       │  ← Darkest
│  ┌───────────────────────────┐  │
│  │ Panel (#141414)           │  │  ← Medium
│  │  ┌─────────────────────┐  │  │
│  │  │ Card (#1E1E1E)      │  │  │  ← Lightest
│  │  └─────────────────────┘  │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
```

#### Semantic Color Mapping

```typescript
// Risk Levels (from TUI)
const riskColors = {
  safe: colors.success.DEFAULT,      // Green
  moderate: colors.warning.DEFAULT,  // Yellow
  high: colors.danger.DEFAULT,       // Red
  critical: colors.danger.DEFAULT,   // Red + Bold/Blink
}

// Status Indicators
const statusColors = {
  online: colors.success.DEFAULT,    // Green
  offline: colors.danger.DEFAULT,    // Red
  loading: colors.primary.DEFAULT,   // Cyan (animated)
  idle: colors.foreground.tertiary,  // Gray
}

// Interactive States
const interactionColors = {
  default: colors.foreground.primary,
  hover: colors.primary.light,
  active: colors.primary.DEFAULT,
  focus: colors.primary.DEFAULT,      // + ring
  disabled: colors.foreground.disabled,
}
```

---

## 📝 Typography

### Font Stack

```css
/* Primary - Monospace */
--font-mono: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;

/* Secondary - Sans (for marketing copy only) */
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;

/* Code Blocks */
--font-code: 'JetBrains Mono', 'Source Code Pro', monospace;
```

### Typography Scale

```typescript
export const typography = {
  // Display (rare, hero sections only)
  display: {
    size: '4rem',        // 64px
    lineHeight: '1.1',
    weight: '700',
    letterSpacing: '-0.02em',
  },

  // Headings
  h1: {
    size: '2.5rem',      // 40px
    lineHeight: '1.2',
    weight: '700',
    letterSpacing: '-0.01em',
  },

  h2: {
    size: '2rem',        // 32px
    lineHeight: '1.3',
    weight: '600',
  },

  h3: {
    size: '1.5rem',      // 24px
    lineHeight: '1.4',
    weight: '600',
  },

  h4: {
    size: '1.25rem',     // 20px
    lineHeight: '1.5',
    weight: '600',
  },

  // Body
  body: {
    size: '1rem',        // 16px
    lineHeight: '1.6',
    weight: '400',
  },

  bodyLarge: {
    size: '1.125rem',    // 18px
    lineHeight: '1.6',
    weight: '400',
  },

  bodySmall: {
    size: '0.875rem',    // 14px
    lineHeight: '1.5',
    weight: '400',
  },

  // Code/Terminal
  code: {
    size: '0.875rem',    // 14px
    lineHeight: '1.6',
    weight: '400',
    fontFamily: 'var(--font-mono)',
  },

  terminal: {
    size: '1rem',        // 16px
    lineHeight: '1.4',   // Tighter for terminal
    weight: '400',
    fontFamily: 'var(--font-mono)',
  },

  // Labels/Captions
  label: {
    size: '0.75rem',     // 12px
    lineHeight: '1.5',
    weight: '500',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
  },
}
```

### Usage Examples

```tsx
// Headings - always monospace
<h1 className="font-mono text-4xl font-bold tracking-tight">cmdai</h1>

// Body text in terminal - monospace
<p className="font-mono text-base leading-relaxed">Generate shell commands...</p>

// Marketing copy ONLY - sans-serif
<p className="font-sans text-lg">Transform natural language into...</p>

// Code blocks - monospace with background
<code className="font-mono text-sm bg-background-tertiary px-2 py-1">
  cargo run -- --tui
</code>
```

---

## 🎭 Component Patterns

### Terminal Window

The signature cmdai UI element - a bordered terminal window with header.

```tsx
interface TerminalWindowProps {
  title?: string;
  headerRight?: React.ReactNode;
  children: React.ReactNode;
  variant?: 'default' | 'accent' | 'danger';
}

// Visual Structure
┌─ cmdai ────────────────────────────────────┐ ← Header (1 line)
│ ⚙ Ollama • bash • Moderate Safety    [?]  │
├────────────────────────────────────────────┤
│                                            │
│             Content Area                   │ ← Main content
│                                            │
├────────────────────────────────────────────┤
│ [Enter] Action  [Ctrl+C] Quit             │ ← Footer (optional)
└────────────────────────────────────────────┘

// Tailwind Implementation
<div className="border border-border rounded-sm overflow-hidden">
  {/* Header */}
  <div className="flex items-center justify-between px-4 py-2 bg-background-secondary border-b border-border">
    <span className="font-mono text-sm text-primary">─ {title} ─</span>
    {headerRight}
  </div>

  {/* Content */}
  <div className="p-4 bg-background-primary">
    {children}
  </div>

  {/* Footer (optional) */}
  <div className="px-4 py-2 bg-background-secondary border-t border-border font-mono text-xs text-foreground-tertiary">
    Keyboard shortcuts...
  </div>
</div>
```

### Status Indicator

Colored dots/icons with text, used for backend status, risk levels, etc.

```tsx
interface StatusIndicatorProps {
  status: 'online' | 'offline' | 'loading';
  label: string;
  size?: 'sm' | 'md' | 'lg';
}

// Visual
⚙ Ollama  ← Icon + text
  └─ colored based on status

// Tailwind
<div className="flex items-center gap-2">
  <div className={`
    w-2 h-2 rounded-full
    ${status === 'online' ? 'bg-success' : 'bg-danger'}
    ${status === 'loading' ? 'animate-pulse' : ''}
  `} />
  <span className="font-mono text-sm">{label}</span>
</div>
```

### Keyboard Shortcut Pill

Shows keyboard shortcuts in a highlighted format.

```tsx
// Visual
[Ctrl+C] Quit
└──┬──┘ └─┬─┘
  Key  Label

// Tailwind
<span className="inline-flex items-center gap-2 font-mono text-xs">
  <kbd className="px-2 py-1 bg-background-tertiary border border-border rounded text-primary font-bold">
    Ctrl+C
  </kbd>
  <span className="text-foreground-secondary">Quit</span>
</span>
```

### Progress Bar

Terminal-style progress visualization.

```tsx
// Visual (ASCII-style)
[████████████░░░░░░░░] 60%

// Tailwind
<div className="w-full">
  <div className="relative h-6 bg-background-tertiary border border-border rounded-sm overflow-hidden">
    <div
      className="absolute inset-y-0 left-0 bg-primary transition-all duration-300"
      style={{ width: `${progress}%` }}
    />
    <div className="absolute inset-0 flex items-center justify-center font-mono text-xs font-bold">
      {progress}%
    </div>
  </div>
</div>
```

### Command Output Block

Displays generated commands with syntax highlighting.

```tsx
// Visual
┌─ Generated Command ──────────────────┐
│ find . -type f -name "*.py"         │
│                                      │
│ 💡 Searches for Python files        │
└──────────────────────────────────────┘

// Tailwind + highlight.js
<div className="border border-border rounded-sm overflow-hidden">
  <div className="px-4 py-2 bg-background-secondary border-b border-border">
    <span className="font-mono text-xs text-foreground-tertiary">─ Generated Command ─</span>
  </div>
  <div className="p-4 bg-background-primary">
    <pre className="font-mono text-sm text-primary">
      <code>{command}</code>
    </pre>
    {explanation && (
      <p className="mt-3 font-mono text-sm text-foreground-secondary flex items-start gap-2">
        <span>💡</span>
        <span>{explanation}</span>
      </p>
    )}
  </div>
</div>
```

---

## ✨ Animation & Motion

### Principles

1. **Fast but Noticeable** - Animations should be quick (100-300ms) but perceptible
2. **Purpose-Driven** - Every animation communicates state or directs attention
3. **Terminal-Authentic** - Use blink, pulse, slide - not bounce or elastic
4. **Respect Preferences** - Honor `prefers-reduced-motion`

### Animation Tokens

```typescript
export const animations = {
  // Durations
  duration: {
    instant: '50ms',
    fast: '150ms',
    normal: '300ms',
    slow: '500ms',
  },

  // Easings (terminal-appropriate)
  easing: {
    linear: 'linear',
    ease: 'ease',
    in: 'cubic-bezier(0.4, 0, 1, 1)',
    out: 'cubic-bezier(0, 0, 0.2, 1)',
  },

  // Common animations
  blink: {
    animation: 'blink 1s step-end infinite',
  },

  pulse: {
    animation: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
  },

  fadeIn: {
    animation: 'fadeIn 300ms ease-out',
  },

  slideUp: {
    animation: 'slideUp 300ms ease-out',
  },
}

// Keyframes
@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { transform: translateY(10px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
```

### Usage Guidelines

```tsx
// Loading states - pulse
<div className="animate-pulse">Loading...</div>

// Cursor - blink
<span className="animate-blink">_</span>

// New content - fade in
<div className="animate-fadeIn">New command generated!</div>

// Modals/Overlays - slide up
<div className="animate-slideUp">Modal content</div>

// Respect reduced motion
<div className="transition-colors duration-300 motion-reduce:transition-none">
  Content
</div>
```

---

## 🎯 Interactive States

### Button States

```tsx
// Default Button
<button className="
  px-4 py-2
  font-mono text-sm
  bg-background-tertiary
  border border-border
  text-foreground-primary
  transition-colors duration-150
  hover:bg-background-secondary
  hover:border-primary
  hover:text-primary
  active:bg-background-primary
  focus:outline-none
  focus:ring-2
  focus:ring-primary
  focus:ring-offset-2
  focus:ring-offset-background-primary
  disabled:opacity-50
  disabled:cursor-not-allowed
">
  Action
</button>

// Primary Button (cyan accent)
<button className="
  px-4 py-2
  font-mono text-sm font-bold
  bg-primary
  border border-primary
  text-background-primary
  transition-all duration-150
  hover:bg-primary-light
  hover:shadow-[0_0_20px_rgba(0,255,255,0.5)]
  active:bg-primary-dark
  focus:outline-none
  focus:ring-2
  focus:ring-primary-light
">
  Generate
</button>

// Danger Button
<button className="
  px-4 py-2
  font-mono text-sm
  bg-transparent
  border border-danger
  text-danger
  hover:bg-danger
  hover:text-background-primary
">
  Delete
</button>
```

### Input States

```tsx
<input className="
  w-full
  px-3 py-2
  font-mono text-sm
  bg-background-primary
  border border-border
  text-foreground-primary
  placeholder:text-foreground-disabled
  transition-colors duration-150
  focus:outline-none
  focus:border-primary
  focus:ring-1
  focus:ring-primary
  disabled:opacity-50
  disabled:cursor-not-allowed
" />
```

---

## 📱 Responsive Design

### Breakpoints

```typescript
export const breakpoints = {
  sm: '640px',   // Mobile landscape
  md: '768px',   // Tablet
  lg: '1024px',  // Laptop
  xl: '1280px',  // Desktop
  '2xl': '1536px', // Large desktop
}
```

### Layout Patterns

```tsx
// Terminal window - stacks on mobile
<div className="
  grid grid-cols-1 gap-4
  md:grid-cols-2
  lg:grid-cols-3
">
  {terminals.map(terminal => (
    <TerminalWindow key={terminal.id} {...terminal} />
  ))}
</div>

// Sidebar layout - collapses on mobile
<div className="flex flex-col lg:flex-row gap-4">
  <aside className="w-full lg:w-64">Sidebar</aside>
  <main className="flex-1">Content</main>
</div>

// Font sizes - responsive
<h1 className="text-2xl sm:text-3xl md:text-4xl lg:text-5xl">
  Heading
</h1>
```

---

## 🎨 ASCII Art & Icons

### Icon Style

Use Unicode box-drawing characters and emojis for icons:

```
⚙  Settings/Backend
🤖 AI/Bot
💡 Explanation/Tip
⚠  Warning
❌ Error/Danger
✓  Success/Safe
📝 Input/Edit
🔍 Search
📊 Statistics
⏳ Loading
🛑 Critical/Stop
```

### Box Drawing Characters

```
┌─┬─┐  Top border
│ │ │  Sides
├─┼─┤  Middle divider
└─┴─┘  Bottom border

╔═╦═╗  Double border (emphasis)
║ ║ ║
╚═╩═╝
```

### Usage

```tsx
// In headers
<span className="font-mono text-primary">─ cmdai ─</span>

// Status indicators
<span>⚙ {backendName}</span>
<span>✓ Safe</span>
<span>⚠ Moderate Risk</span>

// Loading
<span className="animate-pulse">⏳ Generating...</span>
```

---

## 🌐 Accessibility

### Guidelines

1. **Keyboard Navigation**: All interactive elements must be keyboard-accessible
2. **Focus Indicators**: Clear, high-contrast focus rings (cyan)
3. **Color Contrast**: Maintain WCAG AA standards minimum (AAA preferred)
4. **Screen Readers**: Proper semantic HTML and ARIA labels
5. **Reduced Motion**: Respect `prefers-reduced-motion`

### Implementation

```tsx
// Proper semantic HTML
<button aria-label="Generate command" onClick={handleGenerate}>
  Generate
</button>

// Skip links for keyboard users
<a href="#main-content" className="sr-only focus:not-sr-only">
  Skip to main content
</a>

// Announce dynamic content
<div role="status" aria-live="polite">
  {statusMessage}
</div>

// Respect motion preferences
<div className="transition-all motion-reduce:transition-none">
  Content
</div>
```

---

## 📐 Spacing System

```typescript
export const spacing = {
  0: '0',
  1: '0.25rem',  // 4px
  2: '0.5rem',   // 8px
  3: '0.75rem',  // 12px
  4: '1rem',     // 16px
  5: '1.25rem',  // 20px
  6: '1.5rem',   // 24px
  8: '2rem',     // 32px
  10: '2.5rem',  // 40px
  12: '3rem',    // 48px
  16: '4rem',    // 64px
  20: '5rem',    // 80px
  24: '6rem',    // 96px
}

// Usage
<div className="p-4">     // padding: 1rem
<div className="mt-8">    // margin-top: 2rem
<div className="gap-6">   // gap: 1.5rem
```

---

## 🎬 Microinteractions

### Command Generation Flow

```
1. User types → Input highlights (border-primary)
2. User presses Enter → Input dims, loading spinner appears
3. Command generates → Fade in with slide-up animation
4. Safety validation → Risk indicator pulses if warnings
5. User hovers command → Highlight effect, copy icon appears
```

### Status Changes

```
Backend connection:
offline → loading (pulse) → online (solid green + brief glow)
```

### Form Validation

```
Empty field:
normal → (submit) → shake + border-danger + error message
```

---

## 🎨 Theme Variants

While maintaining the 8-bit aesthetic, support light/dark modes:

```typescript
// Dark (default)
background: '#0A0A0A'
foreground: '#FFFFFF'

// Light (optional)
background: '#F5F5F5'
foreground: '#0A0A0A'
accent: '#0099FF' // Adjusted cyan for light backgrounds
```

---

## 📦 Component Library Structure

```
components/
├── ui/                      # Base components
│   ├── Button.tsx
│   ├── Input.tsx
│   ├── Select.tsx
│   ├── Checkbox.tsx
│   └── Radio.tsx
├── terminal/                # Terminal-specific
│   ├── TerminalWindow.tsx
│   ├── CommandOutput.tsx
│   ├── StatusBar.tsx
│   └── HelpFooter.tsx
├── feedback/                # User feedback
│   ├── Alert.tsx
│   ├── Toast.tsx
│   ├── Progress.tsx
│   └── Spinner.tsx
└── layout/                  # Layout components
    ├── Container.tsx
    ├── Grid.tsx
    └── Stack.tsx
```

---

## 🚀 Performance Guidelines

1. **Code Splitting**: Lazy load terminal simulator
2. **Font Loading**: Use `font-display: swap` for monospace fonts
3. **Animations**: Use CSS transforms (GPU-accelerated)
4. **Images**: Minimal use, prefer SVG or CSS for graphics
5. **Bundle Size**: Keep total JS < 100KB gzipped

---

## ✅ Design Checklist

Before shipping any component:

- [ ] Uses monospace font for terminal elements
- [ ] Follows color system (no arbitrary colors)
- [ ] Has proper keyboard navigation
- [ ] Includes focus indicators
- [ ] Respects reduced motion preference
- [ ] Works at all breakpoints
- [ ] Maintains 4.5:1 contrast ratio minimum
- [ ] Uses semantic HTML
- [ ] Includes proper ARIA labels
- [ ] Passes accessibility audit

---

**Next:** Component Architecture for React/Next.js →
