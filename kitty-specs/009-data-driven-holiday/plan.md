# Plan: Data-Driven Holiday Theme Configuration System

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Website Frontend                          │
├─────────────────────────────────────────────────────────────┤
│  Layout.astro          Navigation.astro                      │
│  (early init)          (theme menu)                          │
│       │                      │                               │
│       └──────────┬───────────┘                               │
│                  ▼                                           │
│         ┌─────────────────┐                                  │
│         │ HolidayThemeEngine │ ◄── Core orchestrator        │
│         └────────┬────────┘                                  │
│                  │                                           │
│     ┌────────────┼────────────┐                              │
│     ▼            ▼            ▼                              │
│ ┌────────┐ ┌──────────┐ ┌──────────┐                        │
│ │Holidays│ │ Seasons  │ │ Locale   │                        │
│ │ Config │ │ Config   │ │ Detector │                        │
│ └────────┘ └──────────┘ └──────────┘                        │
│     ▼            ▼                                           │
│ ┌────────────────────────────┐                              │
│ │   Date Range Calculators   │                              │
│ │  (Gregorian, Hebrew, etc)  │                              │
│ └────────────────────────────┘                              │
└─────────────────────────────────────────────────────────────┘
```

## File Structure

```
website/src/
├── config/
│   ├── holidays.ts          # Holiday event definitions
│   ├── seasons.ts           # Seasonal effect definitions
│   └── types.ts             # TypeScript interfaces
├── lib/
│   ├── holiday-engine.ts    # Core engine class
│   ├── date-utils.ts        # Date range calculations
│   ├── locale-detector.ts   # Locale/language detection
│   └── calendar/
│       ├── gregorian.ts     # Standard dates
│       ├── hebrew.ts        # Jewish calendar
│       ├── lunar.ts         # Chinese/lunar calendar
│       └── index.ts         # Calendar factory
├── components/
│   ├── HolidayThemeProvider.astro  # Context provider
│   └── (existing components)
└── layouts/
    └── Layout.astro         # Modified for engine
```

## Implementation Phases

### Phase 1: Foundation (Core Types & Config)

**Goal**: Establish type-safe configuration structure

**Files to create**:
1. `website/src/config/types.ts` - Type definitions
2. `website/src/config/holidays.ts` - Holiday configurations
3. `website/src/config/seasons.ts` - Seasonal effects

**Key decisions**:
- Use TypeScript for type safety
- Embed config in code (not JSON) for tree-shaking
- Support both fixed dates and calculated dates

### Phase 2: Date & Locale Utilities

**Goal**: Build calculation and detection infrastructure

**Files to create**:
1. `website/src/lib/date-utils.ts` - Date range checking
2. `website/src/lib/locale-detector.ts` - User locale detection
3. `website/src/lib/calendar/gregorian.ts` - Fixed date calculator
4. `website/src/lib/calendar/hebrew.ts` - Hebrew calendar (Hanukkah, etc.)

**Dependencies**:
- Consider `@hebcal/core` for Hebrew dates (or lightweight custom)
- Use `Intl` APIs for locale detection

### Phase 3: Theme Engine

**Goal**: Central orchestrator for theme decisions

**Files to create**:
1. `website/src/lib/holiday-engine.ts` - Main engine class

**Engine responsibilities**:
- Get currently active holidays
- Determine default theme based on rules
- Filter visible themes for menu
- Check if seasonal effects should run

**API**:
```typescript
class HolidayThemeEngine {
  getActiveHolidays(): HolidayEvent[]
  getDefaultTheme(): string | null
  getVisibleThemes(): HolidayEvent[]
  isSeasonActive(seasonId: string): boolean
  getUserLocale(): LocaleInfo
}
```

### Phase 4: UI Integration

**Goal**: Connect engine to existing components

**Files to modify**:
1. `website/src/layouts/Layout.astro` - Use engine for init
2. `website/src/components/Navigation.astro` - Dynamic menu

**Changes**:
- Replace hardcoded holiday checks with engine calls
- Dynamic menu population from `getVisibleThemes()`
- Preserve user preference override logic

### Phase 5: Migration & Testing

**Goal**: Migrate existing themes, add new ones

**Tasks**:
1. Migrate Christmas, Hanukkah, New Year to config
2. Add Diwali, Lunar New Year as proof
3. Add one locale-only holiday (July 4th or Bastille Day)
4. Separate snow effect to seasonal config
5. Test all combinations

## Key Implementation Details

### Date Range Checking

```typescript
// Pseudo-code for date range check
function isDateInRange(now: Date, range: DateRange): boolean {
  const { start, end } = getResolvedDates(range, now.getFullYear());
  return now >= start && now <= end;
}

// Handle year boundaries (Dec 25 - Jan 1)
function getResolvedDates(range: DateRange, year: number) {
  // If end < start, spans year boundary
  // Adjust year accordingly
}
```

### Default Theme Priority

```typescript
function getDefaultTheme(activeHolidays: HolidayEvent[], locale: LocaleInfo): string {
  // 1. Filter to locale-matching holidays
  const localeMatches = activeHolidays.filter(h =>
    h.defaultFor?.some(rule =>
      rule.type === 'locale' && rule.values.includes(locale.country)
    )
  );

  // 2. If locale match, pick highest priority
  if (localeMatches.length > 0) {
    return localeMatches.sort((a, b) =>
      getMaxPriority(b) - getMaxPriority(a)
    )[0].id;
  }

  // 3. No locale match = no default (opt-in only)
  return 'none';
}
```

### Menu Visibility Logic

```typescript
function getVisibleThemes(allHolidays: HolidayEvent[], locale: LocaleInfo, now: Date): HolidayEvent[] {
  return allHolidays.filter(holiday => {
    // Must be in date range
    if (!isDateInRange(now, holiday.dateRange)) return false;

    // Locale-only holidays require locale match
    if (holiday.visibility === 'locale-only') {
      return holiday.locales?.includes(locale.country);
    }

    // Global holidays always visible when in range
    return true;
  });
}
```

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Hebrew calendar complexity | Start with approximation, refine later |
| Bundle size from calendar libs | Use code splitting, lazy load |
| Breaking existing functionality | Keep old code as fallback during migration |
| Timezone edge cases | Use user's local time, document behavior |

## Testing Strategy

1. **Unit tests**: Date calculations, priority logic
2. **Integration tests**: Engine with mock dates/locales
3. **Visual testing**: Manual with `?testDate=YYYY-MM-DD` param
4. **Locale testing**: Browser language/timezone override

## Success Metrics

- [ ] Add holiday = config change only (no new code)
- [ ] Menu shows only relevant holidays
- [ ] Israeli user in Dec sees Hanukkah default
- [ ] US user in Dec sees no default (opt-in)
- [ ] Snow effect works independently
- [ ] No regression in existing functionality
