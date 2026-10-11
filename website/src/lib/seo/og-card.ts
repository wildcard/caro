// Share cards, 1200 × 630, one per page. Drawn on the brand paper: the page
// title in Azeret Mono (the display face, src/ui/tokens.css), a small mono
// eyebrow, the page URL behind a signal-red `$` prompt (the card's 5% of
// accent), and the smooth Caro mark, which the brand book reserves for light
// surfaces.
//
// Text is outlined to SVG paths with opentype.js, so a card renders the same
// on any build machine with no system fonts. The system follows the share
// cards of kobi.kadosh.me (apps/site/src/brand/og.ts).
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
// opentype.js ships CommonJS without an `exports` map, so Node's ESM loader
// (which runs astro.config.mjs) sees only a default export.
import opentype, { type Font, type PathCommand } from 'opentype.js';
import sharp from 'sharp';
import { fitTitle } from './og-text';

export const OG_SIZE = { width: 1200, height: 630 } as const;

/** Brand tokens from src/ui/tokens.css (light mode). */
export const CARD_COLORS = {
  /** --caro-beige-100, the paper (--bg). */
  paper: '#f4f1df',
  /** --caro-grey-900 (--fg-strong). */
  ink: '#2b2b2b',
  /** --caro-grey-600 (--fg-muted). */
  muted: '#636363',
  /** --caro-red-500 (--accent). */
  signal: '#ef3333',
} as const;

const require = createRequire(import.meta.url);

const FONT_FILES = {
  /** Azeret Mono Bold: the headline weight (LPHero .headline). */
  display: '@fontsource/azeret-mono/files/azeret-mono-latin-700-normal.woff',
  /** Azeret Mono Medium, for the eyebrow and the URL. */
  label: '@fontsource/azeret-mono/files/azeret-mono-latin-500-normal.woff',
} as const;

interface Fonts {
  display: Font;
  label: Font;
}

let fontsLoading: Promise<Fonts> | undefined;

async function loadFont(specifier: string): Promise<Font> {
  try {
    const file = await readFile(require.resolve(specifier));
    return opentype.parse(file.buffer.slice(file.byteOffset, file.byteOffset + file.byteLength));
  } catch (error) {
    throw new Error(`Share cards: could not load font ${specifier}: ${(error as Error).message}`);
  }
}

function loadFonts(): Promise<Fonts> {
  fontsLoading ??= Promise.all([loadFont(FONT_FILES.display), loadFont(FONT_FILES.label)]).then(
    ([display, label]) => ({ display, label }),
  );
  return fontsLoading;
}

// Layout, in card pixels.
const PAD_X = 80;
const PAD_TOP = 76;
const PAD_BOTTOM = 72;
/** Height of the mark's source box (the PNG with its white margin) on the card. */
const MARK_HEIGHT = 400;
/** Gap between the text column and the mark. */
const COLUMN_GAP = 64;
const LABEL_SIZE = 24;
/** Label tracking, in 1/1000 em. */
const LABEL_TRACKING = 80;
const URL_SIZE = 28;
const TITLE_LINE_HEIGHT = 1.12;
const TITLE_SIZES = [72, 68, 64, 60, 56, 52, 48, 44, 40] as const;
/**
 * Down to this size, a title takes the size that needs the fewest lines, so
 * "Caro vs Warp: Feature Comparison" sets in two lines, not four short ones.
 */
const TITLE_MIN_PREFERRED = 56;
const TITLE_MAX_LINES = 5;

interface TextOptions {
  font: Font;
  size: number;
  tracking?: number;
}

function advance(text: string, { font, size, tracking = 0 }: TextOptions): number {
  // opentype adds tracking after every glyph, including the last one.
  return font.getAdvanceWidth(text, size, { kerning: true, tracking }) - (tracking / 1000) * size;
}

/**
 * SVG path data for `commands`. opentype.js ≥ 1.3.5 can write NaN from its
 * own toPathData (#2020), so the data is written here, as on kobi.kadosh.me.
 */
