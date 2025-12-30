import React, { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { IconButton } from './IconButton';

// ============================================
// ICON COMPONENTS
// ============================================

const SunIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="12" cy="12" r="5" />
    <line x1="12" y1="1" x2="12" y2="3" />
    <line x1="12" y1="21" x2="12" y2="23" />
    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
    <line x1="1" y1="12" x2="3" y2="12" />
    <line x1="21" y1="12" x2="23" y2="12" />
    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
  </svg>
);

const MoonIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
  </svg>
);

const MenuIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <line x1="3" y1="12" x2="21" y2="12" />
    <line x1="3" y1="6" x2="21" y2="6" />
    <line x1="3" y1="18" x2="21" y2="18" />
  </svg>
);

const CloseIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  </svg>
);

const SearchIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="11" cy="11" r="8" />
    <line x1="21" y1="21" x2="16.65" y2="16.65" />
  </svg>
);

const GitHubIcon = () => (
  <svg viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
  </svg>
);

// ============================================
// META
// ============================================

const meta: Meta<typeof IconButton> = {
  title: 'Components/IconButton',
  component: IconButton,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Square icon-only button for actions like theme toggle, menu trigger, etc.

**Sizes**:
- \`sm\`: 32px (with 44px touch target)
- \`md\`: 40px
- \`lg\`: 48px

**Variants**:
- \`default\`: Standard with background and border
- \`ghost\`: Transparent background
- \`brand\`: Orange brand color

**States**:
- \`active\`: Visual highlight for toggle states
- \`loading\`: Shows spinner
- \`disabled\`: Reduced opacity and no interaction
        `,
      },
    },
  },
  argTypes: {
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Button size',
    },
    variant: {
      control: 'select',
      options: ['default', 'ghost', 'brand'],
      description: 'Visual variant',
    },
    active: {
      control: 'boolean',
      description: 'Active state for toggles',
    },
    loading: {
      control: 'boolean',
      description: 'Loading state',
    },
    disabled: {
      control: 'boolean',
      description: 'Disabled state',
    },
    label: {
      control: 'text',
      description: 'Accessible label',
    },
  },
};

export default meta;
type Story = StoryObj<typeof IconButton>;

// ============================================
// BASIC USAGE
// ============================================

export const Default: Story = {
  args: {
    icon: <MenuIcon />,
    label: 'Open menu',
  },
};

export const WithSunIcon: Story = {
  args: {
    icon: <SunIcon />,
    label: 'Toggle light mode',
  },
};

export const WithMoonIcon: Story = {
  args: {
    icon: <MoonIcon />,
    label: 'Toggle dark mode',
  },
};

// ============================================
// SIZES
// ============================================

export const Small: Story = {
  args: {
    icon: <MenuIcon />,
    label: 'Small button',
    size: 'sm',
  },
};

export const Medium: Story = {
  args: {
    icon: <MenuIcon />,
    label: 'Medium button',
    size: 'md',
  },
};

export const Large: Story = {
  args: {
    icon: <MenuIcon />,
    label: 'Large button',
    size: 'lg',
  },
};

export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
      <IconButton icon={<MenuIcon />} label="Small" size="sm" />
      <IconButton icon={<MenuIcon />} label="Medium" size="md" />
      <IconButton icon={<MenuIcon />} label="Large" size="lg" />
    </div>
  ),
};

// ============================================
// VARIANTS
// ============================================

export const Ghost: Story = {
  args: {
    icon: <SearchIcon />,
    label: 'Search',
    variant: 'ghost',
  },
};

export const Brand: Story = {
  args: {
    icon: <GitHubIcon />,
    label: 'View on GitHub',
    variant: 'brand',
  },
};

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
      <IconButton icon={<MenuIcon />} label="Default" variant="default" />
      <IconButton icon={<MenuIcon />} label="Ghost" variant="ghost" />
      <IconButton icon={<MenuIcon />} label="Brand" variant="brand" />
    </div>
  ),
};

// ============================================
// STATES
// ============================================

export const Active: Story = {
  args: {
    icon: <SunIcon />,
    label: 'Light mode active',
    active: true,
  },
};

export const Loading: Story = {
  args: {
    icon: <SearchIcon />,
    label: 'Loading',
    loading: true,
  },
};

export const Disabled: Story = {
  args: {
    icon: <MenuIcon />,
    label: 'Disabled',
    disabled: true,
  },
};

// ============================================
// USE CASES
// ============================================

export const ThemeToggle: Story = {
  render: () => {
    const [isDark, setIsDark] = useState(false);
    return (
      <IconButton
        icon={isDark ? <SunIcon /> : <MoonIcon />}
        label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
        active={isDark}
        onClick={() => setIsDark(!isDark)}
      />
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive theme toggle button that switches between sun and moon icons.',
      },
    },
  },
};

export const MobileMenuToggle: Story = {
  render: () => {
    const [isOpen, setIsOpen] = useState(false);
    return (
      <IconButton
        icon={isOpen ? <CloseIcon /> : <MenuIcon />}
        label={isOpen ? 'Close menu' : 'Open menu'}
        active={isOpen}
        onClick={() => setIsOpen(!isOpen)}
      />
    );
  },
};

export const GitHubButton: Story = {
  args: {
    icon: <GitHubIcon />,
    label: 'View on GitHub',
    variant: 'default',
  },
};

// ============================================
// TOOLBAR EXAMPLE
// ============================================

export const Toolbar: Story = {
  render: () => (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '8px',
        background: 'var(--color-bg-secondary, #f5f5f5)',
        borderRadius: '12px',
      }}
    >
      <IconButton icon={<SearchIcon />} label="Search" variant="ghost" />
      <IconButton icon={<GitHubIcon />} label="GitHub" variant="ghost" />
      <div style={{ width: '1px', height: '24px', background: 'var(--color-border)' }} />
      <IconButton icon={<MoonIcon />} label="Dark mode" variant="ghost" />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Example toolbar with multiple icon buttons.',
      },
    },
  },
};

// ============================================
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  args: {
    icon: <MenuIcon />,
    label: 'Menu',
    size: 'md',
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
  },
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  args: {
    icon: <SunIcon />,
    label: 'Toggle theme',
  },
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
