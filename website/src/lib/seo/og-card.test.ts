import { fileURLToPath } from 'node:url';
import sharp from 'sharp';
import { describe, expect, it } from 'vitest';
import { CARD_COLORS, OG_SIZE, pathData, planCard, renderCard } from './og-card';
import { cardEyebrow, cardTitle, cardUrlLabel } from './routes';

const mark = fileURLToPath(new URL('../../../public/mark-caro-smooth.png', import.meta.url));

const card = (path: string, title: string, publishedTime?: string, readingTime?: string) => ({
  title: cardTitle(title),
  eyebrow: cardEyebrow({ pathname: path, publishedTime, readingTime }),
  url: cardUrlLabel(path),
});

describe('renderCard', () => {
  it('renders a 1200 × 630 PNG on the brand paper', async () => {
    const png = await renderCard(card('/docs/cli', 'CLI Reference - Caro Docs'), mark);
    const image = sharp(png);
    const { width, height, format } = await image.metadata();
    expect({ width, height, format }).toEqual({ ...OG_SIZE, format: 'png' });
    // The top-left corner is paper.
    const { data } = await image.extract({ left: 0, top: 0, width: 1, height: 1 }).raw().toBuffer({ resolveWithObject: true });
    const hex = `#${[...data.subarray(0, 3)].map((v) => v.toString(16).padStart(2, '0')).join('')}`;
    expect(hex).toBe(CARD_COLORS.paper);
  });
});

describe('planCard', () => {
  // The longest titles on the site when this test was written.
  it.each<[string, string, string?, string?]>([
    ['/blog/founders-playbook-adaptation', "Caro's founder arc: adopting Anthropic's playbook with a safety-first execution layer - Caro Blog", '2026-05-25T00:00:00.000Z', '8 min read'],
    ['/blog/claude-skill-launch', 'Introducing the Caro Claude Skill: Safe Shell Commands in Your AI Workflow - Caro Blog', '2025-12-23T00:00:00.000Z', '6 min read'],
    ['/use-cases/sre', 'Caro for SREs & On-Call Engineers | Safe Production Commands'],
    ['/compare/github-copilot-cli', 'Caro vs GitHub Copilot CLI - Feature Comparison | Shell AI Tools'],
  ])('fits %s without cutting the title', async (path, title, date, read) => {
    const plan = await planCard(card(path, title, date, read), mark);
    expect(plan.title.overflow).toBe(false);
    expect(plan.title.lines.join(' ')).toBe(cardTitle(title));
    expect(plan.missingGlyphs).toEqual([]);
  });

  it('drops eyebrow items from the end before it overflows', async () => {
    const plan = await planCard(
      { title: 'T', eyebrow: ['Blog', 'Dec 17, 2025', 'a very long reading time label that cannot fit'], url: 'caro.sh' },
      mark,
    );
    expect(plan.eyebrow).toBe('BLOG  ·  DEC 17, 2025');
  });

  it('reports characters the fonts cannot draw', async () => {
    const plan = await planCard({ title: 'החבר הנאמן שלך', eyebrow: ['Blog'], url: 'caro.sh/he' }, mark);
    expect(plan.missingGlyphs.length).toBeGreaterThan(0);
  });
});

describe('pathData', () => {
  it('writes finite numbers, rounded to two places', () => {
    expect(
      pathData([
        { type: 'M', x: 1, y: 2.000000000000004 },
        { type: 'L', x: -3.456, y: 4 },
        { type: 'Q', x1: 1, y1: 1, x: 2, y: 2 },
        { type: 'Z' },
      ]),
    ).toBe('M1 2.00L-3.46 4Q1 1 2 2Z');
  });
});
