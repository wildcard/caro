import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { Terminal, TerminalLine } from './Terminal';

const meta: Meta<typeof Terminal> = {
  title: 'Components/Terminal',
  component: Terminal,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    backgrounds: {
      default: 'light',
    },
    docs: {
      description: {
        component: `
Terminal window component with macOS-style header.

**Features**:
- Traffic light dots (red, yellow, green)
- Optional title bar
- TerminalLine subcomponent for different line types
- Animated variant that cycles through examples
- Optional copy button
- Responsive design for mobile

**Line Types**:
- \`command\`: Green prompt with blue command text
- \`caro\`: Orange "🐕 Caro:" prefix with brown output
- \`output\`: Standard gray text
- \`status\`: Green checkmark with success message
        `,
      },
    },
  },
  argTypes: {
    title: {
      control: 'text',
      description: 'Optional title in header',
    },
    variant: {
      control: 'select',
      options: ['default', 'animated'],
      description: 'Default shows children, animated cycles through examples',
    },
    showCopy: {
      control: 'boolean',
      description: 'Show copy button in header',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Terminal>;

// ============================================
// BASIC USAGE
// ============================================

export const Default: Story = {
  render: () => (
    <Terminal title="caro — shell companion">
      <TerminalLine type="command">caro "list all files modified today"</TerminalLine>
      <TerminalLine type="caro">find . -type f -mtime 0</TerminalLine>
      <TerminalLine type="status">Safe to run on your system</TerminalLine>
    </Terminal>
  ),
};

export const WithoutTitle: Story = {
  render: () => (
    <Terminal>
      <TerminalLine type="command">caro "find python files"</TerminalLine>
      <TerminalLine type="caro">find . -name "*.py" -type f</TerminalLine>
      <TerminalLine type="status">Safe to run</TerminalLine>
    </Terminal>
  ),
};

// ============================================
// ANIMATED VARIANT
// ============================================

export const Animated: Story = {
  args: {
    variant: 'animated',
    title: 'caro — shell companion',
    examples: [
      {
        command: 'caro "list all files modified today"',
        output: 'find . -type f -mtime 0',
        status: 'Safe to run on your system',
      },
      {
        command: 'caro "find large files over 100MB"',
        output: 'find . -type f -size +100M',
        status: 'Safe to run on your system',
      },
      {
        command: 'caro "show disk usage by folder"',
        output: 'du -sh */ | sort -rh | head -10',
        status: 'Safe to run on your system',
      },
      {
        command: 'caro "find python files modified last week"',
        output: 'find . -name "*.py" -type f -mtime -7',
        status: 'Safe to run on your system',
      },
    ],
    animationInterval: 3000,
  },
  parameters: {
    docs: {
      description: {
        story: 'Animated terminal that cycles through multiple examples. Used on the homepage to showcase different commands.',
      },
    },
  },
};

// ============================================
// WITH COPY BUTTON
// ============================================

export const WithCopyButton: Story = {
  args: {
    title: 'Installation',
    showCopy: true,
    copyText: 'cargo install caro',
  },
  render: (args) => (
    <Terminal {...args}>
      <TerminalLine type="command">cargo install caro</TerminalLine>
    </Terminal>
  ),
};

// ============================================
// LINE TYPES
// ============================================

export const AllLineTypes: Story = {
  render: () => (
    <Terminal title="Line Types Demo">
      <TerminalLine type="command">Command line (type="command")</TerminalLine>
      <TerminalLine type="caro">Caro output (type="caro")</TerminalLine>
      <TerminalLine type="output">Plain output (type="output")</TerminalLine>
      <TerminalLine type="status">Status message (type="status")</TerminalLine>
    </Terminal>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Demonstration of all available line types with their default styling.',
      },
    },
  },
};

export const CustomPrefixes: Story = {
  render: () => (
    <Terminal title="Custom Prefixes">
      <TerminalLine type="command" prefix="→">Custom arrow prefix</TerminalLine>
      <TerminalLine type="output" prefix="[info]">Info message</TerminalLine>
      <TerminalLine type="status" prefix="">No prefix at all</TerminalLine>
    </Terminal>
  ),
};

// ============================================
// USE CASES
// ============================================

export const InstallationExample: Story = {
  render: () => (
    <Terminal title="Quick Install" showCopy copyText="cargo install caro">
      <TerminalLine type="command">cargo install caro</TerminalLine>
      <TerminalLine type="output">Downloading caro v1.0.0</TerminalLine>
      <TerminalLine type="output">Compiling caro v1.0.0</TerminalLine>
      <TerminalLine type="status">Installed successfully</TerminalLine>
    </Terminal>
  ),
};

export const ComplexExample: Story = {
  render: () => (
    <Terminal title="caro — shell companion">
      <TerminalLine type="command">caro "delete all node_modules folders"</TerminalLine>
      <TerminalLine type="caro">find . -name "node_modules" -type d -prune -exec rm -rf '{}' +</TerminalLine>
      <TerminalLine type="output" prefix="⚠️">This command will delete files. Caro requires confirmation.</TerminalLine>
      <TerminalLine type="command">Proceed? [y/N]</TerminalLine>
    </Terminal>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Example showing Caro\'s safety features - warning about potentially destructive commands.',
      },
    },
  },
};

// ============================================
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  render: () => (
    <Terminal title="caro">
      <TerminalLine type="command">caro "list files"</TerminalLine>
      <TerminalLine type="caro">ls -la</TerminalLine>
      <TerminalLine type="status">Safe</TerminalLine>
    </Terminal>
  ),
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
  },
};

// ============================================
// DARK BACKGROUND
// ============================================

export const OnDarkBackground: Story = {
  render: () => (
    <div style={{ padding: '40px', background: '#1a1a2e', borderRadius: '12px' }}>
      <Terminal title="caro — shell companion">
        <TerminalLine type="command">caro "find large files"</TerminalLine>
        <TerminalLine type="caro">find . -type f -size +100M</TerminalLine>
        <TerminalLine type="status">Safe to run</TerminalLine>
      </Terminal>
    </div>
  ),
  parameters: {
    backgrounds: {
      default: 'dark',
    },
  },
};

// ============================================
// STACKED TERMINALS
// ============================================

export const MultipleTerminals: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      <Terminal title="Step 1: Install">
        <TerminalLine type="command">cargo install caro</TerminalLine>
      </Terminal>
      <Terminal title="Step 2: Configure">
        <TerminalLine type="command">caro config --backend ollama</TerminalLine>
      </Terminal>
      <Terminal title="Step 3: Use">
        <TerminalLine type="command">caro "your first command"</TerminalLine>
        <TerminalLine type="caro">echo "Hello, World!"</TerminalLine>
        <TerminalLine type="status">Ready to go!</TerminalLine>
      </Terminal>
    </div>
  ),
};
