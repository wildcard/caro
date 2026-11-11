/**
 * Build-time generators for the occasion system:
 *   - occasionPalette(): accessible derived colors for one occasion
 *   - buildOccasionCss(): the CSS custom properties and shared rules that
 *     dress the site for whichever occasion class is on <html>
 *   - holidayWindows(): opt-in holiday date windows for the inline
 *     pre-paint script (which cannot import TypeScript)
 *
 * The CSS is injected once by Layout.astro. Art files are served by
 * src/pages/occasions/art/[name].svg.ts.
 */
import { OCCASIONS, occasionIdForSource, type Occasion } from '../config/occasions';
import { HOLIDAYS } from '../config/holidays';
import { resolveDateRange } from './date-utils';
import { ensureContrast, mix } from './color';
import { timezonesForCountries } from './locale-detector';

export const PAPER = '#f4f1df';
export const WHITE = '#ffffff';
export const DARK_PAGE = '#2b2b2b';
export const DARK_RAISED = '#3a3a3a';

export interface OccasionPalette {
  /** CTA / banner fill (white text). */
  fill: string;
  fillHover: string;
  /** Occasion color as TEXT on paper and white (≥ 4.5:1). */
  ink: string;
  /** Occasion color as TEXT on the dark page and dark cards (≥ 4.5:1). */
  inkDark: string;
}

export function occasionPalette(o: Occasion): OccasionPalette {
  return {
    fill: o.fill,
    fillHover: mix(o.fill, '#000000', 0.18),
    ink: ensureContrast(o.fill, [PAPER, WHITE], 4.5, '#1a1a1a'),
    inkDark: ensureContrast(o.fill, [DARK_PAGE, DARK_RAISED], 4.5, '#ffffff'),
  };
}

/** Hard-stop stripes: reads as flat bands of color, not a blend. */
export function ribbonBackground(colors: string[]): string {
  if (colors.length === 1) return colors[0];
  const step = 100 / colors.length;
  const stops = colors.map((c, i) => `${c} ${+(i * step).toFixed(2)}% ${+((i + 1) * step).toFixed(2)}%`);
  return `linear-gradient(90deg, ${stops.join(', ')})`;
}

export function artUrl(id: string, variant: 'left' | 'right' | 'band'): string {
  return `/occasions/art/${id}-${variant}.svg`;
}

// Humanity first, holidays last: if a user opts into a holiday while a
// calendar event is also active, the holiday they chose wins the cascade.
const CASCADE_ORDER: Occasion['kind'][] = ['humanity', 'national', 'holiday'];

