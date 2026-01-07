import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://docs.caro.sh',
  integrations: [
    starlight({
      title: 'caro docs',
      description: 'Natural language to shell commands - A Rust CLI powered by local LLMs',
      logo: {
        light: './src/assets/logo-light.svg',
        dark: './src/assets/logo-dark.svg',
        replacesTitle: false,
      },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/wildcard/caro' },
        { icon: 'x.com', label: 'X', href: 'https://x.com/CaroDaShellShib' },
        { icon: 'blueSky', label: 'Bluesky', href: 'https://bsky.app/profile/caro-sh.bsky.social' },
      ],
      editLink: {
        baseUrl: 'https://github.com/wildcard/caro/edit/main/docs-site/',
      },
      customCss: [
        './src/styles/custom.css',
        './src/styles/isolation.css',
      ],
      components: {
        Head: './src/components/Head.astro',
      },
      head: [
        {
          tag: 'link',
          attrs: {
            rel: 'preconnect',
            href: 'https://fonts.googleapis.com',
          },
        },
        {
          tag: 'link',
          attrs: {
            rel: 'preconnect',
            href: 'https://fonts.gstatic.com',
            crossorigin: '',
          },
        },
        {
          tag: 'link',
          attrs: {
            href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&family=Space+Grotesk:wght@500;600;700&display=swap',
            rel: 'stylesheet',
          },
        },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'getting-started/introduction' },
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'Quick Start', slug: 'getting-started/quick-start' },
          ],
        },
        {
          label: 'Product',
          items: [
            { label: 'Jobs To Be Done', slug: 'product/jobs-to-be-done' },
          ],
        },
        {
          label: 'Contributing',
          items: [
            { label: 'Beta Testing', slug: 'contributing/beta-testing' },
            { label: 'Testing Profiles', slug: 'contributing/testing-profiles' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'macOS Setup', slug: 'guides/macos-setup' },
            { label: 'Spec-Kitty Workflow', slug: 'guides/spec-kitty' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Backends', slug: 'reference/backends' },
            { label: 'Configuration', slug: 'reference/configuration' },
            { label: 'Safety Patterns', slug: 'reference/safety' },
            { label: 'Naming History', slug: 'reference/naming-history' },
          ],
        },
        {
          label: 'Project Status',
          collapsed: false,
          items: [
            { label: 'Roadmap', slug: 'status/roadmap' },
            { label: 'Backend Status', slug: 'status/backends' },
          ],
        },
        {
          label: 'Development',
          collapsed: true,
          items: [
            { label: 'TDD Workflow', slug: 'development/tdd-workflow' },
            { label: 'Agent Guidelines', slug: 'development/agents' },
          ],
        },
        {
          label: 'External Docs',
          collapsed: true,
          autogenerate: { directory: 'external' },
        },
      ],
      expressiveCode: {
        themes: ['github-dark', 'github-light'],
        styleOverrides: {
          borderRadius: '0.5rem',
          codeFontFamily: "'JetBrains Mono', monospace",
        },
      },
    }),
  ],
});
