/**
 * Humanity Events Utility Functions
 *
 * Handles date matching and event detection for Caro's Humanity Events Calendar.
 * Supports:
 * - Specific dates (month + day)
 * - Entire months (month only)
 * - Date ranges (month + dayStart + dayEnd)
 * - Calculated dates (e.g., "3rd Monday of January")
 */

import type { HumanityEvent, EventDate } from '../data/humanity-events';
import { HUMANITY_EVENTS } from '../data/humanity-events';

// ============================================================================
// Date Calculations
// ============================================================================

/**
 * Get the nth occurrence of a weekday in a given month.
 * @param year - The year
 * @param month - The month (1-12)
 * @param weekday - The day of week (0 = Sunday, 1 = Monday, etc.)
 * @param n - Which occurrence (1 = first, 2 = second, etc.)
 * @returns The date, or null if it doesn't exist
 */
function getNthWeekdayOfMonth(
  year: number,
  month: number,
  weekday: number,
  n: number
): Date | null {
  const firstDay = new Date(year, month - 1, 1);
  const firstWeekday = firstDay.getDay();

  // Calculate days until the first occurrence of the weekday
  let daysUntil = weekday - firstWeekday;
  if (daysUntil < 0) daysUntil += 7;

  // Calculate the date of the nth occurrence
  const day = 1 + daysUntil + (n - 1) * 7;

  const result = new Date(year, month - 1, day);

  // Check if the date is still in the same month
  if (result.getMonth() !== month - 1) {
    return null;
  }

  return result;
}

/**
 * Calculate a floating holiday date.
 * @param calculation - The calculation name
 * @param year - The year to calculate for
 * @returns The calculated date, or null if unknown calculation
 */
function calculateFloatingDate(calculation: string, year: number): Date | null {
  switch (calculation) {
    case 'mlk-day':
      // Martin Luther King Jr. Day: 3rd Monday of January
      return getNthWeekdayOfMonth(year, 1, 1, 3);

    case 'indigenous-peoples-day':
      // Indigenous Peoples' Day: 2nd Monday of October
      return getNthWeekdayOfMonth(year, 10, 1, 2);

    case 'memorial-day':
      // Memorial Day: Last Monday of May
      // Find all Mondays in May, take the last one
      for (let day = 31; day >= 25; day--) {
        const date = new Date(year, 4, day);
        if (date.getDay() === 1) return date;
      }
      return null;

    case 'labor-day':
      // Labor Day: 1st Monday of September
      return getNthWeekdayOfMonth(year, 9, 1, 1);

    case 'thanksgiving':
      // Thanksgiving: 4th Thursday of November
      return getNthWeekdayOfMonth(year, 11, 4, 4);

    default:
      console.warn(`Unknown date calculation: ${calculation}`);
      return null;
  }
}

// ============================================================================
// Date Matching
// ============================================================================

/**
 * Check if a date matches an event date specification.
 * @param eventDate - The event date specification
 * @param checkDate - The date to check
 * @returns Whether the date matches
 */
function matchesEventDate(eventDate: EventDate, checkDate: Date): boolean {
  const year = checkDate.getFullYear();
  const month = checkDate.getMonth() + 1; // Convert to 1-indexed
  const day = checkDate.getDate();

  // Handle calculated dates (floating holidays)
  if (eventDate.calculate) {
    const calculatedDate = calculateFloatingDate(eventDate.calculate, year);
    if (!calculatedDate) return false;

    return (
      calculatedDate.getMonth() + 1 === month &&
      calculatedDate.getDate() === day
    );
  }

  // Check month first
  if (eventDate.month !== month) return false;

  // If only month specified, match entire month
  if (eventDate.day === undefined && eventDate.dayStart === undefined) {
    return true;
  }

  // Check specific day
  if (eventDate.day !== undefined) {
    return eventDate.day === day;
  }

  // Check date range
  if (eventDate.dayStart !== undefined && eventDate.dayEnd !== undefined) {
    return day >= eventDate.dayStart && day <= eventDate.dayEnd;
  }

  return false;
}

/**
 * Check if an event is active on a given date.
 * @param event - The event to check
 * @param checkDate - The date to check
 * @returns Whether the event is active
 */
export function isEventActive(event: HumanityEvent, checkDate: Date): boolean {
  return event.dates.some((eventDate) => matchesEventDate(eventDate, checkDate));
}

// ============================================================================
// Public API
// ============================================================================

/**
 * Get all active humanity events for a given date.
 * @param checkDate - The date to check (defaults to now)
 * @returns Array of active events, sorted by specificity (specific days first, then months)
 */
export function getActiveEvents(checkDate: Date = new Date()): HumanityEvent[] {
  const activeEvents = HUMANITY_EVENTS.filter((event) =>
    isEventActive(event, checkDate)
  );

  // Sort by specificity: specific days first, then date ranges, then full months
  return activeEvents.sort((a, b) => {
    const aSpecificity = getEventSpecificity(a, checkDate);
    const bSpecificity = getEventSpecificity(b, checkDate);
    return bSpecificity - aSpecificity;
  });
}

/**
 * Calculate the specificity of an event match.
 * Higher specificity = more specific date match.
 * - Calculated dates (floating holidays): 100
 * - Specific day: 90
 * - Date range: 50
 * - Full month: 10
 */
