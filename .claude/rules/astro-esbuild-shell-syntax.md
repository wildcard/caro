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
