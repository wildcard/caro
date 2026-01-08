import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import react from '@astrojs/react';
import db from '@astrojs/db';

// https://astro.build/config
export default defineConfig({
  site: 'https://caro.sh',
  integrations: [
    sitemap(),
    react(),
    db(),
  ],
  build: {
    inlineStylesheets: 'auto',
  },
  output: 'hybrid',
});
