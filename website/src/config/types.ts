/**
 * Holiday Theme Configuration Types
 *
 * This module defines the type system for the data-driven holiday theme
 * configuration. All holidays and seasonal effects are defined using these
 * interfaces to ensure type safety and easy extensibility.
 */

// ============================================================================
// Date Range Types
// ============================================================================

/**
 * Fixed date range using month/day (Gregorian calendar)
 * Example: Christmas is Dec 1-25 every year
 */
export interface FixedDateRange {
  type: 'fixed';
  /** Month (1-12) */
  startMonth: number;
  /** Day of month (1-31) */
  startDay: number;
  /** Month (1-12) */
  endMonth: number;
  /** Day of month (1-31) */
  endDay: number;
}

/**
 * Hebrew calendar date range
 * Dates are calculated based on the Hebrew calendar
 */
export interface HebrewDateRange {
  type: 'hebrew';
  /** Hebrew month name */
  startMonth: HebrewMonth;
  startDay: number;
  endMonth: HebrewMonth;
  endDay: number;
}

/**
 * Chinese/Lunar calendar date range
 * Dates are calculated based on the Chinese lunar calendar
 */
export interface LunarDateRange {
  type: 'lunar';
  /** Lunar month (1-12) */
  startMonth: number;
  startDay: number;
  endMonth: number;
  endDay: number;
}

/**
 * Hindu calendar date range (for Diwali, etc.)
 */
export interface HinduDateRange {
  type: 'hindu';
  /** Hindu month name */
  month: HinduMonth;
  /** Paksha (fortnight) */
  paksha: 'shukla' | 'krishna';
  /** Tithi (lunar day) */
  tithi: number;
  /** Duration in days */
  durationDays: number;
}

/**
 * Islamic calendar date range
 */
export interface IslamicDateRange {
  type: 'islamic';
  /** Islamic month (1-12) */
  startMonth: number;
  startDay: number;
  endMonth: number;
  endDay: number;
}

/**
 * Pre-calculated dates for complex holidays
 * Use when calendar calculation is too complex
 */
export interface CalculatedDateRange {
  type: 'calculated';
  /** Pre-calculated dates by year: { 2024: { start: "2024-12-25", end: "2025-01-01" } } */
  dates: Record<number, { start: string; end: string }>;
}

export type DateRange =
  | FixedDateRange
  | HebrewDateRange
  | LunarDateRange
  | HinduDateRange
  | IslamicDateRange
  | CalculatedDateRange;

// ============================================================================
// Hebrew Calendar Constants
// ============================================================================

export type HebrewMonth =
  | 'Tishrei'
  | 'Cheshvan'
  | 'Kislev'
  | 'Tevet'
  | 'Shevat'
  | 'Adar'
  | 'AdarII' // Leap year
  | 'Nisan'
  | 'Iyar'
  | 'Sivan'
  | 'Tammuz'
  | 'Av'
  | 'Elul';

// ============================================================================
// Hindu Calendar Constants
// ============================================================================

export type HinduMonth =
  | 'Chaitra'
  | 'Vaishakha'
  | 'Jyeshtha'
  | 'Ashadha'
  | 'Shravana'
  | 'Bhadrapada'
  | 'Ashwin'
  | 'Kartik'
  | 'Margashirsha'
  | 'Pausha'
  | 'Magha'
  | 'Phalguna';

// ============================================================================
// Theme Configuration
// ============================================================================

/**
 * Visual theme configuration for a holiday
 */
export interface ThemeConfig {
  /** Primary theme color (hex) */
  primaryColor: string;
  /** Secondary theme color (hex) */
  secondaryColor: string;
  /** Optional accent color (hex) */
  accentColor?: string;
  /** Background color adjustment (hex) */
  bgColor?: string;
  /** Emoji icon for menu display */
  icon: string;
  /** CSS class names for decorations */
  decorations?: string[];
  /** Optional effect component names */
  effects?: string[];
}

// ============================================================================
// Default Selection Rules
// ============================================================================

/**
 * Rule for when to auto-select this holiday as default
 */
export interface DefaultRule {
  /** Type of matching rule */
  type: 'locale' | 'language' | 'timezone';
  /** Values to match (ISO country codes, language codes, or timezone patterns) */
  values: string[];
  /** Priority (higher = more specific, takes precedence) */
  priority: number;
}

// ============================================================================
// Holiday Event
// ============================================================================

/**
 * Complete holiday event configuration
 */
export interface HolidayEvent {
  /** Unique identifier (e.g., "christmas", "diwali") */
  id: string;

  /** Display name in English */
  name: string;

  /** Localized names: { "he": "חנוכה", "es": "Navidad" } */
  localizedNames?: Record<string, string>;

  /** Description for accessibility */
  description?: string;

  // ---- Timing ----

  /** When the theme is active */
  dateRange: DateRange;

  /** Days before the holiday to start showing the theme */
  leadTimeDays?: number;

  // ---- Visibility ----

  /**
   * Visibility scope:
   * - 'global': Available worldwide during date range
   * - 'locale-only': Only visible in specified locales
   */
  visibility: 'global' | 'locale-only';

  /** ISO country codes for locale-only visibility (e.g., ["US", "CA"]) */
  locales?: string[];

  /** ISO language codes for language-based visibility (e.g., ["he", "yi"]) */
  languages?: string[];

  // ---- Theme ----

  /** Visual theme configuration */
  theme: ThemeConfig;

  // ---- Default Selection ----

  /**
   * Rules for when to auto-select this theme as default.
   * If no rules match, theme is available but not pre-selected.
   */
  defaultFor?: DefaultRule[];

  // ---- Metadata ----

  /** Category for grouping in UI */
  category?: 'religious' | 'cultural' | 'national' | 'seasonal';

  /** Whether this is currently enabled */
  enabled?: boolean;
}

// ============================================================================
// Seasonal Effects
// ============================================================================

/**
 * Seasonal visual effect (independent of holidays)
 */
export interface SeasonalEffect {
  /** Unique identifier (e.g., "winter-snow") */
  id: string;

  /** Display name */
  name: string;

  /** Effect component to render */
  effect: 'snow' | 'leaves' | 'petals' | 'confetti' | 'fireworks';

  /** When the effect is active */
  dateRange: DateRange;

  /**
   * Whether to flip dates for southern hemisphere
   * If true, winter (Dec-Feb) becomes summer, etc.
   */
  hemisphereAware: boolean;

  /** Whether effect is enabled by default */
  defaultEnabled: boolean;

  /** Emoji icon for toggle */
  icon: string;
}

// ============================================================================
// Locale Detection
// ============================================================================

/**
 * Detected user locale information
 */
export interface LocaleInfo {
  /** ISO country code (e.g., "US", "IL", "FR") */
  country: string | null;

  /** ISO language code (e.g., "en", "he", "fr") */
  language: string;

  /** IANA timezone (e.g., "America/New_York") */
  timezone: string;

  /** Whether southern hemisphere (for seasonal effects) */
  isSouthernHemisphere: boolean;
}

// ============================================================================
// Engine State
// ============================================================================

/**
 * Current state of the holiday theme engine
 */
export interface HolidayEngineState {
  /** Currently active holidays (in date range) */
  activeHolidays: HolidayEvent[];

  /** Holidays visible in the menu for this user */
  visibleThemes: HolidayEvent[];

  /** Recommended default theme (or null for opt-in only) */
  defaultTheme: string | null;

  /** User's saved preference (from localStorage) */
  userPreference: string | null;

  /** Active seasonal effects */
  activeSeasons: SeasonalEffect[];

  /** Detected user locale */
  locale: LocaleInfo;
}
