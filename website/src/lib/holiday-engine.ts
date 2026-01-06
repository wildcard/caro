/**
 * Holiday Theme Engine
 *
 * Central orchestrator for the holiday theme system. Manages:
 * - Active holiday detection based on current date
 * - Visibility filtering based on locale
 * - Default theme selection using priority rules
 * - Seasonal effect state
 *
 * Usage:
 *   import { holidayEngine } from './holiday-engine';
 *   const state = holidayEngine.getState();
 *   const activeThemes = state.visibleThemes;
 */

import type {
  HolidayEvent,
  SeasonalEffect,
  LocaleInfo,
  HolidayEngineState,
  DefaultRule,
} from '../config/types';
import { HOLIDAYS, getEnabledHolidays } from '../config/holidays';
import { SEASONS, getEnabledSeasons } from '../config/seasons';
import { getCurrentDate, isDateInRange, resolveDateRange } from './date-utils';
import { detectLocale, matchesLocale, matchesLanguage } from './locale-detector';

// ============================================================================
// Holiday Theme Engine Class
// ============================================================================

export class HolidayThemeEngine {
  private locale: LocaleInfo;
  private holidays: HolidayEvent[];
  private seasons: SeasonalEffect[];

  constructor() {
    this.locale = detectLocale();
    this.holidays = getEnabledHolidays();
    this.seasons = getEnabledSeasons();
  }

  /**
   * Refresh locale detection (call after navigation or locale change)
   */
  refreshLocale(): void {
    this.locale = detectLocale();
  }

  /**
   * Get detected user locale
   */
  getLocale(): LocaleInfo {
    return this.locale;
  }

  // ==========================================================================
  // Active Holiday Detection
  // ==========================================================================

  /**
   * Get all holidays that are currently active (within their date range)
   */
  getActiveHolidays(): HolidayEvent[] {
    const now = getCurrentDate();

    return this.holidays.filter((holiday) => {
      return isDateInRange(now, holiday.dateRange, holiday.leadTimeDays || 0);
    });
  }

  /**
   * Check if a specific holiday is currently active
   */
  isHolidayActive(holidayId: string): boolean {
    const holiday = this.holidays.find((h) => h.id === holidayId);
    if (!holiday) return false;

    const now = getCurrentDate();
    return isDateInRange(now, holiday.dateRange, holiday.leadTimeDays || 0);
  }

  // ==========================================================================
  // Visibility Filtering
  // ==========================================================================

  /**
   * Get holidays that should be visible in the menu for the current user
   *
   * Rules:
   * 1. Holiday must be in its active date range
   * 2. Global holidays are visible to everyone
   * 3. Locale-only holidays are visible only to users in those locales
   */
  getVisibleThemes(): HolidayEvent[] {
    const activeHolidays = this.getActiveHolidays();

    return activeHolidays.filter((holiday) => {
      // Global holidays are always visible when active
      if (holiday.visibility === 'global') {
        return true;
      }

      // Locale-only holidays require locale match
      if (holiday.visibility === 'locale-only') {
        return this.matchesHolidayLocale(holiday);
      }

      return false;
    });
  }

  /**
   * Check if user's locale matches a holiday's locale requirements
   */
  private matchesHolidayLocale(holiday: HolidayEvent): boolean {
    if (!holiday.locales || holiday.locales.length === 0) {
      return true; // No locale restriction
    }

    return matchesLocale(this.locale, holiday.locales);
  }

  // ==========================================================================
  // Default Theme Selection
  // ==========================================================================

  /**
   * Get the recommended default theme for the current user
   *
   * Priority system:
   * 1. Higher priority values take precedence
   * 2. Locale matches are typically higher priority than language matches
   * 3. If no rules match, returns null (opt-in only)
   *
   * @returns Holiday ID or null for no default
   */
  getDefaultTheme(): string | null {
    const visibleThemes = this.getVisibleThemes();

    if (visibleThemes.length === 0) {
      return null;
    }

    // Find themes with matching default rules
    const candidates: Array<{ holiday: HolidayEvent; priority: number }> = [];

    for (const holiday of visibleThemes) {
      const matchedPriority = this.getMatchedPriority(holiday);
      if (matchedPriority !== null) {
        candidates.push({ holiday, priority: matchedPriority });
      }
    }

    if (candidates.length === 0) {
      // No matching default rules - return null (user must opt-in)
      return null;
    }

    // Sort by priority (descending) and return highest
    candidates.sort((a, b) => b.priority - a.priority);
    return candidates[0].holiday.id;
  }

  /**
   * Get the matched priority for a holiday's default rules
   * Returns the highest matching priority, or null if no rules match
   */
  private getMatchedPriority(holiday: HolidayEvent): number | null {
    if (!holiday.defaultFor || holiday.defaultFor.length === 0) {
      return null;
    }

    let highestPriority: number | null = null;

    for (const rule of holiday.defaultFor) {
      if (this.matchesRule(rule)) {
        if (highestPriority === null || rule.priority > highestPriority) {
          highestPriority = rule.priority;
        }
      }
    }

    return highestPriority;
  }

