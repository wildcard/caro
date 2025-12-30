import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { Link } from './Link';

const meta: Meta<typeof Link> = {
  title: 'Components/Link',
  component: Link,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Styled link component with multiple variants.

**Variants**:
- \`default\`: Standard text link, turns orange on hover
- \`support\`: Pink colored for sponsor/support links
- \`arrow\`: With animated arrow icon
- \`external\`: With external link icon
- \`nav\`: Navigation-style link

**External Links**:
External links are auto-detected by URL pattern (http/https).
They automatically open in a new tab with proper security attributes.
        `,
      },
    },
  },
  argTypes: {
    href: {
      control: 'text',
      description: 'Link URL',
    },
    variant: {
      control: 'select',
      options: ['default', 'support', 'arrow', 'external', 'nav'],
      description: 'Visual variant',
    },
    external: {
      control: 'boolean',
      description: 'Force external link behavior',
    },
    children: {
      control: 'text',
      description: 'Link content',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Link>;

// ============================================
// VARIANTS
// ============================================

export const Default: Story = {
  args: {
    href: '/features',
    children: 'Features',
  },
};

export const Support: Story = {
  args: {
    href: '/support',
    variant: 'support',
    children: '♥ Sponsor',
  },
};

export const Arrow: Story = {
  args: {
    href: '/blog',
    variant: 'arrow',
    children: 'Read more',
  },
};

export const External: Story = {
  args: {
    href: 'https://github.com/wildcard/caro',
    variant: 'external',
    children: 'GitHub',
  },
};

export const Nav: Story = {
  args: {
    href: '/features',
    variant: 'nav',
    children: 'Features',
  },
};

// ============================================
// ALL VARIANTS
// ============================================

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <Link href="/features">Default Link</Link>
      <Link href="/support" variant="support">♥ Sponsor</Link>
      <Link href="/blog" variant="arrow">Read more</Link>
      <Link href="https://github.com" variant="external">GitHub</Link>
      <Link href="/features" variant="nav">Navigation Link</Link>
    </div>
  ),
};

// ============================================
// USE CASES
// ============================================

export const FooterLinks: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      <Link href="/#features">Features</Link>
      <Link href="/#how-it-works">How It Works</Link>
      <Link href="/explore">Explore</Link>
      <Link href="/#download">Download</Link>
      <Link href="/support" variant="support">♥ Sponsor</Link>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Footer-style link list.',
      },
    },
  },
};

export const BlogLinks: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div>
        <h4 style={{ margin: '0 0 8px 0', color: 'var(--color-text)' }}>
          Announcing Caro
        </h4>
        <Link href="/blog/announcing-caro" variant="arrow">
          Read full story
        </Link>
      </div>
      <div>
        <h4 style={{ margin: '0 0 8px 0', color: 'var(--color-text)' }}>
          Why Caro?
        </h4>
        <Link href="/blog/why-caro" variant="arrow">
          Read full story
        </Link>
      </div>
    </div>
  ),
};

export const NavigationLinks: Story = {
  render: () => (
    <nav style={{ display: 'flex', gap: '24px', alignItems: 'center' }}>
      <Link href="/" variant="nav">Home</Link>
      <Link href="/features" variant="nav">Features</Link>
      <Link href="/blog" variant="nav">Blog</Link>
      <Link href="/support" variant="support">♥ Support</Link>
    </nav>
  ),
};

export const ResourceLinks: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      <Link href="https://github.com/wildcard/caro" variant="external">
        GitHub
      </Link>
      <Link href="https://github.com/wildcard/caro/issues" variant="external">
        Issues
      </Link>
      <Link href="https://github.com/wildcard/caro/contribute" variant="external">
        Contributing
      </Link>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'External resource links with icons.',
      },
    },
  },
};

// ============================================
// AUTO-DETECTED EXTERNAL
// ============================================

export const AutoDetectedExternal: Story = {
  args: {
    href: 'https://caro.sh',
    children: 'External link (auto-detected)',
  },
  parameters: {
    docs: {
      description: {
        story: 'Links starting with http/https are automatically treated as external.',
      },
    },
  },
};

// ============================================
// INLINE TEXT
// ============================================

export const InlineText: Story = {
  render: () => (
    <p style={{ color: 'var(--color-text)', lineHeight: 1.6, maxWidth: '500px' }}>
      Caro is a companion agent that specializes in POSIX shell commands.
      Read more about it on{' '}
      <Link href="https://github.com/wildcard/caro">GitHub</Link> or check out
      the <Link href="/blog" variant="arrow">blog</Link> for the latest updates.
    </p>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Links used within paragraph text.',
      },
    },
  },
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <Link href="/features">Default Link</Link>
      <Link href="/support" variant="support">♥ Sponsor</Link>
      <Link href="/blog" variant="arrow">Read more</Link>
      <Link href="https://github.com" variant="external">GitHub</Link>
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
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <Link href="/features" variant="nav">Features</Link>
      <Link href="/blog" variant="nav">Blog</Link>
      <Link href="/support" variant="support">♥ Sponsor</Link>
    </div>
  ),
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
  },
};
