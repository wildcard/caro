# cmdai Web Component Architecture
## React/Next.js Implementation Guide

**Target Stack:** React 18 + Next.js 14 + Tailwind CSS 3 + TypeScript
**Design System:** See [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md)
**Purpose:** Production-ready component specifications for frontend developers

---

## 🏗️ Project Structure

```
app/                          # Next.js 14 App Router
├── (marketing)/             # Marketing pages
│   ├── page.tsx            # Homepage
│   ├── about/page.tsx
│   └── docs/page.tsx
├── simulator/              # TUI Simulator
│   └── page.tsx           # Interactive demo
└── layout.tsx              # Root layout

components/
├── ui/                      # shadcn/ui base components
│   ├── button.tsx
│   ├── input.tsx
│   ├── select.tsx
│   └── ...
├── terminal/                # cmdai terminal components
│   ├── TerminalWindow/
│   │   ├── index.tsx
│   │   ├── Header.tsx
│   │   ├── Content.tsx
│   │   └── Footer.tsx
│   ├── StatusBar/
│   │   └── index.tsx
│   ├── CommandOutput/
│   │   └── index.tsx
│   ├── InputArea/
│   │   └── index.tsx
│   └── KeyboardShortcut/
│       └── index.tsx
├── simulator/               # Simulator-specific
│   ├── TUISimulator.tsx
│   ├── ReplMode.tsx
│   ├── HistoryMode.tsx
│   └── MockBackend.ts
└── layout/
    ├── Container.tsx
    ├── Grid.tsx
    └── Stack.tsx

lib/
├── utils.ts                 # Utility functions
├── cn.ts                    # Class name merger
└── hooks/
    ├── useKeyboard.ts
    ├── useTerminal.ts
    └── useSimulator.ts

styles/
├── globals.css              # Global styles + Tailwind
└── terminal.css             # Terminal-specific CSS

public/
├── fonts/                   # JetBrains Mono, etc.
└── icons/                   # SVG icons
```

---

## 🧩 Core Components

### 1. TerminalWindow Component

The foundational component - a bordered terminal window.

```tsx
// components/terminal/TerminalWindow/index.tsx
import React from 'react';
import { cn } from '@/lib/utils';
import Header from './Header';
import Footer from './Footer';

export interface TerminalWindowProps {
  title?: string;
  headerRight?: React.ReactNode;
  footer?: React.ReactNode;
  variant?: 'default' | 'accent' | 'danger' | 'success';
  className?: string;
  children: React.ReactNode;
}

export function TerminalWindow({
  title,
  headerRight,
  footer,
  variant = 'default',
  className,
  children,
}: TerminalWindowProps) {
  const borderColors = {
    default: 'border-border',
    accent: 'border-primary',
    danger: 'border-danger',
    success: 'border-success',
  };

  return (
    <div
      className={cn(
        'rounded-sm overflow-hidden border',
        borderColors[variant],
        className
      )}
    >
      {/* Header */}
      {title && (
        <Header title={title} variant={variant}>
          {headerRight}
        </Header>
      )}

      {/* Content */}
      <div className="p-4 bg-background-primary min-h-[200px]">
        {children}
      </div>

      {/* Footer */}
      {footer && <Footer>{footer}</Footer>}
    </div>
  );
}

// components/terminal/TerminalWindow/Header.tsx
interface HeaderProps {
  title: string;
  variant: TerminalWindowProps['variant'];
  children?: React.ReactNode;
}

function Header({ title, variant, children }: HeaderProps) {
  const titleColors = {
    default: 'text-primary',
    accent: 'text-primary',
    danger: 'text-danger',
    success: 'text-success',
  };

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-background-secondary border-b border-border">
      <span className={cn('font-mono text-sm', titleColors[variant])}>
        ─ {title} ─
      </span>
      {children}
    </div>
  );
}

// components/terminal/TerminalWindow/Footer.tsx
interface FooterProps {
  children: React.ReactNode;
}

function Footer({ children }: FooterProps) {
  return (
    <div className="px-4 py-2 bg-background-secondary border-t border-border">
      {children}
    </div>
  );
}

// Usage Example
<TerminalWindow
  title="cmdai"
  variant="accent"
  headerRight={<StatusIndicator status="online" label="Ollama" />}
  footer={<KeyboardShortcuts shortcuts={replShortcuts} />}
>
  <p className="font-mono text-sm">Terminal content here...</p>
</TerminalWindow>
```

