/**
 * Calendar Module Index
 *
 * Factory for calendar-based date calculations. Currently supports:
 * - Gregorian (fixed dates)
 * - Hebrew (via lookup table for major holidays)
 *
 * Future support planned for:
 * - Chinese/Lunar calendar
 * - Hindu calendar
 * - Islamic calendar
 */

import type { DateRange } from '../../config/types';

export { resolveHebrewDate, HEBREW_HOLIDAYS } from './hebrew';

/**
 * Calendar type to resolver mapping
 */
export type CalendarType = 'fixed' | 'hebrew' | 'lunar' | 'hindu' | 'islamic' | 'calculated';

/**
 * Check if a calendar type is supported
 */
export function isCalendarSupported(type: CalendarType): boolean {
  switch (type) {
    case 'fixed':
    case 'calculated':
    case 'hebrew':
      return true;
    case 'lunar':
    case 'hindu':
    case 'islamic':
      // Planned but not yet implemented
      return false;
    default:
      return false;
  }
}

/**
 * Get calendar type display name
 */
export function getCalendarName(type: CalendarType): string {
  const names: Record<CalendarType, string> = {
    fixed: 'Gregorian (Fixed)',
    calculated: 'Pre-calculated',
    hebrew: 'Hebrew Calendar',
    lunar: 'Chinese/Lunar Calendar',
    hindu: 'Hindu Calendar',
    islamic: 'Islamic Calendar',
  };
  return names[type] || 'Unknown';
}
