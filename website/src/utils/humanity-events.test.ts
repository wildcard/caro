/**
 * Humanity Events Utility Tests
 *
 * Comprehensive test suite for date calculations, event detection,
 * and edge cases in the humanity events calendar system.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { getActiveEvents, getEventThemeClasses, getPrimaryEvent } from './humanity-events';
import type { HumanityEvent } from '../data/humanity-events';

// ============================================================================
// Test Helpers
// ============================================================================

/**
 * Create a date from a string for easier testing
 */
function date(str: string): Date {
  return new Date(str);
}

/**
 * Get event by ID from a list
 */
function findEvent(events: HumanityEvent[], id: string): HumanityEvent | undefined {
  return events.find(e => e.id === id);
}

// ============================================================================
// Date Calculation Tests
// ============================================================================

describe('Date Calculations - Floating Holidays', () => {
  describe('Martin Luther King Jr. Day (3rd Monday of January)', () => {
    it('calculates MLK Day correctly for 2025', () => {
      const events = getActiveEvents(date('2025-01-20'));
      const mlkEvent = findEvent(events, 'mlk-day');
      expect(mlkEvent).toBeDefined();
    });

    it('calculates MLK Day correctly for 2026', () => {
      const events = getActiveEvents(date('2026-01-19'));
      const mlkEvent = findEvent(events, 'mlk-day');
      expect(mlkEvent).toBeDefined();
    });

    it('does not show MLK Day on wrong dates', () => {
      const events = getActiveEvents(date('2025-01-19')); // Day before
      const mlkEvent = findEvent(events, 'mlk-day');
      expect(mlkEvent).toBeUndefined();
    });

    it('does not show MLK Day on the next day', () => {
      const events = getActiveEvents(date('2025-01-21')); // Day after
      const mlkEvent = findEvent(events, 'mlk-day');
      expect(mlkEvent).toBeUndefined();
    });
  });

  describe('Indigenous Peoples\' Day (2nd Monday of October)', () => {
    it('calculates Indigenous Peoples\' Day correctly for 2025', () => {
      const events = getActiveEvents(date('2025-10-13'));
      const indigenousEvent = findEvent(events, 'indigenous-peoples-day');
      expect(indigenousEvent).toBeDefined();
    });

    it('calculates Indigenous Peoples\' Day correctly for 2026', () => {
      const events = getActiveEvents(date('2026-10-12'));
      const indigenousEvent = findEvent(events, 'indigenous-peoples-day');
      expect(indigenousEvent).toBeDefined();
    });

    it('does not show on wrong dates', () => {
      const events = getActiveEvents(date('2025-10-12')); // Day before
      const indigenousEvent = findEvent(events, 'indigenous-peoples-day');
      expect(indigenousEvent).toBeUndefined();
    });
  });
});

// ============================================================================
// Event Detection Tests - Specific Days
// ============================================================================

describe('Event Detection - Specific Days', () => {
  it('detects International Women\'s Day (March 8)', () => {
    const events = getActiveEvents(date('2025-03-08'));
    const womensDay = findEvent(events, 'international-womens-day');
    expect(womensDay).toBeDefined();
    expect(womensDay?.message).toContain('women');
  });

  it('detects Earth Day (April 22)', () => {
    const events = getActiveEvents(date('2025-04-22'));
    const earthDay = findEvent(events, 'earth-day');
    expect(earthDay).toBeDefined();
    expect(earthDay?.message).toContain('planet');
  });

  it('detects Juneteenth (June 19)', () => {
    const events = getActiveEvents(date('2025-06-19'));
    const juneteenth = findEvent(events, 'juneteenth');
    expect(juneteenth).toBeDefined();
    expect(juneteenth?.message).toContain('freedom');
  });

  it('detects International Day of Peace (September 21)', () => {
    const events = getActiveEvents(date('2025-09-21'));
    const peaceDay = findEvent(events, 'international-day-of-peace');
    expect(peaceDay).toBeDefined();
    expect(peaceDay?.category).toBe('peace');
  });

  it('detects World Mental Health Day (October 10)', () => {
    const events = getActiveEvents(date('2025-10-10'));
    const mentalHealthDay = findEvent(events, 'world-mental-health-day');
    expect(mentalHealthDay).toBeDefined();
    expect(mentalHealthDay?.category).toBe('awareness');
  });

  it('detects International Human Rights Day (December 10)', () => {
    const events = getActiveEvents(date('2025-12-10'));
    const humanRightsDay = findEvent(events, 'international-human-rights-day');
    expect(humanRightsDay).toBeDefined();
  });

  it('does not detect events on wrong dates', () => {
    const events = getActiveEvents(date('2025-04-21')); // Day before Earth Day
    const earthDay = findEvent(events, 'earth-day');
    expect(earthDay).toBeUndefined();
  });
});

