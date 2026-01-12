
## Astro Blog Post Guidelines

**IMPORTANT**: When writing Astro blog posts (`.astro` files), curly braces `{` and `}` inside `<pre><code>` blocks are parsed as JSX expressions by esbuild, causing build failures.

**Always escape curly braces in code examples:**
- `{` → `&#123;`
- `}` → `&#125;`

**Also escape in Rust code:**
- `&` → `&amp;` (for references like `&self`)
- `<` → `&lt;` (for generics)
- `>` → `&gt;` (for generics)

**Before committing blog changes, always run:**
```bash
cd website && npm run build
```

The website has a pre-commit hook that enforces this automatically.