---

### 2. StatusBar Component

Displays backend status, shell, safety level - matches TUI.

```tsx
// components/terminal/StatusBar/index.tsx
import React from 'react';
import { cn } from '@/lib/utils';

interface BackendStatus {
  name: string;
  available: boolean;
  model?: string;
}

interface StatusBarProps {
  backend: BackendStatus;
  shell: string;
  safetyLevel: 'strict' | 'moderate' | 'permissive';
  showHelp?: boolean;
}

export function StatusBar({
  backend,
  shell,
  safetyLevel,
  showHelp = true,
}: StatusBarProps) {
  const safetyColors = {
    strict: 'text-danger',
    moderate: 'text-warning',
    permissive: 'text-success',
  };

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-background-secondary border-b border-border font-mono text-sm">
      <div className="flex items-center gap-4">
        {/* Backend */}
        <div className="flex items-center gap-2">
          <div
            className={cn(
              'w-2 h-2 rounded-full',
              backend.available ? 'bg-primary' : 'bg-danger'
            )}
          />
          <span className={backend.available ? 'text-primary' : 'text-danger'}>
            ⚙ {backend.name}
          </span>
          {backend.model && (
            <span className="text-foreground-tertiary text-xs">
              ({backend.model})
            </span>
          )}
        </div>

        {/* Shell */}
        <span className="text-success">• {shell}</span>

        {/* Safety Level */}
        <span className={safetyColors[safetyLevel]}>
          • {safetyLevel.charAt(0).toUpperCase() + safetyLevel.slice(1)} Safety
        </span>
      </div>

      {/* Help Indicator */}
      {showHelp && (
        <span className="text-foreground-tertiary">[?] Help</span>
      )}
    </div>
  );
}

// Usage
<StatusBar
  backend={{ name: 'Ollama', available: true, model: 'qwen2.5-coder:7b' }}
  shell="bash"
  safetyLevel="moderate"
/>
```

---

### 3. CommandOutput Component

Displays generated commands with explanation.

```tsx
// components/terminal/CommandOutput/index.tsx
'use client';

import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Check, Copy } from 'lucide-react';

interface CommandOutputProps {
  command: string;
  explanation?: string;
  riskLevel?: 'safe' | 'moderate' | 'high' | 'critical';
  warnings?: string[];
  className?: string;
}

export function CommandOutput({
  command,
  explanation,
  riskLevel = 'safe',
  warnings = [],
  className,
}: CommandOutputProps) {
  const [copied, setCopied] = useState(false);

  const riskColors = {
    safe: 'text-success',
    moderate: 'text-warning',
    high: 'text-danger',
    critical: 'text-danger font-bold',
  };

  const riskIcons = {
    safe: '✓',
    moderate: '⚠',
    high: '❌',
    critical: '🛑',
  };

  const handleCopy = async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={cn('space-y-3', className)}>
      {/* Command */}
      <div className="relative group">
        <pre className="p-4 bg-background-tertiary border border-border rounded-sm font-mono text-sm text-primary overflow-x-auto">
          <code>{command}</code>
        </pre>

        {/* Copy Button */}
        <button
          onClick={handleCopy}
          className="absolute top-2 right-2 p-2 bg-background-primary border border-border rounded opacity-0 group-hover:opacity-100 transition-opacity"
          aria-label="Copy command"
        >
          {copied ? (
            <Check className="w-4 h-4 text-success" />
          ) : (
            <Copy className="w-4 h-4 text-foreground-tertiary" />
          )}
        </button>
      </div>

      {/* Risk Level */}
      {riskLevel !== 'safe' && (
        <div className={cn('flex items-start gap-2 font-mono text-sm', riskColors[riskLevel])}>
          <span>{riskIcons[riskLevel]}</span>
          <span>{riskLevel.toUpperCase()} RISK</span>
        </div>
      )}

      {/* Warnings */}
      {warnings.length > 0 && (
        <div className="space-y-1">
          {warnings.map((warning, i) => (
            <div key={i} className="flex items-start gap-2 font-mono text-sm text-warning">
              <span>⚠</span>
              <span>{warning}</span>
            </div>
          ))}
        </div>
      )}

      {/* Explanation */}
      {explanation && (
        <div className="flex items-start gap-2 font-mono text-sm text-foreground-secondary">
          <span>💡</span>
          <p>{explanation}</p>
        </div>
      )}
    </div>
  );
}
```

