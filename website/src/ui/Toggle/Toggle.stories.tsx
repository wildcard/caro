import React, { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { Toggle } from './Toggle';

const meta: Meta<typeof Toggle> = {
  title: 'Components/Toggle',
  component: Toggle,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
Toggle switch component for binary on/off states.

**Touch Targets**: Both sizes meet the 44px minimum touch target.

**Accessibility**: Uses \`role="switch"\` with proper \`aria-checked\` state.
Supports keyboard navigation with Enter and Space keys.

**Sizes**:
- \`md\` (52×28px): Default for desktop
- \`sm\` (44×24px): Optimized for mobile
        `,
      },
    },
  },
  argTypes: {
    checked: {
      control: 'boolean',
      description: 'Toggle state',
    },
    size: {
      control: 'select',
      options: ['sm', 'md'],
      description: 'Toggle size',
    },
    variant: {
      control: 'select',
      options: ['default', 'branded'],
      description: 'Visual variant (default: green, branded: orange)',
    },
    labelPlacement: {
      control: 'select',
      options: ['left', 'right', 'hidden'],
      description: 'Label position',
    },
    disabled: {
      control: 'boolean',
      description: 'Disabled state',
    },
    label: {
      control: 'text',
      description: 'Accessible label text',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Toggle>;

// ============================================
// INTERACTIVE WRAPPER
// ============================================

function InteractiveToggle(props: React.ComponentProps<typeof Toggle>) {
  const [checked, setChecked] = useState(props.checked ?? false);
  return <Toggle {...props} checked={checked} onChange={setChecked} />;
}

// ============================================
// DEFAULT STATES
// ============================================

export const Default: Story = {
  args: {
    label: 'Enable feature',
    checked: false,
  },
  render: (args) => <InteractiveToggle {...args} />,
};

export const Checked: Story = {
  args: {
    label: 'Feature enabled',
    checked: true,
  },
  render: (args) => <InteractiveToggle {...args} />,
};

// ============================================
// SIZE VARIANTS
// ============================================

export const SizeSmall: Story = {
  args: {
    label: 'Small toggle',
    size: 'sm',
  },
  render: (args) => <InteractiveToggle {...args} />,
};

export const SizeMedium: Story = {
  args: {
    label: 'Medium toggle',
    size: 'md',
  },
  render: (args) => <InteractiveToggle {...args} />,
};

// ============================================
// BRANDED VARIANT (Orange)
// ============================================

export const Branded: Story = {
  args: {
    label: 'Branded toggle',
    variant: 'branded',
    checked: true,
  },
  render: (args) => <InteractiveToggle {...args} />,
};

// ============================================
// LABEL PLACEMENT
// ============================================

export const LabelLeft: Story = {
  args: {
    label: 'Label on left',
    labelPlacement: 'left',
  },
  render: (args) => <InteractiveToggle {...args} />,
};

export const LabelRight: Story = {
  args: {
    label: 'Label on right',
    labelPlacement: 'right',
  },
  render: (args) => <InteractiveToggle {...args} />,
};

export const LabelHidden: Story = {
  args: {
    label: 'Hidden label (still accessible)',
    labelPlacement: 'hidden',
  },
  render: (args) => <InteractiveToggle {...args} />,
  parameters: {
    docs: {
      description: {
        story: 'Label is visually hidden but still accessible to screen readers.',
      },
    },
  },
};

// ============================================
// STATES
// ============================================

export const Disabled: Story = {
  args: {
    label: 'Disabled toggle',
    disabled: true,
  },
};

export const DisabledChecked: Story = {
  args: {
    label: 'Disabled and checked',
    disabled: true,
    checked: true,
  },
};

// ============================================
// USE CASES
// ============================================

export const DarkModeToggle: Story = {
  args: {
    label: 'Dark Mode',
    labelPlacement: 'left',
    variant: 'default',
  },
  render: (args) => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
      <span style={{ fontSize: '18px' }}>🌓</span>
      <InteractiveToggle {...args} />
    </div>
  ),
};

export const SnowEffectToggle: Story = {
  args: {
    label: 'Snow Effect',
    labelPlacement: 'left',
    variant: 'branded',
    checked: true,
  },
  render: (args) => (
    <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
      <span style={{ fontSize: '18px' }}>❄️</span>
      <InteractiveToggle {...args} />
    </div>
  ),
};

// ============================================
// SIZE COMPARISON
// ============================================

export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      <InteractiveToggle label="Small (mobile)" size="sm" />
      <InteractiveToggle label="Medium (desktop)" size="md" />
    </div>
  ),
};

// ============================================
// MOBILE VIEWPORT
// ============================================

export const MobileViewport: Story = {
  args: {
    label: 'Mobile toggle',
    size: 'sm',
  },
  render: (args) => <InteractiveToggle {...args} />,
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
    label: 'Toggle in dark mode',
  },
  render: (args) => <InteractiveToggle {...args} />,
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
