import type { Preview } from '@storybook/react';
import caroTheme from './theme';
import '../src/ui/tokens.css';

// Custom viewports for mobile testing
const customViewports = {
  smallMobile: {
    name: 'Small Mobile (320px)',
    styles: {
      width: '320px',
      height: '568px',
    },
  },
  mobile: {
    name: 'Mobile (375px)',
    styles: {
      width: '375px',
      height: '667px',
    },
  },
  largeMobile: {
    name: 'Large Mobile (428px)',
    styles: {
      width: '428px',
      height: '926px',
    },
  },
  tablet: {
    name: 'Tablet (768px)',
    styles: {
      width: '768px',
      height: '1024px',
    },
  },
  desktop: {
    name: 'Desktop (1200px)',
    styles: {
      width: '1200px',
      height: '900px',
    },
  },
};

const preview: Preview = {
  parameters: {
    docs: {
      theme: caroTheme,
    },
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    viewport: {
      viewports: customViewports,
    },
    backgrounds: {
      default: 'light',
      values: [
        {
          name: 'light',
          value: '#ffffff',
        },
        {
          name: 'dark',
          value: '#1a1a1a',
        },
        {
          name: 'gray',
          value: '#f8f9fa',
        },
      ],
    },
  },
  // Global decorators
  decorators: [
    (Story, context) => {
      // Apply dark class to root when dark background is selected
      const isDark = context.globals.backgrounds?.value === '#1a1a1a';
      document.documentElement.classList.toggle('dark', isDark);
      return Story();
    },
  ],
  globalTypes: {
    theme: {
      description: 'Global theme for components',
      defaultValue: 'light',
      toolbar: {
        title: 'Theme',
        icon: 'circlehollow',
        items: ['light', 'dark'],
        dynamicTitle: true,
      },
    },
  },
};

export default preview;
