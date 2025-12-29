import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import react from '@astrojs/react';

// https://astro.build/config
export default defineConfig({
  site: 'https://caro.sh',
  integrations: [
    sitemap(),
    react(),
  ],
  build: {
    inlineStylesheets: 'auto',
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id'],
    routing: {
      prefixDefaultLocale: false,  // Keep English at / root
      redirectToDefaultLocale: false,
    },
    fallback: {
      es: 'en', fr: 'en', pt: 'en', de: 'en',
      he: 'en', ar: 'en', uk: 'en', ru: 'en',
      ja: 'en', ko: 'en', hi: 'en', ur: 'en',
      fil: 'en', id: 'en'
    }
  }
});
