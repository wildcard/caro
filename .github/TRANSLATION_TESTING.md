# Translation Workflow Testing Guide

This guide covers testing the automated translation workflow both locally and via GitHub Actions.

## Prerequisites

1. **OpenAI API Key**: Required for GPT-4 translations
   ```bash
   export OPENAI_API_KEY="sk-proj-..."
   ```

2. **Node.js 20+**: Required for running translation script
   ```bash
   node --version  # Should be v20.x or higher
   ```

3. **Repository Setup**: Clone and navigate to repo
   ```bash
   git clone <repo-url>
   cd caro
   ```

## Local Testing

### Quick Test (Single Locale)

Test translation for a single locale without triggering GitHub Actions:

```bash
# Set API key
export OPENAI_API_KEY="sk-proj-..."

# Test Spanish translation
./.github/scripts/test-translation-locally.sh es

# Test Hebrew (RTL) translation
./.github/scripts/test-translation-locally.sh he

# Force retranslation (ignore cache)
export FORCE_RETRANSLATE=true
./.github/scripts/test-translation-locally.sh fr
```

### Manual Test (Step by Step)

1. **Install dependencies**:
   ```bash
   npm install openai
   ```

2. **Set environment variables**:
   ```bash
   export OPENAI_API_KEY="sk-proj-..."
   export TARGET_LOCALE="es"
   export FORCE_RETRANSLATE="false"
   ```

3. **Run translation script**:
   ```bash
   node .github/scripts/translate.js
   ```

4. **Verify output**:
   ```bash
   # Check translated files
   ls -lah website/src/i18n/locales/es/

   # Inspect translation
   cat website/src/i18n/locales/es/common.json
   ```

5. **Test website**:
   ```bash
   cd website
   npm run dev

   # Visit http://localhost:4321/es/
   ```

### Verify Translation Quality

Check that the translation:

- ✅ Preserves all JSON keys (only values are translated)
- ✅ Preserves placeholders: `{count}`, `{name}`, `{var}`
- ✅ Preserves brand names: "Caro", "Claude", "GitHub"
- ✅ Preserves technical terms: POSIX, shell, CLI, MLX
- ✅ Preserves code blocks and commands unchanged
- ✅ Preserves emoji and special characters
- ✅ Valid JSON syntax (no parse errors)
- ✅ RTL languages: Text is natural, technical terms stay LTR

## GitHub Actions Testing

### Test Workflow Trigger

The workflow triggers automatically on:
1. **Push to main** with changes to `website/src/i18n/locales/en/**/*.json`
2. **Manual workflow dispatch** (force retranslation option)

### Automatic Trigger Test

1. **Create test branch**:
   ```bash
   git checkout -b test/translation-workflow
   ```

2. **Add test string to English JSON**:
   ```bash
   # Edit website/src/i18n/locales/en/common.json
   # Add: "test": "This is a test string"
   ```

3. **Commit and push**:
   ```bash
   git add website/src/i18n/locales/en/common.json
   git commit -m "test: Add test translation string"
   git push origin test/translation-workflow
   ```

4. **Create PR to main**:
   ```bash
   gh pr create --title "test: Translation workflow test" --body "Testing automated translations"
   ```

5. **Merge PR** (or push to main directly if you have permissions)

6. **Monitor workflow**:
   ```bash
   # Watch workflow runs
   gh run list --workflow=translate.yml

   # View specific run logs
   gh run view <run-id> --log
   ```

### Manual Workflow Dispatch Test

Test the workflow without changing English files:

1. **Trigger via GitHub UI**:
   - Go to: `Actions` → `Translate Website Content` → `Run workflow`
   - Select branch: `main`
   - Check `force_retranslate` to force all translations
   - Click `Run workflow`

2. **Trigger via CLI**:
   ```bash
   # Normal run (skip cached translations)
   gh workflow run translate.yml

   # Force retranslation (ignore cache)
   gh workflow run translate.yml -f force_retranslate=true
   ```

3. **Monitor progress**:
   ```bash
   # List recent runs
   gh run list --workflow=translate.yml --limit 5

   # Watch live logs
   gh run watch
   ```

## Verify GitHub Action Output

### Check Workflow Logs

1. **View matrix job logs**:
   ```bash
   gh run view <run-id> --log
   ```

   Look for:
   - `[es] Found X files to translate`
   - `[es] ✓ Wrote common.json`
   - `[es] Translation completed!`
   - `[es] Summary: X translated, Y skipped (cached), Z failed`

2. **Check for errors**:
   - `✗ Error translating...` - OpenAI API errors
   - `✗ Failed to process...` - File I/O errors
   - `ERROR: OPENAI_API_KEY...` - Missing secret

### Verify Pull Requests

The workflow creates one PR per locale:

1. **List auto-generated PRs**:
   ```bash
   gh pr list --label i18n --label automated
   ```