---

### 4. InputArea Component

Multi-line text input for natural language.

```tsx
// components/terminal/InputArea/index.tsx
'use client';

import React, { useRef, useEffect } from 'react';
import { cn } from '@/lib/utils';

interface InputAreaProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  placeholder?: string;
  disabled?: boolean;
  autoFocus?: boolean;
  className?: string;
}

export function InputArea({
  value,
  onChange,
  onSubmit,
  placeholder = '🤖 Type your command in natural language...',
  disabled = false,
  autoFocus = true,
  className,
}: InputAreaProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (autoFocus && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [autoFocus]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (value.trim()) {
        onSubmit();
      }
    }
  };

  return (
    <div className={cn('relative', className)}>
      <textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        rows={3}
        className={cn(
          'w-full p-3 font-mono text-sm',
          'bg-background-primary border border-border rounded-sm',
          'text-foreground-primary placeholder:text-foreground-disabled',
          'focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary',
          'resize-none transition-colors',
          disabled && 'opacity-50 cursor-not-allowed'
        )}
      />

      {/* Character count (optional) */}
      {value.length > 0 && (
        <div className="mt-1 text-xs text-foreground-tertiary font-mono text-right">
          {value.length} characters
        </div>
      )}
    </div>
  );
}
```

---

### 5. KeyboardShortcut Component

Displays keyboard shortcuts.

```tsx
// components/terminal/KeyboardShortcut/index.tsx
import React from 'react';
import { cn } from '@/lib/utils';

export interface Shortcut {
  key: string;
  description: string;
  enabled?: boolean;
}

interface KeyboardShortcutProps {
  shortcut: Shortcut;
}

export function KeyboardShortcut({ shortcut }: KeyboardShortcutProps) {
  return (
    <span className="inline-flex items-center gap-2 font-mono text-xs">
      <kbd
        className={cn(
          'px-2 py-1 rounded border',
          shortcut.enabled !== false
            ? 'bg-background-tertiary border-border text-primary font-bold'
            : 'bg-background-primary border-border text-foreground-disabled'
        )}
      >
        {shortcut.key}
      </kbd>
      <span
        className={cn(
          shortcut.enabled !== false
            ? 'text-foreground-secondary'
            : 'text-foreground-disabled'
        )}
      >
        {shortcut.description}
      </span>
    </span>
  );
}

interface KeyboardShortcutsProps {
  shortcuts: Shortcut[];
  className?: string;
}

export function KeyboardShortcuts({ shortcuts, className }: KeyboardShortcutsProps) {
  return (
    <div className={cn('flex flex-wrap items-center gap-4', className)}>
      {shortcuts.map((shortcut, i) => (
        <KeyboardShortcut key={i} shortcut={shortcut} />
      ))}
    </div>
  );
}

// Usage
const replShortcuts: Shortcut[] = [
  { key: 'Enter', description: 'Generate' },
  { key: 'Ctrl+L', description: 'Clear' },
  { key: 'Ctrl+C', description: 'Quit' },
];

<KeyboardShortcuts shortcuts={replShortcuts} />
```

---

## 🎣 Custom Hooks

### useKeyboard Hook

Handle keyboard shortcuts globally.

```tsx
// lib/hooks/useKeyboard.ts
import { useEffect, useCallback } from 'react';

interface KeyBinding {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  callback: () => void;
}

export function useKeyboard(bindings: KeyBinding[]) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      for (const binding of bindings) {
        const ctrlMatch = binding.ctrl ? event.ctrlKey || event.metaKey : !event.ctrlKey && !event.metaKey;
        const shiftMatch = binding.shift ? event.shiftKey : !event.shiftKey;
        const altMatch = binding.alt ? event.altKey : !event.altKey;
        const keyMatch = event.key.toLowerCase() === binding.key.toLowerCase();

        if (ctrlMatch && shiftMatch && altMatch && keyMatch) {
          event.preventDefault();
          binding.callback();
        }
      }
    },
    [bindings]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

// Usage
useKeyboard([
  {
    key: 'c',
    ctrl: true,
    callback: () => handleQuit(),
  },
  {
    key: 'l',
    ctrl: true,
    callback: () => handleClear(),
  },
]);
```

