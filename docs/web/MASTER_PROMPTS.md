# Master Prompts for Frontend Implementation
## AI-Assisted Development Guides for cmdai Web

**Purpose:** Copy-paste these prompts into Claude, ChatGPT, or your favorite AI assistant to get high-quality implementation code.

**Prerequisites:** Read [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) and [COMPONENT_ARCHITECTURE.md](./COMPONENT_ARCHITECTURE.md) first.

---

## 🎯 How to Use These Prompts

1. **Copy the entire prompt** including context
2. **Paste into your AI assistant** (Claude, ChatGPT, etc.)
3. **Review the generated code** against our design system
4. **Iterate if needed** by providing feedback
5. **Test in your project** before committing

---

## 📦 Prompt 1: Project Setup

```
I'm building a Next.js 14 web application for cmdai, a CLI tool that converts natural language to shell commands. I need to set up the project with our specific design system.

Requirements:
- Next.js 14 with App Router
- TypeScript
- Tailwind CSS 3
- 8-bit terminal aesthetic (monospace fonts, dark theme, cyan accents)

Design tokens:
- Background colors: #0A0A0A (primary), #141414 (secondary), #1E1E1E (tertiary)
- Primary accent: #00FFFF (cyan)
- Success: #00FF00, Warning: #FFFF00, Danger: #FF0000
- Fonts: JetBrains Mono (monospace), Inter (sans-serif)

Please provide:
1. Complete tailwind.config.ts with all color tokens
2. globals.css with custom CSS for terminal effects
3. package.json dependencies list
4. app/layout.tsx with font imports

Make it production-ready and follow Next.js 14 best practices.
```

---

## 🎨 Prompt 2: TerminalWindow Component

```
Create a reusable TerminalWindow component for a cmdai web app using React + TypeScript + Tailwind.

Design requirements:
- Bordered window with header, content area, and optional footer
- Header has title (left) and optional content (right)
- Support variants: 'default', 'accent', 'danger', 'success' (affects border color)
- Border colors: default=#333333, accent=#00FFFF, danger=#FF0000, success=#00FF00
- Background: header=#141414, content=#0A0A0A
- Font: monospace (JetBrains Mono)
- Title format: "─ title ─" with variant-specific color

Interface:
```typescript
interface TerminalWindowProps {
  title?: string;
  headerRight?: React.ReactNode;
  footer?: React.ReactNode;
  variant?: 'default' | 'accent' | 'danger' | 'success';
  className?: string;
  children: React.ReactNode;
}
```

Provide:
1. Complete component code (TerminalWindow.tsx)
2. Separate Header and Footer sub-components
3. Usage example
4. Tailwind classes (use cn() utility)

Follow React best practices and make it accessible (ARIA labels, keyboard navigation).
```

---

## 🎮 Prompt 3: StatusBar Component

```
Build a StatusBar component for cmdai TUI simulator that displays:
- Backend status (name, availability indicator, model name)
- Shell type (bash, zsh, etc.)
- Safety level (strict/moderate/permissive with color coding)
- Optional help indicator

Design spec:
- Layout: Horizontal flex with space-between
- Font: monospace, text-sm
- Background: #141414, border-bottom: #333333
- Status dot: 2px circle, green if available, red if not
- Safety colors: strict=red, moderate=yellow, permissive=green
- Spacing: gap-4 between items

Interface:
```typescript
interface StatusBarProps {
  backend: { name: string; available: boolean; model?: string };
  shell: string;
  safetyLevel: 'strict' | 'moderate' | 'permissive';
  showHelp?: boolean;
}
```

Include:
1. Complete TypeScript component
2. Tailwind styling with our color tokens
3. Status indicator with pulse animation for "loading" state
4. Usage example

Make it responsive (stack on mobile if needed).
```

---

## ⌨️ Prompt 4: CommandOutput Component

```
Create a CommandOutput component for displaying generated shell commands with:
- Code block with syntax highlighting
- Copy button (appears on hover)
- Risk level indicator (safe/moderate/high/critical)
- Optional warnings list
- Optional explanation text

Design:
- Command: monospace, cyan text, dark background (#1E1E1E)
- Copy button: top-right corner, opacity 0 → 1 on hover
- Risk levels: safe=green checkmark, moderate=yellow warning, high=red X, critical=red stop sign
- Warnings: yellow warning icon + text
- Explanation: lightbulb icon + gray text

Interface:
```typescript
interface CommandOutputProps {
  command: string;
  explanation?: string;
  riskLevel?: 'safe' | 'moderate' | 'high' | 'critical';
  warnings?: string[];
  className?: string;
}
```

Features:
- Copy to clipboard with success feedback
- Icons from lucide-react
- Responsive design
- Accessible (keyboard, screen readers)

Provide complete component + usage example.
```

---

## 🎣 Prompt 5: useKeyboard Hook

