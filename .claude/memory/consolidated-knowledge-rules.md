# Consolidated Knowledge Rules

This document contains the project's agreed-upon standards that must be followed across all documentation, code, and content.

## Rule 1: Canonical Installation Script

**ALWAYS use this exact installation command:**

```bash
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
```

**NEVER use these patterns:**
- `curl -sSL https://caro.sh/install.sh | bash` ❌
- `curl https://caro.sh/install.sh | bash` ❌
- `curl ... | sh` ❌
- Any pipe-to-shell pattern ❌
- Any reference to `caro.sh/install.sh` ❌

**Why the canonical format?**
- `bash <()` - Process substitution is more secure than pipe (no SIGPIPE issues)
- `--proto '=https'` - Enforces HTTPS-only connections
- `--tlsv1.2` - Requires TLS 1.2 minimum for security
- `-sSfL` - Silent, show errors, fail on HTTP errors, follow redirects
- `https://setup.caro.sh` - Canonical setup URL

**Source of truth:** `website/src/components/Download.astro`

## Rule 2: Internal Page Linking

When mentioning pages within the website, always include proper links:

| When you mention... | Link to... |
|---------------------|------------|
| "installation guide" or "download" | `/#download` |
| "FAQ" | `/faq` |
| "telemetry" or "privacy policy" | `/telemetry` |
| "support" or "funding" | `/support` |
| "roadmap" | `/roadmap` |
| "credits" or "acknowledgments" | `/credits` |
| "glossary" | `/glossary` |

**Example - WRONG:**
```
For other options, see our installation guide.
```

**Example - CORRECT:**
```html
For other options, see our <a href="/#download">installation guide</a>.
```

## Rule 3: Configuration Consistency

**Single source of truth for configuration:**

| Configuration | Canonical Location |
|--------------|-------------------|
| Site config (name, social, etc.) | `website/src/config/site.ts` |
| CLI version | `Cargo.toml` (version field) |
| Install script URL | `https://setup.caro.sh` only |
| Page metadata | Each page's `searchMeta` export |

**FORBIDDEN:**
- Hardcoding version numbers in content (use config)
- Using multiple install script URLs
- Duplicating config values across files

## Enforcement

These rules are enforced by:
1. **Git pre-push hook** - Blocks pushes with violations
2. **Constitution validator agent** - Manual validation
3. **Constitution validation skill** - Hookify integration
4. **PR reviews** - Human verification

## Remediation

### Finding violations:

```bash
# Find forbidden install patterns
grep -rn "caro.sh/install.sh" --include="*.md" --include="*.astro" .

# Find pipe-to-bash patterns
grep -rn "curl.*|.*bash" --include="*.md" --include="*.astro" .

# Find missing links for "installation guide"
grep -rn "installation guide" website/ | grep -v "href="
```

### Fixing violations:

1. Replace any `curl ... | bash` with the canonical command
2. Add proper `<a href="...">` links to page references
3. Use config values instead of hardcoded strings

---

*Last updated: 2025-01-04*
*Ratified as part of Constitution v1.1.0*
