# Tasks: Data-Driven Holiday Theme Configuration System

**Feature Status**: COMPLETED
**Completion Date**: 2026-01-02
**Known Issues**: Debug panel date selection - see `docs/development/TECH_DEBT.md`

---

## Work Packages Overview

| WP | Name | Status | Files |
|----|------|--------|-------|
| WP01 | Type Definitions | completed | config/types.ts |
| WP02 | Holiday Configuration | completed | config/holidays.ts |
| WP03 | Seasonal Configuration | completed | config/seasons.ts |
| WP04 | Date Utilities | completed | lib/date-utils.ts |
| WP05 | Locale Detector | completed | lib/locale-detector.ts |
| WP06 | Calendar Calculators | completed | lib/calendar/*.ts |
| WP07 | Holiday Theme Engine | completed | lib/holiday-engine.ts |
| WP08 | Layout Integration | completed | layouts/Layout.astro |
| WP09 | Theme CSS + Decorations | completed | layouts/Layout.astro |
| WP10 | Debug Panel + Testing | completed | components/HolidayDebugPanel.astro |

---

## Summary of Completed Work

### WP01: Type Definitions
- Created `website/src/config/types.ts`
- Defined all TypeScript interfaces: HolidayEvent, DateRange, DefaultRule, ThemeConfig, SeasonalEffect, LocaleInfo
- Used discriminated unions for date range types
- Added JSDoc comments for complex fields

### WP02: Holiday Configuration
- Created `website/src/config/holidays.ts`
- Configured 15+ holidays including Christmas, Hanukkah, Diwali, Lunar New Year, Holi, Eid
- Set up locale-specific holidays: July 4th (US), Bastille Day (FR), German Unity (DE), Yom Ha'atzmaut (IL)
- Implemented priority-based default rules

### WP03: Seasonal Configuration
- Created `website/src/config/seasons.ts`
- Defined winter season with snow effect
- Added hemisphere awareness for future support
- Decoupled snow from Christmas theme

### WP04: Date Utilities
- Created `website/src/lib/date-utils.ts`
- Implemented getCurrentDate(), isDateInRange(), resolveDateRange()
- Added test date override support via window.__CARO_TEST_DATE
- Handled year boundary calculations

### WP05: Locale Detector
- Created `website/src/lib/locale-detector.ts`
- Implemented timezone-to-country mapping for major regions
- Added detectLocale(), matchesLocale(), matchesLanguage()
- Supports 50+ timezone mappings

### WP06: Calendar Calculators
- Created `website/src/lib/calendar/hebrew.ts`
- Pre-calculated Hebrew holiday dates for 2024-2030
- Created calendar factory in `website/src/lib/calendar/index.ts`

### WP07: Holiday Theme Engine
- Created `website/src/lib/holiday-engine.ts`
- Implemented HolidayThemeEngine class
- Methods: getActiveHolidays(), getVisibleThemes(), getDefaultTheme(), getAppliedTheme()
- Priority-based default selection (locale > language > wildcard)

### WP08: Layout Integration
- Updated `website/src/layouts/Layout.astro`
- Added import for HolidayDebugPanel
- Integrated theme initialization logic

### WP09: Theme CSS + Decorations
- Added CSS themes for all holidays in Layout.astro
- Created hero decorations with animations
- Light and dark mode support for all themes

### WP10: Debug Panel + Testing
- Created `website/src/components/HolidayDebugPanel.astro`
- Hotkey: Cmd+Shift+\ (Mac) or Ctrl+Shift+\ (Windows/Linux)
- Features: date picker, quick presets, locale override, theme preview
- **Known Issue**: Date selection doesn't apply themes (parked - see TECH_DEBT.md)
- **Workaround**: Use Preview Theme buttons directly

---

## Files Created/Modified

### New Files
- `website/src/config/types.ts` - TypeScript interfaces
- `website/src/config/holidays.ts` - Holiday configurations
- `website/src/config/seasons.ts` - Seasonal effects
- `website/src/lib/date-utils.ts` - Date utilities
- `website/src/lib/locale-detector.ts` - Locale detection
- `website/src/lib/calendar/hebrew.ts` - Hebrew calendar
- `website/src/lib/calendar/index.ts` - Calendar factory
- `website/src/lib/holiday-engine.ts` - Theme engine
- `website/src/components/HolidayDebugPanel.astro` - Debug panel

### Modified Files
- `website/src/layouts/Layout.astro` - Theme CSS and integration

---

## Related Documentation

- **Tech Debt**: `docs/development/TECH_DEBT.md` - Debug panel issue
- **Skill**: `.claude/skills/multicultural-holidays/SKILL.md` - Feature expertise
- **Agent**: `.claude/agents/cultural-heritage-expert.md` - Cultural guidance
- **Holiday Plan**: `website/GLOBAL_HOLIDAY_THEMES_PLAN.md` - Regional holidays roadmap
