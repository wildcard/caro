# cmdai TUI Web Simulator
## Interactive Demo Specification

**Purpose:** Create an interactive web-based terminal simulator that demonstrates cmdai's TUI experience without requiring installation.

**Target URL:** `https://cmdai.dev/simulator` (or similar)

---

## 🎯 Goals

1. **Showcase TUI Features**: Demonstrate the actual cmdai TUI experience in a browser
2. **Lower Barrier to Entry**: Let users try cmdai before installing
3. **Marketing Tool**: Beautiful, shareable demo for social media/docs
4. **Educational**: Teach users how to use cmdai effectively
5. **Collect Feedback**: Gather user input on UX before full implementation

---

## 🎨 Visual Design

### Full-Page Layout

```
┌────────────────────────────────────────────────────────┐
│  [cmdai Logo]     Simulator    Docs    GitHub    Try   │  ← Nav
├────────────────────────────────────────────────────────┤
│                                                          │
│  ╭─ cmdai TUI Simulator ──────────────────────────╮    │
│  │ ⚙ Mock Backend • bash • Moderate Safety  [?]  │    │
│  ├────────────────────────────────────────────────┤    │
│  │                                                │    │
│  │  🤖 Type a command in natural language...     │    │
│  │                                                │    │
│  │  [Input field with cursor blinking]           │    │
│  │                                                │    │
│  │  ┌─ Generated Command ───────────────────┐    │    │
│  │  │ [Command output shows here]          │    │    │
│  │  └──────────────────────────────────────┘    │    │
│  │                                                │    │
│  ├────────────────────────────────────────────────┤    │
│  │ [Enter] Generate [Ctrl+L] Clear [?] Help     │    │
│  ╰────────────────────────────────────────────────╯    │
│                                                          │
│  Try these examples:                                    │
│  • "find all python files modified today"              │
│  • "compress all log files in /var/log"                │
│  • "show disk usage sorted by size"                    │
│                                                          │
└────────────────────────────────────────────────────────┘
```

---

## ⚙️ Functional Spec

### Core Features

#### 1. Command Input
- Multi-line textarea (auto-resize up to 5 lines)
- Placeholder: "🤖 Type your command in natural language..."
- Supports Enter to submit, Shift+Enter for newline
- Character counter (optional)
- Input validation (min 3 characters)

#### 2. Command Generation (Mock)
- Delay: 800ms - 1500ms (simulate real backend)
- Loading state: "⏳ Generating command..." with pulse animation
- Mock responses for common queries (see Mock Backend section)
- Error handling for unsupported queries

#### 3. Command Output Display
- Syntax-highlighted command (use highlight.js or prismjs)
- Explanation text with 💡 icon
- Risk level indicator with appropriate color/icon
- Copy button (hover to reveal)
- Optional: "Run in Docker" button (future feature)

#### 4. Safety Validation
- Display risk level: Safe ✓, Moderate ⚠, High ❌, Critical 🛑
- Show warnings for dangerous commands
- Suggest safer alternatives
- Modal confirmation for high-risk commands

#### 5. History
- Store commands in localStorage
- Up/Down arrows to navigate history
- Ctrl+R to search history (modal overlay)
- Clear history button

#### 6. Help Modal
- Triggered by [?] button or Ctrl+H
- Shows all keyboard shortcuts
- Links to full documentation
- Escape to close

#### 7. Examples Gallery
- 5-10 pre-written examples users can click
- Each click populates input and auto-generates
- Categories: File Operations, System Info, Git, Docker, etc.

---

## 🤖 Mock Backend

### Mock Response Generator

