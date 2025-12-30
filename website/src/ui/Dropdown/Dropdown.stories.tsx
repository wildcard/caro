import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import {
  Dropdown,
  DropdownTrigger,
  DropdownPanel,
  DropdownItem,
  DropdownDivider,
  DropdownHeader,
} from './Dropdown';

const meta: Meta<typeof Dropdown> = {
  title: 'Components/Dropdown',
  component: Dropdown,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Dropdown component using compound component pattern.

**Subcomponents**:
- \`Dropdown\`: Container with context provider
- \`DropdownTrigger\`: Button that toggles the dropdown
- \`DropdownPanel\`: The dropdown menu container
- \`DropdownItem\`: Menu item with icon, title, and description
- \`DropdownDivider\`: Horizontal separator
- \`DropdownHeader\`: Section header

**Features**:
- Keyboard navigation (Enter, Space, Escape, ArrowDown)
- Click outside to close
- ARIA attributes for accessibility
- Controlled and uncontrolled modes
        `,
      },
    },
  },
  argTypes: {
    closeOnClickOutside: {
      control: 'boolean',
      description: 'Close when clicking outside',
    },
    closeOnEscape: {
      control: 'boolean',
      description: 'Close when pressing Escape',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Dropdown>;

// ============================================
// BASIC USAGE
// ============================================

export const Default: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Compare</DropdownTrigger>
      <DropdownPanel>
        <DropdownItem
          icon="📊"
          title="Overview"
          description="See all comparisons"
          href="/compare"
        />
        <DropdownItem
          icon="⚡"
          title="vs Warp"
          description="AI-native terminal"
          href="/compare/warp"
        />
        <DropdownItem
          icon="🤖"
          title="vs GitHub Copilot CLI"
          description="AI pair programmer"
          href="/compare/github-copilot-cli"
        />
      </DropdownPanel>
    </Dropdown>
  ),
};

// ============================================
// NAVIGATION DROPDOWN
// ============================================

export const CompareDropdown: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Compare</DropdownTrigger>
      <DropdownPanel>
        <DropdownItem
          icon="📊"
          title="Overview"
          description="See all comparisons"
          href="/compare"
        />
        <DropdownItem
          icon="⚡"
          title="vs Warp"
          description="AI-native terminal"
          href="/compare/warp"
        />
        <DropdownItem
          icon="🤖"
          title="vs GitHub Copilot CLI"
          description="AI pair programmer"
          href="/compare/github-copilot-cli"
        />
        <DropdownItem
          icon="☁️"
          title="vs Amazon Q CLI"
          description="AWS AI assistant"
          href="/compare/amazon-q-cli"
        />
      </DropdownPanel>
    </Dropdown>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Navigation dropdown matching the website Compare menu.',
      },
    },
  },
};

export const ResourcesDropdown: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Resources</DropdownTrigger>
      <DropdownPanel>
        <DropdownItem
          icon="📝"
          title="Blog"
          description="News & tutorials"
          href="/#blog"
        />
        <DropdownItem
          icon="🔍"
          title="Explore"
          description="Interactive demo"
          href="/explore"
        />
        <DropdownItem
          icon="🗺️"
          title="Roadmap"
          description="What's coming next"
          href="/roadmap"
        />
        <DropdownItem
          icon="🐙"
          title="GitHub"
          description="Source code"
          href="https://github.com/wildcard/caro"
          external
        />
      </DropdownPanel>
    </Dropdown>
  ),
};

// ============================================
// WITH HEADER AND DIVIDER
// ============================================

export const WithHeaderAndDivider: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Settings</DropdownTrigger>
      <DropdownPanel>
        <DropdownHeader>Theme</DropdownHeader>
        <DropdownItem icon="🎄" title="Christmas" />
        <DropdownItem icon="🕎" title="Hanukkah" />
        <DropdownItem icon="🚫" title="None" />
        <DropdownDivider />
        <DropdownItem icon="❄️" title="Snow Effect" description="Toggle snowfall" />
      </DropdownPanel>
    </Dropdown>
  ),
};

// ============================================
// ALIGNMENT
// ============================================

export const AlignLeft: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Left Aligned</DropdownTrigger>
      <DropdownPanel align="left">
        <DropdownItem icon="📊" title="Item 1" />
        <DropdownItem icon="⚡" title="Item 2" />
        <DropdownItem icon="🤖" title="Item 3" />
      </DropdownPanel>
    </Dropdown>
  ),
};

