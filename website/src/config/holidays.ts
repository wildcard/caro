/**
 * Holiday Configuration
 *
 * This file contains all holiday event definitions. To add a new holiday,
 * simply add a new entry to the HOLIDAYS array following the HolidayEvent
 * interface.
 *
 * Guidelines:
 * - Use 'global' visibility for holidays celebrated worldwide
 * - Use 'locale-only' for national/regional holidays
 * - Set defaultFor rules only when locale-specific defaults make sense
 * - Provide leadTimeDays for holidays that need advance theming
 */

import type { HolidayEvent } from './types';

// ============================================================================
// Holiday Definitions
// ============================================================================

export const HOLIDAYS: HolidayEvent[] = [
  // --------------------------------------------------------------------------
  // WINTER HOLIDAYS (December-January)
  // --------------------------------------------------------------------------

  {
    id: 'christmas',
    name: 'Christmas',
    localizedNames: {
      es: 'Navidad',
      fr: 'Noël',
      de: 'Weihnachten',
      pt: 'Natal',
      it: 'Natale',
    },
    description: 'Christian celebration of the birth of Jesus Christ',
    dateRange: {
      type: 'fixed',
      startMonth: 12,
      startDay: 1,
      endMonth: 12,
      endDay: 25,
    },
    leadTimeDays: 7,
    visibility: 'global',
    theme: {
      primaryColor: '#c41e3a',
      secondaryColor: '#228b22',
      accentColor: '#ffd700',
      bgColor: '#fff5f5',
      icon: '🎄',
      decorations: ['christmas'],
    },
    // Christmas is NOT auto-selected anywhere - users must opt-in
    // This respects cultural diversity
    category: 'religious',
    enabled: true,
  },

  {
    id: 'hanukkah',
    name: 'Hanukkah',
    localizedNames: {
      he: 'חנוכה',
      yi: 'חנוכּה',
    },
    description: 'Jewish Festival of Lights',
    dateRange: {
      type: 'calculated',
      // Hanukkah dates for upcoming years (25 Kislev, 8 days)
      dates: {
        2024: { start: '2024-12-25', end: '2025-01-02' },
        2025: { start: '2025-12-14', end: '2025-12-22' },
        2026: { start: '2026-12-04', end: '2026-12-12' },
        2027: { start: '2027-12-24', end: '2028-01-01' },
        2028: { start: '2028-12-12', end: '2028-12-20' },
        2029: { start: '2029-12-01', end: '2029-12-09' },
        2030: { start: '2030-12-20', end: '2030-12-28' },
      },
    },
    leadTimeDays: 3,
    visibility: 'global',
    theme: {
      primaryColor: '#0038b8',
      secondaryColor: '#4169e1',
      accentColor: '#ffd700',
      bgColor: '#f0f5ff',
      icon: '🕎',
      decorations: ['hanukkah'],
    },
    defaultFor: [
      // Default for users in Israel
      { type: 'locale', values: ['IL'], priority: 100 },
      // Also default for Hebrew language users
      { type: 'language', values: ['he', 'yi'], priority: 90 },
    ],
    category: 'religious',
    enabled: true,
  },

  {
    id: 'newyear',
    name: 'New Year',
    localizedNames: {
      es: 'Año Nuevo',
      fr: 'Nouvel An',
      de: 'Neujahr',
      pt: 'Ano Novo',
      ja: '正月',
      zh: '新年',
    },
    description: 'Celebration of the new calendar year',
    dateRange: {
      type: 'fixed',
      startMonth: 12,
      startDay: 31,
      endMonth: 1,
      endDay: 1,
    },
    visibility: 'global',
    theme: {
      primaryColor: '#ffd700',
      secondaryColor: '#c44569',
      accentColor: '#4ecdc4',
      bgColor: '#fffef0',
      icon: '🎊',
      decorations: ['newyear'],
      effects: ['fireworks'],
    },
    // New Year defaults for everyone on Dec 31 - Jan 1
    defaultFor: [{ type: 'locale', values: ['*'], priority: 50 }],
    category: 'cultural',
    enabled: true,
  },

  // --------------------------------------------------------------------------
  // SOUTH ASIAN HOLIDAYS
  // --------------------------------------------------------------------------

  {
    id: 'diwali',
    name: 'Diwali',
    localizedNames: {
      hi: 'दीवाली',
      ta: 'தீபாவளி',
      te: 'దీపావళి',
      gu: 'દિવાળી',
    },
    description: 'Hindu Festival of Lights',
    dateRange: {
      type: 'calculated',
      // Diwali dates (Amavasya of Kartik month)
      dates: {
        2024: { start: '2024-10-29', end: '2024-11-03' },
        2025: { start: '2025-10-18', end: '2025-10-23' },
        2026: { start: '2026-11-06', end: '2026-11-11' },
        2027: { start: '2027-10-26', end: '2027-10-31' },
        2028: { start: '2028-10-14', end: '2028-10-19' },
        2029: { start: '2029-11-02', end: '2029-11-07' },
        2030: { start: '2030-10-22', end: '2030-10-27' },
      },
    },
    leadTimeDays: 5,
    visibility: 'global',
    theme: {
      primaryColor: '#ff9933',
      secondaryColor: '#800080',
      accentColor: '#ffd700',
      bgColor: '#fff8f0',
      icon: '🪔',
      decorations: ['diwali'],
      effects: ['fireworks'],
    },
    defaultFor: [
      { type: 'locale', values: ['IN', 'NP', 'LK', 'MU', 'FJ'], priority: 100 },
      { type: 'language', values: ['hi', 'ta', 'te', 'gu', 'mr', 'bn'], priority: 90 },
    ],
    category: 'religious',
    enabled: true,
  },

  {
    id: 'holi',
    name: 'Holi',
    localizedNames: {
      hi: 'होली',
    },
    description: 'Hindu Festival of Colors',
    dateRange: {
      type: 'calculated',
      dates: {
        2025: { start: '2025-03-12', end: '2025-03-14' },
        2026: { start: '2026-03-02', end: '2026-03-04' },
        2027: { start: '2027-03-21', end: '2027-03-23' },
        2028: { start: '2028-03-10', end: '2028-03-12' },
        2029: { start: '2029-02-28', end: '2029-03-02' },
        2030: { start: '2030-03-18', end: '2030-03-20' },
      },
    },
    leadTimeDays: 3,
    visibility: 'global',
    theme: {
      primaryColor: '#ff1493',
      secondaryColor: '#00ff00',
      accentColor: '#ffff00',
      bgColor: '#fff0f5',
      icon: '🎨',
      decorations: ['holi'],
      effects: ['confetti'],
    },
    defaultFor: [
      { type: 'locale', values: ['IN', 'NP'], priority: 100 },
    ],
    category: 'religious',
    enabled: true,
  },

  // --------------------------------------------------------------------------
  // EAST ASIAN HOLIDAYS
  // --------------------------------------------------------------------------

  {
    id: 'lunar-new-year',
    name: 'Lunar New Year',
    localizedNames: {
      zh: '春节',
      ko: '설날',
      vi: 'Tết',
      ja: '旧正月',
    },
    description: 'Traditional East Asian New Year celebration',
    dateRange: {
      type: 'calculated',
      // Lunar New Year dates (1st day of 1st lunar month)
      dates: {
        2024: { start: '2024-02-04', end: '2024-02-18' }, // Year of Dragon
        2025: { start: '2025-01-25', end: '2025-02-08' }, // Year of Snake
        2026: { start: '2026-02-13', end: '2026-02-27' }, // Year of Horse
        2027: { start: '2027-02-02', end: '2027-02-16' }, // Year of Goat
        2028: { start: '2028-01-22', end: '2028-02-05' }, // Year of Monkey
        2029: { start: '2029-02-09', end: '2029-02-23' }, // Year of Rooster
        2030: { start: '2030-01-29', end: '2030-02-12' }, // Year of Dog
      },
    },
    leadTimeDays: 7,
    visibility: 'global',
    theme: {
      primaryColor: '#de2910',
      secondaryColor: '#ffde00',
      accentColor: '#ffffff',
      bgColor: '#fff5f5',
      icon: '🐉',
      decorations: ['lunar-new-year'],
      effects: ['fireworks'],
    },
    defaultFor: [
      { type: 'locale', values: ['CN', 'TW', 'HK', 'SG', 'MY', 'KR', 'VN'], priority: 100 },
      { type: 'language', values: ['zh', 'ko', 'vi'], priority: 90 },
    ],
    category: 'cultural',
    enabled: true,
  },

  {
    id: 'mid-autumn',
    name: 'Mid-Autumn Festival',
    localizedNames: {
      zh: '中秋节',
      ko: '추석',
      vi: 'Tết Trung Thu',
    },
    description: 'East Asian harvest moon festival',
    dateRange: {
      type: 'calculated',
      dates: {
        2024: { start: '2024-09-15', end: '2024-09-18' },
        2025: { start: '2025-10-04', end: '2025-10-07' },
        2026: { start: '2026-09-23', end: '2026-09-26' },
        2027: { start: '2027-09-12', end: '2027-09-15' },
        2028: { start: '2028-09-30', end: '2028-10-03' },
        2029: { start: '2029-09-20', end: '2029-09-23' },
        2030: { start: '2030-09-10', end: '2030-09-13' },
      },
    },
    leadTimeDays: 3,
    visibility: 'global',
    theme: {
      primaryColor: '#ffa500',
      secondaryColor: '#8b4513',
      accentColor: '#fffacd',
      bgColor: '#fff8f0',
      icon: '🥮',
      decorations: ['mid-autumn'],
    },
    defaultFor: [
      { type: 'locale', values: ['CN', 'TW', 'HK', 'SG', 'MY', 'KR', 'VN'], priority: 100 },
    ],
    category: 'cultural',
    enabled: true,
  },

  // --------------------------------------------------------------------------
  // ISLAMIC HOLIDAYS
  // --------------------------------------------------------------------------

  {
    id: 'eid-al-fitr',
    name: 'Eid al-Fitr',
    localizedNames: {
      ar: 'عيد الفطر',
      id: 'Idul Fitri',
      ms: 'Hari Raya Aidilfitri',
      tr: 'Ramazan Bayramı',
    },
    description: 'Islamic celebration marking end of Ramadan',
    dateRange: {
      type: 'calculated',
      dates: {
        2024: { start: '2024-04-09', end: '2024-04-12' },
        2025: { start: '2025-03-29', end: '2025-04-01' },
        2026: { start: '2026-03-19', end: '2026-03-22' },
        2027: { start: '2027-03-08', end: '2027-03-11' },
        2028: { start: '2028-02-25', end: '2028-02-28' },
        2029: { start: '2029-02-13', end: '2029-02-16' },
        2030: { start: '2030-02-03', end: '2030-02-06' },
      },
    },
    leadTimeDays: 3,
    visibility: 'global',
    theme: {
      primaryColor: '#009639',
      secondaryColor: '#ffd700',
      accentColor: '#ffffff',
      bgColor: '#f0fff0',
      icon: '🌙',
      decorations: ['eid'],
    },
    defaultFor: [
      {
        type: 'locale',
        values: ['SA', 'AE', 'EG', 'ID', 'MY', 'PK', 'BD', 'TR', 'IR'],
        priority: 100,
      },
      { type: 'language', values: ['ar', 'id', 'ms', 'tr', 'fa', 'ur'], priority: 90 },
    ],
    category: 'religious',
    enabled: true,
  },

  {
    id: 'eid-al-adha',
    name: 'Eid al-Adha',
    localizedNames: {
      ar: 'عيد الأضحى',
      id: 'Idul Adha',
      tr: 'Kurban Bayramı',
    },
    description: 'Islamic Festival of Sacrifice',
    dateRange: {
      type: 'calculated',
      dates: {
        2024: { start: '2024-06-16', end: '2024-06-19' },
        2025: { start: '2025-06-06', end: '2025-06-09' },
        2026: { start: '2026-05-26', end: '2026-05-29' },
        2027: { start: '2027-05-16', end: '2027-05-19' },
        2028: { start: '2028-05-04', end: '2028-05-07' },
        2029: { start: '2029-04-23', end: '2029-04-26' },
        2030: { start: '2030-04-12', end: '2030-04-15' },
      },
    },
    leadTimeDays: 3,
    visibility: 'global',
    theme: {
      primaryColor: '#009639',
      secondaryColor: '#ffd700',
      accentColor: '#ffffff',
      bgColor: '#f0fff0',
      icon: '🐑',
      decorations: ['eid'],
    },
    defaultFor: [
      {
        type: 'locale',
        values: ['SA', 'AE', 'EG', 'ID', 'MY', 'PK', 'BD', 'TR', 'IR'],
        priority: 100,
      },
    ],
    category: 'religious',
    enabled: true,
  },

  // --------------------------------------------------------------------------
  // NATIONAL HOLIDAYS (Locale-Only)
  // --------------------------------------------------------------------------

  {
    id: 'july-4th',
    name: 'Independence Day',
    description: 'United States Independence Day',
    dateRange: {
      type: 'fixed',
      startMonth: 7,
      startDay: 1,
      endMonth: 7,
      endDay: 4,
    },
    visibility: 'locale-only',
    locales: ['US'],
    theme: {
      primaryColor: '#b22234',
      secondaryColor: '#3c3b6e',
      accentColor: '#ffffff',
      bgColor: '#f8f9fa',
      icon: '🇺🇸',
      decorations: ['usa'],
      effects: ['fireworks'],
    },
    defaultFor: [{ type: 'locale', values: ['US'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'canada-day',
    name: 'Canada Day',
    localizedNames: {
      fr: 'Fête du Canada',
    },
    description: 'Canadian national day',
    dateRange: {
      type: 'fixed',
      startMonth: 6,
      startDay: 28,
      endMonth: 7,
      endDay: 1,
    },
    visibility: 'locale-only',
    locales: ['CA'],
    theme: {
      primaryColor: '#ff0000',
      secondaryColor: '#ffffff',
      icon: '🇨🇦',
      decorations: ['canada'],
    },
    defaultFor: [{ type: 'locale', values: ['CA'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'bastille-day',
    name: 'Bastille Day',
    localizedNames: {
      fr: 'Fête Nationale',
    },
    description: 'French National Day',
    dateRange: {
      type: 'fixed',
      startMonth: 7,
      startDay: 10,
      endMonth: 7,
      endDay: 14,
    },
    visibility: 'locale-only',
    locales: ['FR'],
    theme: {
      primaryColor: '#0055a4',
      secondaryColor: '#ef4135',
      accentColor: '#ffffff',
      icon: '🇫🇷',
      decorations: ['france'],
      effects: ['fireworks'],
    },
    defaultFor: [{ type: 'locale', values: ['FR'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'german-unity',
    name: 'German Unity Day',
    localizedNames: {
      de: 'Tag der Deutschen Einheit',
    },
    description: 'German national day celebrating reunification',
    dateRange: {
      type: 'fixed',
      startMonth: 10,
      startDay: 1,
      endMonth: 10,
      endDay: 3,
    },
    visibility: 'locale-only',
    locales: ['DE'],
    theme: {
      primaryColor: '#000000',
      secondaryColor: '#dd0000',
      accentColor: '#ffcc00',
      icon: '🇩🇪',
      decorations: ['germany'],
    },
    defaultFor: [{ type: 'locale', values: ['DE'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'yom-haatzmaut',
    name: "Yom Ha'atzmaut",
    localizedNames: {
      he: 'יום העצמאות',
    },
    description: 'Israeli Independence Day',
    dateRange: {
      type: 'calculated',
      // 5 Iyar in Hebrew calendar
      dates: {
        2024: { start: '2024-05-13', end: '2024-05-14' },
        2025: { start: '2025-05-01', end: '2025-05-02' },
        2026: { start: '2026-04-21', end: '2026-04-22' },
        2027: { start: '2027-05-11', end: '2027-05-12' },
        2028: { start: '2028-05-01', end: '2028-05-02' },
        2029: { start: '2029-04-19', end: '2029-04-20' },
        2030: { start: '2030-05-08', end: '2030-05-09' },
      },
    },
    visibility: 'locale-only',
    locales: ['IL'],
    theme: {
      primaryColor: '#0038b8',
      secondaryColor: '#ffffff',
      icon: '🇮🇱',
      decorations: ['israel'],
    },
    defaultFor: [{ type: 'locale', values: ['IL'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'uae-national-day',
    name: 'UAE National Day',
    localizedNames: {
      ar: 'اليوم الوطني',
    },
    description: 'United Arab Emirates National Day',
    dateRange: {
      type: 'fixed',
      startMonth: 12,
      startDay: 1,
      endMonth: 12,
      endDay: 3,
    },
    visibility: 'locale-only',
    locales: ['AE'],
    theme: {
      primaryColor: '#009639',
      secondaryColor: '#ffffff',
      accentColor: '#ff0000',
      icon: '🇦🇪',
      decorations: ['uae'],
    },
    defaultFor: [{ type: 'locale', values: ['AE'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'indonesia-independence',
    name: 'Indonesian Independence Day',
    localizedNames: {
      id: 'Hari Kemerdekaan',
    },
    description: 'Indonesian Independence Day',
    dateRange: {
      type: 'fixed',
      startMonth: 8,
      startDay: 14,
      endMonth: 8,
      endDay: 17,
    },
    visibility: 'locale-only',
    locales: ['ID'],
    theme: {
      primaryColor: '#ff0000',
      secondaryColor: '#ffffff',
      icon: '🇮🇩',
      decorations: ['indonesia'],
    },
    defaultFor: [{ type: 'locale', values: ['ID'], priority: 100 }],
    category: 'national',
    enabled: true,
  },

  {
    id: 'philippines-independence',
    name: 'Philippine Independence Day',
    localizedNames: {
      tl: 'Araw ng Kalayaan',
    },
    description: 'Philippine Independence Day',
    dateRange: {
      type: 'fixed',
      startMonth: 6,
      startDay: 10,
      endMonth: 6,
      endDay: 12,
    },
    visibility: 'locale-only',
    locales: ['PH'],
    theme: {
      primaryColor: '#0038a8',
      secondaryColor: '#ce1126',
      accentColor: '#fcd116',
      icon: '🇵🇭',
      decorations: ['philippines'],
    },
    defaultFor: [{ type: 'locale', values: ['PH'], priority: 100 }],
    category: 'national',
    enabled: true,
  },
];

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get a holiday by ID
 */
export function getHolidayById(id: string): HolidayEvent | undefined {
  return HOLIDAYS.find((h) => h.id === id);
}

/**
 * Get all enabled holidays
 */
export function getEnabledHolidays(): HolidayEvent[] {
  return HOLIDAYS.filter((h) => h.enabled !== false);
}

/**
 * Get holidays by category
 */
export function getHolidaysByCategory(
  category: HolidayEvent['category']
): HolidayEvent[] {
  return HOLIDAYS.filter((h) => h.category === category && h.enabled !== false);
}

/**
 * Get holidays for a specific locale
 */
export function getHolidaysForLocale(localeCode: string): HolidayEvent[] {
  return HOLIDAYS.filter((h) => {
    if (h.enabled === false) return false;
    if (h.visibility === 'global') return true;
    return h.locales?.includes(localeCode);
  });
}