// ============================================================================
// Event Detection Tests - Full Months
// ============================================================================

describe('Event Detection - Full Months', () => {
  it('detects Black History Month throughout February', () => {
    const feb1 = getActiveEvents(date('2025-02-01'));
    const feb15 = getActiveEvents(date('2025-02-15'));
    const feb28 = getActiveEvents(date('2025-02-28'));

    expect(findEvent(feb1, 'black-history-month')).toBeDefined();
    expect(findEvent(feb15, 'black-history-month')).toBeDefined();
    expect(findEvent(feb28, 'black-history-month')).toBeDefined();
  });

  it('does not detect Black History Month outside February', () => {
    const jan31 = getActiveEvents(date('2025-01-31'));
    const mar1 = getActiveEvents(date('2025-03-01'));

    expect(findEvent(jan31, 'black-history-month')).toBeUndefined();
    expect(findEvent(mar1, 'black-history-month')).toBeUndefined();
  });

  it('detects Women\'s History Month throughout March', () => {
    const mar1 = getActiveEvents(date('2025-03-01'));
    const mar15 = getActiveEvents(date('2025-03-15'));
    const mar31 = getActiveEvents(date('2025-03-31'));

    expect(findEvent(mar1, 'womens-history-month')).toBeDefined();
    expect(findEvent(mar15, 'womens-history-month')).toBeDefined();
    expect(findEvent(mar31, 'womens-history-month')).toBeDefined();
  });

  it('detects Pride Month throughout June', () => {
    const jun1 = getActiveEvents(date('2025-06-01'));
    const jun15 = getActiveEvents(date('2025-06-15'));
    const jun30 = getActiveEvents(date('2025-06-30'));

    expect(findEvent(jun1, 'pride-month')).toBeDefined();
    expect(findEvent(jun15, 'pride-month')).toBeDefined();
    expect(findEvent(jun30, 'pride-month')).toBeDefined();
  });

  it('detects Asian Pacific American Heritage Month throughout May', () => {
    const may1 = getActiveEvents(date('2025-05-01'));
    const may31 = getActiveEvents(date('2025-05-31'));

    expect(findEvent(may1, 'asian-pacific-heritage-month')).toBeDefined();
    expect(findEvent(may31, 'asian-pacific-heritage-month')).toBeDefined();
  });

  it('detects Native American Heritage Month throughout November', () => {
    const nov1 = getActiveEvents(date('2025-11-01'));
    const nov30 = getActiveEvents(date('2025-11-30'));

    expect(findEvent(nov1, 'native-american-heritage-month')).toBeDefined();
    expect(findEvent(nov30, 'native-american-heritage-month')).toBeDefined();
  });
});

// ============================================================================
// Event Detection Tests - Date Ranges
// ============================================================================

describe('Event Detection - Date Ranges', () => {
  it('detects Hispanic Heritage Month (Sept 15 - Oct 15)', () => {
    const sept15 = getActiveEvents(date('2025-09-15'));
    const sept30 = getActiveEvents(date('2025-09-30'));
    const oct1 = getActiveEvents(date('2025-10-01'));
    const oct15 = getActiveEvents(date('2025-10-15'));

    expect(findEvent(sept15, 'hispanic-heritage-month')).toBeDefined();
    expect(findEvent(sept30, 'hispanic-heritage-month')).toBeDefined();
    expect(findEvent(oct1, 'hispanic-heritage-month')).toBeDefined();
    expect(findEvent(oct15, 'hispanic-heritage-month')).toBeDefined();
  });

  it('does not detect Hispanic Heritage Month outside the range', () => {
    const sept14 = getActiveEvents(date('2025-09-14')); // Day before
    const oct16 = getActiveEvents(date('2025-10-16')); // Day after

    expect(findEvent(sept14, 'hispanic-heritage-month')).toBeUndefined();
    expect(findEvent(oct16, 'hispanic-heritage-month')).toBeUndefined();
  });
});

