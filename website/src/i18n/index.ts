/**
 * Translation Index
 *
 * Central export point for all translation JSON files.
 * Import this file to access translations for any locale.
 */

// English translations
import navigationEn from './locales/en/navigation.json';
import heroEn from './locales/en/hero.json';
import featuresEn from './locales/en/features.json';
import downloadEn from './locales/en/download.json';
import commonEn from './locales/en/common.json';
import landingEn from './locales/en/landing.json';
import compareEn from './locales/en/compare.json';

/**
 * English translation object
 * Combines all section translations into a single object
 */
export const en = {
  ...navigationEn,
  ...heroEn,
  ...featuresEn,
  ...downloadEn,
  ...commonEn,
  ...landingEn,
  ...compareEn
};

/**
 * Placeholder for other locales
 * These will be populated in WP07 (GitHub Action automation)
 */
export const es = en; // TODO: Replace with Spanish translations
export const fr = en; // TODO: Replace with French translations
export const pt = en; // TODO: Replace with Portuguese translations
export const de = en; // TODO: Replace with German translations
export const he = en; // TODO: Replace with Hebrew translations
export const ar = en; // TODO: Replace with Arabic translations
export const uk = en; // TODO: Replace with Ukrainian translations
export const ru = en; // TODO: Replace with Russian translations
export const ja = en; // TODO: Replace with Japanese translations
export const ko = en; // TODO: Replace with Korean translations
export const hi = en; // TODO: Replace with Hindi translations
export const ur = en; // TODO: Replace with Urdu translations
export const fil = en; // TODO: Replace with Filipino translations
export const id = en; // TODO: Replace with Indonesian translations

/**
 * Map of all translations by locale code
 */
export const translations = {
  en,
  es,
  fr,
  pt,
  de,
  he,
  ar,
  uk,
  ru,
  ja,
  ko,
  hi,
  ur,
  fil,
  id
};
