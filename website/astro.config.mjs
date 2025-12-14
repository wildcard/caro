import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
  site: 'https://caro.sh',
  integrations: [sitemap()],
  build: {
    inlineStylesheets: 'auto',
  },
});