// ============================================================================
// Event Prioritization Tests
// ============================================================================

describe('Event Prioritization and Specificity', () => {
  it('prioritizes Juneteenth over Pride Month (specific day > full month)', () => {
    const events = getActiveEvents(date('2025-06-19'));
    const primary = getPrimaryEvent(date('2025-06-19'));

    // Both events should be active
    expect(findEvent(events, 'juneteenth')).toBeDefined();
    expect(findEvent(events, 'pride-month')).toBeDefined();

    // But Juneteenth should be primary (more specific)
    expect(primary?.id).toBe('juneteenth');
  });

  it('prioritizes International Women\'s Day over Women\'s History Month (specific day > full month)', () => {
    const events = getActiveEvents(date('2025-03-08'));
    const primary = getPrimaryEvent(date('2025-03-08'));

    // Both events should be active
    expect(findEvent(events, 'international-womens-day')).toBeDefined();
    expect(findEvent(events, 'womens-history-month')).toBeDefined();

    // But Women's Day should be primary
    expect(primary?.id).toBe('international-womens-day');
  });

  it('handles overlapping events correctly', () => {
    const events = getActiveEvents(date('2025-03-08'));

    // Should have both events
    expect(events.length).toBeGreaterThanOrEqual(2);

    // Events should be sorted by specificity
    const specificities = events.map(e => {
      if (e.dates.some(d => d.day !== undefined && d.dayStart === undefined)) {
        return 3; // Specific day
      } else if (e.dates.some(d => d.dayStart !== undefined)) {
        return 2; // Date range
      } else {
        return 1; // Full month
      }
    });

    // Verify descending order
    for (let i = 1; i < specificities.length; i++) {
      expect(specificities[i]).toBeLessThanOrEqual(specificities[i - 1]);
    }
  });
});

// ============================================================================
// Edge Case Tests
// ============================================================================

describe('Edge Cases', () => {
  describe('Year Boundaries', () => {
    it('handles December 31st correctly', () => {
      const events = getActiveEvents(date('2025-12-31'));
      // Should not show events from next year
      const feb = findEvent(events, 'black-history-month');
      expect(feb).toBeUndefined();
    });

    it('handles January 1st correctly', () => {
      const events = getActiveEvents(date('2025-01-01'));
      // Should not show events from previous year's December
      const humanRights = findEvent(events, 'international-human-rights-day');
      expect(humanRights).toBeUndefined();
    });

    it('handles events across multiple years consistently', () => {
      const mlk2025 = getActiveEvents(date('2025-01-20'));
      const mlk2026 = getActiveEvents(date('2026-01-19'));

      expect(findEvent(mlk2025, 'mlk-day')).toBeDefined();
      expect(findEvent(mlk2026, 'mlk-day')).toBeDefined();
    });
  });

  describe('Leap Years', () => {
    it('handles February 29 in leap year (2024)', () => {
      const events = getActiveEvents(date('2024-02-29'));
      const blackHistory = findEvent(events, 'black-history-month');
      expect(blackHistory).toBeDefined();
    });

    it('handles February 28 in non-leap year (2025)', () => {
      const events = getActiveEvents(date('2025-02-28'));
      const blackHistory = findEvent(events, 'black-history-month');
      expect(blackHistory).toBeDefined();
    });
  });

  describe('Month Boundaries', () => {
    it('handles last day of month correctly', () => {
      const mar31 = getActiveEvents(date('2025-03-31'));
      const apr1 = getActiveEvents(date('2025-04-01'));

      expect(findEvent(mar31, 'womens-history-month')).toBeDefined();
      expect(findEvent(apr1, 'womens-history-month')).toBeUndefined();
    });

    it('handles first day of month correctly', () => {
      const feb28 = getActiveEvents(date('2025-02-28'));
      const mar1 = getActiveEvents(date('2025-03-01'));

      expect(findEvent(feb28, 'black-history-month')).toBeDefined();
      expect(findEvent(mar1, 'black-history-month')).toBeUndefined();
    });
  });

  describe('Date Range Boundaries', () => {
    it('includes start date of Hispanic Heritage Month', () => {
      const events = getActiveEvents(date('2025-09-15'));
      expect(findEvent(events, 'hispanic-heritage-month')).toBeDefined();
    });

    it('includes end date of Hispanic Heritage Month', () => {
      const events = getActiveEvents(date('2025-10-15'));
      expect(findEvent(events, 'hispanic-heritage-month')).toBeDefined();
    });

    it('excludes day before start date', () => {
      const events = getActiveEvents(date('2025-09-14'));
      expect(findEvent(events, 'hispanic-heritage-month')).toBeUndefined();
    });

    it('excludes day after end date', () => {
      const events = getActiveEvents(date('2025-10-16'));
      expect(findEvent(events, 'hispanic-heritage-month')).toBeUndefined();
    });
  });

  describe('No Active Events', () => {
    it('returns empty array when no events are active', () => {
      // Pick a random date with no events
      const events = getActiveEvents(date('2025-01-05'));
      expect(events).toEqual([]);
    });

    it('returns null for primary event when no events active', () => {
      const primary = getPrimaryEvent(date('2025-01-05'));
      expect(primary).toBeNull();
    });
  });
});

