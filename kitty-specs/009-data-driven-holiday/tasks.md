# Tasks: Data-Driven Holiday Theme Configuration System

## Work Packages Overview

| WP | Name | Status | Files |
|----|------|--------|-------|
| WP01 | Type Definitions | **completed** | config/types.ts |
| WP02 | Holiday Configuration | **completed** | config/holidays.ts |
| WP03 | Seasonal Configuration | **completed** | config/seasons.ts |
| WP04 | Date Utilities | **completed** | lib/date-utils.ts |
| WP05 | Locale Detector | **completed** | lib/locale-detector.ts |
| WP06 | Calendar Calculators | **completed** | lib/calendar/*.ts |
| WP07 | Holiday Theme Engine | **completed** | lib/holiday-engine.ts |
| WP08 | Layout Integration | pending | layouts/Layout.astro |
| WP09 | Navigation Integration | pending | components/Navigation.astro |
| WP10 | Testing & Validation | pending | (manual + automated) |

---

## WP01: Type Definitions

**Status**: pending
**Estimated effort**: Small

### Description
Create comprehensive TypeScript interfaces for the holiday and seasonal configuration system.

### Files
- Create: `website/src/config/types.ts`

### Tasks
- [ ] Define `HolidayEvent` interface
- [ ] Define `DateRange` interface with type discriminator
- [ ] Define `DefaultRule` interface
- [ ] Define `ThemeConfig` interface
- [ ] Define `SeasonalEffect` interface
- [ ] Define `LocaleInfo` interface
- [ ] Export all types

### Acceptance Criteria
- All interfaces are properly typed
- Discriminated unions for date range types
- JSDoc comments for complex fields

---

## WP02: Holiday Configuration

**Status**: pending
**Estimated effort**: Medium

### Description
Create the holiday configuration with initial set of holidays migrated from current implementation plus new additions.

### Files
- Create: `website/src/config/holidays.ts`

### Tasks
- [ ] Create Christmas configuration (global)
- [ ] Create Hanukkah configuration (global, Hebrew calendar)
- [ ] Create New Year configuration (global)
- [ ] Create Diwali configuration (global, Hindu calendar)
- [ ] Create Lunar New Year configuration (global)
- [ ] Create July 4th configuration (US locale-only)
- [ ] Create Yom Ha'atzmaut configuration (IL locale-only)
- [ ] Export holidays array and helper functions

### Acceptance Criteria
- All holidays have complete configuration
- Default rules properly set for locale-specific holidays
- Theme colors match existing implementation

---

## WP03: Seasonal Configuration

**Status**: pending
**Estimated effort**: Small

### Description
Create seasonal effects configuration separate from holiday themes.

### Files
- Create: `website/src/config/seasons.ts`

### Tasks
- [ ] Define winter season with snow effect
- [ ] Set date range (Dec 1 - Feb 28 for northern hemisphere)
- [ ] Add hemisphere awareness flag
- [ ] Export seasons array

### Acceptance Criteria
- Snow effect decoupled from Christmas
- Works during entire winter period
- Hemisphere flag for future southern support

---

## WP04: Date Utilities

**Status**: pending
**Estimated effort**: Medium

### Description
Create utility functions for date range checking and calculations.

### Files
- Create: `website/src/lib/date-utils.ts`

### Tasks
- [ ] Create `isDateInRange(date, range)` function
- [ ] Handle year boundary spanning (Dec-Jan)
- [ ] Create `getDateRangeForYear(range, year)` function
- [ ] Support fixed date ranges
- [ ] Create hook for calendar-based ranges
- [ ] Add test date override support (for development)

### Acceptance Criteria
- Correctly handles year boundaries
- Works with test date override
- Clean API for calendar integration

---

## WP05: Locale Detector

**Status**: pending
**Estimated effort**: Small

### Description
Create utility to detect user's locale, language, and approximate location.

### Files
- Create: `website/src/lib/locale-detector.ts`

### Tasks
- [ ] Create `detectLocale()` function
- [ ] Extract country from timezone
- [ ] Extract language from navigator
- [ ] Create timezone-to-country mapping for major regions
- [ ] Export `LocaleInfo` result

### Acceptance Criteria
- Correctly identifies IL for Asia/Jerusalem
- Correctly identifies US for America/* timezones
- Falls back gracefully when detection fails

---

## WP06: Calendar Calculators

**Status**: pending
**Estimated effort**: Medium-Large

### Description
Create calendar calculation modules for non-Gregorian date systems.

### Files
- Create: `website/src/lib/calendar/index.ts`
- Create: `website/src/lib/calendar/gregorian.ts`
- Create: `website/src/lib/calendar/hebrew.ts`

### Tasks
- [ ] Create calendar factory in index.ts
- [ ] Implement Gregorian fixed date calculator
- [ ] Implement Hebrew calendar date calculator (for Hanukkah)
- [ ] Create simplified Hanukkah date lookup table (2024-2030)
- [ ] Add placeholder for lunar calendar (Lunar New Year)
- [ ] Add placeholder for Hindu calendar (Diwali)

### Acceptance Criteria
- Hanukkah dates correct for next 5 years
- Extensible for future calendar types
- Minimal bundle size (lookup tables vs libraries)

---

## WP07: Holiday Theme Engine

**Status**: pending
**Estimated effort**: Large

### Description
Create the central orchestrator class that manages theme selection and visibility.

### Files
- Create: `website/src/lib/holiday-engine.ts`

### Tasks
- [ ] Create `HolidayThemeEngine` class
- [ ] Implement `getActiveHolidays()` method
- [ ] Implement `getVisibleThemes(locale)` method
- [ ] Implement `getDefaultTheme(locale)` method
- [ ] Implement `isSeasonActive(seasonId)` method
- [ ] Implement priority-based default selection
- [ ] Handle locale-only visibility filtering
- [ ] Create singleton instance export
- [ ] Add debug logging for development

### Acceptance Criteria
- Returns correct holidays for given date/locale
- Priority system works correctly
- Locale-only holidays properly filtered
- Easy to test with mock data

---

## WP08: Layout Integration

**Status**: pending
**Estimated effort**: Medium

### Description
Integrate the holiday engine with Layout.astro for early theme initialization.

### Files
- Modify: `website/src/layouts/Layout.astro`

### Tasks
- [ ] Import holiday engine
- [ ] Replace hardcoded `isHolidaySeason()` with engine
- [ ] Use engine for default theme selection
- [ ] Integrate seasonal effects with engine
- [ ] Maintain backward compatibility during transition
- [ ] Keep test date override working

### Acceptance Criteria
- No visible change in current behavior
- Engine provides theme data
- Snow effect controlled by seasons config
- Test date override still works

---

## WP09: Navigation Integration

**Status**: pending
**Estimated effort**: Medium

### Description
Integrate the holiday engine with Navigation.astro for dynamic menu population.

### Files
- Modify: `website/src/components/Navigation.astro`

### Tasks
- [ ] Import holiday engine
- [ ] Replace static theme options with `getVisibleThemes()`
- [ ] Dynamic menu rendering based on active holidays
- [ ] Update desktop dropdown to use engine data
- [ ] Update mobile select to use engine data
- [ ] Maintain user preference override logic
- [ ] Update theme icons dynamically

### Acceptance Criteria
- Menu shows only currently active holidays
- Locale-only holidays respect visibility rules
- User selection still overrides default
- No regression in toggle functionality

---

## WP10: Testing & Validation

**Status**: pending
**Estimated effort**: Medium

### Description
Comprehensive testing of the new system.

### Tasks
- [ ] Test Christmas visibility in December
- [ ] Test Hanukkah date calculation
- [ ] Test New Year visibility (Dec 31 - Jan 1)
- [ ] Test July 4th visibility (US only)
- [ ] Test Israeli locale detection
- [ ] Test default selection priority
- [ ] Test user preference persistence
- [ ] Test snow effect independence
- [ ] Visual testing with testDate parameter
- [ ] Test mobile and desktop menus

### Acceptance Criteria
- All date ranges work correctly
- Locale detection works for US, IL, FR
- Default selection follows priority rules
- No regression in user experience

---

## Dependencies

```
WP01 ──┬──► WP02 ──┬──► WP07 ──┬──► WP08 ──► WP10
       │          │          │
       ├──► WP03 ─┤          ├──► WP09 ──► WP10
       │          │          │
       └──► WP04 ─┴──► WP06 ─┘
             │
             └──► WP05 ───────────────────► WP07
```

## Implementation Order

Recommended sequence:
1. WP01 (Types) - Foundation
2. WP04 (Date Utils) - Core functionality
3. WP05 (Locale Detector) - Core functionality
4. WP06 (Calendar) - Date calculations
5. WP02 (Holidays Config) - Configuration
6. WP03 (Seasons Config) - Configuration
7. WP07 (Engine) - Integration
8. WP08 (Layout) - UI Integration
9. WP09 (Navigation) - UI Integration
10. WP10 (Testing) - Validation