```typescript
// lib/simulator/mockBackend.ts

interface MockResponse {
  command: string;
  explanation: string;
  riskLevel: 'safe' | 'moderate' | 'high' | 'critical';
  warnings?: string[];
  alternatives?: string[];
}

const mockResponses: Record<string, MockResponse> = {
  // File operations
  'find all python files': {
    command: 'find . -type f -name "*.py"',
    explanation: 'Searches current directory recursively for all Python files',
    riskLevel: 'safe',
  },

  'find python files modified today': {
    command: 'find . -type f -name "*.py" -mtime -1',
    explanation: 'Finds Python files modified in the last 24 hours',
    riskLevel: 'safe',
  },

  'delete all log files': {
    command: 'rm -rf *.log',
    explanation: 'Recursively deletes all files ending in .log',
    riskLevel: 'high',
    warnings: [
      'Recursive deletion without confirmation',
      'Cannot be undone',
    ],
    alternatives: [
      'rm -i *.log  # Interactive deletion',
      'mv *.log ~/.trash/  # Move to trash instead',
    ],
  },

  'show disk usage': {
    command: 'df -h',
    explanation: 'Displays disk usage in human-readable format',
    riskLevel: 'safe',
  },

  'compress all logs': {
    command: 'tar -czf logs.tar.gz *.log',
    explanation: 'Creates a gzip-compressed tarball of all log files',
    riskLevel: 'safe',
  },

  // Git operations
  'show git status': {
    command: 'git status --short',
    explanation: 'Shows the current state of the Git repository in short format',
    riskLevel: 'safe',
  },

  'create git branch': {
    command: 'git checkout -b feature/new-branch',
    explanation: 'Creates and switches to a new branch named feature/new-branch',
    riskLevel: 'safe',
  },

  // System info
  'show running processes': {
    command: 'ps aux | head -20',
    explanation: 'Lists the top 20 running processes with details',
    riskLevel: 'safe',
  },

  'kill all node processes': {
    command: 'pkill -9 node',
    explanation: 'Forcefully terminates all Node.js processes',
    riskLevel: 'moderate',
    warnings: [
      'Will kill ALL node processes, including important services',
    ],
    alternatives: [
      'pkill node  # Graceful termination',
      'ps aux | grep node  # Find specific process first',
    ],
  },
};

export function generateMockResponse(input: string): MockResponse {
  const normalizedInput = input.toLowerCase().trim();

  // Check for exact matches first
  if (mockResponses[normalizedInput]) {
    return mockResponses[normalizedInput];
  }

  // Fuzzy matching (contains keywords)
  for (const [key, response] of Object.entries(mockResponses)) {
    const keywords = key.split(' ');
    if (keywords.every(keyword => normalizedInput.includes(keyword))) {
      return response;
    }
  }

  // Fallback for unknown queries
  return {
    command: 'echo "Command generation would happen here"',
    explanation: `This is a mock simulator. In the real cmdai, your query "${input}" would be processed by a local LLM.`,
    riskLevel: 'safe',
  };
}

// Simulate network delay
export async function simulateGeneration(
  input: string
): Promise<MockResponse> {
  const delay = Math.random() * 700 + 800; // 800-1500ms
  await new Promise(resolve => setTimeout(resolve, delay));
  return generateMockResponse(input);
}
```

---

## 🎮 Interactive Features

### Keyboard Shortcuts

```typescript
const keyboardShortcuts = {
  'Enter': 'Generate command',
  'Shift+Enter': 'New line in input',
  'Ctrl+L': 'Clear input',
  'Ctrl+C': 'Cancel generation',
  'Ctrl+R': 'Search history',
  'Up Arrow': 'Previous command',
  'Down Arrow': 'Next command',
  '?': 'Toggle help',
  'Esc': 'Close modal',
};
```

### Mouse Interactions

- **Hover Command**: Show copy button
- **Click Example**: Auto-fill and generate
- **Click Shortcut**: Show tooltip with description
- **Click Status**: Show backend info tooltip

---

## 🎬 Animations & Transitions

### Loading State

```tsx
<div className="flex items-center gap-2 text-primary animate-pulse">
  <span className="animate-spin">⏳</span>
  <span>Generating command...</span>
</div>
```

### Command Appears

```tsx
<div className="animate-slideUp">
  <CommandOutput {...response} />
</div>
```

### Cursor Blink

```tsx
<span className="animate-blink">_</span>
```

### Risk Level Flash

High/Critical risk commands should briefly flash to draw attention:

```tsx
<div className="animate-pulse" style={{ animationIterationCount: 3 }}>
  ❌ HIGH RISK
</div>
```

---

## 📱 Responsive Design

### Desktop (> 768px)
- Full terminal window with all features
- Side-by-side examples and terminal
- Multi-column help modal

### Tablet (768px - 1024px)
- Stacked layout (examples above terminal)
- Scrollable help modal

### Mobile (< 768px)
- Single column
- Collapsible examples section
- Full-screen terminal on focus
- Simplified help (scrollable list)

---

## 🔄 State Management

