# Coder Agent Rule: Shell Syntax in Astro/JSX Templates

**Applies to:** `.astro`, `.jsx`, `.tsx` files in `website/src/`

## The Problem

Astro 5.x uses **esbuild** to parse `.astro` template sections. esbuild treats
every `{` character in template content (HTML/text) as a JSX expression boundary.

Shell command syntax like these will **crash the build**:

```
<code>:(){ :|:& };</code>        <!-- fork bomb -->
<code>awk '{print $2}'</code>    <!-- awk command -->
<code>find . -exec ... {} +</code>  <!-- find placeholder -->
```

The `:` after `{` is not valid JSX, so the parser throws:
```
Expected "}" but found ":"
Unexpected ":"
```

## Solutions (in order of preference)

### 1. Move data to external `.ts` files (BEST)

For **frontmatter data arrays** containing shell commands:

```typescript
// src/data/use-cases.ts
export const useCases = [
  { output: "awk '{print $2}' | xargs kill" },
  { output: "find . -name '*.py' -exec ... {} +" },
];
```

```astro
---
import { useCases } from '../data/use-cases';
---
{useCases.map(u => <code>{u.output}</code>)}
```

✅ Works because `.ts` files are compiled by TypeScript, not esbuild JSX parser.

### 2. Astro expression for literal text in template

For **inline shell code** in HTML:

```astro
<!-- BAD (build crash) -->
<code>:(){ :|:& };</code>

<!-- GOOD (wrapping in expression tells Astro it's a string) -->
<code>:{':(){:|:&};:'}</code>
```

Or for longer blocks:
```astro
<code set:html={`awk '{print $2}' | xargs kill`}></code>
```

### 3. HTML entity escaping

For **pre-rendered code with highlights**:

```astro
<code>awk '&#123;print $2&#125;'</code>
```

⚠️ Only use in `<pre><code>` blocks that are already escaped HTML
(e.g., `<span>` highlighted). Not for JSX expressions.

## What NOT to do

❌ Don't use backtick template literals in double quotes for fork bombs:
```
<!-- Still crashes -->
<code>{`:{\`}:{\|:&\};:`}</code>
```

❌ Don't blanket-escape `{` and `}` across all template content:
This will break JSX expressions like `{variable}` and Astro directives.

❌ Don't use the `---` data-in-template pattern:
```
---\n// comment\n---\nconst data = [...];\n---\n<section>
```
This is invalid for Astro — always use exactly 2 `---` markers with data
either in frontmatter OR in an external file.

## Fix Pattern for Existing Problems

1. **Find all fork bomb patterns:**
   ```bash
   grep -rn ':(){\|:(){ :|' website/src/pages/ website/src/components/ --include='*.astro'
   ```

2. **Find all template-level `const` blocks with shell data:**
   Look for files with 3+ `---` markers:
   ```bash
   grep -rln '^---$' website/src/ | xargs -I{} sh -c 'grep -c "^---$" {} | grep -v "^2$"'
   ```

3. **Fix:**
   - Extract `const` arrays to `website/src/data/<name>.ts`
   - Add import in frontmatter
   - Reduce `---` markers to exactly 2

## Verified Working Patterns

| Context | Safe Approach | Example |
|---------|--------------|---------|
| Frontmatter data array | External `.ts` file | `import { data } from '../data/file'` |
| Inline `<code>` shell text | JSX expression string | `<code>:{':(){:|:&};:'}</code>` |
| Highlighted code blocks | HTML entities `&#123;` | In `<pre><code>` span trees |
| Template `{{.Name}}` | `{'{{.Name}}'}` or external file | Docker inspect examples |

## Lessons Learned (Prevent Repetition)

### L1: Translation Key Mismatch Between Component and JSON
**Date:** 2026-04-21
**Symptom:** Root `/` page showed raw keys like `landing.hero.headline.line1`
instead of text. Component referenced keys that didn't exist in any locale JSON.
**Root Cause:** `hero.json` existed with flat `{ "hero": { ... } }` structure,
but `LPHero.astro` expected nested `{ "landing": { "hero": { "headline": { ... } } } }`.
The Spanish `landing.json` had these keys, but English didn't — creating a
silent fallback-to-raw-key behavior.
**Fix:** Added missing keys (`headline`, `socialProof`, `trustBadges`,
`cta.primary/secondary`) to `en/landing.json` hero section.
**Rule:** When a component uses `t(lang, 'path.to.key')`, verify the key exists
in ALL locale files (especially `en/`). A missing EN key causes raw key display
on ALL locale pages since EN is the fallback base.

### L2: Pattern Quality - No Literal `...` Placeholders
**Date:** 2026-04-21
**Symptom:** `dangerous-commands.ts` had `curl ... | sudo bash` — the literal
`...` doesn't match real-world patterns and reduces effectiveness.
**Fix:** Changed to `curl https://example.com | sudo bash` — a realistic pattern.
**Rule:** All dangerous command patterns must use realistic, matchable syntax.
Never use ellipsis (`...`) as placeholder text.

### L3: Pattern Quality - No Exact Duplicates
**Date:** 2026-04-21
**Symptom:** `ddSda: 'dd if=/dev/zero of=/dev/sda'` was identical to `ddZero`,
wasting a pattern slot and potentially causing confusion.
**Fix:** Renamed to `ddTruncate: 'dd if=/dev/zero of=/dev/sda bs=1M'` with
differentiated parameters.
**Rule:** When adding patterns, check for existing near-duplicates. Each pattern
should cover a distinct scenario or variant.