export function pathData(commands: PathCommand[]): string {
  const number = (v: number) => (Math.round(v) === v ? String(v) : v.toFixed(2));
  const values = (...vs: number[]) =>
    vs.map((v, i) => (v >= 0 && i > 0 ? ' ' : '') + number(v)).join('');
  let d = '';
  for (const cmd of commands) {
    if (cmd.type === 'M' || cmd.type === 'L') d += cmd.type + values(cmd.x, cmd.y);
    else if (cmd.type === 'C') d += `C${values(cmd.x1, cmd.y1, cmd.x2, cmd.y2, cmd.x, cmd.y)}`;
    else if (cmd.type === 'Q') d += `Q${values(cmd.x1, cmd.y1, cmd.x, cmd.y)}`;
    else if (cmd.type === 'Z') d += 'Z';
  }
  return d;
}

function textPath(
  text: string,
  x: number,
  baseline: number,
  { font, size, tracking = 0 }: TextOptions,
  fill: string,
): string {
  const d = pathData(font.getPath(text, x, baseline, size, { kerning: true, tracking }).commands);
  // An SVG renderer stops a path at its first NaN, so the line would vanish.
  if (/NaN|Infinity/.test(d)) {
    throw new Error(`opentype.js outlined "${text}" at ${size}px with non-finite coordinates`);
  }
  return `<path d="${d}" fill="${fill}"/>`;
}

/** Characters `font` has no glyph for: they would print as empty boxes. */
export function missingGlyphs(text: string, font: Font): string[] {
  return [...new Set(text)].filter(
    (char) => char.trim() !== '' && font.charToGlyphIndex(char) === 0,
  );
}

/** Cuts `text` with an ellipsis until it fits `maxWidth`. */
function fitLine(text: string, options: TextOptions, maxWidth: number): string {
  if (advance(text, options) <= maxWidth) return text;
  let kept = text;
  while (kept.length > 1 && advance(`${kept}…`, options) > maxWidth) kept = kept.slice(0, -1);
  return `${kept}…`;
}

