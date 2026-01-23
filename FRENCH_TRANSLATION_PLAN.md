# French Translation Plan

**Target:** Improve French coverage from 62.4% to 90%+
**Cultural Context:** Paris - sophisticated, elegant, precise
**Locale Code:** `fr`

## Current Status

Based on latest status check:
- Current coverage: ~62.4% (204/327 keys)
- Target coverage: 90% (294/327 keys)
- Keys to translate: ~90 keys

## Cultural Context (Paris)

### Tone & Style
- **Sophisticated, elegant, precise**
- Avoid anglicisms - use French equivalents:
  - "courriel" not "email"
  - "logiciel" not "software"
  - "télécharger" not "downloader"
- Formal register with "vous" (not "tu")
- Balance professionalism with modern tech culture

### Pop Culture References
- Cinema (Nouvelle Vague)
- Fashion
- Café culture
- Wine appreciation
- Tour de France

## Translation Command

```bash
export ANTHROPIC_API_KEY='[new-key-here]'
TARGET_LOCALE=fr TRANSLATION_BACKEND=claude node .github/scripts/translate-multi-backend.js
```

## Expected Results

After translation:
- French coverage: ~90% (294/327 keys)
- All 10 JSON files translated with Paris cultural context
- Build verified for French route `/fr/`
- Validation passed with protected terms preserved

## Quality Checklist

- [ ] Anglicisms avoided (use French equivalents)
- [ ] Formal "vous" used consistently
- [ ] Technical terms preserved (Caro, Claude, POSIX, CLI)
- [ ] Placeholders maintained ({count}, {name}, etc.)
- [ ] Build succeeds without errors
- [ ] Validation passes

## Post-Translation

After completing French:
1. Verify coverage: `node scripts/i18n/status.mjs | grep "^fr"`
2. Validate translations: `node scripts/i18n/validate.mjs`
3. Test build: `cd website && npm run build`
4. Commit changes
5. Proceed to German (Berlin context)

## Next Priority Languages (Tier 1)

1. ✅ Spanish (es) - 84.1% - Complete
2. 🔄 French (fr) - 62.4% → 90% - In Progress
3. ⏭️ German (de) - 60.9% → 90% - Next
4. ⏭️ Portuguese (pt) - 53.2% → 90% - After German
5. ⏭️ Japanese (ja) - 62.7% → 90% - After Portuguese
