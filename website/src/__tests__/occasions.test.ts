/**
 * Regression guards for the occasion system (holidays, national days,
 * humanity events). Fails the build if an occasion would ship a CTA or
 * banner that is not WCAG AA, or if a calendar entry has no look.
 */
import { describe, expect, it } from 'vitest';
import { OCCASIONS, getOccasion, occasionIdForSource } from '../config/occasions';
import { HOLIDAYS } from '../config/holidays';
import { HUMANITY_EVENTS } from '../data/humanity-events';
import { isEventActive } from '../utils/humanity-events';
import { isDateInRange } from '../lib/date-utils';
import { contrast } from '../lib/color';
import {
  DARK_PAGE,
  DARK_RAISED,
  PAPER,
  WHITE,
  buildOccasionCss,
  holidayWindows,
  occasionPalette,
} from '../lib/occasion-css';
import { ART_VARIANTS, renderOccasionArt } from '../lib/occasion-art';

const AA = 4.5;

describe('occasion palettes meet WCAG AA', () => {
  it.each(OCCASIONS.map((o) => [o.id, o] as const))('%s', (_id, o) => {
    const p = occasionPalette(o);
    // White CTA and banner labels on the solid fill, hover included.
    expect(contrast(WHITE, p.fill)).toBeGreaterThanOrEqual(AA);
    expect(contrast(WHITE, p.fillHover)).toBeGreaterThanOrEqual(AA);
    // Occasion-colored text, both color modes.
    expect(contrast(p.ink, PAPER)).toBeGreaterThanOrEqual(AA);
    expect(contrast(p.ink, WHITE)).toBeGreaterThanOrEqual(AA);
    expect(contrast(p.inkDark, DARK_PAGE)).toBeGreaterThanOrEqual(AA);
    expect(contrast(p.inkDark, DARK_RAISED)).toBeGreaterThanOrEqual(AA);
  });
});

describe('occasion registry', () => {
  it('has unique ids', () => {
    const ids = OCCASIONS.map((o) => o.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('maps every enabled holiday to a look', () => {
    const missing = HOLIDAYS.filter((h) => h.enabled !== false && !occasionIdForSource(h.id)).map((h) => h.id);
    expect(missing).toEqual([]);
  });

  it('maps every humanity event to a look', () => {
    const missing = HUMANITY_EVENTS.filter((e) => !occasionIdForSource(e.id)).map((e) => e.id);
    expect(missing).toEqual([]);
  });

  it('uses the runtime class name for humanity events that add one', () => {
    for (const e of HUMANITY_EVENTS) {
      const cls = e.theme?.className;
      if (cls) expect(getOccasion(cls)?.sources).toContain(e.id);
    }
  });

  it.each(OCCASIONS.map((o) => [o.id, o] as const))('%s sampleDate falls inside its window', (_id, o) => {
    const [y, m, d] = o.sampleDate.split('-').map(Number);
    const date = new Date(y, m - 1, d, 12);
    const inside = o.sources.some((src) => {
      const h = HOLIDAYS.find((x) => x.id === src);
      if (h) return isDateInRange(date, h.dateRange, h.leadTimeDays || 0);
      const e = HUMANITY_EVENTS.find((x) => x.id === src);
      return e ? isEventActive(e, date) : false;
    });
    expect(inside).toBe(true);
  });
});

describe('generated occasion CSS', () => {
  const css = buildOccasionCss();

  it('declares every occasion', () => {
    for (const o of OCCASIONS) expect(css).toContain(`html.${o.id}{`);
  });

  it('paints CTAs with a solid fill, never a gradient', () => {
    for (const m of css.matchAll(/--occasion-fill(?:-hover)?:([^;]+);/g)) {
      expect(m[1]).toMatch(/^#[0-9a-f]{6}$/);
    }
  });

  it('stops hero art animation under reduced motion', () => {
    expect(css).toMatch(/prefers-reduced-motion:reduce\)\{[^}]*animation:none/);
  });

  it('builds opt-in holiday windows for the build year', () => {
    const w = holidayWindows(2026);
    expect(w.christmas.some(([s, e]) => s <= 20261220 && e >= 20261220)).toBe(true);
    expect(w.eid.length).toBeGreaterThan(0);
  });
});

describe('occasion art', () => {
  it.each(OCCASIONS.map((o) => [o.id, o] as const))('%s renders clean SVG', (_id, o) => {
    for (const v of ART_VARIANTS) {
      const svg = renderOccasionArt(o, v);
      expect(svg.startsWith('<svg')).toBe(true);
      expect(svg).toContain('aria-hidden="true"');
      expect(svg).not.toMatch(/NaN|undefined|Infinity/);
      expect(svg.length).toBeLessThan(24_000);
      // Deterministic: same occasion and variant, same art.
      expect(renderOccasionArt(o, v)).toBe(svg);
    }
  });
});
