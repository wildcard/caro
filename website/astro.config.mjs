import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import react from '@astrojs/react';
import db from '@astrojs/db';
import vercel from '@astrojs/vercel';

// https://astro.build/config
export default defineConfig({
  site: 'https://caro.sh',
  // Astro 5 default output is 'static'. The /api/waitlist route declares
  // `export const prerender = false` to opt that single endpoint into
  // serverless rendering on Vercel — keeps the rest of the site fully
  // static for cacheability while letting the Turso-backed signup API
  // run at request time. The Vercel adapter is required so that the
  // single non-prerendered route can be deployed as a serverless
  // function; the rest of the site continues to ship as static assets.
  // Do NOT set `output: 'hybrid'`; that mode was removed in Astro 5
  // (use `output: 'server'` + per-page prerender if every API route
  // ever needs server rendering).
  output: 'static',
  adapter: vercel(),
  integrations: [
    sitemap(),
    react(),
    db(),
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
