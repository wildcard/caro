import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { DistroSelector } from './DistroSelector';

const meta: Meta<typeof DistroSelector> = {
  title: 'Components/DistroSelector',
  component: DistroSelector,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
An engaging, interactive component for users to select their operating system and shell preferences.

**Features**:
- Auto-detection from browser user agent
- Comprehensive Linux distro selection (70+ distributions!)
- Shell preference selection (Bash, Zsh, Fish, Nushell, etc.)
- Categorized browsing (Popular, Debian-based, Arch-based, etc.)
- Search functionality for quick distro lookup
- PostHog analytics integration
- localStorage persistence across sessions
- Fun animations and distro-specific colors

**Use Cases**:
- Personalize documentation examples to user's setup
- Show relevant installation instructions
- Customize terminal styling
- Filter content based on package manager
        `,
      },
    },
  },
  argTypes: {
    compact: {
      control: 'boolean',
      description: 'Compact mode shows only the icon and current selection',
    },
    showShell: {
      control: 'boolean',
      description: 'Whether to show the shell selection tab',
    },
  },
};

export default meta;
type Story = StoryObj<typeof DistroSelector>;

// ============================================
// DEFAULT
// ============================================

export const Default: Story = {
  args: {
    compact: false,
    showShell: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Default full-size selector with OS, distro, and shell selection tabs.',
      },
    },
  },
};

// ============================================
// COMPACT MODE
// ============================================

export const Compact: Story = {
  args: {
    compact: true,
    showShell: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Compact mode for use in navigation bars or tight spaces. Shows only the distro icon.',
      },
    },
  },
};

// ============================================
// WITHOUT SHELL SELECTION
// ============================================

export const WithoutShell: Story = {
  args: {
    compact: false,
    showShell: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Selector without the shell tab, useful when shell preference is not relevant.',
      },
    },
  },
};

// ============================================
// IN NAVIGATION BAR
// ============================================

export const InNavigationBar: Story = {
  render: () => (
    <nav
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
        padding: '12px 24px',
        background: 'var(--color-surface, #fff)',
        borderRadius: '12px',
        border: '1px solid var(--color-border, #e5e7eb)',
      }}
    >
      <a href="/" style={{ color: 'var(--color-text)', textDecoration: 'none', fontWeight: 600 }}>
        Caro
      </a>
      <span style={{ flex: 1 }} />
      <DistroSelector compact showShell={false} />
      <button
        style={{
          padding: '8px 16px',
          background: 'var(--color-primary, #ff8c42)',
          color: 'white',
          border: 'none',
          borderRadius: '8px',
          cursor: 'pointer',
        }}
      >
        Download
      </button>
    </nav>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Compact selector integrated into a navigation bar.',
      },
    },
  },
};

// ============================================
// WITH CALLBACK
// ============================================

export const WithCallback: Story = {
  render: () => {
    const [lastChange, setLastChange] = React.useState<string>('No changes yet');

    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', alignItems: 'center' }}>
        <DistroSelector
          onPreferencesChange={(prefs) => {
            setLastChange(
              `Changed to ${prefs.osFamily}${prefs.distro ? ` (${prefs.distro})` : ''} with ${prefs.shell}`
            );
          }}
        />
        <div
          style={{
            padding: '12px 16px',
            background: 'var(--color-surface-secondary, #f3f4f6)',
            borderRadius: '8px',
            fontSize: '14px',
            color: 'var(--color-text-muted, #6b7280)',
          }}
        >
          Last change: {lastChange}
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Using the `onPreferencesChange` callback to react to preference changes.',
      },
    },
  },
};

// ============================================
// DOCUMENTATION EXAMPLE
// ============================================

export const DocumentationExample: Story = {
  render: () => {
    const [prefs, setPrefs] = React.useState({
      osFamily: 'linux' as const,
      distro: 'ubuntu' as const,
      shell: 'bash' as const,
    });

    const getInstallCommand = () => {
      switch (prefs.osFamily) {
        case 'macos':
          return 'brew install caro';
        case 'windows':
          return 'winget install caro';
        case 'linux':
          if (prefs.distro?.includes('arch')) {
            return 'yay -S caro';
          }
          if (prefs.distro?.includes('fedora')) {
            return 'dnf install caro';
          }
          return 'curl -sSL https://caro.sh/install | sh';
        default:
          return 'curl -sSL https://caro.sh/install | sh';
      }
    };

    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '500px' }}>
        <div>
          <h3 style={{ margin: '0 0 12px', fontSize: '18px', fontWeight: 600 }}>
            Customize for your setup
          </h3>
          <DistroSelector
            onPreferencesChange={(p) => setPrefs({ osFamily: p.osFamily, distro: p.distro as any, shell: p.shell })}
          />
        </div>

        <div
          style={{
            padding: '16px',
            background: '#1e1e2e',
            borderRadius: '12px',
            fontFamily: 'monospace',
          }}
        >
          <div style={{ color: '#6b7280', marginBottom: '8px', fontSize: '12px' }}>
            Installation for {prefs.distro || prefs.osFamily}
          </div>
          <code style={{ color: '#cdd6f4', fontSize: '14px' }}>{getInstallCommand()}</code>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Example of using the selector to customize documentation content.',
      },
    },
  },
};

// ============================================
// DARK MODE
// ============================================

export const DarkMode: Story = {
  args: {
    compact: false,
    showShell: true,
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
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  args: {
    compact: false,
    showShell: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile1',
    },
    docs: {
      description: {
        story: 'On mobile, the dropdown panel becomes a bottom sheet for easier interaction.',
      },
    },
  },
};

// ============================================
// ALL STATES SHOWCASE
// ============================================

export const AllStates: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
      <div>
        <h4 style={{ margin: '0 0 12px', color: 'var(--color-text-muted)' }}>Full Size</h4>
        <DistroSelector showShell />
      </div>
      <div>
        <h4 style={{ margin: '0 0 12px', color: 'var(--color-text-muted)' }}>Compact</h4>
        <DistroSelector compact />
      </div>
      <div>
        <h4 style={{ margin: '0 0 12px', color: 'var(--color-text-muted)' }}>Without Shell</h4>
        <DistroSelector showShell={false} />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Comparison of all component variants.',
      },
    },
  },
};
