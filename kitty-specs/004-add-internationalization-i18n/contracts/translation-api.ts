/**
 * Translation API Contract
 *
 * Defines the public interfaces for the i18n translation system.
 * These type definitions serve as the contract between components and the translation system.
 */

/**
 * Supported locale codes (ISO 639-1)
 */
export type Locale =
  | 'en'  // English (default)
  | 'es'  // Spanish
  | 'fr'  // French
  | 'pt'  // Portuguese
  | 'de'  // German
  | 'he'  // Hebrew (RTL)
  | 'ar'  // Arabic (RTL)
  | 'uk'  // Ukrainian
  | 'ru'  // Russian
  | 'ja'  // Japanese
  | 'ko'  // Korean
  | 'hi'  // Hindi
  | 'ur'  // Urdu (RTL)
  | 'fil' // Filipino
  | 'id'; // Indonesian

/**
 * Text direction for rendering
 */
export type TextDirection = 'ltr' | 'rtl';

/**
 * Locale metadata configuration
 */
export interface LocaleConfig {
  /** ISO 639-1 language code */
  code: Locale;

  /** Language name in native script (e.g., "Español", "עברית") */
  nativeName: string;

  /** Language name in English */
  englishName: string;

  /** Text direction */
  direction: TextDirection;

  /** Font family for this locale (optional, for RTL languages) */
  fontFamily?: string;

  /** Whether this is the default locale */
  isDefault: boolean;
}

/**
 * Translation section names (maps to JSON files)
 */
export type TranslationSection =
  | 'common'       // Shared UI strings (buttons, labels, status)
  | 'navigation'   // Nav bar and footer
  | 'hero'         // Hero section
  | 'features'     // Features section
  | 'download'     // Download section
  | 'faq'          // FAQ section
  | 'landing'      // Landing page copy
  | 'compare';     // Comparison pages

/**
 * Nested translation object structure
 * Allows dot-notation key access (e.g., "navigation.links.features")
 */
export type TranslationObject = {
  [key: string]: string | TranslationObject;
};

/**
 * Complete translations for a single section
 */
export interface SectionTranslations {
  [key: string]: string | TranslationObject;
}

/**
 * Core translation function
 *
 * @param locale - Target locale code
 * @param key - Dot-notation key path (e.g., "navigation.links.features")
 * @returns Translated string, or English fallback if not found
 *
 * @example
 * t('es', 'navigation.links.features') // => "Características"
 * t('he', 'hero.tagline') // => "חבר הנאמן שלך לשורת הפקודה"
 * t('ja', 'common.buttons.getStarted') // => "始める"
 */
export function t(locale: Locale, key: string): string;

/**
 * Get all translations for a specific section
 *
 * @param locale - Target locale code
 * @returns Object containing all translations for the locale
 *
 * @example
 * const nav = getTranslations('es').navigation;
 * nav.links.features // => "Características"
 */
export function getTranslations(locale: Locale): Record<TranslationSection, SectionTranslations>;

/**
 * Check if a locale uses RTL text direction
 *
 * @param locale - Locale code to check
 * @returns true if RTL, false if LTR
 *
 * @example
 * isRtl('he') // => true
 * isRtl('es') // => false
 */
export function isRtl(locale: Locale): boolean;

/**
 * Get locale configuration
 *
 * @param locale - Locale code
 * @returns Locale metadata
 *
 * @example
 * getLocaleConfig('ar') // => { code: 'ar', nativeName: 'العربية', direction: 'rtl', ... }
 */
export function getLocaleConfig(locale: Locale): LocaleConfig;

/**
 * Get all available locales
 *
 * @returns Array of all supported locale configurations
 *
 * @example
 * getAllLocales() // => [{ code: 'en', ... }, { code: 'es', ... }, ...]
 */
export function getAllLocales(): LocaleConfig[];

/**
 * Validate if a string is a supported locale
 *
 * @param code - String to validate
 * @returns true if valid locale, false otherwise
 *
 * @example
 * isValidLocale('es') // => true
 * isValidLocale('xx') // => false
 */
export function isValidLocale(code: string): code is Locale;

/**
 * Component prop interface for locale-aware components
 */
export interface LocaleProps {
  /** Current locale */
  lang?: Locale;
}

/**
 * Language switcher state
 */
export interface LanguageSwitcherState {
  /** Currently active locale */
  currentLocale: Locale;

  /** Available locales to switch to */
  availableLocales: LocaleConfig[];

  /** Function to change locale */
  setLocale: (locale: Locale) => void;
}
