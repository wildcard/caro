// Ported with og-text.ts from kobi.kadosh.me (apps/site/src/brand/og-text.test.ts).
import { describe, expect, it } from 'vitest';
import { balancedWrap, fitTitle, type Measure, wrap } from './og-text';

// A monospace stand-in: every character is 0.5 em wide.
const measure: Measure = (text, size) => text.length * size * 0.5;

const base = { lineHeight: 1.1, maxLines: 4 } as const;

describe('wrap', () => {
  it('fills each line greedily', () => {
    expect(wrap(['aaaa', 'bb', 'cc', 'dddd'], 10, 40, measure)).toEqual(['aaaa bb', 'cc dddd']);
  });

  it('never drops an over-wide word', () => {
    expect(wrap(['aaaaaaaaaaaa', 'b'], 10, 20, measure)).toEqual(['aaaaaaaaaaaa', 'b']);
  });
});

describe('balancedWrap', () => {
  it('evens out lines without adding one', () => {
    const tokens = 'one two three four five six seven'.split(' ');
    const greedy = wrap(tokens, 10, 140, measure);
    const balanced = balancedWrap(tokens, 10, 140, measure);
    expect(balanced).toHaveLength(greedy.length);
    const spread = (lines: string[]) =>
      Math.max(...lines.map((l) => l.length)) - Math.min(...lines.map((l) => l.length));
    expect(spread(balanced)).toBeLessThanOrEqual(spread(greedy));
  });
});

describe('fitTitle', () => {
  it('uses the largest size that fits', () => {
    // One line of room: 80 needs two lines, 60 fits on one.
    const result = fitTitle('Disposable Software', measure, {
      ...base,
      maxWidth: 600,
      maxHeight: 80,
      sizes: [80, 60, 40],
    });
    expect(result.size).toBe(60);
    expect(result.lines).toEqual(['Disposable Software']);
  });

  it('wraps rather than shrinks when there is height to spare', () => {
    const result = fitTitle('Disposable Software', measure, {
      ...base,
      maxWidth: 600,
      maxHeight: 300,
      sizes: [80, 60, 40],
    });
    expect(result).toEqual({ size: 80, lines: ['Disposable', 'Software'], overflow: false });
  });

  it('breaks after a subtitle colon when that fits', () => {
    const result = fitTitle('Silhouette: Building a Logo in Stages', measure, {
      ...base,
      maxWidth: 700,
      maxHeight: 300,
      sizes: [60, 50, 40],
    });
    expect(result.lines[0]).toBe('Silhouette:');
    expect(result.lines.join(' ')).toBe('Silhouette: Building a Logo in Stages');
  });

  it('never strands a number at the start of a line', () => {
    const result = fitTitle('The Feedback Loop Series Part 1: Test-Driven Development', measure, {
      ...base,
      maxWidth: 300,
      maxHeight: 400,
      sizes: [20],
    });
    expect(result.lines.some((line) => /^1:/.test(line))).toBe(false);
    expect(result.lines.join(' ')).toBe('The Feedback Loop Series Part 1: Test-Driven Development');
  });

  it('never starts a line with a lone "&"', () => {
    // 14 characters a line: without the rule, "& Secure" would open line 3.
    const result = fitTitle('Caro for Air-Gapped & Secure Environments', measure, {
      ...base,
      maxWidth: 140,
      maxHeight: 400,
      sizes: [20],
    });
    expect(result.lines.some((line) => line.startsWith('&'))).toBe(false);
    expect(result.lines.join(' ')).toBe('Caro for Air-Gapped & Secure Environments');
  });

  it('keeps a single over-wide word inside the column', () => {
    const result = fitTitle('https://example.com/a-very-long-unbroken-path-segment', measure, {
      ...base,
      maxWidth: 200,
      maxHeight: 400,
      sizes: [20],
    });
    expect(result.overflow).toBe(true);
    expect(result.lines[0]?.endsWith('…')).toBe(true);
    for (const line of result.lines) expect(measure(line, 20)).toBeLessThanOrEqual(200);
  });

  it('keeps every word when the title fits', () => {
    const title =
      'AI Is Not Your New Compiler: Vibe Engineering, the Ghost in the Machine, and the Future of Software Fundamentals';
    const result = fitTitle(title, measure, {
      ...base,
      maxLines: 5,
      maxWidth: 620,
      maxHeight: 300,
      sizes: [64, 56, 48, 44, 40, 36],
    });
    expect(result.lines.join(' ')).toBe(title);
    expect(result.lines.length).toBeLessThanOrEqual(5);
  });

  it('ends on an ellipsis only when nothing fits', () => {
    const result = fitTitle('word '.repeat(80).trim(), measure, {
      ...base,
      maxLines: 2,
      maxWidth: 200,
      maxHeight: 100,
      sizes: [40, 20],
    });
    expect(result.size).toBe(20);
    expect(result.overflow).toBe(true);
    expect(result.lines).toHaveLength(2);
    expect(result.lines[1]?.endsWith('…')).toBe(true);
    for (const line of result.lines) expect(measure(line, 20)).toBeLessThanOrEqual(200);
  });
});
