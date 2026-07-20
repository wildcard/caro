import { describe, expect, it } from 'vitest';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const SRC = join(process.cwd(), 'src');

function walk(directory: string): string[] {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

describe('Caro design-system brand mark', () => {
  it('keeps the retired kyaro glyph out of components and the icon pack', () => {
    const offenders = walk(SRC)
      .filter((file) => /\.(astro|tsx?)$/.test(file))
      .filter((file) => /name=["']kyaro-mark["']/.test(readFileSync(file, 'utf8')))
      .map((file) => relative(SRC, file));

    expect(offenders).toEqual([]);
    expect(existsSync(join(process.cwd(), 'public/icons/kyaro-mark.svg'))).toBe(false);

    const manifest = JSON.parse(
      readFileSync(join(SRC, 'data/icon-manifest.json'), 'utf8'),
    ) as { icons: Record<string, unknown> };
    expect(manifest.icons).not.toHaveProperty('kyaro-mark');
  });

  it('keeps the generated Kyaro image family available on legacy landing surfaces', () => {
    const mark = readFileSync(join(SRC, 'components/KyaroMark.astro'), 'utf8');
    const navigation = readFileSync(
      join(SRC, 'components/landing/LPNavigation.astro'),
      'utf8',
    );
    const personas = readFileSync(
      join(SRC, 'components/landing/LPPersonas.astro'),
      'utf8',
    );

    expect(mark).toContain('<img');
    expect(mark).not.toContain('<svg');
    expect(mark).toContain("variant?: 'idle' | 'happy' | 'alert'");
    expect(mark).toContain('/brand/kyaro/kyaro-idle-web.png');
    expect(mark).toContain('/brand/kyaro/kyaro-happy-web.png');
    expect(mark).toContain('/brand/kyaro/kyaro-alert-web.png');
    expect(navigation).toContain('<KyaroMark');
    expect(personas.match(/<KyaroMark/g)).toHaveLength(3);
    expect(personas).toContain('variant="happy"');
    expect(personas).toContain('variant="alert"');
    expect(personas).toContain('variant="idle"');
  });

  it('ships high-resolution RGBA PNGs rather than custom vectors', () => {
    for (const variant of ['idle', 'happy', 'alert']) {
      const file = join(
        process.cwd(),
        `public/brand/kyaro/kyaro-${variant}.png`,
      );
      const png = readFileSync(file);
      expect(png.subarray(1, 4).toString()).toBe('PNG');
      expect(png.readUInt32BE(16)).toBe(1024);
      expect(png.readUInt32BE(20)).toBe(1024);
      // PNG IHDR colour type 6 = truecolour with alpha.
      expect(png[25]).toBe(6);

      const webPng = readFileSync(
        join(process.cwd(), `public/brand/kyaro/kyaro-${variant}-web.png`),
      );
      expect(webPng.subarray(1, 4).toString()).toBe('PNG');
      expect(webPng.readUInt32BE(16)).toBe(256);
      expect(webPng.readUInt32BE(20)).toBe(256);
      expect(webPng[25]).toBe(6);
    }
  });

  it('keeps the icon family grounded in all Kyaro source states', () => {
    const grounding = join(
      process.cwd(),
      'docs/brand/kyaro-icon-system',
    );
    const palette = JSON.parse(
      readFileSync(join(grounding, 'kyaro-source-palette.json'), 'utf8'),
    ) as { frame_count: number; top_colours: Array<{ hex: string }> };

    expect(existsSync(join(grounding, 'kyaro-state-grounding.jpg'))).toBe(true);
    expect(existsSync(join(grounding, 'kyaro-all-frames-contact-sheet.jpg'))).toBe(true);
    expect(palette.frame_count).toBe(99);
    expect(palette.top_colours.slice(0, 4).map(({ hex }) => hex)).toEqual([
      '#343434',
      '#000000',
      '#FFFFFF',
      '#D9A066',
    ]);
  });
});

describe('Claude Design homepage handoff', () => {
  it('keeps the imported Claude source and production entrypoint connected', () => {
    const root = process.cwd();
    const source = join(root, 'design-system/claude');
    const page = readFileSync(join(SRC, 'pages/index.astro'), 'utf8');

    expect(existsSync(join(source, 'HANDOFF.md'))).toBe(true);
    expect(existsSync(join(source, 'README.md'))).toBe(true);
    expect(existsSync(join(source, 'manifest.json'))).toBe(true);
    expect(existsSync(join(source, 'landing-template/index.html'))).toBe(true);
    expect(page).toContain('import BrandHomepageLayout');
    expect(page).toContain('import ClaudeHomepage');
    expect(page).not.toContain('LPVideoDemo');
    expect(page).not.toContain('LPPersonas');
  });

  it('uses the paper-and-ink identity without deprecated visual treatments', () => {
    const homepage = readFileSync(
      join(SRC, 'components/brand/ClaudeHomepage.astro'),
      'utf8',
    );
    const tokens = readFileSync(join(SRC, 'ui/caro-brand.css'), 'utf8');
    const brandSurface = `${homepage}\n${tokens}`;

    expect(tokens).toContain('--caro-beige-100: #f4f1df');
    expect(tokens).toContain('--caro-grey-700: #4f4f4f');
    expect(tokens).toContain('--caro-red-500: #ef3333');
    expect(tokens).toContain("--font-display: 'Azeret Mono'");
    expect(tokens).toContain("--font-body:    'Figtree'");
    expect(brandSurface).not.toMatch(/linear-gradient|radial-gradient/);
    expect(brandSurface).not.toMatch(/#ff8c42|#ff6b35/i);
  });

  it('uses only official raster marks and Kyaro sprite states on the homepage', () => {
    const homepage = readFileSync(
      join(SRC, 'components/brand/ClaudeHomepage.astro'),
      'utf8',
    );
    const publicRoot = join(process.cwd(), 'public/brand/caro');

    expect(homepage).toContain('<CaroBrandLockup');
    expect(homepage).toContain('<KyaroSprite state="idle"');
    expect(homepage).toContain('<KyaroSprite state="shocked"');
    expect(homepage).toContain('<KyaroSprite state="happy"');
    expect(homepage).not.toContain('kyaro-mark.svg');
    expect(existsSync(join(publicRoot, 'logo-caro-horizontal.png'))).toBe(true);
    expect(existsSync(join(publicRoot, 'kyaro/idle.gif'))).toBe(true);
    expect(existsSync(join(publicRoot, 'kyaro/happy-bounce.gif'))).toBe(true);
    expect(existsSync(join(publicRoot, 'kyaro/shocked.gif'))).toBe(true);
  });

  it('preserves Claude voice and casing rules in the flagship copy', () => {
    const homepage = readFileSync(
      join(SRC, 'components/brand/ClaudeHomepage.astro'),
      'utf8',
    );

    expect(homepage).toContain('Your loyal <span>shell companion.</span>');
    expect(homepage).toContain('Built like a shell.<br />Feels like a friend.');
    expect(homepage).toContain('She pauses on the dangerous stuff.');
    expect(homepage).not.toContain('Your Loyal Shell Companion');
    expect(homepage).not.toContain('Built Like A Shell');
  });
});
