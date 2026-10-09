# Development Process - Caro

## Branch and PR Workflow

1. **ALWAYS work on a feature branch**, never on `main`
   ```bash
   # Use the project's helper if available
   bin/sk-new-feature "description"
   
   # OR manually
   git checkout -b fix/description-NNNN
   ```

2. **Commit early, push often, push to remote when done**
   - Every meaningful change should be committed
   - Always push to `origin` when work is complete
   - The AGENTS.md "Landing the Plane" rules apply

3. **Create a PR for review**
   ```bash
   gh pr create --title "fix(scope): concise description" --body "description"
   ```

4. **Verify CI before requesting review**
   - All Rust builds pass
   - All unit tests pass
   - Vercel deploys succeed (especially `caro-foss-website`)

## CI/CD Pipeline

### Rust Tests (GitHub Actions)
- `cargo test` — unit tests across all modules
- `cargo clippy` — lints with zero warnings
- Smoke tests with SmolLM 135M on macOS + Linux
- Evaluation suite for embedded models

### Website Deploy (GitHub Pages + Vercel)
- **caro-foss-website** → Vercel (primary website at caro.sh)
- **caro-docs** → Vercel (documentation)
- **GitHub Pages** → `deploy-website.yml` workflow (secondary)

### Release Workflow
```bash
# Pre-release grooming
caro release.acceptance  # dry-run audit
caro release.publish     # create tag + GitHub release
```

The release workflow (`.github/workflows/release.yml`) extracts the `CHANGELOG.md`
section for the version being released and uses it as the GitHub Release body.

## Frontend Build

```bash
cd website
npm run build      # Build for production (58 pages generated)
npm run dev        # Dev server with hot reload
```

**Known issue (#873):** esbuild v0.25 in Astro 5.x treats `{` in template content as JSX expressions.
**Fix patterns are documented in:** `.claude/rules/astro-esbuild-shell-syntax.md`

### Common Patterns
| What you want | How to do it |
|--------------|-------------|
| Shell commands in frontmatter data | Extract to `src/data/<name>.ts` |
| Fork bomb in `<code>` tag | `<code>{':(){:|:&};:'}</code>` |
| Highlighted code with braces | HTML entities: `&#123;` |

## Translation Workflow

Website uses Astro i18n routing with 15 locales. Source strings live in
`website/src/i18n/locales/en/*.json`. Non-English locales override keys
and fall back to English for missing keys.

```bash
cd website
# Check translation coverage
node scripts/i18n/status.mjs

# Validate all translations
node scripts/i18n/validate.mjs --strict

# Add new English strings
edit src/i18n/locales/en/*.json

# Run auto-translate (GitHub Actions)
gh workflow run translate.yml
```

Full i18n guide: `website/I18N_TRANSLATION_GUIDE.md`

## Code Style

- **Rust:** `cargo fmt`, `cargo clippy -- -D warnings`, `thiserror` for error types
- **TypeScript:** strict mode, no `any`, explicit return types
- **Astro:** prefer `.ts` data files over frontmatter for complex data
- **Commit messages:** conventional commits (`fix:`, `feat:`, `docs:`, `chore:`)

## Testing Standards

- Unit tests for all new Rust modules
- Integration tests for safety validation
- Website tests run via `vitest` in `website/`
- No false positives in safety patterns (validated by extensive test suite)
