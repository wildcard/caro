/**
 * Occasion presentation registry — the single source of truth for how every
 * holiday, national day and humanity event LOOKS on caro.sh.
 *
 * WHEN an occasion is active comes from `holidays.ts` (opt-in holidays) and
 * `data/humanity-events.ts` (calendar events). This file only says what the
 * site wears while it is active. The `id` is the class set on <html>.
 *
 * Design rules (WCAG 2.2 AA + the Caro brand book):
 * - `fill` paints CTAs and the event banner as ONE solid color. White labels
 *   must reach 4.5:1 on it — `occasions.test.ts` fails the build otherwise.
 *   No gradients on CTAs, ever (brand rule, no exceptions for occasions).
 * - Links and body copy keep the brand palette; an occasion is a costume,
 *   not a rebrand. Only the CTA fill, the top ribbon, the hero art and the
 *   banner change.
 * - `colors` and `ribbon` are decorative only. They never carry text, so
 *   they may use the occasion's traditional colors unchanged.
 * - Motifs stay simple and respectful: lights, lanterns, a crescent with
 *   four-point sparkles. No scripture, crests or sacred symbols. Where no
 *   motif can represent a group without flattening it (Indigenous peoples'
 *   occasions), the occasion uses `none`: ribbon, fill and banner only.
 * - Reviewed by the cultural-heritage-expert agent (2026-10-10).
 */

export type Motif =
  | 'snowflakes'
  | 'candles'
  | 'diyas'
  | 'fireworks'
  | 'powder'
  | 'lanterns'
  | 'moon-lanterns'
  | 'crescent'
  | 'leaves'
  | 'blossoms'
  | 'rainbow'
  | 'sunburst'
  | 'stars'
  | 'bunting'
  | 'orbits'
  | 'hearts'
  | 'waves'
  | 'none';

export interface Occasion {
  /** CSS class on <html>; must match the class the runtime applies. */
  id: string;
  name: string;
  kind: 'holiday' | 'national' | 'humanity';
  /** Ids in holidays.ts or humanity-events.ts that map to this look. */
  sources: string[];
  /** Solid CTA and banner fill. White text on it must be ≥ 4.5:1. */
  fill: string;
  /** Art style for the hero motif. */
  motif: Motif;
  /** Decorative motif colors (never used behind or as text). */
  colors: string[];
  /** Hard-stop stripes for the 4px ribbon above the page. */
  ribbon: string[];
  /** A date (YYYY-MM-DD) inside the occasion, for previews and tests. */
  sampleDate: string;
}

