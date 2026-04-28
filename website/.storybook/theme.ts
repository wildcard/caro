import { create } from '@storybook/theming/create';

export default create({
  base: 'light',

  // Brand
  brandTitle: 'Caro Design System',
  brandUrl: 'https://caro.sh',
  brandTarget: '_blank',

  // Colors
  colorPrimary: '#ef3333',
  colorSecondary: '#c02020',

  // UI
  appBg: '#f8f8f8',
  appContentBg: '#ffffff',
  appBorderColor: '#e0e0e0',
  appBorderRadius: 8,

  // Typography
  fontBase: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  fontCode: '"Monaco", "Menlo", "Ubuntu Mono", monospace',

  // Text colors
  textColor: '#1a1a1a',
  textInverseColor: '#ffffff',

  // Toolbar default and active colors
  barTextColor: '#666666',
  barSelectedColor: '#ef3333',
  barBg: '#ffffff',

  // Form colors
  inputBg: '#ffffff',
  inputBorder: '#e0e0e0',
  inputTextColor: '#1a1a1a',
  inputBorderRadius: 8,
});
