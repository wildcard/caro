import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { CopyCodeBlock } from './CopyCodeBlock';

const meta: Meta<typeof CopyCodeBlock> = {
  title: 'Components/CopyCodeBlock',
  component: CopyCodeBlock,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Code block with copy-to-clipboard functionality.

**Variants**:
- \`dark\`: Dark terminal-style background
- \`light\`: Light background for inline code
- \`brand\`: Semi-transparent for use on brand gradient backgrounds

**Sizes**:
- \`sm\`: Compact padding, 13px font
- \`md\`: Standard padding, 14px font
- \`lg\`: Generous padding, 16px font
        `,
      },
    },
  },
  argTypes: {
    code: {
      control: 'text',
      description: 'Code content to display',
    },
    variant: {
      control: 'select',
      options: ['dark', 'light', 'brand'],
      description: 'Visual variant',
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Size variant',
    },
    showLineNumbers: {
      control: 'boolean',
      description: 'Show line numbers',
    },
    language: {
      control: 'text',
      description: 'Language label for header',
    },
    disableCopy: {
      control: 'boolean',
      description: 'Disable copy button',
    },
  },
};

export default meta;
type Story = StoryObj<typeof CopyCodeBlock>;

// ============================================
// BASIC USAGE
// ============================================

export const Default: Story = {
  args: {
    code: 'cargo install caro',
    variant: 'dark',
  },
};

export const WithLanguage: Story = {
  args: {
    code: 'npm install @caro/sdk',
    language: 'bash',
    variant: 'dark',
  },
};

// ============================================
// VARIANTS
// ============================================

export const Dark: Story = {
  args: {
    code: 'cargo install caro',
    variant: 'dark',
    language: 'shell',
  },
};

export const Light: Story = {
  args: {
    code: 'const caro = require("caro")',
    variant: 'light',
    language: 'javascript',
  },
  parameters: {
    backgrounds: {
      default: 'light',
    },
  },
};

export const Brand: Story = {
  args: {
    code: 'bash <(curl --proto \'=https\' --tlsv1.2 -sSfL https://setup.caro.sh)',
    variant: 'brand',
    size: 'lg',
  },
  decorators: [
    (Story) => (
      <div
        style={{
          background: 'linear-gradient(135deg, #ff8c42 0%, #ff6b35 100%)',
          padding: '40px',
          borderRadius: '12px',
          maxWidth: '600px',
        }}
      >
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story:
          'Brand variant designed for use on orange gradient backgrounds, like the download section.',
      },
    },
  },
};

// ============================================
// SIZES
// ============================================

export const Small: Story = {
  args: {
    code: 'caro "list files"',
    size: 'sm',
    variant: 'dark',
  },
};

export const Medium: Story = {
  args: {
    code: 'caro "find files modified today"',
    size: 'md',
    variant: 'dark',
  },
};

export const Large: Story = {
  args: {
    code: 'bash <(curl -sSfL https://setup.caro.sh)',
    size: 'lg',
    variant: 'dark',
  },
};

// ============================================
// LINE NUMBERS
// ============================================

export const WithLineNumbers: Story = {
  args: {
    code: `use caro::Config;

fn main() {
    let config = Config::default();
    config.run();
}`,
    variant: 'dark',
    language: 'rust',
    showLineNumbers: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Multi-line code with line numbers for better readability.',
      },
    },
  },
};

// ============================================
// MULTILINE CODE
// ============================================

export const MultilineCode: Story = {
  args: {
    code: `# Install Caro
cargo install caro

# Configure your backend
caro config --backend ollama

# Start using Caro
caro "list all files"`,
    variant: 'dark',
    language: 'bash',
  },
};

// ============================================
// WITHOUT COPY
// ============================================

export const NoCopyButton: Story = {
  args: {
    code: 'caro "show disk usage"',
    disableCopy: true,
    variant: 'dark',
  },
  parameters: {
    docs: {
      description: {
        story: 'Code block without copy functionality, useful for display-only scenarios.',
      },
    },
  },
};

// ============================================
// INSTALLATION EXAMPLES
// ============================================

export const CargoInstall: Story = {
  args: {
    code: 'cargo install caro',
    variant: 'dark',
    language: 'shell',
  },
  parameters: {
    docs: {
      description: {
        story: 'Standard Rust cargo installation command.',
      },
    },
  },
};

export const CurlInstall: Story = {
  args: {
    code: 'bash <(curl --proto \'=https\' --tlsv1.2 -sSfL https://setup.caro.sh)',
    variant: 'dark',
    language: 'shell',
    size: 'md',
  },
  parameters: {
    docs: {
      description: {
        story: 'Curl-based installation script for quick setup.',
      },
    },
  },
};

// ============================================
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  args: {
    code: 'bash <(curl -sSfL https://setup.caro.sh)',
    variant: 'dark',
    size: 'md',
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
  },
};

// ============================================
// ALL VARIANTS COMPARISON
// ============================================

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '500px' }}>
      <div>
        <h4 style={{ marginBottom: '8px', color: '#666' }}>Dark</h4>
        <CopyCodeBlock code="cargo install caro" variant="dark" language="shell" />
      </div>
      <div>
        <h4 style={{ marginBottom: '8px', color: '#666' }}>Light</h4>
        <CopyCodeBlock
          code="cargo install caro"
          variant="light"
          language="shell"
        />
      </div>
      <div
        style={{
          background: 'linear-gradient(135deg, #ff8c42 0%, #ff6b35 100%)',
          padding: '20px',
          borderRadius: '8px',
        }}
      >
        <h4 style={{ marginBottom: '8px', color: 'white' }}>Brand</h4>
        <CopyCodeBlock code="cargo install caro" variant="brand" />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Comparison of all three variants side by side.',
      },
    },
  },
};

// ============================================
// ALL SIZES COMPARISON
// ============================================

export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '500px' }}>
      <div>
        <h4 style={{ marginBottom: '8px', color: '#666' }}>Small</h4>
        <CopyCodeBlock code="caro help" variant="dark" size="sm" />
      </div>
      <div>
        <h4 style={{ marginBottom: '8px', color: '#666' }}>Medium</h4>
        <CopyCodeBlock code="cargo install caro" variant="dark" size="md" />
      </div>
      <div>
        <h4 style={{ marginBottom: '8px', color: '#666' }}>Large</h4>
        <CopyCodeBlock code="bash <(curl -sSfL https://setup.caro.sh)" variant="dark" size="lg" />
      </div>
    </div>
  ),
};
