/**
 * Date Utilities for Holiday Theme System
 *
 * Provides functions for checking if dates fall within holiday ranges,
 * handling year boundaries, and supporting test date overrides.
 */

import type { DateRange, FixedDateRange, CalculatedDateRange } from '../config/types';

// ============================================================================
// Test Date Override
// ============================================================================

/**
 * Get the current date, respecting test date override for development
 */
export function getCurrentDate(): Date {
  // Check for test date override (set by Layout.astro for localhost testing)
  if (typeof window !== 'undefined' && (window as any).__CARO_TEST_DATE) {
    return new Date((window as any).__CARO_TEST_DATE);
  }
  return new Date();
}

// ============================================================================
// Date Range Resolution
// ============================================================================

/**
 * Resolve a fixed date range to actual Date objects for a given year
 */
function resolveFixedDateRange(
  range: FixedDateRange,
  year: number
): { start: Date; end: Date } {
  let startYear = year;
  let endYear = year;

  // Handle year boundary (e.g., Dec 31 - Jan 1)
  if (
    range.endMonth < range.startMonth ||
    (range.endMonth === range.startMonth && range.endDay < range.startDay)
  ) {
    // End is in the next year
    endYear = year + 1;
  }

  // Also handle the case where we're checking in January for a Dec-Jan range
  // In this case, the start should be in the previous year

  return {
    start: new Date(startYear, range.startMonth - 1, range.startDay),
    end: new Date(endYear, range.endMonth - 1, range.endDay, 23, 59, 59),
  };
}

/**
 * Resolve a calculated date range for a given year
 */
function resolveCalculatedDateRange(
  range: CalculatedDateRange,
  year: number
): { start: Date; end: Date } | null {
  const yearData = range.dates[year];
  if (!yearData) {
    // Try to find the closest year
    const years = Object.keys(range.dates).map(Number).sort();
    const closest = years.reduce((prev, curr) =>
      Math.abs(curr - year) < Math.abs(prev - year) ? curr : prev
    );

    // Only use if within 1 year
    if (Math.abs(closest - year) <= 1) {
      const fallback = range.dates[closest];
      // Adjust the year in the dates
      const startDate = new Date(fallback.start);
      const endDate = new Date(fallback.end);
      startDate.setFullYear(year);
      endDate.setFullYear(year);
      return { start: startDate, end: endDate };
    }

    return null;
  }

  return {
    start: new Date(yearData.start),
    end: new Date(yearData.end + 'T23:59:59'),
  };
}

/**
 * Resolve any date range type to start/end Date objects
 */
export function resolveDateRange(
  range: DateRange,
  year: number
): { start: Date; end: Date } | null {
  switch (range.type) {
    case 'fixed':
      return resolveFixedDateRange(range, year);

    case 'calculated':
      return resolveCalculatedDateRange(range, year);

    case 'hebrew':
    case 'lunar':
    case 'hindu':
    case 'islamic':
      // These require calendar-specific calculations
      // For now, return null - will be handled by calendar modules
      console.warn(`Calendar type "${range.type}" requires calendar module`);
      return null;

    default:
      console.warn(`Unknown date range type`);
      return null;
  }
}

// ============================================================================
// Date Range Checking
// ============================================================================

/**
 * Check if a date falls within a date range
 *
 * @param date - Date to check
 * @param range - Date range configuration
 * @param leadTimeDays - Optional days before start to also consider "in range"
 */
export function isDateInRange(
  date: Date,
  range: DateRange,
  leadTimeDays: number = 0
): boolean {
  const year = date.getFullYear();

  // Try current year
  let resolved = resolveDateRange(range, year);

  if (resolved) {
    // Apply lead time
    const adjustedStart = new Date(resolved.start);
    if (leadTimeDays > 0) {
      adjustedStart.setDate(adjustedStart.getDate() - leadTimeDays);
    }

    if (date >= adjustedStart && date <= resolved.end) {
      return true;
    }
  }

  // For fixed ranges that span year boundary, also check previous year's range
  if (range.type === 'fixed') {
    const prevYearResolved = resolveDateRange(range, year - 1);
    if (prevYearResolved) {
      const adjustedStart = new Date(prevYearResolved.start);
      if (leadTimeDays > 0) {
        adjustedStart.setDate(adjustedStart.getDate() - leadTimeDays);
      }

      if (date >= adjustedStart && date <= prevYearResolved.end) {
        return true;
      }
    }
  }

  return false;
}

/**
 * Get the resolved date range for display purposes
 */
export function getDisplayDateRange(
  range: DateRange,
  year?: number
): { startStr: string; endStr: string } | null {
  const targetYear = year ?? new Date().getFullYear();
  const resolved = resolveDateRange(range, targetYear);

  if (!resolved) {
    return null;
  }

  const options: Intl.DateTimeFormatOptions = {
    month: 'short',
    day: 'numeric',
  };

  return {
    startStr: resolved.start.toLocaleDateString('en-US', options),
    endStr: resolved.end.toLocaleDateString('en-US', options),
  };
}

/**
 * Check if today is within or approaching a date range
 */
export function getDateRangeStatus(
  range: DateRange,
  leadTimeDays: number = 0
): 'active' | 'upcoming' | 'past' | 'unknown' {
  const now = getCurrentDate();
  const year = now.getFullYear();

  const resolved = resolveDateRange(range, year);
  if (!resolved) {
    return 'unknown';
  }

  const { start, end } = resolved;

  // Check if currently active
  if (now >= start && now <= end) {
    return 'active';
  }

  // Check if upcoming (within lead time)
  const upcomingStart = new Date(start);
  upcomingStart.setDate(upcomingStart.getDate() - leadTimeDays);
  if (now >= upcomingStart && now < start) {
    return 'upcoming';
  }

  // Check if past
  if (now > end) {
    return 'past';
  }

  return 'upcoming';
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get days until a date range starts
 */
export function getDaysUntilStart(range: DateRange): number | null {
  const now = getCurrentDate();
  const year = now.getFullYear();
  const resolved = resolveDateRange(range, year);

  if (!resolved) {
    return null;
  }

  const diffTime = resolved.start.getTime() - now.getTime();
  if (diffTime < 0) {
    return 0; // Already started
  }

  return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
}

/**
 * Check if a month is in the "winter" season for a given hemisphere
 */
export function isWinterMonth(month: number, isSouthern: boolean): boolean {
  // Northern hemisphere winter: Dec, Jan, Feb (months 12, 1, 2)
  const northernWinter = [12, 1, 2];
  // Southern hemisphere winter: Jun, Jul, Aug (months 6, 7, 8)
  const southernWinter = [6, 7, 8];

  const winterMonths = isSouthern ? southernWinter : northernWinter;
  return winterMonths.includes(month);
}