### useTerminal Hook

Manage terminal state and command history.

```tsx
// lib/hooks/useTerminal.ts
import { useState, useCallback } from 'react';

interface Command {
  input: string;
  output: string;
  explanation?: string;
  timestamp: Date;
  riskLevel?: 'safe' | 'moderate' | 'high' | 'critical';
}

export function useTerminal() {
  const [history, setHistory] = useState<Command[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  const addCommand = useCallback((command: Command) => {
    setHistory((prev) => [...prev, command]);
    setHistoryIndex(-1);
  }, []);

  const getPreviousCommand = useCallback(() => {
    if (history.length === 0) return null;
    const newIndex = historyIndex + 1;
    if (newIndex >= history.length) return null;
    setHistoryIndex(newIndex);
    return history[history.length - 1 - newIndex].input;
  }, [history, historyIndex]);

  const getNextCommand = useCallback(() => {
    if (historyIndex <= 0) {
      setHistoryIndex(-1);
      return '';
    }
    const newIndex = historyIndex - 1;
    setHistoryIndex(newIndex);
    return history[history.length - 1 - newIndex].input;
  }, [history, historyIndex]);

  const clearHistory = useCallback(() => {
    setHistory([]);
    setHistoryIndex(-1);
  }, []);

  return {
    history,
    addCommand,
    getPreviousCommand,
    getNextCommand,
    clearHistory,
  };
}
```

---

## 🎨 Tailwind Configuration

```typescript
// tailwind.config.ts
import type { Config } from 'tailwindcss';

const config: Config = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#00FFFF',
          light: '#66FFFF',
          dark: '#00CCCC',
          glow: 'rgba(0, 255, 255, 0.3)',
        },
        success: {
          DEFAULT: '#00FF00',
          light: '#66FF66',
          dark: '#00CC00',
        },
        warning: {
          DEFAULT: '#FFFF00',
          light: '#FFFF66',
          dark: '#CCCC00',
        },
        danger: {
          DEFAULT: '#FF0000',
          light: '#FF6666',
          dark: '#CC0000',
        },
        info: {
          DEFAULT: '#0099FF',
          light: '#66B3FF',
          dark: '#0077CC',
        },
        background: {
          primary: '#0A0A0A',
          secondary: '#141414',
          tertiary: '#1E1E1E',
        },
        foreground: {
          primary: '#FFFFFF',
          secondary: '#B0B0B0',
          tertiary: '#808080',
          disabled: '#4A4A4A',
        },
        border: {
          DEFAULT: '#333333',
          light: '#4A4A4A',
          dark: '#1A1A1A',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', 'monospace'],
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
      keyframes: {
        blink: {
          '0%, 50%': { opacity: '1' },
          '51%, 100%': { opacity: '0' },
        },
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        slideUp: {
          from: { transform: 'translateY(10px)', opacity: '0' },
          to: { transform: 'translateY(0)', opacity: '1' },
        },
      },
      animation: {
        blink: 'blink 1s step-end infinite',
        fadeIn: 'fadeIn 300ms ease-out',
        slideUp: 'slideUp 300ms ease-out',
      },
    },
  },
  plugins: [],
};

export default config;
```

---

## 🚀 Next Steps for Implementers

1. **Setup Project**: `npx create-next-app@latest cmdai-web --typescript --tailwind --app`
2. **Install Dependencies**: `npm install lucide-react clsx tailwind-merge`
3. **Add Fonts**: Download JetBrains Mono, add to `public/fonts`
4. **Build Base Components**: Start with TerminalWindow, StatusBar
5. **Create TUI Simulator**: See [WEB_SIMULATOR_SPEC.md](./WEB_SIMULATOR_SPEC.md)
6. **Test Accessibility**: Run Lighthouse, test keyboard navigation
7. **Deploy**: Vercel, Netlify, or Cloudflare Pages

---

**See Also:**
- [Design System](./DESIGN_SYSTEM.md) - Visual design tokens
- [Web Simulator Spec](./WEB_SIMULATOR_SPEC.md) - Interactive demo specs
- [Master Prompts](./MASTER_PROMPTS.md) - AI-assisted implementation guides