```
Build a custom React hook for handling global keyboard shortcuts in a terminal simulator.

Requirements:
- Support common modifiers: Ctrl, Shift, Alt
- Works with multiple key bindings
- Prevents default browser behavior
- Type-safe with TypeScript
- Cleanup on unmount

Interface:
```typescript
interface KeyBinding {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  callback: () => void;
}

function useKeyboard(bindings: KeyBinding[]): void
```

Example usage:
```typescript
useKeyboard([
  { key: 'c', ctrl: true, callback: () => handleQuit() },
  { key: 'l', ctrl: true, callback: () => handleClear() },
  { key: 'Enter', callback: () => handleSubmit() },
]);
```

Provide:
1. Complete hook implementation
2. Tests (using @testing-library/react)
3. Usage example in a component
4. Handle edge cases (Cmd vs Ctrl on Mac)
```

---

## 🤖 Prompt 6: Mock Backend

```
Create a mock backend for cmdai web simulator that generates realistic shell commands from natural language.

Requirements:
- 15-20 pre-defined command mappings
- Fuzzy matching (keywords-based)
- Simulated network delay (800-1500ms)
- TypeScript with full type safety
- Categories: file ops, git, system info, docker

Response structure:
```typescript
interface MockResponse {
  command: string;
  explanation: string;
  riskLevel: 'safe' | 'moderate' | 'high' | 'critical';
  warnings?: string[];
  alternatives?: string[];
}
```

Mock responses should include:
- File operations: find, compress, delete
- Git: status, branch, commit
- System: disk usage, processes, network
- Dangerous commands: rm -rf (with warnings)

Functions needed:
1. `generateMockResponse(input: string): MockResponse`
2. `simulateGeneration(input: string): Promise<MockResponse>`

Include error handling and fallback for unknown queries.
```

---

## 📱 Prompt 7: Responsive Terminal Layout

```
Design a responsive layout for cmdai terminal simulator that works on desktop, tablet, and mobile.

Layout structure:
- Desktop (>1024px): Side-by-side terminal + examples
- Tablet (768-1024px): Stacked vertical
- Mobile (<768px): Full-width terminal, collapsible examples

Components:
- TerminalWindow (main simulator)
- Examples gallery (clickable queries)
- Help section
- Footer with keyboard shortcuts

Requirements:
- Next.js 14 App Router page component
- Tailwind responsive utilities
- Smooth transitions between breakpoints
- Accessibility (skip links, focus management)
- Performance (lazy load examples)

Provide:
1. Complete page.tsx for /simulator route
2. Layout with Tailwind grid/flex
3. State management (useState for input/output)
4. Example gallery component
5. Mobile optimizations

Make it production-ready with error boundaries and loading states.
```

---

## ✨ Prompt 8: Animation System

```
Implement a comprehensive animation system for cmdai terminal UI using Tailwind + Framer Motion.

Animations needed:
1. Loading state: Pulsing spinner + text
2. Command appears: Slide up + fade in
3. Cursor blink: 1s interval
4. Risk flash: 3 pulses for high/critical
5. Modal enter/exit: Scale + fade
6. Button hover: Glow effect on primary color

Design principles:
- Fast (100-300ms) but noticeable
- Terminal-authentic (no bounce/elastic)
- Respect prefers-reduced-motion
- GPU-accelerated (transforms)

Provide:
1. Tailwind keyframes in tailwind.config.ts
2. Framer Motion variants for modal
3. Reusable animation components (LoadingSpinner, BlinkingCursor)
4. Accessibility support
5. Usage examples

Use Tailwind's animation utilities where possible, Framer Motion for complex sequences.
```

---

## 🎯 Prompt 9: TUI Simulator Component

```
Build the complete TUI Simulator component that ties everything together.

Features:
- Text input with multi-line support
- Command generation with mock backend
- Display output with syntax highlighting
- History navigation (up/down arrows)
- Keyboard shortcuts (Ctrl+L, Ctrl+R, etc.)
- Help modal
- Examples gallery
- localStorage for history

State management:
```typescript
interface SimulatorState {
  input: string;
  isGenerating: boolean;
  currentResponse: MockResponse | null;
  commandHistory: Array<{ input: string; response: MockResponse; timestamp: Date }>;
  historyIndex: number;
  showHelp: boolean;
}
```

Components to integrate:
- TerminalWindow
- StatusBar
- InputArea
- CommandOutput
- KeyboardShortcuts
- HelpModal

Provide:
1. Main TUISimulator.tsx component
2. State management with useState
3. All keyboard shortcuts implemented
4. History management
5. localStorage persistence
6. Complete working example

Make it production-ready with error handling, loading states, and accessibility.
```

---

## 🧪 Prompt 10: Testing Suite

