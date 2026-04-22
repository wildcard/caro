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
import landingEs from './locales/es/landing.json';
import commonFr from './locales/fr/common.json';
import landingFr from './locales/fr/landing.json';
import commonPt from './locales/pt/common.json';
import landingPt from './locales/pt/landing.json';
import commonDe from './locales/de/common.json';
import landingDe from './locales/de/landing.json';
import commonHe from './locales/he/common.json';
import landingHe from './locales/he/landing.json';
import commonAr from './locales/ar/common.json';
import landingAr from './locales/ar/landing.json';
import commonUk from './locales/uk/common.json';
import landingUk from './locales/uk/landing.json';
import commonRu from './locales/ru/common.json';
import landingRu from './locales/ru/landing.json';
import commonJa from './locales/ja/common.json';
import landingJa from './locales/ja/landing.json';
import commonKo from './locales/ko/common.json';
import landingKo from './locales/ko/landing.json';
import commonHi from './locales/hi/common.json';
import landingHi from './locales/hi/landing.json';
import commonUr from './locales/ur/common.json';
import landingUr from './locales/ur/landing.json';
import commonFil from './locales/fil/common.json';
import landingFil from './locales/fil/landing.json';
import commonId from './locales/id/common.json';
import landingId from './locales/id/landing.json';

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
export const es = { ...en, ...commonEs, ...landingEs };
export const fr = { ...en, ...commonFr, ...landingFr };
export const pt = { ...en, ...commonPt, ...landingPt };
export const de = { ...en, ...commonDe, ...landingDe };
export const he = { ...en, ...commonHe, ...landingHe };
export const ar = { ...en, ...commonAr, ...landingAr };
export const uk = { ...en, ...commonUk, ...landingUk };
export const ru = { ...en, ...commonRu, ...landingRu };
export const ja = { ...en, ...commonJa, ...landingJa };
export const ko = { ...en, ...commonKo, ...landingKo };
export const hi = { ...en, ...commonHi, ...landingHi };
export const ur = { ...en, ...commonUr, ...landingUr };
export const fil = { ...en, ...commonFil, ...landingFil };
export const id = { ...en, ...commonId, ...landingId };

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
