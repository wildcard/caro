/**
 * Hebrew Calendar Utilities
 *
 * Provides lookup tables for major Jewish holidays. Rather than implementing
 * full Hebrew calendar calculations (which are complex), we use pre-calculated
 * dates for the next several years.
 *
 * Data source: https://www.hebcal.com/
 */

/**
 * Pre-calculated Hebrew holiday dates by Gregorian year
 */
export const HEBREW_HOLIDAYS: Record<
  string,
  Record<number, { start: string; end: string }>
> = {
  // Rosh Hashanah (1-2 Tishrei)
  'rosh-hashanah': {
    2024: { start: '2024-10-02', end: '2024-10-04' },
    2025: { start: '2025-09-22', end: '2025-09-24' },
    2026: { start: '2026-09-11', end: '2026-09-13' },
    2027: { start: '2027-10-01', end: '2027-10-03' },
    2028: { start: '2028-09-20', end: '2028-09-22' },
    2029: { start: '2029-09-09', end: '2029-09-11' },
    2030: { start: '2030-09-27', end: '2030-09-29' },
  },

  // Yom Kippur (10 Tishrei)
  'yom-kippur': {
    2024: { start: '2024-10-11', end: '2024-10-12' },
    2025: { start: '2025-10-01', end: '2025-10-02' },
    2026: { start: '2026-09-20', end: '2026-09-21' },
    2027: { start: '2027-10-10', end: '2027-10-11' },
    2028: { start: '2028-09-29', end: '2028-09-30' },
    2029: { start: '2029-09-18', end: '2029-09-19' },
    2030: { start: '2030-10-06', end: '2030-10-07' },
  },

  // Sukkot (15-21 Tishrei)
  sukkot: {
    2024: { start: '2024-10-16', end: '2024-10-23' },
    2025: { start: '2025-10-06', end: '2025-10-13' },
    2026: { start: '2026-09-25', end: '2026-10-02' },
    2027: { start: '2027-10-15', end: '2027-10-22' },
    2028: { start: '2028-10-04', end: '2028-10-11' },
    2029: { start: '2029-09-23', end: '2029-09-30' },
    2030: { start: '2030-10-11', end: '2030-10-18' },
  },

  // Hanukkah (25 Kislev - 2/3 Tevet, 8 days)
  hanukkah: {
    2024: { start: '2024-12-25', end: '2025-01-02' },
    2025: { start: '2025-12-14', end: '2025-12-22' },
    2026: { start: '2026-12-04', end: '2026-12-12' },
    2027: { start: '2027-12-24', end: '2028-01-01' },
    2028: { start: '2028-12-12', end: '2028-12-20' },
    2029: { start: '2029-12-01', end: '2029-12-09' },
    2030: { start: '2030-12-20', end: '2030-12-28' },
  },

  // Purim (14 Adar)
  purim: {
    2024: { start: '2024-03-23', end: '2024-03-24' },
    2025: { start: '2025-03-13', end: '2025-03-14' },
    2026: { start: '2026-03-02', end: '2026-03-03' },
    2027: { start: '2027-03-22', end: '2027-03-23' },
    2028: { start: '2028-03-11', end: '2028-03-12' },
    2029: { start: '2029-02-28', end: '2029-03-01' },
    2030: { start: '2030-03-18', end: '2030-03-19' },
  },

  // Passover (15-22 Nisan)
  passover: {
    2024: { start: '2024-04-22', end: '2024-04-30' },
    2025: { start: '2025-04-12', end: '2025-04-20' },
    2026: { start: '2026-04-01', end: '2026-04-09' },
    2027: { start: '2027-04-21', end: '2027-04-29' },
    2028: { start: '2028-04-10', end: '2028-04-18' },
    2029: { start: '2029-03-30', end: '2029-04-07' },
    2030: { start: '2030-04-17', end: '2030-04-25' },
  },

  // Shavuot (6-7 Sivan)
  shavuot: {
    2024: { start: '2024-06-11', end: '2024-06-13' },
    2025: { start: '2025-06-01', end: '2025-06-03' },
    2026: { start: '2026-05-21', end: '2026-05-23' },
    2027: { start: '2027-06-10', end: '2027-06-12' },
    2028: { start: '2028-05-29', end: '2028-05-31' },
    2029: { start: '2029-05-18', end: '2029-05-20' },
    2030: { start: '2030-06-06', end: '2030-06-08' },
  },

  // Yom Ha'atzmaut (5 Iyar, Israeli Independence Day)
  'yom-haatzmaut': {
    2024: { start: '2024-05-13', end: '2024-05-14' },
    2025: { start: '2025-05-01', end: '2025-05-02' },
    2026: { start: '2026-04-21', end: '2026-04-22' },
    2027: { start: '2027-05-11', end: '2027-05-12' },
    2028: { start: '2028-05-01', end: '2028-05-02' },
    2029: { start: '2029-04-19', end: '2029-04-20' },
    2030: { start: '2030-05-08', end: '2030-05-09' },
  },
};

/**
 * Resolve a Hebrew holiday to Gregorian dates for a given year
 *
 * @param holidayId - The holiday identifier (e.g., "hanukkah", "passover")
 * @param year - The Gregorian year
 * @returns Date range or null if not found
 */
export function resolveHebrewDate(
  holidayId: string,
  year: number
): { start: Date; end: Date } | null {
  const holidayDates = HEBREW_HOLIDAYS[holidayId];

  if (!holidayDates) {
    console.warn(`Unknown Hebrew holiday: ${holidayId}`);
    return null;
  }

  const yearData = holidayDates[year];

  if (!yearData) {
    // Try to extrapolate or find nearest year
    const years = Object.keys(holidayDates).map(Number).sort();

    if (years.length === 0) {
      return null;
    }

    // Find the closest available year
    const closest = years.reduce((prev, curr) =>
      Math.abs(curr - year) < Math.abs(prev - year) ? curr : prev
    );

    // Only use if within 1 year
    if (Math.abs(closest - year) > 1) {
      console.warn(
        `No data for ${holidayId} in year ${year}, closest is ${closest}`
      );
      return null;
    }

    const fallbackData = holidayDates[closest];
    console.warn(
      `Using ${closest} data for ${holidayId} in year ${year} (approximation)`
    );

    return {
      start: new Date(fallbackData.start),
      end: new Date(fallbackData.end + 'T23:59:59'),
    };
  }

  return {
    start: new Date(yearData.start),
    end: new Date(yearData.end + 'T23:59:59'),
  };
}

/**
 * Get all Hebrew holidays for a given year
 */
export function getHebrewHolidaysForYear(
  year: number
): Record<string, { start: Date; end: Date } | null> {
  const result: Record<string, { start: Date; end: Date } | null> = {};

  for (const holidayId of Object.keys(HEBREW_HOLIDAYS)) {
    result[holidayId] = resolveHebrewDate(holidayId, year);
  }

  return result;
}
