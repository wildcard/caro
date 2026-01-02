/**
 * Locale Detection Utility
 *
 * Detects user's locale, language, and approximate location using
 * browser APIs. This is privacy-respecting - we only use timezone
 * and language settings, not IP geolocation.
 */

import type { LocaleInfo } from '../config/types';

// ============================================================================
// Timezone to Country Mapping
// ============================================================================

/**
 * Map of timezone prefixes/names to ISO country codes
 * This is a best-effort mapping for major regions
 */
const TIMEZONE_TO_COUNTRY: Record<string, string> = {
  // Israel
  'Asia/Jerusalem': 'IL',
  'Asia/Tel_Aviv': 'IL',

  // United States (major zones)
  'America/New_York': 'US',
  'America/Chicago': 'US',
  'America/Denver': 'US',
  'America/Los_Angeles': 'US',
  'America/Phoenix': 'US',
  'America/Anchorage': 'US',
  'Pacific/Honolulu': 'US',
  'America/Detroit': 'US',
  'America/Indianapolis': 'US',
  'America/Boise': 'US',

  // Canada
  'America/Toronto': 'CA',
  'America/Vancouver': 'CA',
  'America/Montreal': 'CA',
  'America/Edmonton': 'CA',
  'America/Winnipeg': 'CA',
  'America/Halifax': 'CA',
  'America/St_Johns': 'CA',

  // United Kingdom
  'Europe/London': 'GB',

  // France
  'Europe/Paris': 'FR',

  // Germany
  'Europe/Berlin': 'DE',

  // Spain
  'Europe/Madrid': 'ES',

  // Portugal
  'Europe/Lisbon': 'PT',

  // Italy
  'Europe/Rome': 'IT',

  // Netherlands
  'Europe/Amsterdam': 'NL',

  // Belgium
  'Europe/Brussels': 'BE',

  // Switzerland
  'Europe/Zurich': 'CH',

  // Austria
  'Europe/Vienna': 'AT',

  // Russia
  'Europe/Moscow': 'RU',

  // China
  'Asia/Shanghai': 'CN',
  'Asia/Beijing': 'CN',
  'Asia/Hong_Kong': 'HK',

  // Japan
  'Asia/Tokyo': 'JP',

  // Korea
  'Asia/Seoul': 'KR',

  // Taiwan
  'Asia/Taipei': 'TW',

  // Singapore
  'Asia/Singapore': 'SG',

  // Malaysia
  'Asia/Kuala_Lumpur': 'MY',

  // Indonesia
  'Asia/Jakarta': 'ID',
  'Asia/Makassar': 'ID',
  'Asia/Jayapura': 'ID',

  // Philippines
  'Asia/Manila': 'PH',

  // Thailand
  'Asia/Bangkok': 'TH',

  // Vietnam
  'Asia/Ho_Chi_Minh': 'VN',
  'Asia/Saigon': 'VN',

  // India
  'Asia/Kolkata': 'IN',
  'Asia/Calcutta': 'IN',

  // Pakistan
  'Asia/Karachi': 'PK',

  // Bangladesh
  'Asia/Dhaka': 'BD',

  // UAE
  'Asia/Dubai': 'AE',

  // Saudi Arabia
  'Asia/Riyadh': 'SA',

  // Egypt
  'Africa/Cairo': 'EG',

  // South Africa
  'Africa/Johannesburg': 'ZA',

  // Australia
  'Australia/Sydney': 'AU',
  'Australia/Melbourne': 'AU',
  'Australia/Brisbane': 'AU',
  'Australia/Perth': 'AU',
  'Australia/Adelaide': 'AU',

  // New Zealand
  'Pacific/Auckland': 'NZ',

  // Brazil
  'America/Sao_Paulo': 'BR',
  'America/Rio_de_Janeiro': 'BR',

  // Argentina
  'America/Buenos_Aires': 'AR',
  'America/Argentina/Buenos_Aires': 'AR',

  // Mexico
  'America/Mexico_City': 'MX',

  // Turkey
  'Europe/Istanbul': 'TR',

  // Iran
  'Asia/Tehran': 'IR',
};

/**
 * Southern hemisphere countries (for seasonal effect flipping)
 */
