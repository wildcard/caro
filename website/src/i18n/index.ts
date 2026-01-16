/**
 * Translation Index
 *
 * Central export point for all translation JSON files.
 * Import this file to access translations for any locale.
 */

// English translations (complete set)
import navigationEn from './locales/en/navigation.json';
import heroEn from './locales/en/hero.json';
import featuresEn from './locales/en/features.json';
import downloadEn from './locales/en/download.json';
import commonEn from './locales/en/common.json';
import landingEn from './locales/en/landing.json';
import compareEn from './locales/en/compare.json';

// Localized common translations (partial - falls back to English for missing keys)
import commonEs from './locales/es/common.json';
import commonFr from './locales/fr/common.json';
import commonPt from './locales/pt/common.json';
import commonDe from './locales/de/common.json';
import commonHe from './locales/he/common.json';
import commonAr from './locales/ar/common.json';
import commonUk from './locales/uk/common.json';
import commonRu from './locales/ru/common.json';
import commonJa from './locales/ja/common.json';
import commonKo from './locales/ko/common.json';
import commonHi from './locales/hi/common.json';
import commonUr from './locales/ur/common.json';
import commonFil from './locales/fil/common.json';
import commonId from './locales/id/common.json';

/**
 * English translation object (base/fallback)
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
 * Localized translations
 * Each locale spreads English as base, then overrides with locale-specific translations.
 * This provides automatic fallback to English for any missing keys.
 */
export const es = { ...en, ...commonEs };
export const fr = { ...en, ...commonFr };
export const pt = { ...en, ...commonPt };
export const de = { ...en, ...commonDe };
export const he = { ...en, ...commonHe };
export const ar = { ...en, ...commonAr };
export const uk = { ...en, ...commonUk };
export const ru = { ...en, ...commonRu };
export const ja = { ...en, ...commonJa };
export const ko = { ...en, ...commonKo };
export const hi = { ...en, ...commonHi };
export const ur = { ...en, ...commonUr };
export const fil = { ...en, ...commonFil };
export const id = { ...en, ...commonId };

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
