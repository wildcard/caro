import type { Meta, StoryObj } from '@storybook/react';
import { CodeBlock } from './CodeBlock';

const meta: Meta<typeof CodeBlock> = {
  title: 'Components/CodeBlock',
  component: CodeBlock,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Minimal code block with copy-to-clipboard functionality.

**Features**:
- Automatic inline/block detection based on content
- Low-profile copy button (appears on hover)
- Optional line numbers for multiline blocks
- Works with both single-line and multiline code
- Accessible keyboard navigation

**Design Philosophy**:
- Subtle UX that doesn't distract from content
- Copy button only visible when needed
- Optimized for both usability and maintainability
        `,
      },
    },
  },
  argTypes: {
    children: {
      control: 'text',
      description: 'Code content to display',
    },
    variant: {
      control: 'select',
      options: ['default', 'muted'],
      description: 'Visual variant',
    },
    lang: {
      control: 'text',
      description: 'Language for accessibility (not displayed)',
    },
    lineNumbers: {
      control: 'boolean',
      description: 'Show line numbers',
    },
    noCopy: {
      control: 'boolean',
      description: 'Disable copy functionality',
    },
  },
};

export default meta;
type Story = StoryObj<typeof CodeBlock>;

// ============================================
// INLINE CODE (single line)
// ============================================

export const Inline: Story = {
  args: {
    children: 'npm install caro',
  },
  parameters: {
    docs: {
      description: {
        story: 'Single-line code automatically renders inline with a subtle copy button.',
      },
    },
  },
};

export const InlineCommand: Story = {
  args: {
    children: 'cargo install caro',
    lang: 'bash',
  },
};

export const InlineLong: Story = {
  args: {
    children: 'bash <(curl -sSfL https://setup.caro.sh)',
    lang: 'bash',
  },
};

// ============================================
// BLOCK CODE (multiline)
// ============================================

export const Block: Story = {
  args: {
    children: `const greeting = 'Hello, World!';
console.log(greeting);`,
    lang: 'typescript',
  },
  parameters: {
    docs: {
      description: {
        story: 'Multiline code automatically renders as a block with copy button in top-right.',
      },
    },
  },
};

export const BlockWithLineNumbers: Story = {
  args: {
    children: `function fibonacci(n: number): number {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log(fibonacci(10));`,
    lang: 'typescript',
    lineNumbers: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Enable `lineNumbers` for longer code blocks to improve readability.',
      },
    },
  },
};

export const BlockLong: Story = {
  args: {
    children: `use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut input = String::new();

    print!("Describe what you want to do: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;

    let command = generate_command(&input.trim());
    println!("Generated: {}", command);

    Ok(())
}`,
    lang: 'rust',
    lineNumbers: true,
  },
};

// ============================================
// VARIANTS
// ============================================

export const VariantDefault: Story = {
  args: {
    children: 'const x = 42;',
    variant: 'default',
  },
};

export const VariantMuted: Story = {
  args: {
    children: 'const x = 42;',
    variant: 'muted',
  },
};

// ============================================
// WITHOUT COPY
// ============================================

export const NoCopy: Story = {
  args: {
    children: 'read-only code',
    noCopy: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Set `noCopy` to hide the copy button for read-only displays.',
      },
    },
  },
};

// ============================================
// USE CASES
// ============================================

export const InstallCommand: Story = {
  args: {
    children: 'cargo install caro',
    lang: 'bash',
  },
  parameters: {
    docs: {
      description: {
        story: 'Typical usage for install commands.',
      },
    },
  },
};

export const ConfigExample: Story = {
  args: {
    children: `{
  "model": "mlx",
  "safety": true,
  "confirm": "always"
}`,
    lang: 'json',
    lineNumbers: true,
  },
};

export const ShellScript: Story = {
  args: {
    children: `#!/bin/bash
set -e

echo "Installing Caro..."
cargo install caro

echo "Done!"`,
    lang: 'bash',
    lineNumbers: true,
  },
};

// ============================================
// ALL VARIANTS
// ============================================

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div>
        <p style={{ margin: '0 0 8px', fontSize: '12px', color: '#666' }}>Default</p>
        <CodeBlock variant="default">npm install caro</CodeBlock>
      </div>
      <div>
        <p style={{ margin: '0 0 8px', fontSize: '12px', color: '#666' }}>Muted</p>
        <CodeBlock variant="muted">npm install caro</CodeBlock>
      </div>
    </div>
  ),
};

// ============================================
// INLINE VS BLOCK COMPARISON
// ============================================

export const InlineVsBlock: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '500px' }}>
      <div>
        <p style={{ margin: '0 0 8px', fontSize: '12px', color: '#666' }}>
          Inline (auto-detected from single line)
        </p>
        <CodeBlock>npm install caro</CodeBlock>
      </div>
      <div>
        <p style={{ margin: '0 0 8px', fontSize: '12px', color: '#666' }}>
          Block (auto-detected from multiline)
        </p>
        <CodeBlock>{`npm install caro
caro --version`}</CodeBlock>
      </div>
    </div>
  ),
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <CodeBlock>npm install caro</CodeBlock>
      <CodeBlock lineNumbers>{`const x = 1;
const y = 2;
console.log(x + y);`}</CodeBlock>
    </div>
  ),
  parameters: {
    backgrounds: {
      default: 'dark',
    },
  },
  decorators: [
    (Story) => {
      document.documentElement.classList.add('dark');
      return <Story />;
    },
  ],
};

// ============================================
// MOBILE
// ============================================

export const Mobile: Story = {
  args: {
    children: `// On mobile, copy button is always visible
const isMobile = window.innerWidth < 600;`,
    lineNumbers: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile1',
    },
  },
};
