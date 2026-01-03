/**
 * Humanity Events Calendar
 *
 * Caro celebrates humanity, diversity, peace, and cultural heritage.
 * This calendar powers automatic website transformations during important
 * events that have a positive impact on humanity.
 *
 * NOTE: Caro does not participate in political events. These are celebrations
 * of human unity, cultural heritage, and causes that bring people together.
 */

// ============================================================================
// Types
// ============================================================================

/**
 * Event categories reflect Caro's values:
 * - peace: Anti-war, pro-peace celebrations
 * - diversity: LGBTQ+, racial equality, gender equality
 * - culture: Cultural celebrations and festivals
 * - heritage: Indigenous peoples, historical recognition
 * - awareness: Mental health, humanitarian, environmental causes
 */
export type EventCategory =
  | 'peace'
  | 'diversity'
  | 'culture'
  | 'heritage'
  | 'awareness';

/**
 * Date specification for an event.
 * Events can be:
 * - A specific date (month + day)
 * - An entire month (month only)
 * - A date range within a month (month + dayStart + dayEnd)
 * - A calculated date (using `calculate` function name)
 */
export interface EventDate {
  /** Month (1-12) */
  month: number;
  /** Specific day of month (1-31) */
  day?: number;
  /** Start day for a range */
  dayStart?: number;
  /** End day for a range */
  dayEnd?: number;
  /**
   * Named calculation for floating dates.
   * Supported: 'mlk-day' (3rd Monday of January)
   */
  calculate?: string;
}

/**
 * Optional theme customization for an event.
 * If provided, these CSS variables will be applied during the event.
 */
export interface EventTheme {
  /** Primary accent color */
  accent: string;
  /** Secondary accent color */
  accentSecondary?: string;
  /** Tertiary accent color */
  accentTertiary?: string;
  /** Background tint color */
  bgTint?: string;
  /** CSS class to add to html element */
  className?: string;
}

/**
 * A humanity event that Caro celebrates.
 */
export interface HumanityEvent {
  /** Unique identifier (kebab-case) */
  id: string;
  /** Display name */
  name: string;
  /** Event category */
  category: EventCategory;
  /** When the event occurs */
  dates: EventDate[];
  /** Banner message shown during the event */
  message: string;
  /** Emoji representing the event */
  emoji: string;
  /** Hero decorations (left and right emojis) */
  decorations?: {
    left: string;
    right: string;
  };
  /** Optional link to learn more */
  link?: string;
  /** Optional theme customization */
  theme?: EventTheme;
  /** Whether this event applies a full theme (like holidays) */
  hasFullTheme?: boolean;
}

// ============================================================================
// Event Definitions
// ============================================================================

/**
 * Caro's Humanity Events Calendar
 *
 * Guidelines for adding events:
 * 1. Events should celebrate humanity, diversity, peace, or cultural heritage
 * 2. Events should NOT be political in nature
 * 3. Each event needs a clear, positive message
 * 4. Theme colors should be appropriate and respectful
 */