```typescript
// State structure
interface SimulatorState {
  // Input
  input: string;
  cursorPosition: number;

  // Generation
  isGenerating: boolean;
  currentResponse: MockResponse | null;
  error: string | null;

  // History
  commandHistory: Array<{
    input: string;
    response: MockResponse;
    timestamp: Date;
  }>;
  historyIndex: number;

  // UI
  showHelp: boolean;
  showHistorySearch: boolean;
}

// Example with React hooks
function useSimulator() {
  const [state, setState] = useState<SimulatorState>({
    input: '',
    cursorPosition: 0,
    isGenerating: false,
    currentResponse: null,
    error: null,
    commandHistory: [],
    historyIndex: -1,
    showHelp: false,
    showHistorySearch: false,
  });

  const generateCommand = async () => {
    setState(prev => ({ ...prev, isGenerating: true, error: null }));

    try {
      const response = await simulateGeneration(state.input);
      setState(prev => ({
        ...prev,
        isGenerating: false,
        currentResponse: response,
        commandHistory: [
          ...prev.commandHistory,
          { input: prev.input, response, timestamp: new Date() },
        ],
      }));
    } catch (error) {
      setState(prev => ({
        ...prev,
        isGenerating: false,
        error: 'Generation failed. Please try again.',
      }));
    }
  };

  return { state, setState, generateCommand };
}
```

---

## 🎨 Example Gallery

### Pre-Built Examples

```typescript
const examples = [
  {
    category: 'File Operations',
    queries: [
      'find all python files modified today',
      'compress all log files in current directory',
      'show disk usage sorted by size',
    ],
  },
  {
    category: 'Git',
    queries: [
      'show git status',
      'create a new branch called feature/auth',
      'show last 5 commits with file changes',
    ],
  },
  {
    category: 'System Info',
    queries: [
      'show CPU and memory usage',
      'list running docker containers',
      'show network connections',
    ],
  },
  {
    category: 'Text Processing',
    queries: [
      'find and replace "foo" with "bar" in all JS files',
      'count lines in all Python files',
      'search for "TODO" in current directory',
    ],
  },
];

// Component
<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
  {examples.map(category => (
    <div key={category.category}>
      <h3 className="font-mono text-sm text-primary mb-2">
        {category.category}
      </h3>
      <div className="space-y-1">
        {category.queries.map(query => (
          <button
            key={query}
            onClick={() => {
              setInput(query);
              generateCommand();
            }}
            className="w-full text-left px-3 py-2 font-mono text-xs bg-background-tertiary border border-border rounded hover:border-primary hover:text-primary transition-colors"
          >
            "{query}"
          </button>
        ))}
      </div>
    </div>
  ))}
</div>
```

---

## 🚀 Implementation Checklist

### Phase 1: Basic Simulator
- [ ] Setup Next.js project with Tailwind
- [ ] Create TerminalWindow component
- [ ] Implement InputArea with keyboard shortcuts
- [ ] Build mock backend with 10 responses
- [ ] Add CommandOutput display
- [ ] Test on desktop/mobile

### Phase 2: Enhanced UX
- [ ] Add history (localStorage)
- [ ] Implement Up/Down navigation
- [ ] Create help modal
- [ ] Add examples gallery
- [ ] Implement copy-to-clipboard

### Phase 3: Polish
- [ ] Add loading animations
- [ ] Implement risk level warnings
- [ ] Create confirmation modals
- [ ] Add syntax highlighting
- [ ] Accessibility audit

### Phase 4: Analytics & Feedback
- [ ] Add event tracking (optional)
- [ ] Create feedback form
- [ ] Monitor popular queries
- [ ] A/B test different UX approaches

---

## 📊 Success Metrics

- **Engagement**: Average session duration > 2 minutes
- **Conversion**: 30%+ click "Install cmdai" after using simulator
- **Queries**: Average 5+ commands generated per session
- **Social**: 100+ shares on Twitter/Reddit
- **Performance**: Lighthouse score > 95

---

## 🔗 Integration Points

### With Marketing Site
- Hero section: "Try it now →" button opens simulator
- Docs: Embedded simulator for live examples
- Blog posts: Interactive code samples

### With Main CLI
- "Try online" link in `cmdai --help`
- QR code in terminal for mobile users
- Link in error messages for learning

---

## 🎭 Easter Eggs (Optional)

Fun surprises for engaged users:

1. **Konami Code**: Up, Up, Down, Down, Left, Right, Left, Right, B, A → Matrix rain animation
2. **Type "make me a sandwich"**: Response: "What? Make it yourself." / "sudo make me a sandwich" → "Okay."
3. **Type "hello"**: Friendly response with tips
4. **100th command**: Confetti animation + "Power user unlocked!" badge

---

**See Also:**
- [Component Architecture](./COMPONENT_ARCHITECTURE.md) - How to build components
- [Design System](./DESIGN_SYSTEM.md) - Visual design tokens
- [Master Prompts](./MASTER_PROMPTS.md) - AI implementation guides