// ============================================================================
// Theme Application Tests
// ============================================================================

describe('Theme Application', () => {
  it('returns theme classes for Black History Month', () => {
    const classes = getEventThemeClasses(date('2025-02-15'));
    expect(classes).toContain('black-history-month');
  });

  it('returns theme classes for Pride Month', () => {
    const classes = getEventThemeClasses(date('2025-06-15'));
    expect(classes).toContain('pride-month');
  });

  it('returns theme classes for Women\'s History Month', () => {
    const classes = getEventThemeClasses(date('2025-03-15'));
    expect(classes).toContain('womens-history-month');
  });

  it('returns empty array when no themed events are active', () => {
    const classes = getEventThemeClasses(date('2025-01-05'));
    expect(classes).toEqual([]);
  });

  it('returns theme for specific day events that have themes', () => {
    // Check if any specific day events have themes and test them
    const womensDay = getActiveEvents(date('2025-03-08'));
    if (womensDay.length > 0 && womensDay[0].theme) {
      const classes = getEventThemeClasses(date('2025-03-08'));
      expect(classes.length).toBeGreaterThan(0);
    }
  });
});

// ============================================================================
// Data Validation Tests
// ============================================================================

describe('Event Data Validation', () => {
  it('all events have required fields', () => {
    const allDates = [
      '2025-01-20', '2025-02-15', '2025-03-08', '2025-03-15',
      '2025-04-22', '2025-05-15', '2025-06-15', '2025-06-19',
      '2025-09-15', '2025-09-21', '2025-10-10', '2025-10-13',
      '2025-11-15', '2025-12-10'
    ];

    allDates.forEach(dateStr => {
      const events = getActiveEvents(date(dateStr));
      events.forEach(event => {
        expect(event.id).toBeTruthy();
        expect(event.name).toBeTruthy();
        expect(event.category).toBeTruthy();
        expect(event.message).toBeTruthy();
        expect(event.emoji).toBeTruthy();
        expect(event.dates).toBeDefined();
        expect(event.dates.length).toBeGreaterThan(0);
      });
    });
  });

  it('all event categories are valid', () => {
    const validCategories = ['peace', 'diversity', 'culture', 'heritage', 'awareness'];
    const allDates = [
      '2025-01-20', '2025-02-15', '2025-03-08', '2025-04-22',
      '2025-05-15', '2025-06-15', '2025-06-19', '2025-09-21',
      '2025-10-10', '2025-11-15', '2025-12-10'
    ];

    allDates.forEach(dateStr => {
      const events = getActiveEvents(date(dateStr));
      events.forEach(event => {
        expect(validCategories).toContain(event.category);
      });
    });
  });

  it('all event links are valid URLs', () => {
    const allDates = [
      '2025-01-20', '2025-02-15', '2025-03-08', '2025-04-22',
      '2025-05-15', '2025-06-15', '2025-06-19', '2025-09-21',
      '2025-10-10', '2025-11-15', '2025-12-10'
    ];

    allDates.forEach(dateStr => {
      const events = getActiveEvents(date(dateStr));
      events.forEach(event => {
        if (event.link) {
          expect(event.link).toMatch(/^https?:\/\//);
        }
      });
    });
  });
});
