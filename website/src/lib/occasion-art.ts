/**
 * Occasion art — deterministic, decorative SVG motifs for the hero frame.
 *
 * Each occasion gets three static SVG files (served from
 * /occasions/art/<id>-<variant>.svg, generated at build time):
 *   - left / right: 320×640 side panels that frame wide heroes
 *   - band:         720×112 strip used above the hero on narrow screens
 *
 * The art is purely decorative: it is painted with CSS background-image,
 * never carries text, and is hidden from assistive tech by construction.
 * Placement is seeded from the occasion id, so builds are reproducible and
 * the same occasion always looks the same.
 */
import type { Motif, Occasion } from '../config/occasions';

export type ArtVariant = 'left' | 'right' | 'band';

const SIZE: Record<ArtVariant, [number, number]> = {
  left: [320, 640],
  right: [320, 640],
  band: [720, 112],
};

/** mulberry32 — tiny seeded PRNG. */
function rng(seed: string): () => number {
  let h = 1779033703 ^ seed.length;
  for (let i = 0; i < seed.length; i++) {
    h = Math.imul(h ^ seed.charCodeAt(i), 3432918353);
    h = (h << 13) | (h >>> 19);
  }
  let a = h >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const f = (n: number) => Math.round(n * 10) / 10;

interface Ctx {
  w: number;
  h: number;
  r: () => number;
  pick: <T>(xs: T[]) => T;
  between: (a: number, b: number) => number;
  colors: string[];
  variant: ArtVariant;
}

/** Spread `n` points over the panel without heavy clumping (jittered grid). */
function scatter(c: Ctx, n: number, pad = 24): Array<[number, number]> {
  const cols = c.variant === 'band' ? Math.ceil(n / 2) : 2;
  const rows = Math.ceil(n / cols);
  const cw = (c.w - pad * 2) / cols;
  const rh = (c.h - pad * 2) / rows;
  const pts: Array<[number, number]> = [];
  for (let i = 0; i < n; i++) {
    const col = i % cols;
    const row = Math.floor(i / cols);
    pts.push([
      pad + col * cw + c.between(0.15, 0.85) * cw,
      pad + row * rh + c.between(0.15, 0.85) * rh,
    ]);
  }
  return pts;
}

function star(cx: number, cy: number, outer: number, inner: number, points = 5, rot = -90): string {
  const pts: string[] = [];
  for (let i = 0; i < points * 2; i++) {
    const rad = ((rot + (i * 180) / points) * Math.PI) / 180;
    const len = i % 2 === 0 ? outer : inner;
    pts.push(`${f(cx + Math.cos(rad) * len)},${f(cy + Math.sin(rad) * len)}`);
  }
  return `<polygon points="${pts.join(' ')}"/>`;
}

function flame(cx: number, baseY: number, h: number): string {
  const w = h * 0.42;
  return `<path d="M${f(cx)} ${f(baseY - h)} C ${f(cx + w)} ${f(baseY - h * 0.45)} ${f(cx + w * 0.8)} ${f(baseY)} ${f(cx)} ${f(baseY)} C ${f(cx - w * 0.8)} ${f(baseY)} ${f(cx - w)} ${f(baseY - h * 0.45)} ${f(cx)} ${f(baseY - h)} Z"/>`;
}

function glow(cx: number, cy: number, r: number, color: string): string {
  return `<g fill="${color}"><circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r)}" opacity=".12"/><circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r * 0.6)}" opacity=".18"/></g>`;
}

const MOTIFS: Record<Motif, (c: Ctx) => string> = {
  snowflakes(c) {
    const n = c.variant === 'band' ? 9 : 12;
    return scatter(c, n).map(([x, y]) => {
      const s = c.between(10, c.variant === 'band' ? 22 : 34);
      const col = c.pick(c.colors);
      const rot = c.between(0, 60);
      let arms = '';
      for (let k = 0; k < 6; k++) {
        const a = ((rot + k * 60) * Math.PI) / 180;
        const ex = x + Math.cos(a) * s;
        const ey = y + Math.sin(a) * s;
        const bx = x + Math.cos(a) * s * 0.6;
        const by = y + Math.sin(a) * s * 0.6;
        const b1 = a + 0.6;
        const b2 = a - 0.6;
        arms += `M${f(x)} ${f(y)}L${f(ex)} ${f(ey)}M${f(bx)} ${f(by)}L${f(bx + Math.cos(b1) * s * 0.3)} ${f(by + Math.sin(b1) * s * 0.3)}M${f(bx)} ${f(by)}L${f(bx + Math.cos(b2) * s * 0.3)} ${f(by + Math.sin(b2) * s * 0.3)}`;
      }
      return `<path d="${arms}" stroke="${col}" stroke-width="${f(c.between(1.4, 2.2))}" stroke-linecap="round" fill="none" opacity="${f(c.between(0.55, 0.95))}"/>`;
    }).join('');
  },

  candles(c) {
    // Nine slim lights along a gentle arc, the middle one raised: an
    // abstract "festival of lights" without drawing a ritual object.
    const [wax, , gold, silver] = [c.colors[0], c.colors[1], c.colors[2], c.colors[3] ?? c.colors[0]];
    const band = c.variant === 'band';
    const n = 9;
    const span = band ? c.w * 0.5 : c.w * 0.8;
    const x0 = (c.w - span) / 2;
    const base = band ? c.h - 14 : c.h * 0.78;
    let out = `<path d="M${f(x0 - 10)} ${f(base)} Q ${f(c.w / 2)} ${f(base + (band ? 8 : 26))} ${f(x0 + span + 10)} ${f(base)}" stroke="${silver}" stroke-width="3" fill="none" stroke-linecap="round" opacity=".7"/>`;
    for (let i = 0; i < n; i++) {
      const x = x0 + (span * i) / (n - 1);
      const mid = i === 4;
      const ch = (band ? 26 : 70) + (mid ? (band ? 14 : 34) : 0);
      const top = base - ch;
      out += `<rect x="${f(x - 4)}" y="${f(top)}" width="8" height="${f(ch)}" rx="2" fill="${i % 2 ? wax : c.colors[1]}" opacity=".85"/>`;
      out += glow(x, top - 10, band ? 12 : 18, gold);
      out += `<g fill="${gold}">${flame(x, top - 2, band ? 14 : 20)}</g>`;
    }
    const sparks = band ? 6 : 10;
    for (let i = 0; i < sparks; i++) {
      out += `<g fill="${c.pick([gold, c.colors[1]])}" opacity="${f(c.between(0.4, 0.8))}">${star(c.between(20, c.w - 20), c.between(16, band ? 40 : base - 160), c.between(3, 7), 1.4, 4, 0)}</g>`;
    }
    return out;
  },

  diyas(c) {
    const [flameCol, gold, clay, magenta, purple] = c.colors;
    const n = c.variant === 'band' ? 6 : 6;
    let out = '';
    for (const [x, y] of scatter(c, n, 40)) {
      const s = c.between(0.7, c.variant === 'band' ? 0.9 : 1.25);
      out += glow(x, y - 22 * s, 26 * s, gold);
      out += `<path d="M${f(x - 26 * s)} ${f(y)} Q ${f(x)} ${f(y + 26 * s)} ${f(x + 26 * s)} ${f(y)} Z" fill="${clay}"/>`;
      out += `<path d="M${f(x - 26 * s)} ${f(y)} L${f(x + 26 * s)} ${f(y)}" stroke="${gold}" stroke-width="2.5" stroke-linecap="round"/>`;
      out += `<g fill="${flameCol}">${flame(x, y - 2, 26 * s)}</g>`;
    }
    // Rangoli-style dot rings.
    const rings = c.variant === 'band' ? 2 : 3;
    for (let i = 0; i < rings; i++) {
      const cx = c.between(40, c.w - 40);
      const cy = c.between(40, c.h - 40);
      const rr = c.between(14, 26);
      for (let k = 0; k < 12; k++) {
        const a = (k * Math.PI) / 6;
        out += `<circle cx="${f(cx + Math.cos(a) * rr)}" cy="${f(cy + Math.sin(a) * rr)}" r="2.6" fill="${k % 2 ? magenta : purple}" opacity=".7"/>`;
      }
      out += `<circle cx="${f(cx)}" cy="${f(cy)}" r="5" fill="${gold}" opacity=".8"/>`;
    }
    return out;
  },

  fireworks(c) {
    const n = c.variant === 'band' ? 5 : 5;
    let out = '';
    for (const [x, y] of scatter(c, n, 50)) {
      const col = c.pick(c.colors);
      const rays = Math.round(c.between(12, 18));
      const len = c.between(c.variant === 'band' ? 20 : 30, c.variant === 'band' ? 36 : 62);
      let d = '';
      let dots = '';
      for (let k = 0; k < rays; k++) {
        const a = (k / rays) * Math.PI * 2 + c.between(-0.06, 0.06);
        const l = len * c.between(0.75, 1);
        d += `M${f(x + Math.cos(a) * l * 0.35)} ${f(y + Math.sin(a) * l * 0.35)}L${f(x + Math.cos(a) * l)} ${f(y + Math.sin(a) * l)}`;
        dots += `<circle cx="${f(x + Math.cos(a) * (l + 6))}" cy="${f(y + Math.sin(a) * (l + 6))}" r="2"/>`;
      }
      out += `<g opacity="${f(c.between(0.6, 0.95))}"><path d="${d}" stroke="${col}" stroke-width="2" stroke-linecap="round"/><g fill="${col}">${dots}</g></g>`;
    }
    return out;
  },

  powder(c) {
    const n = c.variant === 'band' ? 10 : 12;
    let out = '';
    for (const [x, y] of scatter(c, n, 30)) {
      const col = c.pick(c.colors);
      const r = c.between(c.variant === 'band' ? 14 : 20, c.variant === 'band' ? 26 : 46);
      out += `<g fill="${col}"><circle cx="${f(x)}" cy="${f(y)}" r="${f(r)}" opacity=".16"/><circle cx="${f(x + r * 0.3)}" cy="${f(y - r * 0.2)}" r="${f(r * 0.65)}" opacity=".22"/><circle cx="${f(x - r * 0.25)}" cy="${f(y + r * 0.2)}" r="${f(r * 0.4)}" opacity=".3"/></g>`;
      for (let k = 0; k < 6; k++) {
        out += `<circle cx="${f(x + c.between(-r * 1.4, r * 1.4))}" cy="${f(y + c.between(-r * 1.4, r * 1.4))}" r="${f(c.between(1.2, 3))}" fill="${col}" opacity=".6"/>`;
      }
    }
    return out;
  },

  lanterns(c) {
    const [red, gold, deep] = c.colors;
    const band = c.variant === 'band';
    const n = band ? 6 : 5;
    let out = '';
    for (let i = 0; i < n; i++) {
      const x = band ? 40 + (i * (c.w - 80)) / (n - 1) : c.between(50, c.w - 50);
      const hang = band ? c.between(8, 30) : c.between(40, c.h - 160);
      const w = band ? c.between(26, 34) : c.between(44, 66);
      const h = w * 1.15;
      const cy = hang + h / 2 + 10;
      out += `<path d="M${f(x)} 0V${f(hang)}" stroke="${gold}" stroke-width="1.5" opacity=".7"/>`;
      out += `<rect x="${f(x - w * 0.22)}" y="${f(hang)}" width="${f(w * 0.44)}" height="7" rx="2" fill="${gold}"/>`;
      out += `<ellipse cx="${f(x)}" cy="${f(cy)}" rx="${f(w / 2)}" ry="${f(h / 2)}" fill="${i % 2 ? red : deep}"/>`;
      out += `<path d="M${f(x)} ${f(cy - h / 2)}V${f(cy + h / 2)}M${f(x - w * 0.25)} ${f(cy - h * 0.46)}Q ${f(x - w * 0.42)} ${f(cy)} ${f(x - w * 0.25)} ${f(cy + h * 0.46)}M${f(x + w * 0.25)} ${f(cy - h * 0.46)}Q ${f(x + w * 0.42)} ${f(cy)} ${f(x + w * 0.25)} ${f(cy + h * 0.46)}" stroke="${gold}" stroke-width="1.4" fill="none" opacity=".75"/>`;
      out += `<rect x="${f(x - w * 0.22)}" y="${f(cy + h / 2 - 2)}" width="${f(w * 0.44)}" height="6" rx="2" fill="${gold}"/>`;
      out += `<path d="M${f(x)} ${f(cy + h / 2 + 4)}v${band ? 10 : 22}M${f(x - 4)} ${f(cy + h / 2 + 6)}v${band ? 8 : 18}M${f(x + 4)} ${f(cy + h / 2 + 6)}v${band ? 8 : 18}" stroke="${gold}" stroke-width="1.6" stroke-linecap="round"/>`;
    }
    return out;
  },

  'moon-lanterns'(c) {
    const [moon, lantern, deep, pale] = c.colors;
    const band = c.variant === 'band';
    const mr = band ? 40 : 86;
    const mx = c.variant === 'right' ? c.w * 0.38 : c.w * 0.6;
    const my = band ? c.h / 2 : c.h * 0.24;
    let out = `<circle cx="${f(mx)}" cy="${f(my)}" r="${f(mr * 1.35)}" fill="${pale}" opacity=".25"/><circle cx="${f(mx)}" cy="${f(my)}" r="${f(mr)}" fill="${moon}" opacity=".9"/>`;
    for (let i = 0; i < 4; i++) {
      out += `<circle cx="${f(mx + c.between(-mr * 0.5, mr * 0.5))}" cy="${f(my + c.between(-mr * 0.5, mr * 0.5))}" r="${f(c.between(mr * 0.06, mr * 0.14))}" fill="${pale}" opacity=".55"/>`;
    }
    const n = band ? 4 : 3;
    for (let i = 0; i < n; i++) {
      const x = band ? (i < 2 ? 60 + i * 90 : c.w - 60 - (i - 2) * 90) : c.between(50, c.w - 50);
      const y = band ? c.between(30, 70) : c.between(c.h * 0.48, c.h - 70);
      const w = band ? 24 : c.between(36, 50);
      out += `<path d="M${f(x)} ${f(y - w)}V${f(y - w * 1.8)}" stroke="${deep}" stroke-width="1.4" opacity=".6"/>`;
      out += `<rect x="${f(x - w / 2)}" y="${f(y - w * 0.8)}" width="${f(w)}" height="${f(w * 1.4)}" rx="${f(w * 0.3)}" fill="${lantern}" opacity=".9"/>`;
      out += `<path d="M${f(x - w / 2)} ${f(y - w * 0.1)}H${f(x + w / 2)}M${f(x - w / 2)} ${f(y + w * 0.3)}H${f(x + w / 2)}" stroke="${moon}" stroke-width="1.4" opacity=".8"/>`;
    }
    return out;
  },

  crescent(c) {
    const [gold, green, teal, pale] = c.colors;
    const band = c.variant === 'band';
    const r = band ? 38 : 80;
    const cx = c.variant === 'right' ? c.w * 0.36 : c.w * 0.58;
    const cy = band ? c.h / 2 : c.h * 0.26;
    let out = `<circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r * 1.3)}" fill="${pale}" opacity=".18"/>`;
    out += `<path d="M${f(cx)} ${f(cy - r)} A ${f(r)} ${f(r)} 0 1 0 ${f(cx)} ${f(cy + r)} A ${f(r * 0.78)} ${f(r * 0.78)} 0 1 1 ${f(cx)} ${f(cy - r)} Z" fill="${gold}" transform="rotate(-20 ${f(cx)} ${f(cy)})"/>`;
    const n = band ? 7 : 11;
    for (const [x, y] of scatter(c, n, 20)) {
      if (Math.hypot(x - cx, y - cy) < r * 1.4) continue;
      out += `<g fill="${c.pick([gold, gold, teal, green])}" opacity="${f(c.between(0.55, 0.95))}">${star(x, y, c.between(5, band ? 9 : 14), c.between(1.6, 3), 4, 0)}</g>`;
    }
    return out;
  },

  leaves(c) {
    const n = c.variant === 'band' ? 9 : 13;
    return scatter(c, n, 26).map(([x, y]) => {
      const s = c.between(c.variant === 'band' ? 12 : 16, c.variant === 'band' ? 20 : 32);
      const col = c.pick(c.colors);
      const rot = c.between(0, 360);
      return `<g transform="rotate(${f(rot)} ${f(x)} ${f(y)})" opacity="${f(c.between(0.6, 0.95))}"><path d="M${f(x)} ${f(y - s)} Q ${f(x + s * 0.8)} ${f(y)} ${f(x)} ${f(y + s)} Q ${f(x - s * 0.8)} ${f(y)} ${f(x)} ${f(y - s)} Z" fill="${col}"/><path d="M${f(x)} ${f(y - s * 0.8)}V${f(y + s * 1.25)}" stroke="#f4f1df" stroke-width="1.2" opacity=".7"/></g>`;
    }).join('');
  },

  blossoms(c) {
    const n = c.variant === 'band' ? 8 : 10;
    let out = '';
    for (const [x, y] of scatter(c, n, 30)) {
      const s = c.between(c.variant === 'band' ? 9 : 12, c.variant === 'band' ? 15 : 24);
      const col = c.pick(c.colors);
      const rot = c.between(0, 72);
      let petals = '';
      for (let k = 0; k < 5; k++) {
        const a = rot + k * 72;
        petals += `<ellipse cx="${f(x)}" cy="${f(y - s * 0.62)}" rx="${f(s * 0.42)}" ry="${f(s * 0.62)}" transform="rotate(${f(a)} ${f(x)} ${f(y)})"/>`;
      }
      out += `<g fill="${col}" opacity="${f(c.between(0.6, 0.9))}">${petals}</g><circle cx="${f(x)}" cy="${f(y)}" r="${f(s * 0.24)}" fill="${c.colors[c.colors.length - 1]}"/>`;
    }
    for (let i = 0; i < (c.variant === 'band' ? 6 : 10); i++) {
      const x = c.between(10, c.w - 10);
      const y = c.between(10, c.h - 10);
      out += `<ellipse cx="${f(x)}" cy="${f(y)}" rx="3" ry="5.5" fill="${c.pick(c.colors)}" opacity=".5" transform="rotate(${f(c.between(0, 180))} ${f(x)} ${f(y)})"/>`;
    }
    return out;
  },

  rainbow(c) {
    const band = c.variant === 'band';
    const cx = band ? c.w / 2 : c.variant === 'left' ? 0 : c.w;
    const cy = band ? c.h + 30 : c.h * 0.62;
    const step = band ? 9 : 16;
    const r0 = band ? 50 : 120;
    let out = '';
    c.colors.forEach((col, i) => {
      out += `<circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r0 + (c.colors.length - 1 - i) * step)}" fill="none" stroke="${col}" stroke-width="${f(step - 2)}" opacity=".85"/>`;
    });
    for (let i = 0; i < (band ? 6 : 9); i++) {
      out += `<g fill="${c.pick(c.colors)}" opacity=".7">${star(c.between(16, c.w - 16), c.between(14, band ? 50 : c.h * 0.3), c.between(4, 8), 1.6, 4, 0)}</g>`;
    }
    return out;
  },

  sunburst(c) {
    const band = c.variant === 'band';
    const [a, b, d] = c.colors;
    const cx = band ? c.w / 2 : c.variant === 'left' ? c.w * 0.55 : c.w * 0.45;
    const cy = band ? c.h / 2 : c.h * 0.3;
    const r = band ? 22 : 46;
    const rays = 16;
    let out = '';
    for (let k = 0; k < rays; k++) {
      const ang = (k / rays) * Math.PI * 2;
      const inner = r * 1.25;
      const outer = r * (k % 2 ? 1.9 : 2.5);
      const w = 0.09;
      out += `<path d="M${f(cx + Math.cos(ang - w) * inner)} ${f(cy + Math.sin(ang - w) * inner)}L${f(cx + Math.cos(ang) * outer)} ${f(cy + Math.sin(ang) * outer)}L${f(cx + Math.cos(ang + w) * inner)} ${f(cy + Math.sin(ang + w) * inner)}Z" fill="${k % 2 ? b : a}" opacity=".8"/>`;
    }
    out += `<circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r)}" fill="${b}"/><circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r * 0.62)}" fill="${a}" opacity=".9"/>`;
    const dots = band ? 10 : 18;
    for (let i = 0; i < dots; i++) {
      const x = c.between(14, c.w - 14);
      const y = band ? c.between(10, c.h - 10) : c.between(c.h * 0.55, c.h - 14);
      if (Math.hypot(x - cx, y - cy) < r * 2.7) continue;
      out += `<circle cx="${f(x)}" cy="${f(y)}" r="${f(c.between(2, 5))}" fill="${c.pick([a, b, d])}" opacity=".7"/>`;
    }
    return out;
  },

  stars(c) {
    const n = c.variant === 'band' ? 10 : 14;
    return scatter(c, n, 20).map(([x, y]) => {
      const s = c.between(c.variant === 'band' ? 6 : 8, c.variant === 'band' ? 13 : 22);
      const col = c.pick(c.colors);
      return c.r() < 0.3
        ? `<g fill="none" stroke="${col}" stroke-width="1.8" stroke-linejoin="round" opacity=".8">${star(x, y, s, s * 0.45)}</g>`
        : `<g fill="${col}" opacity="${f(c.between(0.55, 0.95))}">${star(x, y, s, s * 0.45)}</g>`;
    }).join('');
  },

  bunting(c) {
    // Papel-picado style garlands: strings of cut-paper flags.
    const band = c.variant === 'band';
    const lines = band ? 1 : 4;
    let out = '';
    for (let l = 0; l < lines; l++) {
      const y0 = band ? 10 : 40 + l * ((c.h - 120) / (lines - 1));
      const sag = band ? 22 : c.between(18, 34);
      out += `<path d="M0 ${f(y0)} Q ${f(c.w / 2)} ${f(y0 + sag * 2)} ${f(c.w)} ${f(y0)}" stroke="#7a7a7a" stroke-width="1.2" fill="none" opacity=".6"/>`;
      const flags = band ? 14 : 6;
      for (let k = 0; k < flags; k++) {
        const t = (k + 0.5) / flags;
        const x = t * c.w;
        const y = (1 - t) * (1 - t) * y0 + 2 * (1 - t) * t * (y0 + sag * 2) + t * t * y0;
        const w = band ? 30 : 38;
        const h = band ? 34 : 44;
        const col = c.colors[(k + l) % c.colors.length];
        out += `<g opacity=".88"><path d="M${f(x - w / 2)} ${f(y)}H${f(x + w / 2)}V${f(y + h)}L${f(x + w / 4)} ${f(y + h - 6)}L${f(x)} ${f(y + h)}L${f(x - w / 4)} ${f(y + h - 6)}L${f(x - w / 2)} ${f(y + h)}Z" fill="${col}"/>`;
        // Cut-out pattern: small paper-colored holes.
        out += `<g fill="#f4f1df" opacity=".85"><circle cx="${f(x)}" cy="${f(y + h * 0.42)}" r="${f(w * 0.14)}"/><circle cx="${f(x - w * 0.27)}" cy="${f(y + h * 0.22)}" r="2"/><circle cx="${f(x + w * 0.27)}" cy="${f(y + h * 0.22)}" r="2"/><circle cx="${f(x - w * 0.27)}" cy="${f(y + h * 0.62)}" r="2"/><circle cx="${f(x + w * 0.27)}" cy="${f(y + h * 0.62)}" r="2"/></g></g>`;
      }
    }
    return out;
  },

  orbits(c) {
    const band = c.variant === 'band';
    const cx = band ? c.w / 2 : c.variant === 'left' ? c.w * 0.45 : c.w * 0.55;
    const cy = band ? c.h / 2 : c.h * 0.4;
    let out = '';
    const rings = band ? 3 : 5;
    for (let i = 0; i < rings; i++) {
      const rx = (band ? 60 : 50) + i * (band ? 70 : 26);
      const ry = band ? 16 + i * 10 : rx * 0.42;
      const col = c.colors[i % c.colors.length];
      const rot = band ? 0 : -18 + i * 9;
      out += `<ellipse cx="${f(cx)}" cy="${f(cy)}" rx="${f(rx)}" ry="${f(ry)}" fill="none" stroke="${col}" stroke-width="1.6" opacity=".7" transform="rotate(${f(rot)} ${f(cx)} ${f(cy)})"/>`;
      const a = c.between(0, Math.PI * 2);
      const px = cx + Math.cos(a) * rx;
      const py = cy + Math.sin(a) * ry;
      out += `<circle cx="${f(px)}" cy="${f(py)}" r="${f(c.between(3, 6))}" fill="${col}" transform="rotate(${f(rot)} ${f(cx)} ${f(cy)})"/>`;
    }
    out += `<circle cx="${f(cx)}" cy="${f(cy)}" r="${band ? 10 : 18}" fill="${c.colors[0]}" opacity=".85"/>`;
    return out;
  },

  hearts(c) {
    const n = c.variant === 'band' ? 9 : 11;
    return scatter(c, n, 26).map(([x, y]) => {
      const s = c.between(c.variant === 'band' ? 8 : 10, c.variant === 'band' ? 15 : 24);
      const col = c.pick(c.colors);
      const d = `M${f(x)} ${f(y + s * 0.9)} C ${f(x - s * 1.6)} ${f(y - s * 0.2)} ${f(x - s * 0.6)} ${f(y - s * 1.2)} ${f(x)} ${f(y - s * 0.35)} C ${f(x + s * 0.6)} ${f(y - s * 1.2)} ${f(x + s * 1.6)} ${f(y - s * 0.2)} ${f(x)} ${f(y + s * 0.9)} Z`;
      return c.r() < 0.35
        ? `<path d="${d}" fill="none" stroke="${col}" stroke-width="2" opacity=".8"/>`
        : `<path d="${d}" fill="${col}" opacity="${f(c.between(0.5, 0.9))}"/>`;
    }).join('');
  },
};

/** Render one art panel for an occasion as a standalone SVG document. */
export function renderOccasionArt(o: Occasion, variant: ArtVariant): string {
  const [w, h] = SIZE[variant];
  const r = rng(`${o.id}:${variant}`);
  const ctx: Ctx = {
    w,
    h,
    r,
    variant,
    colors: o.colors,
    pick: (xs) => xs[Math.floor(r() * xs.length)],
    between: (a, b) => a + r() * (b - a),
  };
  const body = MOTIFS[o.motif](ctx);
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" aria-hidden="true">${body}</svg>`;
}

export const ART_VARIANTS: ArtVariant[] = ['left', 'right', 'band'];