export const OCCASIONS: Occasion[] = [
  // ---------------------------------------------------------------- holidays
  {
    id: 'christmas', name: 'Christmas', kind: 'holiday', sources: ['christmas'],
    fill: '#b3202f', motif: 'snowflakes',
    colors: ['#c41e3a', '#1e7b34', '#c9a227', '#8fb3c9'],
    ribbon: ['#c41e3a', '#1e7b34'], sampleDate: '2026-12-20',
  },
  {
    id: 'hanukkah', name: 'Hanukkah', kind: 'holiday', sources: ['hanukkah'],
    fill: '#1e40af', motif: 'candles',
    colors: ['#1e40af', '#4169e1', '#e0a800', '#9aa5b1'],
    ribbon: ['#1e40af', '#9aa5b1'], sampleDate: '2026-12-06',
  },
  {
    id: 'newyear', name: 'New Year', kind: 'holiday', sources: ['newyear'],
    fill: '#5b2a86', motif: 'fireworks',
    colors: ['#d4a017', '#c44569', '#2a9d8f', '#7c3aed'],
    ribbon: ['#d4a017', '#c44569', '#2a9d8f'], sampleDate: '2026-12-31',
  },
  {
    id: 'diwali', name: 'Diwali', kind: 'holiday', sources: ['diwali'],
    fill: '#7e22ce', motif: 'diyas',
    colors: ['#f59e0b', '#fbbf24', '#b45309', '#db2777', '#7e22ce'],
    ribbon: ['#f59e0b', '#db2777', '#7e22ce'], sampleDate: '2026-11-08',
  },
  {
    id: 'holi', name: 'Holi', kind: 'holiday', sources: ['holi'],
    fill: '#be185d', motif: 'powder',
    colors: ['#ec4899', '#22c55e', '#facc15', '#3b82f6', '#a855f7', '#f97316'],
    ribbon: ['#ec4899', '#facc15', '#22c55e', '#3b82f6', '#a855f7'], sampleDate: '2027-03-22',
  },
  {
    id: 'lunar-new-year', name: 'Lunar New Year', kind: 'holiday', sources: ['lunar-new-year'],
    fill: '#b91c1c', motif: 'lanterns',
    colors: ['#dc2626', '#eab308', '#b91c1c', '#f59e0b'],
    ribbon: ['#dc2626', '#eab308'], sampleDate: '2027-02-06',
  },
  {
    id: 'mid-autumn', name: 'Mid-Autumn Festival', kind: 'holiday', sources: ['mid-autumn'],
    fill: '#9a3412', motif: 'moon-lanterns',
    colors: ['#f5c542', '#ea580c', '#b45309', '#fde68a'],
    ribbon: ['#ea580c', '#f5c542'], sampleDate: '2027-09-14',
  },
  {
    id: 'eid', name: 'Eid', kind: 'holiday', sources: ['eid-al-fitr', 'eid-al-adha'],
    fill: '#047857', motif: 'crescent',
    colors: ['#d4a017', '#047857', '#0f766e', '#e9c46a'],
    ribbon: ['#047857', '#d4a017'], sampleDate: '2027-03-09',
  },

  // ----------------------------------------------------------- national days
  {
    id: 'july-4th', name: 'Independence Day (US)', kind: 'national', sources: ['july-4th'],
    fill: '#b22234', motif: 'fireworks',
    colors: ['#b22234', '#3c3b6e', '#c9a227'],
    ribbon: ['#b22234', '#3c3b6e'], sampleDate: '2027-07-04',
  },
  {
    id: 'canada-day', name: 'Canada Day', kind: 'national', sources: ['canada-day'],
    fill: '#c8102e', motif: 'leaves',
    colors: ['#d80621', '#a50d1f', '#e85d6a'],
    ribbon: ['#d80621', '#f4f1df', '#d80621'], sampleDate: '2027-07-01',
  },
  {
    id: 'bastille-day', name: 'Bastille Day', kind: 'national', sources: ['bastille-day'],
    fill: '#0055a4', motif: 'fireworks',
    colors: ['#0055a4', '#ef4135', '#9aa5b1'],
    ribbon: ['#0055a4', '#f4f1df', '#ef4135'], sampleDate: '2027-07-14',
  },
  {
    id: 'german-unity', name: 'German Unity Day', kind: 'national', sources: ['german-unity'],
    fill: '#c00000', motif: 'fireworks',
    colors: ['#4f4f4f', '#dd0000', '#ffcc00'],
    ribbon: ['#2b2b2b', '#dd0000', '#ffcc00'], sampleDate: '2027-10-03',
  },
  {
    id: 'yom-haatzmaut', name: "Yom Ha'atzmaut", kind: 'national', sources: ['yom-haatzmaut'],
    fill: '#0038b8', motif: 'fireworks',
    colors: ['#0038b8', '#4169e1', '#9aa5b1'],
    ribbon: ['#0038b8', '#f4f1df', '#0038b8'], sampleDate: '2027-05-12',
  },
  {
    id: 'uae-national-day', name: 'UAE National Day', kind: 'national', sources: ['uae-national-day'],
    fill: '#00732f', motif: 'fireworks',
    colors: ['#00732f', '#ff0000', '#4f4f4f'],
    ribbon: ['#ff0000', '#00732f', '#f4f1df', '#2b2b2b'], sampleDate: '2026-12-02',
  },
  {
    id: 'indonesia-independence', name: 'Indonesian Independence Day', kind: 'national',
    sources: ['indonesia-independence'],
    fill: '#c8102e', motif: 'fireworks',
    colors: ['#ce1126', '#e85d6a', '#9aa5b1'],
    ribbon: ['#ce1126', '#f4f1df'], sampleDate: '2027-08-17',
  },
  {
    id: 'philippines-independence', name: 'Philippine Independence Day', kind: 'national',
    sources: ['philippines-independence'],
    fill: '#0038a8', motif: 'sunburst',
    colors: ['#fcd116', '#0038a8', '#ce1126'],
    ribbon: ['#0038a8', '#ce1126'], sampleDate: '2027-06-12',
  },

  // ---------------------------------------------------------- humanity events
  {
    id: 'mlk-day', name: 'Martin Luther King Jr. Day', kind: 'humanity', sources: ['mlk-day'],
    fill: '#1a365d', motif: 'stars',
    colors: ['#1a365d', '#2c5282', '#c9a227'],
    ribbon: ['#1a365d', '#c9a227'], sampleDate: '2027-01-18',
  },
  {
    id: 'black-history-month', name: 'Black History Month', kind: 'humanity',
    sources: ['black-history-month'],
    fill: '#1f6b2c', motif: 'sunburst',
    colors: ['#d4a017', '#228b22', '#c41e3a'],
    ribbon: ['#c41e3a', '#2b2b2b', '#228b22'], sampleDate: '2027-02-15',
  },
  {
    id: 'womens-day', name: "International Women's Day", kind: 'humanity',
    sources: ['international-womens-day'],
    fill: '#6b21a8', motif: 'blossoms',
    colors: ['#7c3aed', '#a855f7', '#16a34a'],
    ribbon: ['#6b21a8', '#16a34a'], sampleDate: '2027-03-08',
  },
  {
    id: 'womens-history-month', name: "Women's History Month", kind: 'humanity',
    sources: ['womens-history-month'],
    fill: '#6d28d9', motif: 'blossoms',
    colors: ['#7c3aed', '#c026d3', '#f0abfc'],
    ribbon: ['#7c3aed', '#c026d3'], sampleDate: '2027-03-20',
  },
  {
    id: 'earth-day', name: 'Earth Day', kind: 'humanity', sources: ['earth-day'],
    fill: '#15803d', motif: 'leaves',
    colors: ['#16a34a', '#0ea5e9', '#65a30d'],
    ribbon: ['#16a34a', '#0ea5e9'], sampleDate: '2027-04-22',
  },
  {
    id: 'aapi-heritage-month',
    name: 'Asian American, Native Hawaiian and Pacific Islander Heritage Month', kind: 'humanity',
    sources: ['asian-pacific-heritage-month'],
    fill: '#0f766e', motif: 'waves',
    colors: ['#0f766e', '#f97316', '#d4a017'],
    ribbon: ['#0f766e', '#d4a017'], sampleDate: '2027-05-10',
  },
  {
    id: 'mental-health-month', name: 'Mental Health Awareness Month', kind: 'humanity',
    sources: ['mental-health-awareness-month'],
    fill: '#15803d', motif: 'hearts',
    colors: ['#22c55e', '#16a34a', '#86efac'],
    ribbon: ['#22c55e', '#15803d'], sampleDate: '2027-05-20',
  },
  {
    id: 'pride-month', name: 'Pride Month', kind: 'humanity', sources: ['pride-month'],
    fill: '#750787', motif: 'rainbow',
    colors: ['#e40303', '#ff8c00', '#ffed00', '#008026', '#004dff', '#750787'],
    ribbon: ['#e40303', '#ff8c00', '#ffed00', '#008026', '#004dff', '#750787'], sampleDate: '2027-06-10',
  },
  {
    id: 'juneteenth', name: 'Juneteenth', kind: 'humanity', sources: ['juneteenth'],
    fill: '#bf0a30', motif: 'stars',
    colors: ['#bf0a30', '#002868', '#c9a227'],
    ribbon: ['#002868', '#bf0a30'], sampleDate: '2027-06-19',
  },
  {
    id: 'hispanic-heritage-month', name: 'Hispanic Heritage Month', kind: 'humanity',
    sources: ['hispanic-heritage-month'],
    fill: '#b91c1c', motif: 'bunting',
    colors: ['#dc2626', '#ea580c', '#16a34a', '#eab308', '#db2777', '#2563eb'],
    ribbon: ['#dc2626', '#eab308', '#16a34a'], sampleDate: '2026-10-05',
  },
  {
    id: 'peace-day', name: 'International Day of Peace', kind: 'humanity',
    sources: ['international-day-of-peace'],
    fill: '#0369a1', motif: 'leaves',
    colors: ['#6b7c2e', '#9aa86b', '#0369a1'],
    ribbon: ['#009edb', '#f4f1df', '#009edb'], sampleDate: '2027-09-21',
  },
  {
    id: 'mental-health-day', name: 'World Mental Health Day', kind: 'humanity',
    sources: ['world-mental-health-day'],
    fill: '#15803d', motif: 'hearts',
    colors: ['#22c55e', '#16a34a', '#86efac'],
    ribbon: ['#22c55e', '#15803d'], sampleDate: '2026-10-10',
  },
  {
    id: 'indigenous-peoples-day', name: "Indigenous Peoples' Day", kind: 'humanity',
    sources: ['indigenous-peoples-day'],
    fill: '#92400e', motif: 'none',
    colors: ['#92400e', '#b45309', '#4d7c0f'],
    ribbon: ['#92400e', '#4d7c0f'], sampleDate: '2026-10-12',
  },
  {
    id: 'native-american-heritage-month', name: 'Native American Heritage Month', kind: 'humanity',
    sources: ['native-american-heritage-month'],
    fill: '#9a3412', motif: 'none',
    colors: ['#92400e', '#b45309', '#4d7c0f'],
    ribbon: ['#92400e', '#4d7c0f'], sampleDate: '2026-11-15',
  },
  {
    id: 'human-rights-day', name: 'Human Rights Day', kind: 'humanity',
    sources: ['international-human-rights-day'],
    fill: '#0369a1', motif: 'orbits',
    colors: ['#009edb', '#0077b6', '#9aa5b1'],
    ribbon: ['#009edb', '#0077b6'], sampleDate: '2026-12-10',
  },
];

/** Every occasion class name, for scripts that clear the current occasion. */
export const OCCASION_IDS = OCCASIONS.map((o) => o.id);

export function getOccasion(id: string): Occasion | undefined {
  return OCCASIONS.find((o) => o.id === id);
}

/** Map a holidays.ts / humanity-events.ts id to its occasion (look) id. */
export function occasionIdForSource(sourceId: string): string | undefined {
  return OCCASIONS.find((o) => o.sources.includes(sourceId))?.id;
}
