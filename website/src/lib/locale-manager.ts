/**
 * Locale Detection and Persistence Manager
 *
 * Implements a waterfall detection strategy:
 * 1. URL path prefix (/es/, /fr/, etc.)
 * 2. localStorage preference (user-selected)
 * 3. Browser Accept-Language header
 * 4. Fallback: English (en)
 */

import type { Locale } from '../i18n/config';
import { isValidLocale, languages } from '../i18n/config';

const STORAGE_KEY = 'caro-locale';

/**
 * Extract locale from URL path
 * Examples: /es/ → 'es', /fr/features → 'fr', / → null
 */
export function getLocaleFromPath(pathname: string): Locale | null {
  const pathParts = pathname.split('/').filter(Boolean);
  if (pathParts.length === 0) return null;

  const firstSegment = pathParts[0];
  return isValidLocale(firstSegment) ? firstSegment : null;
}

/**
 * Detect the user's preferred locale using waterfall strategy
 *
 * This function is intended for client-side use where browser APIs are available.
 * For SSR, use getLocaleFromPath() with the request URL.
 */
export function detectLocale(): Locale {
  // 1. Check URL path (highest priority)
  if (typeof window !== 'undefined') {
    const pathLocale = getLocaleFromPath(window.location.pathname);
    if (pathLocale) return pathLocale;

    // 2. Check localStorage preference
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored && isValidLocale(stored)) {
        return stored as Locale;
      }
    } catch (e) {
      // localStorage may not be available (SSR, private browsing)
    }

    // 3. Check browser language preference
    if (navigator && navigator.language) {
      // Extract the language code (e.g., 'en-US' → 'en')
      const browserLang = navigator.language.split('-')[0];
      if (isValidLocale(browserLang)) {
        return browserLang as Locale;
      }
    }
  }

  // 4. Default fallback
  return 'en';
}

/**
 * Persist user's locale preference to localStorage
 * This will be checked on future page loads.
 */
export function setLocalePreference(locale: Locale): void {
  if (typeof window === 'undefined') return;

  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch (e) {
    console.warn('Failed to persist locale preference:', e);
  }
}

/**
 * Get the stored locale preference from localStorage
 * Returns null if no preference is stored or if localStorage is unavailable.
 */
export function getStoredLocale(): Locale | null {
  if (typeof window === 'undefined') return null;

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored && isValidLocale(stored) ? (stored as Locale) : null;
  } catch (e) {
    return null;
  }
}

/**
 * Clear the stored locale preference
 * User will go back to browser/default detection.
 */
export function clearLocalePreference(): void {
  if (typeof window === 'undefined') return;

  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (e) {
    console.warn('Failed to clear locale preference:', e);
  }
}

/**
 * Get locale configuration with metadata
 * Useful for displaying language names and direction info.
 */
export function getLocaleMetadata(locale: Locale) {
  return languages[locale];
}

/**
 * Check if the current page is in the default locale (English)
 */
export function isDefaultLocale(locale: Locale): boolean {
  return locale === 'en';
}