  /**
   * Check if user matches a default rule
   */
  private matchesRule(rule: DefaultRule): boolean {
    switch (rule.type) {
      case 'locale':
        // Wildcard matches everyone
        if (rule.values.includes('*')) {
          return true;
        }
        return matchesLocale(this.locale, rule.values);

      case 'language':
        return matchesLanguage(this.locale, rule.values);

      case 'timezone':
        // Check if user's timezone matches any pattern
        return rule.values.some(
          (tz) =>
            this.locale.timezone === tz ||
            this.locale.timezone.startsWith(tz + '/')
        );

      default:
        return false;
    }
  }

  // ==========================================================================
  // Seasonal Effects
  // ==========================================================================

  /**
   * Get currently active seasonal effects
   */
  getActiveSeasons(): SeasonalEffect[] {
    const now = getCurrentDate();

    return this.seasons.filter((season) => {
      return this.isSeasonActive(season, now);
    });
  }

  /**
   * Check if a specific seasonal effect is currently active
   */
  isSeasonActive(season: SeasonalEffect, date?: Date): boolean {
    const checkDate = date || getCurrentDate();

    // For hemisphere-aware effects, we need to check if we're in the
    // southern hemisphere and potentially flip the dates
    if (season.hemisphereAware && this.locale.isSouthernHemisphere) {
      // For southern hemisphere, we flip the date range
      // Winter (Dec-Feb) becomes Summer (Jun-Aug) and vice versa
      const flippedRange = this.flipDateRangeForHemisphere(season.dateRange);
      return isDateInRange(checkDate, flippedRange);
    }

    return isDateInRange(checkDate, season.dateRange);
  }

  /**
   * Flip a fixed date range for southern hemisphere
   */
  private flipDateRangeForHemisphere(range: any): any {
    if (range.type !== 'fixed') {
      return range; // Only flip fixed ranges
    }

    // Add 6 months to flip seasons
    const flip = (month: number) => ((month + 5) % 12) + 1;

    return {
      type: 'fixed',
      startMonth: flip(range.startMonth),
      startDay: range.startDay,
      endMonth: flip(range.endMonth),
      endDay: range.endDay,
    };
  }

  /**
   * Check if snow effect should be active
   */
  isSnowActive(): boolean {
    const winterSeason = this.seasons.find((s) => s.id === 'winter-snow');
    if (!winterSeason || !winterSeason.defaultEnabled) {
      return false;
    }
    return this.isSeasonActive(winterSeason);
  }

  // ==========================================================================
  // State Aggregation
  // ==========================================================================

  /**
   * Get complete engine state
   */
  getState(): HolidayEngineState {
    const activeHolidays = this.getActiveHolidays();
    const visibleThemes = this.getVisibleThemes();
    const defaultTheme = this.getDefaultTheme();
    const activeSeasons = this.getActiveSeasons();

    // Get user preference from localStorage
    let userPreference: string | null = null;
    if (typeof localStorage !== 'undefined') {
      userPreference = localStorage.getItem('holidayTheme');
    }

    return {
      activeHolidays,
      visibleThemes,
      defaultTheme,
      userPreference,
      activeSeasons,
      locale: this.locale,
    };
  }

  /**
   * Get the theme that should be applied
   * (user preference overrides default)
   */
  getAppliedTheme(): string | null {
    if (typeof localStorage !== 'undefined') {
      const userPref = localStorage.getItem('holidayTheme');
      if (userPref) {
        // If user explicitly set 'none', respect it
        if (userPref === 'none') {
          return null;
        }
        // Verify the user's preference is still valid (holiday is active)
        if (this.isHolidayActive(userPref)) {
          return userPref;
        }
        // User's preference is for an inactive holiday, clear it
        localStorage.removeItem('holidayTheme');
      }
    }

    // No user preference, use default
    return this.getDefaultTheme();
  }

  // ==========================================================================
  // Debug Helpers
  // ==========================================================================

  /**
   * Get debug info for development
   */
  getDebugInfo(): object {
    const state = this.getState();
    const now = getCurrentDate();

    return {
      currentDate: now.toISOString(),
      locale: this.locale,
      activeHolidayIds: state.activeHolidays.map((h) => h.id),
      visibleThemeIds: state.visibleThemes.map((h) => h.id),
      defaultTheme: state.defaultTheme,
      userPreference: state.userPreference,
      appliedTheme: this.getAppliedTheme(),
      snowActive: this.isSnowActive(),
      totalHolidays: this.holidays.length,
    };
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

/**
 * Singleton engine instance for use across the application
 */
export const holidayEngine = new HolidayThemeEngine();

// ============================================================================
// Convenience Exports
// ============================================================================

export {
  getCurrentDate,
  isDateInRange,
  detectLocale,
};