function hexToRgb(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

type Mark = { png: Buffer; width: number; height: number };

const marks = new Map<string, Promise<Mark>>();

/**
 * The mark, without its white margin, scaled as if its box were MARK_HEIGHT
 * tall, and mapped onto the line from ink (black) to paper (white). The source is black on a white box; mapped, the box
 * becomes the card's own paper and the mark sits on it.
 */
function loadMark(file: string): Promise<Mark> {
  let loading = marks.get(file);
  if (!loading) {
    loading = (async () => {
      // Trim the source's white margin, at the scale the full box would
      // have, so the visible mark lines up with the card's padding.
      const source = await sharp(file).metadata();
      const trimmed = await sharp(file)
        .flatten({ background: '#ffffff' })
        .trim({ background: '#ffffff', threshold: 10 })
        .toBuffer({ resolveWithObject: true });
      const scale = MARK_HEIGHT / (source.height ?? MARK_HEIGHT);
      const { data, info } = await sharp(trimmed.data)
        .resize({ height: Math.round(trimmed.info.height * scale) })
        .greyscale()
        .raw()
        .toBuffer({ resolveWithObject: true });
      const [dr, dg, db] = hexToRgb(CARD_COLORS.ink);
      const [lr, lg, lb] = hexToRgb(CARD_COLORS.paper);
      const pixels = Buffer.alloc(info.width * info.height * 3);
      for (let i = 0, j = 0; j < pixels.length; i += info.channels, j += 3) {
        const t = (data[i] as number) / 255;
        pixels[j] = Math.round(dr + (lr - dr) * t);
        pixels[j + 1] = Math.round(dg + (lg - dg) * t);
        pixels[j + 2] = Math.round(db + (lb - db) * t);
      }
      const png = await sharp(pixels, {
        raw: { width: info.width, height: info.height, channels: 3 },
      })
        .png()
        .toBuffer();
      return { png, width: info.width, height: info.height };
    })();
    marks.set(file, loading);
  }
  return loading;
}

export interface CardInput {
  title: string;
  /** Short labels over the title, e.g. section, date, reading time. */
  eyebrow: string[];
  /** The page URL label printed after the `$` prompt, e.g. `caro.sh/docs/cli`. */
  url: string;
}

export interface CardPlan {
  title: { size: number; lines: string[]; overflow: boolean };
  eyebrow: string;
  url: string;
  /** Characters the card's fonts have no glyph for; they would print as empty boxes. */
  missingGlyphs: string[];
}

interface ComposedCard {
  plan: CardPlan;
  parts: string[];
}

function compose(input: CardInput, fonts: Fonts, markWidth: number): ComposedCard {
  const right = OG_SIZE.width - PAD_X - markWidth - COLUMN_GAP;
  const columnWidth = right - PAD_X;
  const parts: string[] = [];

  // Eyebrow: as many labels as fit, dropped from the end.
  const label: TextOptions = { font: fonts.label, size: LABEL_SIZE, tracking: LABEL_TRACKING };
  let items = input.eyebrow.filter(Boolean);
  let eyebrow = items.join('  ·  ').toUpperCase();
  while (items.length > 1 && advance(eyebrow, label) > columnWidth) {
    items = items.slice(0, -1);
    eyebrow = items.join('  ·  ').toUpperCase();
  }
  eyebrow = fitLine(eyebrow, label, columnWidth);
  const eyebrowBaseline = PAD_TOP + LABEL_SIZE;
  parts.push(textPath(eyebrow, PAD_X, eyebrowBaseline, label, CARD_COLORS.muted));

  // Footer: `$ caro.sh/path`, the prompt in signal red.
  const prompt: TextOptions = { font: fonts.display, size: URL_SIZE };
  const urlText: TextOptions = { font: fonts.label, size: URL_SIZE };
  const footerBaseline = OG_SIZE.height - PAD_BOTTOM;
  const promptWidth = advance('$ ', prompt);
  const url = fitLine(input.url, urlText, columnWidth - promptWidth);
  parts.push(textPath('$', PAD_X, footerBaseline, prompt, CARD_COLORS.signal));
  parts.push(textPath(url, PAD_X + promptWidth, footerBaseline, urlText, CARD_COLORS.ink));

  // Title: the largest size whose balanced lines fit between the two.
  const measure = (text: string, size: number) => advance(text, { font: fonts.display, size });
  const titleTop = eyebrowBaseline + 52;
  const titleBottom = footerBaseline - URL_SIZE - 56;
  const fit = (sizes: readonly number[]) =>
    fitTitle(input.title, measure, {
      maxWidth: columnWidth,
      maxHeight: titleBottom - titleTop,
      lineHeight: TITLE_LINE_HEIGHT,
      sizes,
      maxLines: TITLE_MAX_LINES,
    });
  // Fewest lines first, then the larger size, among the preferred sizes that
  // fit; below them, the largest size that fits at all.
  const preferred = TITLE_SIZES.filter((size) => size >= TITLE_MIN_PREFERRED)
    .map((size) => fit([size]))
    .filter((result) => !result.overflow)
    .sort((a, b) => a.lines.length - b.lines.length || b.size - a.size);
  const title = preferred[0] ?? fit(TITLE_SIZES);
  const os2 = fonts.display.tables.os2 as { sCapHeight?: number } | undefined;
  const capHeight = (os2?.sCapHeight ?? 0.7 * fonts.display.unitsPerEm) / fonts.display.unitsPerEm;
  title.lines.forEach((line, i) => {
    const baseline = titleTop + title.size * capHeight + i * title.size * TITLE_LINE_HEIGHT;
    parts.push(
      textPath(line, PAD_X, baseline, { font: fonts.display, size: title.size }, CARD_COLORS.ink),
    );
  });

  const missing = new Set([
    ...missingGlyphs(input.title, fonts.display),
    ...missingGlyphs(`${eyebrow} ${url}`, fonts.label),
  ]);
  return { plan: { title, eyebrow, url, missingGlyphs: [...missing] }, parts };
}

/** Plans a card exactly as it would render, without encoding it. */
export async function planCard(input: CardInput, markFile: string): Promise<CardPlan> {
  const [fonts, mark] = await Promise.all([loadFonts(), loadMark(markFile)]);
  return compose(input, fonts, mark.width).plan;
}

/** Renders one card to PNG. */
export async function renderCard(input: CardInput, markFile: string): Promise<Buffer> {
  const [fonts, mark] = await Promise.all([loadFonts(), loadMark(markFile)]);
  const { parts } = compose(input, fonts, mark.width);
  const { width, height } = OG_SIZE;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><rect width="${width}" height="${height}" fill="${CARD_COLORS.paper}"/>${parts.join('')}</svg>`;
  return (
    sharp(Buffer.from(svg))
      .composite([
        {
          input: mark.png,
          left: width - PAD_X - mark.width,
          top: Math.round((height - mark.height) / 2),
        },
      ])
      // Everything lies on a few brand colors, so a 256-color palette is
      // effectively lossless and keeps each card small.
      .png({ palette: true, colours: 256, compressionLevel: 9 })
      .toBuffer()
  );
}