export function buildOccasionCss(): string {
  const ordered = CASCADE_ORDER.flatMap((k) => OCCASIONS.filter((o) => o.kind === k));
  const any = `html:is(${OCCASIONS.map((o) => '.' + o.id).join(',')})`;
  // Where hero art appears: any element that opts in, plus the homepage
  // hero (first child of <main>), so pages need no markup change.
  const art = ':is([data-occasion-art],#main-content>.hero:first-child)';

  const perOccasion = ordered.map((o) => {
    const p = occasionPalette(o);
    return (
      `html.${o.id}{` +
      `--occasion-fill:${p.fill};--occasion-fill-hover:${p.fillHover};` +
      `--occasion-ink:${p.ink};` +
      `--occasion-ribbon:${ribbonBackground(o.ribbon)};` +
      `--occasion-art-l:url(${artUrl(o.id, 'left')});` +
      `--occasion-art-r:url(${artUrl(o.id, 'right')});` +
      `--occasion-art-band:url(${artUrl(o.id, 'band')});` +
      `--accent-fill:var(--occasion-fill);--accent-fill-hover:var(--occasion-fill-hover);}` +
      `html.${o.id}.dark{--occasion-ink:${p.inkDark};}`
    );
  });

  const shared = `
/* Ribbon: a 4px band of the occasion's colors above everything else. */
${any} body::before{content:"";display:block;height:4px;background:var(--occasion-ribbon);}
/* Hero art: decorative side panels behind the hero content. */
${any} ${art}{position:relative;isolation:isolate;}
${any} ${art}::before,${any} ${art}::after{
  content:"";position:absolute;top:0;bottom:0;z-index:-1;pointer-events:none;
  width:min(22vw,300px,calc((100vw - 900px) / 2));
  background-repeat:no-repeat;background-size:contain;
  animation:occasion-float 14s ease-in-out infinite alternate;}
${any} ${art}::before{left:0;background-image:var(--occasion-art-l);background-position:left center;
  -webkit-mask-image:linear-gradient(to right,#000 55%,transparent);mask-image:linear-gradient(to right,#000 55%,transparent);}
${any} ${art}::after{right:0;background-image:var(--occasion-art-r);background-position:right center;
  -webkit-mask-image:linear-gradient(to left,#000 55%,transparent);mask-image:linear-gradient(to left,#000 55%,transparent);
  animation-delay:-7s;}
html.dark:is(${OCCASIONS.map((o) => '.' + o.id).join(',')}) ${art}::before,
html.dark:is(${OCCASIONS.map((o) => '.' + o.id).join(',')}) ${art}::after{filter:brightness(1.6) saturate(1.1);}
@media (max-width:1100px){
  ${any} ${art}::before{top:0;bottom:auto;left:0;right:0;width:auto;height:72px;
    background-image:var(--occasion-art-band);background-size:auto 100%;background-position:center top;background-repeat:repeat-x;
    -webkit-mask-image:linear-gradient(to bottom,#000 60%,transparent);mask-image:linear-gradient(to bottom,#000 60%,transparent);}
  ${any} ${art}::after{display:none;}
}
@keyframes occasion-float{from{transform:translateY(0)}to{transform:translateY(-10px)}}
@media (prefers-reduced-motion:reduce){${any} ${art}::before,${any} ${art}::after{animation:none;}}
`;

  return perOccasion.join('\n') + '\n' + shared.replace(/\n\s*/g, '\n').trim();
}

/** YYYYMMDD as a number, for cheap comparisons in the inline script. */
function ymd(d: Date): number {
  return d.getFullYear() * 10000 + (d.getMonth() + 1) * 100 + d.getDate();
}

/**
 * Opt-in holiday windows (lead time included) keyed by occasion id, for the
 * years around the build date. The site rebuilds on every deploy, so a
 * ±1/+2 year horizon is enough.
 */
export function holidayWindows(buildYear = new Date().getFullYear()): Record<string, Array<[number, number]>> {
  const out: Record<string, Array<[number, number]>> = {};
  for (const h of HOLIDAYS) {
    if (h.enabled === false) continue;
    const id = occasionIdForSource(h.id);
    if (!id) continue;
    for (let y = buildYear - 1; y <= buildYear + 2; y++) {
      const r = resolveDateRange(h.dateRange, y);
      if (!r) continue;
      const start = new Date(r.start);
      start.setDate(start.getDate() - (h.leadTimeDays || 0));
      (out[id] ||= []).push([ymd(start), ymd(r.end)]);
    }
  }
  return out;
}

/** Locale restriction per occasion id, for national days (ISO country codes). */
export function holidayLocales(): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  for (const h of HOLIDAYS) {
    const id = occasionIdForSource(h.id);
    if (id && h.visibility === 'locale-only' && h.locales?.length) out[id] = h.locales;
  }
  return out;
}

/**
 * Timezones allowed per locale-only occasion (national days). The pre-paint
 * script drops a saved national-day theme outside these zones, so a visitor
 * who travels or changes their clock does not keep another country's day.
 */
export function holidayTimezones(): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  for (const [id, countries] of Object.entries(holidayLocales())) {
    out[id] = timezonesForCountries(countries);
  }
  return out;
}