export const HUMANITY_EVENTS: HumanityEvent[] = [
  // =========================================================================
  // JANUARY
  // =========================================================================
  {
    id: 'mlk-day',
    name: 'Martin Luther King Jr. Day',
    category: 'heritage',
    dates: [{ month: 1, calculate: 'mlk-day' }],
    message: 'Honoring Dr. King\'s dream of equality and justice for all',
    emoji: '✊',
    decorations: { left: '✊', right: '🕊️' },
    link: 'https://thekingcenter.org/',
    theme: {
      accent: '#1a365d',
      accentSecondary: '#2c5282',
      bgTint: '#ebf8ff',
      className: 'mlk-day',
    },
  },

  // =========================================================================
  // FEBRUARY
  // =========================================================================
  {
    id: 'black-history-month',
    name: 'Black History Month',
    category: 'heritage',
    dates: [{ month: 2 }],
    message: 'Celebrating Black history, culture, and achievements',
    emoji: '✊🏿',
    decorations: { left: '✊🏿', right: '🌍' },
    link: 'https://www.blackhistorymonth.gov/',
    theme: {
      // Pan-African colors: Red, Black, Green + Gold
      accent: '#c41e3a',
      accentSecondary: '#228b22',
      accentTertiary: '#ffd700',
      bgTint: '#fffaf0',
      className: 'black-history-month',
    },
    hasFullTheme: true,
  },

  // =========================================================================
  // MARCH
  // =========================================================================
  {
    id: 'international-womens-day',
    name: 'International Women\'s Day',
    category: 'diversity',
    dates: [{ month: 3, day: 8 }],
    message: 'Celebrating women\'s achievements and advocating for equality',
    emoji: '♀️',
    decorations: { left: '💜', right: '♀️' },
    link: 'https://www.internationalwomensday.com/',
    theme: {
      // IWD official color: Purple
      accent: '#6b21a8',
      accentSecondary: '#7c3aed',
      bgTint: '#faf5ff',
      className: 'womens-day',
    },
  },
  {
    id: 'womens-history-month',
    name: 'Women\'s History Month',
    category: 'diversity',
    dates: [{ month: 3 }],
    message: 'Honoring women who shaped our world',
    emoji: '👩',
    decorations: { left: '💜', right: '🌸' },
    link: 'https://www.womenshistorymonth.gov/',
    theme: {
      // Purple and magenta
      accent: '#7c3aed',
      accentSecondary: '#c026d3',
      bgTint: '#fdf4ff',
      className: 'womens-history-month',
    },
    hasFullTheme: true,
  },

  // =========================================================================
  // APRIL
  // =========================================================================
  {
    id: 'earth-day',
    name: 'Earth Day',
    category: 'awareness',
    dates: [{ month: 4, day: 22 }],
    message: 'Celebrating our planet and commitment to protecting it',
    emoji: '🌍',
    decorations: { left: '🌱', right: '🌍' },
    link: 'https://www.earthday.org/',
    theme: {
      // Earth Day green and blue
      accent: '#16a34a',
      accentSecondary: '#0ea5e9',
      bgTint: '#f0fdf4',
      className: 'earth-day',
    },
  },

  // =========================================================================
  // MAY
  // =========================================================================
  {
    id: 'asian-pacific-heritage-month',
    name: 'Asian Pacific American Heritage Month',
    category: 'heritage',
    dates: [{ month: 5 }],
    message: 'Celebrating Asian and Pacific Islander heritage and contributions',
    emoji: '🌸',
    decorations: { left: '🌸', right: '🏯' },
    link: 'https://asianpacificheritage.gov/',
    theme: {
      // Cherry blossom pink with red and gold
      accent: '#db2777',
      accentSecondary: '#dc2626',
      accentTertiary: '#eab308',
      bgTint: '#fdf2f8',
      className: 'aapi-heritage-month',
    },
    hasFullTheme: true,
  },
  {
    id: 'mental-health-awareness-month',
    name: 'Mental Health Awareness Month',
    category: 'awareness',
    dates: [{ month: 5 }],
    message: 'Breaking the stigma and supporting mental wellness',
    emoji: '💚',
    decorations: { left: '💚', right: '🧠' },
    link: 'https://www.mhanational.org/mental-health-month',
    theme: {
      // Mental Health Awareness green
      accent: '#22c55e',
      accentSecondary: '#16a34a',
      bgTint: '#f0fdf4',
    },
  },

  // =========================================================================
  // JUNE
  // =========================================================================
  {
    id: 'pride-month',
    name: 'Pride Month',
    category: 'diversity',
    dates: [{ month: 6 }],
    message: 'Celebrating LGBTQ+ pride, diversity, and love',
    emoji: '🏳️‍🌈',
    decorations: { left: '🏳️‍🌈', right: '❤️' },
    link: 'https://www.loc.gov/lgbt-pride-month/',
    theme: {
      // Rainbow Pride flag colors (using key colors)
      accent: '#e40303',
      accentSecondary: '#ff8c00',
      accentTertiary: '#008026',
      bgTint: '#fefce8',
      className: 'pride-month',
    },
    hasFullTheme: true,
  },
  {
    id: 'juneteenth',
    name: 'Juneteenth',
    category: 'heritage',
    dates: [{ month: 6, day: 19 }],
    message: 'Commemorating the end of slavery and celebrating freedom',
    emoji: '✊🏿',
    decorations: { left: '✊🏿', right: '🎉' },
    link: 'https://www.juneteenth.gov/',
    theme: {
      // Juneteenth flag colors: Red, White, Blue with star burst
      accent: '#bf0a30',
      accentSecondary: '#002868',
      bgTint: '#fef2f2',
      className: 'juneteenth',
    },
  },

  // =========================================================================
  // SEPTEMBER
  // =========================================================================
  {
    id: 'hispanic-heritage-month',
    name: 'Hispanic Heritage Month',
    category: 'heritage',
    dates: [
      { month: 9, dayStart: 15, dayEnd: 30 },
      { month: 10, dayStart: 1, dayEnd: 15 },
    ],
    message: 'Celebrating Hispanic and Latino heritage and culture',
    emoji: '🎺',
    decorations: { left: '💃', right: '🎺' },
    link: 'https://www.hispanicheritagemonth.gov/',
    theme: {
      // Warm colors representing Latin American flags
      accent: '#dc2626',
      accentSecondary: '#ea580c',
      accentTertiary: '#16a34a',
      bgTint: '#fef2f2',
      className: 'hispanic-heritage-month',
    },
    hasFullTheme: true,
  },
  {
    id: 'international-day-of-peace',
    name: 'International Day of Peace',
    category: 'peace',
    dates: [{ month: 9, day: 21 }],
    message: 'Celebrating peace, unity, and non-violence worldwide',
    emoji: '🕊️',
    decorations: { left: '🕊️', right: '☮️' },
    link: 'https://www.un.org/en/observances/international-day-peace',
    theme: {
      // UN blue
      accent: '#009edb',
      accentSecondary: '#0077b6',
      bgTint: '#ecfeff',
      className: 'peace-day',
    },
  },

  // =========================================================================
  // OCTOBER
  // =========================================================================
  {
    id: 'world-mental-health-day',
    name: 'World Mental Health Day',
    category: 'awareness',
    dates: [{ month: 10, day: 10 }],
    message: 'Raising awareness for mental health and wellbeing',
    emoji: '💚',
    decorations: { left: '💚', right: '🌻' },
    link: 'https://www.who.int/campaigns/world-mental-health-day',
    theme: {
      // WHO green
      accent: '#22c55e',
      accentSecondary: '#15803d',
      bgTint: '#f0fdf4',
      className: 'mental-health-day',
    },
  },
  {
    id: 'indigenous-peoples-day',
    name: 'Indigenous Peoples\' Day',
    category: 'heritage',
    dates: [{ month: 10, calculate: 'indigenous-peoples-day' }],
    message: 'Honoring Indigenous peoples and their enduring cultures',
    emoji: '🪶',
    decorations: { left: '🪶', right: '🦅' },
    link: 'https://americanindian.si.edu/',
    theme: {
      // Earth tones with turquoise
      accent: '#b45309',
      accentSecondary: '#0d9488',
      accentTertiary: '#15803d',
      bgTint: '#fffbeb',
      className: 'indigenous-peoples-day',
    },
  },

  // =========================================================================
  // NOVEMBER
  // =========================================================================
  {
    id: 'native-american-heritage-month',
    name: 'Native American Heritage Month',
    category: 'heritage',
    dates: [{ month: 11 }],
    message: 'Celebrating Native American cultures and contributions',
    emoji: '🪶',
    decorations: { left: '🪶', right: '🦬' },
    link: 'https://nativeamericanheritagemonth.gov/',
    theme: {
      // Earth tones: amber, turquoise, terracotta
      accent: '#d97706',
      accentSecondary: '#0d9488',
      accentTertiary: '#b45309',
      bgTint: '#fffbeb',
      className: 'native-american-heritage-month',
    },
    hasFullTheme: true,
  },

  // =========================================================================
  // DECEMBER
  // =========================================================================
  {
    id: 'international-human-rights-day',
    name: 'Human Rights Day',
    category: 'diversity',
    dates: [{ month: 12, day: 10 }],
    message: 'Celebrating universal human rights for all people',
    emoji: '🌐',
    decorations: { left: '🌐', right: '🤝' },
    link: 'https://www.un.org/en/observances/human-rights-day',
    theme: {
      // UN blue
      accent: '#009edb',
      accentSecondary: '#0077b6',
      bgTint: '#ecfeff',
      className: 'human-rights-day',
    },
  },
];