export const AlignRight: Story = {
  render: () => (
    <div style={{ display: 'flex', justifyContent: 'flex-end', width: '300px' }}>
      <Dropdown>
        <DropdownTrigger>Right Aligned</DropdownTrigger>
        <DropdownPanel align="right">
          <DropdownItem icon="📊" title="Item 1" />
          <DropdownItem icon="⚡" title="Item 2" />
          <DropdownItem icon="🤖" title="Item 3" />
        </DropdownPanel>
      </Dropdown>
    </div>
  ),
};

// ============================================
// WITHOUT CHEVRON
// ============================================

export const WithoutChevron: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger showChevron={false}>More ⋯</DropdownTrigger>
      <DropdownPanel>
        <DropdownItem icon="✏️" title="Edit" />
        <DropdownItem icon="📋" title="Copy" />
        <DropdownDivider />
        <DropdownItem icon="🗑️" title="Delete" />
      </DropdownPanel>
    </Dropdown>
  ),
};

// ============================================
// SIMPLE ITEMS (NO DESCRIPTION)
// ============================================

export const SimpleItems: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Actions</DropdownTrigger>
      <DropdownPanel>
        <DropdownItem icon="👁️" title="View" href="/view" />
        <DropdownItem icon="✏️" title="Edit" href="/edit" />
        <DropdownItem icon="📤" title="Share" href="/share" />
        <DropdownDivider />
        <DropdownItem icon="🗑️" title="Delete" onClick={() => alert('Delete clicked')} />
      </DropdownPanel>
    </Dropdown>
  ),
};

// ============================================
// CONTROLLED MODE
// ============================================

export const Controlled: Story = {
  render: () => {
    const [open, setOpen] = React.useState(false);
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', alignItems: 'center' }}>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button onClick={() => setOpen(true)}>Open</button>
          <button onClick={() => setOpen(false)}>Close</button>
        </div>
        <Dropdown open={open} onOpenChange={setOpen}>
          <DropdownTrigger>Controlled Dropdown</DropdownTrigger>
          <DropdownPanel>
            <DropdownItem icon="📊" title="Item 1" />
            <DropdownItem icon="⚡" title="Item 2" />
          </DropdownPanel>
        </Dropdown>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Controlled mode allows external control of the open state.',
      },
    },
  },
};

// ============================================
// NAVIGATION BAR EXAMPLE
// ============================================

export const NavigationBar: Story = {
  render: () => (
    <nav
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '12px 24px',
        background: 'var(--color-bg, #fff)',
        borderRadius: '12px',
        border: '1px solid var(--color-border, #e5e7eb)',
      }}
    >
      <a href="/" style={{ color: 'var(--color-text)', textDecoration: 'none', fontWeight: 500 }}>
        Features
      </a>
      <Dropdown>
        <DropdownTrigger>Compare</DropdownTrigger>
        <DropdownPanel>
          <DropdownItem icon="📊" title="Overview" href="/compare" />
          <DropdownItem icon="⚡" title="vs Warp" href="/compare/warp" />
        </DropdownPanel>
      </Dropdown>
      <Dropdown>
        <DropdownTrigger>Resources</DropdownTrigger>
        <DropdownPanel>
          <DropdownItem icon="📝" title="Blog" href="/blog" />
          <DropdownItem icon="🐙" title="GitHub" href="https://github.com" external />
        </DropdownPanel>
      </Dropdown>
    </nav>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Example of multiple dropdowns in a navigation bar.',
      },
    },
  },
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Dark Mode Dropdown</DropdownTrigger>
      <DropdownPanel>
        <DropdownItem icon="📊" title="Overview" description="See all" />
        <DropdownItem icon="⚡" title="vs Warp" description="Compare" />
        <DropdownDivider />
        <DropdownItem icon="🐙" title="GitHub" external />
      </DropdownPanel>
    </Dropdown>
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
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  render: () => (
    <Dropdown>
      <DropdownTrigger>Menu</DropdownTrigger>
      <DropdownPanel align="left">
        <DropdownItem icon="📊" title="Overview" />
        <DropdownItem icon="⚡" title="Compare" />
        <DropdownItem icon="📝" title="Blog" />
      </DropdownPanel>
    </Dropdown>
  ),
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
  },
};
