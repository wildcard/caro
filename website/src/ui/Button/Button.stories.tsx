import type { Meta, StoryObj } from '@storybook/react';
import { Button } from './Button';

const meta: Meta<typeof Button> = {
  title: 'Components/Button',
  component: Button,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Button component for user interactions. Follows the Caro design system with
playful, gradient-based primary buttons and clear visual hierarchy.

**Touch Targets**: All sizes meet or exceed the 44px minimum touch target.

**Accessibility**: Includes focus-visible states, aria-busy for loading,
and supports reduced motion preferences.
        `,
      },
    },
  },
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'ghost', 'support'],
      description: 'Visual style variant',
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Button size',
    },
    fullWidth: {
      control: 'boolean',
      description: 'Whether button fills container width',
    },
    loading: {
      control: 'boolean',
      description: 'Show loading spinner',
    },
    disabled: {
      control: 'boolean',
      description: 'Disable the button',
    },
    children: {
      control: 'text',
      description: 'Button content',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Button>;

// ============================================
// PRIMARY VARIANT
// ============================================

export const Primary: Story = {
  args: {
    variant: 'primary',
    children: 'Get Started',
  },
};

export const PrimaryLarge: Story = {
  args: {
    variant: 'primary',
    size: 'lg',
    children: 'Get Started',
  },
};

export const PrimaryWithIcon: Story = {
  args: {
    variant: 'primary',
    children: 'Download',
    rightIcon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <path
          d="M3 8H13M13 8L9 4M13 8L9 12"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    ),
  },
};

// ============================================
// SECONDARY VARIANT
// ============================================

export const Secondary: Story = {
  args: {
    variant: 'secondary',
    children: 'Learn More',
  },
};

export const SecondarySmall: Story = {
  args: {
    variant: 'secondary',
    size: 'sm',
    children: 'View Docs',
  },
};

// ============================================
// GHOST VARIANT
// ============================================

export const Ghost: Story = {
  args: {
    variant: 'ghost',
    children: 'Cancel',
  },
};

// ============================================
// SUPPORT VARIANT (Pink)
// ============================================

export const Support: Story = {
  args: {
    variant: 'support',
    children: 'Sponsor',
    leftIcon: (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 3C8.5 2 9.5 1 11 1C13 1 15 2.5 15 5C15 9 8 14 8 14C8 14 1 9 1 5C1 2.5 3 1 5 1C6.5 1 7.5 2 8 3Z" />
      </svg>
    ),
  },
};

// ============================================
// STATES
// ============================================

export const Loading: Story = {
  args: {
    variant: 'primary',
    children: 'Loading...',
    loading: true,
  },
};

export const Disabled: Story = {
  args: {
    variant: 'primary',
    children: 'Disabled',
    disabled: true,
  },
};

export const FullWidth: Story = {
  args: {
    variant: 'primary',
    children: 'Full Width Button',
    fullWidth: true,
  },
  parameters: {
    layout: 'padded',
  },
};

// ============================================
// SIZE COMPARISON
// ============================================

export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
      <Button size="sm">Small</Button>
      <Button size="md">Medium</Button>
      <Button size="lg">Large</Button>
    </div>
  ),
};

// ============================================
// VARIANT COMPARISON
// ============================================

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '16px', flexWrap: 'wrap' }}>
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="support">Support</Button>
    </div>
  ),
};

// ============================================
// MOBILE VIEWPORT
// ============================================

export const MobileFullWidth: Story = {
  args: {
    variant: 'primary',
    size: 'lg',
    children: 'Get Started',
    fullWidth: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    layout: 'padded',
  },
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  args: {
    variant: 'secondary',
    children: 'Dark Mode',
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
