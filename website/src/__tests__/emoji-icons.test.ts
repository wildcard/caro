/**
 * Regression guard for emoji used as icons (docs/brand/ICONOGRAPHY.md,
 * docs/brand/icons/DEFINITIONS.md).
 *
 * The site is moving from emoji to Kyaro pixel icons and terminal glyphs,
 * slot by slot. Two checks keep that from sliding back:
 * - every <TermGlyph> uses a glyph from the terminal family
 * - the emoji count under src/ never grows. Lower EMOJI_BUDGET when a PR
 *   removes emoji; never raise it to make room for a new one.
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const EMOJI_BUDGET = 2135;

const GLYPHS = ['▸', '■', '$', '❯', '>_', '✓', '✗', '♥'];
// Pictographic code points that render as plain text in our fonts.
const TEXT_SYMBOLS = new Set(['©', '®', '™', '♥', '✓', '✗', '✕', '↔', '↩', '↪', '▶', '◀']);

const src = fileURLToPath(new URL('..', import.meta.url));

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === '__tests__' ? [] : sourceFiles(path);
    return /\.(astro|tsx?|jsx|mdx?|json|css)$/.test(name) ? [path] : [];
  });
}

const files = sourceFiles(src).map((path) => ({
  file: path.slice(src.length),
  text: readFileSync(path, 'utf8'),
}));

function emojiIn(text: string): string[] {
  return [...text.matchAll(/\p{Extended_Pictographic}/gu)]
    .map((m) => m[0])
    .filter((c) => !TEXT_SYMBOLS.has(c));
}

describe('emoji used as icons', () => {
  it('every <TermGlyph> uses a terminal glyph', () => {
    const bad = files.flatMap(({ file, text }) =>
      [...text.matchAll(/<TermGlyph\b[^>]*\bg="([^"]*)"/g)]
        .filter((m) => !GLYPHS.includes(m[1]))
        .map((m) => `${file}: g="${m[1]}"`),
    );
    expect(bad).toEqual([]);
  });

  it(`src/ holds at most ${EMOJI_BUDGET} emoji`, () => {
    const counts = files
      .map(({ file, text }) => ({ file, n: emojiIn(text).length }))
      .filter(({ n }) => n > 0);
    const total = counts.reduce((sum, { n }) => sum + n, 0);
    const where = counts.map(({ file, n }) => `${file}: ${n}`).join('\n');
    expect(total, `emoji per file:\n${where}`).toBeLessThanOrEqual(EMOJI_BUDGET);
  });
});
