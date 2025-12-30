import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { Card } from './Card';

const meta: Meta<typeof Card> = {
  title: 'Components/Card',
  component: Card,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Card component for features, blog posts, and summaries.

**Variants**:
- \`feature\`: Icon + status badge + title + description (centered)
- \`blog\`: Date + read time + title + description + read more link
- \`summary\`: Simple card for compact content

**Status Badges** (feature variant):
- \`implemented\`: Green badge
- \`in-progress\`: Blue badge
- \`planned\`: Purple badge
        `,
      },
    },
  },
  argTypes: {
    variant: {
      control: 'select',
      options: ['feature', 'blog', 'summary'],
      description: 'Card variant',
    },
    status: {
      control: 'select',
      options: ['implemented', 'in-progress', 'planned'],
      description: 'Feature status (for feature variant)',
    },
    title: {
      control: 'text',
      description: 'Card title',
    },
    description: {
      control: 'text',
      description: 'Card description',
    },
    icon: {
      control: 'text',
      description: 'Icon or emoji (for feature variant)',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Card>;

// ============================================
// FEATURE VARIANT
// ============================================

export const Feature: Story = {
  args: {
    variant: 'feature',
    icon: '🛡️',
    title: 'Safety Guardian',
    description:
      'Comprehensive validation blocks dangerous commands like rm -rf /, fork bombs, and destructive operations. 52 predefined safety patterns with risk-level assessment.',
    status: 'implemented',
    statusLabel: 'Available Now',
  },
};

export const FeatureInProgress: Story = {
  args: {
    variant: 'feature',
    icon: '⚡',
    title: 'Lightning Fast',
    description:
      'Target: Sub-100ms startup, sub-2s inference on Apple Silicon. MLX framework integration for GPU acceleration on M-series chips.',
    status: 'in-progress',
    statusLabel: 'In Development',
  },
};

export const FeaturePlanned: Story = {
  args: {
    variant: 'feature',
    icon: '🔮',
    title: 'Future Feature',
    description:
      'This feature is planned for a future release. Stay tuned for updates!',
    status: 'planned',
    statusLabel: 'Coming Soon',
  },
};

export const FeatureGrid: Story = {
  render: () => (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
        gap: '40px',
        maxWidth: '900px',
      }}
    >
      <Card
        variant="feature"
        icon="🛡️"
        title="Safety Guardian"
        description="Comprehensive validation blocks dangerous commands."
        status="implemented"
        statusLabel="Available Now"
      />
      <Card
        variant="feature"
        icon="🌍"
        title="Cross-Platform"
        description="Works across macOS, Linux, Windows, GNU, and BSD."
        status="implemented"
        statusLabel="Available Now"
      />
      <Card
        variant="feature"
        icon="⚡"
        title="Lightning Fast"
        description="Sub-100ms startup, sub-2s inference on Apple Silicon."
        status="in-progress"
        statusLabel="In Development"
      />
    </div>
  ),
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        story: 'Feature cards in a responsive grid layout, matching the homepage.',
      },
    },
  },
};

// ============================================
// BLOG VARIANT
// ============================================

export const Blog: Story = {
  args: {
    variant: 'blog',
    title: 'Announcing Caro: Your Terminal\'s AI Companion',
    description:
      'We\'re excited to announce that cmdai has been renamed to caro! Thanks to @aeplay for graciously transferring the crate name.',
    metadata: {
      date: '2025-12-20',
      readTime: '5 min read',
    },
    href: '/blog/announcing-caro',
  },
};

export const BlogWithBadge: Story = {
  args: {
    variant: 'blog',
    title: 'Announcing Caro: Your Terminal\'s AI Companion',
    description:
      'We\'re excited to announce that cmdai has been renamed to caro! Thanks to @aeplay for graciously transferring the crate name.',
    badge: 'NEW',
    metadata: {
      date: '2025-12-20',
      readTime: '5 min read',
    },
    href: '/blog/announcing-caro',
  },
};

export const BlogGrid: Story = {
  render: () => (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
        gap: '30px',
        maxWidth: '1000px',
      }}
    >
      <Card
        variant="blog"
        title="Announcing Caro"
        description="We're excited to announce that cmdai has been renamed to caro!"
        badge="NEW"
        metadata={{ date: '2025-12-20', readTime: '5 min read' }}
        href="/blog/announcing-caro"
      />
      <Card
        variant="blog"
        title="Caro Claude Skill"
        description="Generate safe shell commands directly in your Claude sessions."
        metadata={{ date: '2025-12-19', readTime: '6 min read' }}
        href="/blog/claude-skill"
      />
      <Card
        variant="blog"
        title="Why Caro?"
        description="The story behind your terminal companion."
        metadata={{ date: '2025-12-17', readTime: '8 min read' }}
        href="/blog/why-caro"
      />
    </div>
  ),
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        story: 'Blog cards in a responsive grid layout.',
      },
    },
  },
};

// ============================================
// SUMMARY VARIANT
// ============================================

export const Summary: Story = {
  args: {
    variant: 'summary',
    title: 'Quick Summary',
    description:
      'A compact card for displaying brief information or summaries.',
  },
};

// ============================================
// INTERACTIVE STATES
// ============================================

export const Clickable: Story = {
  args: {
    variant: 'feature',
    icon: '🔗',
    title: 'Clickable Card',
    description: 'This card has an onClick handler. Click to trigger an action.',
  },
  render: (args) => (
    <Card {...args} onClick={() => alert('Card clicked!')} />
  ),
};

export const WithLink: Story = {
  args: {
    variant: 'feature',
    icon: '🔗',
    title: 'Card with Link',
    description: 'This card navigates to a URL when clicked.',
    href: 'https://caro.sh',
  },
};

// ============================================
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  args: {
    variant: 'feature',
    icon: '📱',
    title: 'Mobile Card',
    description: 'This card is optimized for mobile viewports.',
    status: 'implemented',
    statusLabel: 'Available',
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
    variant: 'feature',
    icon: '🌙',
    title: 'Dark Mode Card',
    description: 'This card is displayed in dark mode.',
    status: 'implemented',
    statusLabel: 'Available',
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

// ============================================
// ALL STATUS BADGES
// ============================================

export const AllStatusBadges: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', maxWidth: '400px' }}>
      <Card
        variant="feature"
        icon="✅"
        title="Implemented Feature"
        description="This feature is available now."
        status="implemented"
        statusLabel="Available Now"
      />
      <Card
        variant="feature"
        icon="🔄"
        title="In Progress Feature"
        description="This feature is being developed."
        status="in-progress"
        statusLabel="In Development"
      />
      <Card
        variant="feature"
        icon="📅"
        title="Planned Feature"
        description="This feature is planned."
        status="planned"
        statusLabel="Coming Soon"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All three status badge variants side by side.',
      },
    },
  },
};
