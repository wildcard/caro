import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import react from '@astrojs/react';

// CMS Mode: Set to true to enable Keystatic CMS admin UI
// When true, requires: npm run dev (not static build)
const CMS_MODE = process.env.KEYSTATIC_CMS === 'true';

// Conditionally import Keystatic when in CMS mode
const keystatic = CMS_MODE
  ? (await import('@keystatic/astro')).default
  : null;

// https://astro.build/config
export default defineConfig({
  site: 'https://caro.sh',
  output: CMS_MODE ? 'hybrid' : 'static',
  integrations: [
    sitemap(),
    react(),
    ...(keystatic ? [keystatic()] : []),
  ],
  build: {
    inlineStylesheets: 'auto',
  },
});