function getEventSpecificity(event: HumanityEvent, checkDate: Date): number {
  for (const eventDate of event.dates) {
    if (!matchesEventDate(eventDate, checkDate)) continue;

    if (eventDate.calculate) return 100;
    if (eventDate.day !== undefined) return 90;
    if (eventDate.dayStart !== undefined) return 50;
    return 10; // Full month
  }
  return 0;
}

/**
 * Get the primary (most specific) active event.
 * @param checkDate - The date to check (defaults to now)
 * @returns The most specific active event, or null if none
 */
export function getPrimaryEvent(checkDate: Date = new Date()): HumanityEvent | null {
  const events = getActiveEvents(checkDate);
  return events.length > 0 ? events[0] : null;
}

/**
 * Get all events with full themes that are active.
 * These events should apply visual transformations to the website.
 * @param checkDate - The date to check (defaults to now)
 * @returns Array of active events with full themes
 */
export function getThemedEvents(checkDate: Date = new Date()): HumanityEvent[] {
  return getActiveEvents(checkDate).filter((event) => event.hasFullTheme);
}

/**
 * Get the CSS class names for active themed events.
 * @param checkDate - The date to check (defaults to now)
 * @returns Array of CSS class names to apply
 */
export function getEventThemeClasses(checkDate: Date = new Date()): string[] {
  return getThemedEvents(checkDate)
    .filter((event) => event.theme?.className)
    .map((event) => event.theme!.className!);
}

/**
 * Generate inline CSS variables for an event's theme.
 * @param event - The event
 * @returns CSS variable declarations as a string
 */
export function getEventThemeCSS(event: HumanityEvent): string {
  if (!event.theme) return '';

  const vars: string[] = [];

  if (event.theme.accent) {
    vars.push(`--humanity-event-accent: ${event.theme.accent}`);
  }
  if (event.theme.accentSecondary) {
    vars.push(`--humanity-event-accent-secondary: ${event.theme.accentSecondary}`);
  }
  if (event.theme.accentTertiary) {
    vars.push(`--humanity-event-accent-tertiary: ${event.theme.accentTertiary}`);
  }
  if (event.theme.bgTint) {
    vars.push(`--humanity-event-bg-tint: ${event.theme.bgTint}`);
  }

  return vars.join('; ');
}

/**
 * Check if there are any active events.
 * @param checkDate - The date to check (defaults to now)
 * @returns Whether there are active events
 */
export function hasActiveEvents(checkDate: Date = new Date()): boolean {
  return getActiveEvents(checkDate).length > 0;
}

/**
 * Get an event by its ID.
 * @param id - The event ID
 * @returns The event, or undefined if not found
 */
export function getEventById(id: string): HumanityEvent | undefined {
  return HUMANITY_EVENTS.find((event) => event.id === id);
}

/**
 * Format the event date(s) for display.
 *
 * NOTE: This is an internal utility function for testing/debugging purposes.
 * It intentionally uses English ('en-US') formatting. User-facing date displays
 * should use toLocaleDateString(lang, {...}) directly in components.
 *
 * @param event - The event
 * @param year - The year (for calculated dates)
 * @returns Human-readable date string (English format)
 */
export function formatEventDates(event: HumanityEvent, year?: number): string {
  const dateStrings: string[] = [];

  for (const eventDate of event.dates) {
    if (eventDate.calculate && year) {
      const date = calculateFloatingDate(eventDate.calculate, year);
      if (date) {
        dateStrings.push(date.toLocaleDateString('en-US', {
          month: 'long',
          day: 'numeric',
        }));
      }
    } else if (eventDate.day !== undefined) {
      const date = new Date(2000, eventDate.month - 1, eventDate.day);
      dateStrings.push(date.toLocaleDateString('en-US', {
        month: 'long',
        day: 'numeric',
      }));
    } else if (eventDate.dayStart !== undefined && eventDate.dayEnd !== undefined) {
      const monthName = new Date(2000, eventDate.month - 1, 1).toLocaleDateString('en-US', {
        month: 'long',
      });
      dateStrings.push(`${monthName} ${eventDate.dayStart}-${eventDate.dayEnd}`);
    } else {
      dateStrings.push(new Date(2000, eventDate.month - 1, 1).toLocaleDateString('en-US', {
        month: 'long',
      }));
    }
  }

  return dateStrings.join(' - ');
}

/**
 * Get a sample test date for an event.
 * @param event - The event
 * @param year - The year (defaults to current year)
 * @returns A date string in YYYY-MM-DD format
 */
export function getTestDateForEvent(event: HumanityEvent, year?: number): string {
  const y = year || new Date().getFullYear();
  const eventDate = event.dates[0];

  if (eventDate.calculate) {
    const date = calculateFloatingDate(eventDate.calculate, y);
    if (date) {
      return date.toISOString().split('T')[0];
    }
    // Fallback for unknown calculations
    const month = String(eventDate.month).padStart(2, '0');
    return `${y}-${month}-15`;
  }

  if (eventDate.day) {
    const month = String(eventDate.month).padStart(2, '0');
    const day = String(eventDate.day).padStart(2, '0');
    return `${y}-${month}-${day}`;
  }

  if (eventDate.dayStart) {
    const month = String(eventDate.month).padStart(2, '0');
    const day = String(eventDate.dayStart).padStart(2, '0');
    return `${y}-${month}-${day}`;
  }

  // Full month - use the 15th
  const month = String(eventDate.month).padStart(2, '0');
  return `${y}-${month}-15`;
}
