import type { Meta, StoryObj } from '@storybook/react';
import { Badge } from './Badge';

const meta: Meta<typeof Badge> = {
  title: 'Components/Badge',
  component: Badge,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Badge component for status indicators and labels.

Used throughout the Caro website for:
- Feature status (Implemented, In Progress, Planned)
- Version tags (Alpha, Beta, v1.0)
- Category labels

**Variants**:
- \`primary\`: Orange - brand accent
- \`success\`: Green - implemented, complete
- \`warning\`: Amber - in progress, pending
- \`info\`: Blue - informational
- \`neutral\`: Gray - default, subtle
        `,
      },
    },
  },
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'success', 'warning', 'info', 'neutral'],
      description: 'Visual style variant',
    },
    size: {
      control: 'select',
      options: ['sm', 'md'],
      description: 'Badge size',
    },
    children: {
      control: 'text',
      description: 'Badge content',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Badge>;

// ============================================
// DEFAULT VARIANTS
// ============================================

export const Primary: Story = {
  args: {
    variant: 'primary',
    children: 'New Feature',
  },
};

export const Success: Story = {
  args: {
    variant: 'success',
    children: 'Implemented',
  },
};

export const Warning: Story = {
  args: {
    variant: 'warning',
    children: 'In Progress',
  },
};

export const Info: Story = {
  args: {
    variant: 'info',
    children: 'Alpha',
  },
};

export const Neutral: Story = {
  args: {
    variant: 'neutral',
    children: 'Default',
  },
};

// ============================================
// SIZE VARIANTS
// ============================================

export const Small: Story = {
  args: {
    size: 'sm',
    variant: 'primary',
    children: 'Small Badge',
  },
};

export const Medium: Story = {
  args: {
    size: 'md',
    variant: 'primary',
    children: 'Medium Badge',
  },
};

// ============================================
// WITH ICONS
// ============================================

export const WithEmoji: Story = {
  args: {
    variant: 'primary',
    icon: '🐕',
    children: 'Companion',
  },
};

export const WithSvgIcon: Story = {
  args: {
    variant: 'success',
    icon: (
      <svg viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 0a8 8 0 1 0 0 16A8 8 0 0 0 8 0zm3.5 6.5l-4 4a.5.5 0 0 1-.7 0l-2-2a.5.5 0 0 1 .7-.7L7 9.3l3.5-3.5a.5.5 0 0 1 .7.7z" />
      </svg>
    ),
    children: 'Complete',
  },
};

// ============================================
// ALL VARIANTS
// ============================================

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
      <Badge variant="primary">Primary</Badge>
      <Badge variant="success">Success</Badge>
      <Badge variant="warning">Warning</Badge>
      <Badge variant="info">Info</Badge>
      <Badge variant="neutral">Neutral</Badge>
    </div>
  ),
};

// ============================================
// ALL SIZES
// ============================================

export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
      <Badge size="sm" variant="primary">
        Small
      </Badge>
      <Badge size="md" variant="primary">
        Medium
      </Badge>
    </div>
  ),
};

// ============================================
// USE CASES
// ============================================

export const FeatureStatus: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span style={{ width: '150px' }}>Safety Validation</span>
        <Badge variant="success">Implemented</Badge>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span style={{ width: '150px' }}>MLX Backend</span>
        <Badge variant="warning">In Progress</Badge>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span style={{ width: '150px' }}>Multi-device Sync</span>
        <Badge variant="neutral">Planned</Badge>
      </div>
    </div>
  ),
};

export const VersionBadges: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '12px' }}>
      <Badge variant="info" size="sm">
        v0.1.0
      </Badge>
      <Badge variant="warning" size="sm">
        Alpha
      </Badge>
      <Badge variant="primary" size="sm" icon="🐕">
        Companion
      </Badge>
    </div>
  ),
};

export const HeroBadge: Story = {
  args: {
    variant: 'primary',
    icon: '🐕',
    children: 'Your loyal shell companion',
  },
  parameters: {
    docs: {
      description: {
        story: 'As used in the hero section of the Caro website.',
      },
    },
  },
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
      <Badge variant="primary">Primary</Badge>
      <Badge variant="success">Success</Badge>
      <Badge variant="warning">Warning</Badge>
      <Badge variant="info">Info</Badge>
      <Badge variant="neutral">Neutral</Badge>
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