```
Create a comprehensive testing suite for cmdai React components using:
- Vitest (test runner)
- @testing-library/react (component testing)
- @testing-library/user-event (interaction testing)

Test cases for:
1. TerminalWindow: renders correctly, variant colors, accessibility
2. StatusBar: displays backend status, safety levels, tooltips
3. CommandOutput: copy functionality, risk indicators, warnings
4. useKeyboard: key combinations, cleanup, edge cases
5. Mock Backend: response generation, fuzzy matching, delays
6. TUISimulator: full user flow, history, shortcuts

Requirements:
- 80%+ code coverage
- Accessibility tests (ARIA, keyboard nav)
- Snapshot tests for UI components
- Integration tests for user flows

Provide:
1. Vitest config (vitest.config.ts)
2. Test setup file
3. Example tests for 3 components
4. Mock implementations
5. GitHub Actions CI workflow

Follow testing best practices (arrange-act-assert, descriptive names).
```

---

## 🎨 Prompt 11: Brand Guidelines Document

```
Create a comprehensive brand guidelines document for cmdai that extends the 8-bit terminal aesthetic to all marketing materials.

Include:
1. Logo usage (primary, secondary, icon-only)
2. Color palette with usage rules
3. Typography system (headings, body, code)
4. Voice & tone (technical but friendly)
5. Photography/imagery style
6. Social media templates
7. Documentation style
8. Email signature format

Design principles:
- 8-bit nostalgia meets modern professionalism
- Monospace fonts for technical credibility
- Cyan as signature color
- Dark theme primary, light theme optional
- ASCII art encouraged

Provide:
1. Markdown guidelines document
2. Figma file structure (describe layers)
3. Social media templates (dimensions)
4. Example usage (do's and don'ts)

Make it comprehensive enough for designers and marketers to maintain consistency.
```

---

## 📊 Prompt 12: Analytics Integration

```
Implement privacy-respecting analytics for cmdai web simulator.

Track:
- Page views
- Command generations (query categories, not actual queries)
- Example clicks
- Copy button usage
- Modal opens
- Error rates
- Session duration

Privacy requirements:
- No PII collection
- Respect DNT header
- GDPR compliant
- Optional (user can opt-out)
- No third-party tracking pixels

Tech stack options:
1. Plausible Analytics (preferred)
2. Simple Analytics
3. Custom solution with Next.js API routes

Provide:
1. Analytics integration code
2. Privacy policy snippet
3. Cookie banner component (if needed)
4. Dashboard layout (what metrics to show)
5. Event tracking wrapper

Make it production-ready and respectful of user privacy.
```

---

## 🚀 Deployment Prompts

### Prompt 13: Vercel Deployment

```
Provide complete deployment instructions for cmdai Next.js app on Vercel.

Include:
1. vercel.json configuration
2. Environment variables setup
3. Build optimizations
4. Performance checklist (Core Web Vitals)
5. Error monitoring setup (Sentry)
6. CDN configuration for fonts/assets

Optimize for:
- Lighthouse score > 95
- First Contentful Paint < 1.5s
- Time to Interactive < 3s
- Bundle size < 150KB

Provide step-by-step deployment guide + troubleshooting tips.
```

---

## ✅ Quality Checklist Prompt

```
Review my cmdai web implementation and provide a comprehensive quality audit.

Check:
1. Accessibility (WCAG AA compliance)
2. Performance (Lighthouse scores)
3. SEO (meta tags, Open Graph)
4. Security (no XSS vulnerabilities)
5. Code quality (TypeScript strict mode)
6. Component reusability
7. Error handling
8. Loading states
9. Mobile responsiveness
10. Browser compatibility

Provide:
- Checklist of issues found
- Priority (high/medium/low)
- Fix recommendations
- Code examples for fixes

Be thorough and actionable.
```

---

## 💡 Usage Tips

### For Best Results:

1. **Provide Context**: Always include design system details
2. **Be Specific**: Mention exact versions (Next.js 14, React 18)
3. **Request Examples**: Ask for usage examples and tests
4. **Iterate**: If output isn't perfect, refine the prompt
5. **Cross-Reference**: Check generated code against our design docs

### Common Follow-Up Questions:

- "Make this more accessible"
- "Add responsive design for mobile"
- "Include unit tests using Vitest"
- "Optimize for performance"
- "Add error handling"

---

## 📚 Reference Documents

Before using these prompts, review:

1. [Design System](./DESIGN_SYSTEM.md) - Colors, typography, components
2. [Component Architecture](./COMPONENT_ARCHITECTURE.md) - React patterns
3. [Web Simulator Spec](./WEB_SIMULATOR_SPEC.md) - Feature requirements

---

## 🤝 Contributing

Have a better prompt? Submit a PR with:
- Clear use case
- Expected output description
- Example of good vs. bad results

---

**Last Updated:** 2025-11-19
**Maintained By:** Architecture Team
