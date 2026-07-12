/**
 * Design-system regression guards.
 *
 * These lock in the paper-and-ink brand migration so it cannot silently
 * regress: no deprecated orange, all manifest icons resolve to real files,
 * and the self-hosted Figtree face is actually declared and applied.
 *
 * Run: `npm test` (vitest). cwd is the `website/` package root.
 */
import { describe, it, expect } from 'vitest';
import { readdirSync, readFileSync, statSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = process.cwd();
const SRC = join(ROOT, 'src');

/** Deprecated Caro orange (migrated to signal-red #ef3333 / var(--accent)). */
const ORANGE =
  /#ff8c42|#ff6b35|#ffa76c|255,\s*140,\s*66|255,\s*107,\s*53|255,\s*167,\s*108/i;

const TEXT_EXT = /\.(astro|css|ts|tsx|js|mjs|cjs|json|md)$/;

/**
 * This branch's design pass now covers the WHOLE tree — including the files the
 * 4 in-flight design PRs (#1155/#1156/#1158/#1159) touched, which this branch
 * supersedes — so no file is exempt from the no-orange guard.
 */
const IN_FLIGHT_PR_FILES = new Set<string>([]);

/** This test file itself contains the orange regex literal — exclude it from its own scan. */
const SELF = 'src/design-system.regression.test.ts';

function walk(dir: string, out: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) walk(full, out);
    else if (TEXT_EXT.test(entry)) out.push(full);
  }
  return out;
}

function rel(abs: string): string {
  return abs.slice(ROOT.length + 1);
}

describe('design system: no deprecated orange', () => {
  it('has no #ff8c42/#ff6b35 orange outside in-flight PR files', () => {
    const offenders = walk(SRC)
      .filter((f) => ORANGE.test(readFileSync(f, 'utf8')))
      .map(rel)
      .filter((r) => r !== SELF && !IN_FLIGHT_PR_FILES.has(r));
    expect(
      offenders,
      `Deprecated orange must be var(--accent)/#ef3333. Found in:\n${offenders.join('\n')}`
    ).toEqual([]);
  });
});

describe('icon system: manifest references resolve', () => {
  const manifest = JSON.parse(
    readFileSync(join(SRC, 'data/icon-manifest.json'), 'utf8')
  ) as { icons: Record<string, { path: string }> };

  it('every declared icon points to a file that exists in public/', () => {
    const missing = Object.entries(manifest.icons)
      .map(([name, def]) => ({ name, file: join(ROOT, 'public', def.path) }))
      .filter(({ file }) => !existsSync(file));
    expect(
      missing.map((m) => m.name),
      `Manifest icons missing their SVG: ${missing.map((m) => m.file).join(', ')}`
    ).toEqual([]);
  });

  it('the brand mark and pixel logo assets exist', () => {
    expect(existsSync(join(ROOT, 'public/favicon.svg'))).toBe(true);
    expect(existsSync(join(ROOT, 'public/caro-pixel.png'))).toBe(true);
  });
});

describe('icon system: retired kyaro-mark glyph stays gone', () => {
  it('no component references the horned kyaro-mark glyph', () => {
    const offenders = walk(SRC)
      .filter((f) => /name=["']kyaro-mark["']/.test(readFileSync(f, 'utf8')))
      .map(rel)
      .filter((r) => r !== SELF);
    expect(
      offenders,
      `The kyaro-mark pixel-glyph was retired (horned read at <=32px); use <KyaroMark> (the real mascot sprite) instead. Found in:\n${offenders.join('\n')}`
    ).toEqual([]);
  });

  it('the icon manifest no longer declares kyaro-mark', () => {
    const manifest = JSON.parse(
      readFileSync(join(SRC, 'data/icon-manifest.json'), 'utf8')
    ) as { icons: Record<string, unknown> };
    expect(Object.keys(manifest.icons)).not.toContain('kyaro-mark');
  });
});

describe('typography: Figtree is self-hosted and applied', () => {
  const fontsCss = readFileSync(join(SRC, 'ui/fonts.css'), 'utf8');

  it('declares an @font-face for Figtree', () => {
    expect(fontsCss).toMatch(/@font-face/);
    expect(fontsCss).toMatch(/font-family:\s*['"]Figtree['"]/i);
  });

  it('every Figtree src file referenced by fonts.css exists in public/fonts', () => {
    const refs = [...fontsCss.matchAll(/url\(['"]?(\/fonts\/[^'")]+)['"]?\)/g)].map(
      (m) => m[1]
    );
    expect(refs.length).toBeGreaterThan(0);
    const missing = refs.filter((r) => !existsSync(join(ROOT, 'public', r.replace(/^\//, ''))));
    expect(missing, `Missing font files: ${missing.join(', ')}`).toEqual([]);
  });

  it('LandingPage applies the body + display fonts via tokens', () => {
    const lp = readFileSync(join(SRC, 'layouts/LandingPage.astro'), 'utf8');
    expect(lp).toMatch(/font-family:\s*var\(--font-body\)/);
    expect(lp).toMatch(/font-family:\s*var\(--font-display\)/);
    expect(lp).toMatch(/import\s+['"]\.\.\/ui\/fonts\.css['"]/);
  });
});