2. **Review PR details**:
   ```bash
   gh pr view <pr-number>
   ```

   Check for:
   - Title: `i18n: Update {locale} translations`
   - Labels: `i18n`, `automated`
   - Branch: `i18n/auto-translate-{locale}`
   - Checklist in PR body

3. **Review translated files**:
   ```bash
   gh pr diff <pr-number>

   # Or checkout PR locally
   gh pr checkout <pr-number>
   git diff main website/src/i18n/locales/es/
   ```

4. **Test PR locally**:
   ```bash
   gh pr checkout <pr-number>
   cd website
   npm run dev
   # Visit http://localhost:4321/es/
   ```

## Translation Cache Testing

The workflow uses MD5 hashing to skip unchanged files:

### Test Cache Behavior

1. **First run** (no cache):
   - All files translated
   - Cache created: `website/src/i18n/locales/.translation-cache.json`

2. **Second run** (with cache, no changes):
   ```bash
   ./.github/scripts/test-translation-locally.sh es
   ```
   - Output: `[es] ⊘ Skipping common.json (unchanged, cached)`
   - Zero translations, all skipped

3. **After English file changes**:
   - Edit `website/src/i18n/locales/en/common.json`
   - Run translation again
   - Output: `[es] ✓ Wrote common.json` (only changed file)

4. **Force retranslation**:
   ```bash
   export FORCE_RETRANSLATE=true
   ./.github/scripts/test-translation-locally.sh es
   ```
   - All files retranslated regardless of cache

### Inspect Cache File

```bash
cat website/src/i18n/locales/.translation-cache.json
```

Expected format:
```json
{
  "es": {
    "common.json": {
      "sourceHash": "abc123...",
      "timestamp": "2025-12-29T10:54:50.000Z"
    }
  }
}
```

## Troubleshooting

### OpenAI API Errors

**Error**: `OPENAI_API_KEY environment variable is not set`
- **Fix**: Add secret to GitHub repo settings or export locally

**Error**: `Rate limit exceeded`
- **Fix**: Script has 1s delay between files; check OpenAI tier limits
- **Workaround**: Run workflow with `max-parallel: 1` in matrix

**Error**: `JSON parsing failed`
- **Fix**: Script has automatic retry with stricter prompt
- **Debug**: Check OpenAI response in logs for markdown formatting

### GitHub Actions Errors

**Error**: `create-pull-request: No file changes detected`
- **Cause**: All translations were cached, no changes made
- **Fix**: Either modify English JSON or use `force_retranslate: true`

**Error**: `permissions denied`
- **Fix**: Verify workflow has `contents: write` and `pull-requests: write`

**Error**: `Matrix job failed`
- **Fix**: Check individual locale logs for specific errors
- **Retry**: Re-run failed jobs in GitHub UI

### Translation Quality Issues

**Issue**: Placeholders translated (e.g., `{count}` became `{contador}`)
- **Fix**: Report to OpenAI or add stronger prompt constraints
- **Manual**: Review and fix in PR before merging

**Issue**: Brand names translated (e.g., "Caro" became "Expensive")
- **Fix**: Add to PRESERVE list in system prompt
- **Manual**: Review and fix in PR before merging

**Issue**: RTL text displays incorrectly
- **Fix**: Verify `Layout.astro` has RTL CSS rules
- **Test**: Visit `/{locale}/` and check text direction

## Test Checklist

Before merging translation PRs:

- [ ] All matrix jobs succeeded (14/14 locales)
- [ ] PRs created for each locale
- [ ] Translated JSON files have valid syntax
- [ ] Placeholders preserved: `{count}`, `{name}`, etc.
- [ ] Brand names preserved: "Caro", "Claude", etc.
- [ ] Technical terms preserved: POSIX, CLI, shell, etc.
- [ ] Emoji and special characters intact
- [ ] RTL languages (he, ar, ur) render correctly
- [ ] Website builds without errors: `npm run build`
- [ ] All locale routes accessible: `/{locale}/`
- [ ] No console errors in browser

## Cost Estimation

**Per full translation** (all 14 locales):
- Files: ~8 JSON files per locale
- Input tokens: ~1000-2000 per file
- Output tokens: ~1000-2000 per file
- Cost: ~$0.30-0.50 per locale
- **Total**: ~$4-7 per full run

**With caching** (incremental changes):
- Only changed files are retranslated
- Cost: $0.30-0.50 per locale per changed file

**Recommendation**: Use caching for iterative development, force retranslation only for major content updates.

## Next Steps

After successful testing:

1. **Merge translation PRs**: Review and merge auto-generated PRs
2. **Set up OPENAI_API_KEY secret**: Add to repository settings
3. **Monitor first real run**: Watch workflow on actual English content changes
4. **Adjust max-parallel**: Tune based on OpenAI rate limits
5. **Create release**: Deploy translated website