const SOUTHERN_HEMISPHERE_COUNTRIES = new Set([
  'AU', // Australia
  'NZ', // New Zealand
  'ZA', // South Africa
  'AR', // Argentina
  'CL', // Chile
  'BR', // Brazil (partially)
  'UY', // Uruguay
  'PY', // Paraguay
  'BO', // Bolivia (partially)
  'PE', // Peru (partially)
]);

// ============================================================================
// Language Extraction
// ============================================================================

/**
 * Extract primary language code from navigator.language
 * e.g., "en-US" -> "en", "zh-TW" -> "zh"
 */
function extractLanguageCode(navigatorLanguage: string): string {
  return navigatorLanguage.split('-')[0].toLowerCase();
}

/**
 * Get all user languages in preference order
 */
function getUserLanguages(): string[] {
  if (typeof navigator === 'undefined') {
    return ['en'];
  }

  const languages: string[] = [];

  // navigator.languages gives all preferred languages
  if (navigator.languages) {
    for (const lang of navigator.languages) {
      const code = extractLanguageCode(lang);
      if (!languages.includes(code)) {
        languages.push(code);
      }
    }
  }

  // Fallback to navigator.language
  if (navigator.language) {
    const code = extractLanguageCode(navigator.language);
    if (!languages.includes(code)) {
      languages.push(code);
    }
  }

  return languages.length > 0 ? languages : ['en'];
}

// ============================================================================
// Country Detection
// ============================================================================

/**
 * Detect country from timezone
 */
function detectCountryFromTimezone(timezone: string): string | null {
  // Direct match
  if (TIMEZONE_TO_COUNTRY[timezone]) {
    return TIMEZONE_TO_COUNTRY[timezone];
  }

  // Try prefix matching for America/* timezones (fallback to US)
  if (timezone.startsWith('America/')) {
    // Check if it's a known non-US timezone
    const knownCanadian = [
      'Toronto',
      'Vancouver',
      'Montreal',
      'Edmonton',
      'Winnipeg',
      'Halifax',
      'St_Johns',
    ];
    const city = timezone.split('/')[1];
    if (knownCanadian.includes(city)) {
      return 'CA';
    }
    // Default to US for other America/* timezones
    // (This is imperfect but reasonable for our use case)
  }

  return null;
}

// ============================================================================
// Main Detection Function
// ============================================================================

/**
 * Detect user locale information
 *
 * Returns LocaleInfo with:
 * - country: ISO country code (or null if unknown)
 * - language: Primary language code
 * - timezone: IANA timezone string
 * - isSouthernHemisphere: Boolean for seasonal effect flipping
 */
export function detectLocale(): LocaleInfo {
  // Defaults for SSR or unknown
  let timezone = 'UTC';
  let country: string | null = null;
  let language = 'en';

  if (typeof window !== 'undefined' && typeof Intl !== 'undefined') {
    try {
      // Get timezone
      timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';

      // Detect country from timezone
      country = detectCountryFromTimezone(timezone);

      // Get primary language
      const languages = getUserLanguages();
      language = languages[0] || 'en';
    } catch (e) {
      console.warn('Locale detection failed:', e);
    }
  }

  // Determine hemisphere
  const isSouthernHemisphere = country
    ? SOUTHERN_HEMISPHERE_COUNTRIES.has(country)
    : false;

  return {
    country,
    language,
    timezone,
    isSouthernHemisphere,
  };
}

/**
 * Check if user's locale matches a list of country codes
 */
export function matchesLocale(
  locale: LocaleInfo,
  countryCodes: string[]
): boolean {
  if (!locale.country) {
    return false;
  }

  // Wildcard matches everything
  if (countryCodes.includes('*')) {
    return true;
  }

  return countryCodes.includes(locale.country);
}

/**
 * Check if user's language matches a list of language codes
 */
export function matchesLanguage(
  locale: LocaleInfo,
  languageCodes: string[]
): boolean {
  return languageCodes.includes(locale.language);
}

/**
 * Get localized name for a holiday if available
 */
export function getLocalizedName(
  baseName: string,
  localizedNames: Record<string, string> | undefined,
  language: string
): string {
  if (localizedNames && localizedNames[language]) {
    return localizedNames[language];
  }
  return baseName;
}
