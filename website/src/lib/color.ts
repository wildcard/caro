/**
 * Small, dependency-free color helpers for WCAG contrast math.
 *
 * Used at build time to derive accessible occasion palettes and by the
 * contrast regression tests. Formulas follow WCAG 2.2 (relative luminance,
 * contrast ratio). Colors are #rrggbb strings.
 */

function channels(hex: string): [number, number, number] {
  const h = hex.replace('#', '');
  const full = h.length === 3 ? [...h].map((c) => c + c).join('') : h;
  return [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16)) as [number, number, number];
}

function toHex([r, g, b]: [number, number, number]): string {
  return '#' + [r, g, b].map((c) => Math.round(Math.max(0, Math.min(255, c))).toString(16).padStart(2, '0')).join('');
}

/** WCAG relative luminance of a #rrggbb color. */
export function luminance(hex: string): number {
  const [r, g, b] = channels(hex).map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two colors (1–21). */
export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

/** Linear mix of two colors; t = 0 gives `a`, t = 1 gives `b`. */
export function mix(a: string, b: string, t: number): string {
  const ca = channels(a);
  const cb = channels(b);
  return toHex([0, 1, 2].map((i) => ca[i] + (cb[i] - ca[i]) * t) as [number, number, number]);
}

/**
 * Move `color` toward `toward` (white to lighten, near-black to darken) in
 * small steps until it reaches `ratio` against every background. Returns the
 * first passing color, so the result stays as close to the original hue as
 * possible.
 */
export function ensureContrast(color: string, backgrounds: string[], ratio: number, toward: string): string {
  for (let t = 0; t <= 1.0001; t += 0.02) {
    const candidate = mix(color, toward, t);
    if (backgrounds.every((bg) => contrast(candidate, bg) >= ratio)) return candidate;
  }
  return toward;
}