// ============================================================================
// Category Metadata
// ============================================================================

export interface CategoryMeta {
  name: string;
  description: string;
  defaultColors: {
    primary: string;
    secondary: string;
    bgTint: string;
  };
}

export const CATEGORY_META: Record<EventCategory, CategoryMeta> = {
  peace: {
    name: 'Peace',
    description: 'Anti-war and pro-peace celebrations',
    defaultColors: {
      primary: '#3b82f6',
      secondary: '#60a5fa',
      bgTint: '#eff6ff',
    },
  },
  diversity: {
    name: 'Diversity',
    description: 'Celebrating all people equally',
    defaultColors: {
      primary: '#9b59b6',
      secondary: '#e91e8c',
      bgTint: '#faf5ff',
    },
  },
  culture: {
    name: 'Culture',
    description: 'Cultural celebrations and festivals',
    defaultColors: {
      primary: '#f59e0b',
      secondary: '#d97706',
      bgTint: '#fffbeb',
    },
  },
  heritage: {
    name: 'Heritage',
    description: 'Historical and cultural heritage',
    defaultColors: {
      primary: '#b45309',
      secondary: '#92400e',
      bgTint: '#fffbeb',
    },
  },
  awareness: {
    name: 'Awareness',
    description: 'Mental health and humanitarian causes',
    defaultColors: {
      primary: '#10b981',
      secondary: '#059669',
      bgTint: '#ecfdf5',
    },
  },
};
