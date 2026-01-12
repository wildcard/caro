
## Astro Blog Post Guidelines

**IMPORTANT**: When writing Astro blog posts (`.astro` files), curly braces `{` and `}` inside `<pre><code>` blocks are parsed as JSX expressions by esbuild, causing build failures.

**Always escape curly braces in code examples:**
- `{` → `&#123;`
- `}` → `&#125;`

**Also escape in Rust code:**
- `&` → `&amp;` (for references like `&self`)
- `<` → `&lt;` (for generics)
- `>` → `&gt;` (for generics)

**Pre-commit hook (husky)** automatically runs `npm run build` when website files are staged. If the build fails, the commit is blocked with helpful error messages.

To manually verify: `cd website && npm run build`
