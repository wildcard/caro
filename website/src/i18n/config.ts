/**
 * Translation Configuration and Utilities
 *
 * Provides type-safe translation functions for all components.
 * Implements the API contract defined in contracts/translation-api.ts
 */

export type Locale =
  | 'en' | 'es' | 'fr' | 'pt' | 'de'
  | 'he' | 'ar' | 'uk' | 'ru' | 'ja'
  | 'ko' | 'hi' | 'ur' | 'fil' | 'id';

export type TextDirection = 'ltr' | 'rtl';

export interface LocaleConfig {
  code: Locale;
  nativeName: string;
  englishName: string;
  direction: TextDirection;
  fontFamily?: string;
  isDefault: boolean;
}

/**
 * Language configuration for all 15 supported locales
 */
export const languages: Record<Locale, LocaleConfig> = {
  en: {
    code: 'en',
    nativeName: 'English',
    englishName: 'English',
    direction: 'ltr',
    isDefault: true
  },
  es: {
    code: 'es',
    nativeName: 'Español',
    englishName: 'Spanish',
    direction: 'ltr',
    isDefault: false
  },
  fr: {
    code: 'fr',
    nativeName: 'Français',
    englishName: 'French',
    direction: 'ltr',
    isDefault: false
  },
  pt: {
    code: 'pt',
    nativeName: 'Português',
    englishName: 'Portuguese',
    direction: 'ltr',
    isDefault: false
  },
  de: {
    code: 'de',
    nativeName: 'Deutsch',
    englishName: 'German',
    direction: 'ltr',
    isDefault: false
  },
  he: {
    code: 'he',
    nativeName: 'עברית',
    englishName: 'Hebrew',
    direction: 'rtl',
    fontFamily: "'Noto Sans Hebrew', sans-serif",
    isDefault: false
  },
  ar: {
    code: 'ar',
    nativeName: 'العربية',
    englishName: 'Arabic',
    direction: 'rtl',
    fontFamily: "'Noto Sans Arabic', sans-serif",
    isDefault: false
  },
  uk: {
    code: 'uk',
    nativeName: 'Українська',
    englishName: 'Ukrainian',
    direction: 'ltr',
    isDefault: false
  },
  ru: {
    code: 'ru',
    nativeName: 'Русский',
    englishName: 'Russian',
    direction: 'ltr',
    isDefault: false
  },
  ja: {
    code: 'ja',
    nativeName: '日本語',
    englishName: 'Japanese',
    direction: 'ltr',
    isDefault: false
  },
  ko: {
    code: 'ko',
    nativeName: '한국어',
    englishName: 'Korean',
    direction: 'ltr',
    isDefault: false
  },
  hi: {
    code: 'hi',
    nativeName: 'हिन्दी',
    englishName: 'Hindi',
    direction: 'ltr',
    isDefault: false
  },
  ur: {
    code: 'ur',
    nativeName: 'اردو',
    englishName: 'Urdu',
    direction: 'rtl',
    fontFamily: "'Noto Nastaliq Urdu', serif",
    isDefault: false
  },
  fil: {
    code: 'fil',
    nativeName: 'Filipino',
    englishName: 'Filipino',
    direction: 'ltr',
    isDefault: false
  },
  id: {
    code: 'id',
    nativeName: 'Bahasa Indonesia',
    englishName: 'Indonesian',
    direction: 'ltr',
    isDefault: false
  },
};

/**
 * Core translation function
 *
 * @param locale - Target locale code
 * @param key - Dot-notation key path (e.g., "navigation.links.features")
 * @returns Translated string, or English fallback if not found
 *
 * NOTE: This is a placeholder implementation for Phase 1.
 * Actual translation loading from JSON files happens in WP02.
 */
export function t(locale: Locale, key: string): string {
  // Placeholder: return key until JSON files are created in WP02
  return key;
}

/**
 * Check if a locale uses RTL text direction
 *
 * @param locale - Locale code to check
 * @returns true if RTL, false if LTR
 */
export function isRtl(locale: Locale): boolean {
  return languages[locale]?.direction === 'rtl';
}

/**
 * Get locale configuration
 *
 * @param locale - Locale code
 * @returns Locale metadata
 */
export function getLocaleConfig(locale: Locale): LocaleConfig {
  return languages[locale];
}

/**
 * Get all available locales
 *
 * @returns Array of all supported locale configurations
 */
export function getAllLocales(): LocaleConfig[] {
  return Object.values(languages);
}

/**
 * Validate if a string is a supported locale
 *
 * @param code - String to validate
 * @returns true if valid locale, false otherwise
 */
export function isValidLocale(code: string): code is Locale {
  return code in languages;
}

/**
 * Component prop interface for locale-aware components
 */
export interface LocaleProps {
  lang?: Locale;
}
