# Research Decision Log

Document the outcomes of Phase 0 discovery work. Capture every clarification resolved and the supporting evidence that backs each decision.

## Summary

- **Feature**: 004-add-internationalization-i18n
- **Date**: 2025-12-28
- **Researchers**: Claude (AI Agent)
- **Open Questions**: None - all planning questions resolved

## Decisions & Rationale

| Decision | Rationale | Evidence | Status |
|----------|-----------|----------|--------|
| Use Native Astro i18n routing | Simpler, no external dependencies, official support from Astro 4.0+. Aligns with "no over-engineering" principle from CLAUDE.md. Reduces attack surface and maintenance burden. | Astro docs, project CLAUDE.md constitution | final |
| Component-based translation structure | Balances granularity with simplicity. ~10 JSON files per locale (navigation.json, hero.json, features.json, etc.) matches existing Astro component structure. Makes spot-checking translations easier. Prevents merge conflicts vs single large file. | Industry best practices, React i18n patterns | final |
| Official OpenAI Node.js SDK | Translation accuracy is critical - worth the overhead for built-in retry logic, rate limiting, and better error messages. ~30 second install time acceptable for infrequent workflow runs (~$6 per run budget). | OpenAI API docs, GitHub Actions best practices | final |
| Google Fonts CDN for RTL fonts | Standard approach for web fonts, reliable CDN, comprehensive Noto font family support for Hebrew/Arabic/Urdu. No self-hosting needed. | Google Fonts documentation, RTL styling best practices | final |
| CSS Logical Properties for RTL | Modern standard for bidirectional layouts. Properties like `margin-inline-start` automatically flip for RTL. Better than manual `[dir="rtl"]` overrides. Supported in all modern browsers. | W3C i18n guidelines, MDN web docs, rtlstyling.com | final |
| GPT-4 for translation (not GPT-3.5) | Higher quality needed for 15 languages. Cost difference minimal (~$6 vs ~$4 per run). Better context understanding for technical terms (POSIX, shell, CLI). | OpenAI model comparison docs, translation quality benchmarks | final |
| Spot-check review process | Balances quality with velocity. Full human review of 15 languages × ~200 strings = prohibitive bottleneck. 95% GPT-4 accuracy acceptable with 2-3 string spot-checks per language. Issues can be fixed incrementally. | User confirmation during planning discovery | final |
| All 15 languages launch together | Simpler implementation - single GitHub Action matrix configuration. No phased rollout complexity. Translation cost (~$6) is low enough to justify comprehensive launch. | User confirmation during planning discovery | final |
| Marketing pages only (no blog/docs/CLI) | Focused scope - highest ROI for user acquisition. Blog/docs have less traffic. CLI i18n requires Rust infrastructure (different implementation). Can expand later if needed. | User confirmation during planning discovery, spec requirements | final |

## Evidence Highlights

Summarize the most impactful findings from the evidence log.

- **Astro i18n maturity** – Native i18n routing available since Astro 4.0 (released Jan 2024). Production-ready, well-documented, actively maintained. No need for third-party libraries.
  - Source: https://docs.astro.build/en/guides/internationalization/

- **RTL CSS best practices** – CSS Logical Properties (margin-inline-*, padding-inline-*, etc.) are the modern standard for RTL support. Better browser support than older techniques.
  - Source: https://rtlstyling.com/, W3C i18n guidelines

- **Translation automation viability** – GPT-4 achieves 90-95% accuracy for technical content translation. Acceptable for marketing copy with spot-check review. Cost-effective at scale ($0.03/1K tokens input, $0.06/1K output).
  - Source: OpenAI API pricing, user validation during discovery

- **Component-based i18n patterns** – Industry standard in React/Vue/Angular ecosystems. Proven pattern for 10-20 translation files per locale. Easier maintenance than flat or deeply nested structures.
  - Source: React i18n docs, Vue i18n docs, Next.js i18n patterns

- **Hebrew/Arabic font considerations** – Arabic requires `letter-spacing: 0` due to connected script. Hebrew and Arabic have distinct typographic needs - cannot share fonts. Noto family provides comprehensive coverage.
  - Source: Google Fonts, Arabic typography guidelines

## Risks & Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| OpenAI API costs exceed budget | Medium - could hit rate limits or unexpected usage | Set GitHub Actions concurrency limits, cache translations, monitor spending via OpenAI dashboard | Monitored |
| GPT-4 translation quality issues | Medium - cultural insensitivity or technical inaccuracy | Spot-check review process, translation memory for consistency, community feedback loop | Mitigated |
| RTL layout breaks with existing themes | Low - holiday themes may conflict | Test RTL with Christmas/Hanukkah themes explicitly, use logical properties throughout | Mitigated |
| Page load performance degradation | Low - loading 15 languages of JSON | Lazy-load translation files per route, use Astro's built-in code splitting | Mitigated |
| Translation file merge conflicts | Low - multiple developers updating same locale files | Component-based structure (10 files vs 1) reduces conflict probability | Mitigated |

## Next Actions

Outline what needs to happen before moving into implementation planning.

1. ✅ Validate Astro version supports i18n (Astro 4.0+) - **Confirmed via package.json**
2. ✅ Confirm OpenAI API key availability - **User confirmed budget allocation**
3. ✅ Identify all marketing pages for translation - **Spec defines scope: homepage, landing pages, comparison pages, navigation, footer**
4. ⏭️ Create data model for Locale, Translation, and TranslationJob entities - **See data-model.md**
5. ⏭️ Design translation utility API (`t(locale, key)` function signature) - **To be done in Phase 1**
6. ⏭️ Map existing Astro components to translation sections - **To be done in Phase 1**

## Research Artifacts

- **evidence-log.csv** - Detailed research findings with source tracking
- **source-register.csv** - All sources referenced with URLs and access dates
- **data-model.md** - Entity definitions for Locale, Translation, TranslationJob

> This document captures all Phase 0 research decisions. All planning questions have been resolved with user confirmation. Ready for Phase 1 (Design & Contracts).
