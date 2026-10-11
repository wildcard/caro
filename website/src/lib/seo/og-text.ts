// Title layout for the share cards: pick the largest size at which a title
// fits its box, wrap it in balanced lines, and prefer breaking after a
// subtitle colon ("Why Caro? / The Story Behind…"). Pure, so it is testable
// without fonts: callers pass a `measure` that returns an advance width.
//
// Ported unchanged from kobi.kadosh.me (apps/site/src/brand/og-text.ts), the
// card system this site's cards follow.

/** Advance width of `text` at font size `size`, in the same units as `maxWidth`. */
export type Measure = (text: string, size: number) => number;

interface FitOptions {
  maxWidth: number;
  maxHeight: number;
  /** Line height as a multiple of the font size. */
  lineHeight: number;
  /** Candidate font sizes, largest first. */
  sizes: readonly number[];
  maxLines: number;
}

interface FitResult {
  size: number;
  lines: string[];
  /**
   * True when the title doesn't fit even at the smallest size, so it was cut
   * with an ellipsis (or one word overruns the width). A card should never
   * ship like this: shorten the title or give the post an `og.title`.
   */
  overflow: boolean;
}

/**
 * Splits a title into wrap units. A bare number rides with the word before it,
 * so "Part 1:" never breaks across lines.
 */
function words(text: string): string[] {
  const tokens: string[] = [];
  for (const token of text.trim().split(/\s+/).filter(Boolean)) {
    const previous = tokens[tokens.length - 1];
    if (previous && /^\d+[:.,;]?$/.test(token)) tokens[tokens.length - 1] = `${previous} ${token}`;
    else tokens.push(token);
  }
  return tokens;
}

/** Greedy wrap: as many words per line as fit in `width`. */
export function wrap(tokens: string[], size: number, width: number, measure: Measure): string[] {
  const lines: string[] = [];
  let line = '';
  for (const token of tokens) {
    const candidate = line ? `${line} ${token}` : token;
    if (line && measure(candidate, size) > width) {
      lines.push(line);
      line = token;
    } else {
      line = candidate;
    }
  }
  if (line) lines.push(line);
  return lines;
}

/**
 * Balanced wrap: the same number of lines greedy wrapping needs at `maxWidth`,
 * but at the narrowest width that still needs no more lines, so the lines come
 * out even instead of a long line over a short one.
 */
export function balancedWrap(
  tokens: string[],
  size: number,
  maxWidth: number,
  measure: Measure
): string[] {
  const target = wrap(tokens, size, maxWidth, measure).length;
  if (target <= 1) return wrap(tokens, size, maxWidth, measure);
  let lo = 0;
  let hi = maxWidth;
  for (let i = 0; i < 24; i++) {
    const mid = (lo + hi) / 2;
    if (wrap(tokens, size, mid, measure).length <= target) hi = mid;
    else lo = mid;
  }
  return wrap(tokens, size, hi, measure);
}

function fits(lines: string[], size: number, measure: Measure, options: FitOptions): boolean {
  return (
    lines.length <= options.maxLines &&
    lines.length * size * options.lineHeight <= options.maxHeight &&
    lines.every((line) => measure(line, size) <= options.maxWidth)
  );
}

/** Splits "Head: rest of the title" after the first colon, if it has one. */
function colonSplit(text: string): [string[], string[]] | null {
  const match = /^(.+?:)\s+(.+)$/.exec(text.trim());
  if (!match) return null;
  return [words(match[1] as string), words(match[2] as string)];
}

/**
 * Cuts `text` to fit `maxWidth` and ends it on an ellipsis: whole words first,
 * then characters, so even a single over-wide word (a URL, a hash) stays inside
 * the column.
 */
function ellipsize(text: string, size: number, maxWidth: number, measure: Measure): string {
  const trim = (value: string) => value.replace(/[\s,:;.–—-]+$/, '');
  let tokens = words(text);
  while (tokens.length > 1 && measure(`${trim(tokens.join(' '))}…`, size) > maxWidth) {
    tokens = tokens.slice(0, -1);
  }
  let kept = trim(tokens.join(' '));
  while (kept.length > 1 && measure(`${kept}…`, size) > maxWidth) {
    kept = trim(kept.slice(0, -1));
  }
  return `${kept}…`;
}

export function fitTitle(text: string, measure: Measure, options: FitOptions): FitResult {
  const tokens = words(text);
  const split = colonSplit(text);

  for (const size of options.sizes) {
    if (split) {
      const [head, tail] = split;
      const lines = [
        ...balancedWrap(head, size, options.maxWidth, measure),
        ...balancedWrap(tail, size, options.maxWidth, measure),
      ];
      if (fits(lines, size, measure, options)) return { size, lines, overflow: false };
    }
    const lines = balancedWrap(tokens, size, options.maxWidth, measure);
    if (fits(lines, size, measure, options)) return { size, lines, overflow: false };
  }

  // Nothing fits even at the smallest size: keep the lines that fit the box and
  // end on an ellipsis. Every returned line stays within `maxWidth`, so the card
  // never spills, while `overflow` tells the check to reject it.
  const size = options.sizes[options.sizes.length - 1] as number;
  const byHeight = Math.floor(options.maxHeight / (size * options.lineHeight));
  const room = Math.max(1, Math.min(options.maxLines, byHeight));
  const lines = wrap(tokens, size, options.maxWidth, measure);
  const cut = lines.length > room;
  const kept = lines.slice(0, room).map((line, i) => {
    if (cut && i === room - 1) {
      return ellipsize(`${line} ${lines[room]}`, size, options.maxWidth, measure);
    }
    return measure(line, size) > options.maxWidth
      ? ellipsize(line, size, options.maxWidth, measure)
      : line;
  });
  return { size, lines: kept, overflow: true };
}
