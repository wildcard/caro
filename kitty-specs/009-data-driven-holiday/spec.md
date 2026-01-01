# Spec: Data-Driven Holiday Theme Configuration System

## Overview

Transform the current hardcoded holiday theme system into a data-driven, configurable architecture that supports timely, locale-aware holiday themes with intelligent default selection.

## Problem Statement

The current holiday theme implementation has several limitations:
1. **Hardcoded themes** - Adding new holidays requires code changes
2. **No locale awareness** - Cannot show Israeli holidays only in Israel
3. **Always visible** - All themes show in menu regardless of date relevance
4. **No separation of concerns** - Seasonal effects (snow) are tied to holidays
5. **Limited default rules** - Only basic timezone detection for Israel

## Goals

1. **Data-driven configuration** - Define holidays in a structured config file
2. **Timely visibility** - Themes only appear in menu during their active period
3. **Smart defaults** - Apply themes based on date + locale + language rules
4. **Local vs Global** - Support locale-specific vs worldwide holidays
5. **Seasons separation** - Decouple seasonal effects from holiday calendar
6. **Extensible** - Easy to add new holidays without code changes

## Non-Goals

- Implementing all 150+ holidays from the global plan (future work)
- Summer/regional seasonal effects (future work)
- User preference persistence beyond localStorage
- Server-side rendering of theme detection

## Core Concepts

### Holiday Event Configuration

Each holiday event should have:

```typescript
interface HolidayEvent {
  id: string;                    // Unique identifier (e.g., "christmas", "diwali")
  name: string;                  // Display name
  localizedNames?: Record<string, string>;  // Translations

  // Timing
  dateRange: DateRange;          // When theme is active

  // Visibility rules
  visibility: 'global' | 'locale-only';
  locales?: string[];            // ISO country codes if locale-only
  languages?: string[];          // ISO language codes for preference

  // Theme configuration
  theme: ThemeConfig;            // Colors, effects, decorations

  // Default selection rules
  defaultFor?: DefaultRule[];    // When to auto-select this theme
}

interface DateRange {
  type: 'fixed' | 'lunar' | 'hebrew' | 'islamic' | 'calculated';
  start: string | DateCalculation;
  end: string | DateCalculation;
}

interface DefaultRule {
  type: 'locale' | 'language' | 'timezone';
  values: string[];
  priority: number;  // Higher = more specific
}

interface ThemeConfig {
  primaryColor: string;
  secondaryColor: string;
  accentColor?: string;
  icon: string;       // Emoji for menu
  decorations?: string[];  // CSS class names
}
```

### Seasonal Effects Configuration

Separate from holidays:

```typescript
interface SeasonalEffect {
  id: string;           // e.g., "winter-snow"
  name: string;
  effect: string;       // Component name
  dateRange: DateRange;
  hemisphereAware: boolean;  // Flip for southern hemisphere
  enabled: boolean;     // Global toggle
}
```

## Visibility Rules

### Menu Visibility
1. Theme appears in menu ONLY during its active `dateRange`
2. Locale-only themes appear ONLY for matching locales
3. Global themes appear for everyone during their period

### Default Selection Priority

When user has no saved preference:

1. **Date check** - Is any holiday active right now?
2. **Locale match** - Does user's location match a locale-specific holiday?
3. **Language match** - Does user's language match a holiday preference?
4. **Fallback** - Default to 'none' (opt-in only)

Example priority:
- User in Israel, December ’ Hanukkah (locale match)
- User in USA, December ’ Christmas available, but defaults to 'none'
- User in India, October ’ Diwali (locale match)
- User in France, July 14 ’ Bastille Day (locale + date match)

## Implementation Approach

### Phase 1: Configuration Structure
1. Create `holiday-config.ts` with type definitions
2. Create `holidays.json` or embedded config with initial holidays
3. Create `seasons.json` for seasonal effects

### Phase 2: Theme Engine
1. Build `HolidayThemeEngine` class to process config
2. Implement date range calculations (including lunar calendars)
3. Implement locale/language detection
4. Build priority-based default selection

### Phase 3: UI Integration
1. Modify `Navigation.astro` to use engine
2. Dynamic menu population based on active holidays
3. Maintain user preference override

### Phase 4: Migration
1. Migrate existing themes (Christmas, Hanukkah, New Year) to config
2. Add 3-5 new holidays as proof of concept
3. Separate snow effect into seasonal config

## Initial Holiday Config

Start with these holidays:

| Holiday | Visibility | Locales | Date Range |
|---------|------------|---------|------------|
| Christmas | Global | - | Dec 1-25 |
| Hanukkah | Global | - | Hebrew calendar |
| New Year | Global | - | Dec 31 - Jan 1 |
| Diwali | Global | - | Hindu calendar |
| Lunar New Year | Global | - | Chinese calendar |
| Yom Ha'atzmaut | Locale-only | IL | Hebrew calendar |
| July 4th | Locale-only | US | Jul 1-4 |
| Bastille Day | Locale-only | FR | Jul 10-14 |

## Technical Considerations

### Calendar Libraries
- Hebrew: `@hebcal/core` or custom
- Chinese/Lunar: `chinese-lunar-calendar`
- Islamic: `hijri-converter`
- Hindu: API or calculation

### Locale Detection
1. `navigator.language` for language
2. `Intl.DateTimeFormat().resolvedOptions().timeZone` for rough location
3. Optional: IP-based geolocation (privacy concern)

### Performance
- Lazy load calendar calculations
- Cache computed dates per session
- Minimize bundle size with code splitting

## Success Criteria

1. [ ] Configuration-driven: Add holiday by editing config, no code changes
2. [ ] Timely: Themes only visible during their period
3. [ ] Locale-aware: Israeli holidays in Israel, French in France
4. [ ] User choice preserved: Manual selection overrides defaults
5. [ ] Seasonal separation: Snow works independently of holidays
6. [ ] Backward compatible: Existing themes work during migration

## Open Questions

1. How far in advance should themes be visible? (1 week? 2 weeks?)
2. Should we support "preview" mode for testing themes?
3. How to handle holidays that span midnight across timezones?
4. Should locale-only themes be completely hidden or just not defaulted?

## References

- Global Holiday Plan: `website/GLOBAL_HOLIDAY_THEMES_PLAN.md`
- Current Implementation: `website/src/components/Navigation.astro`
- Layout Theme Init: `website/src/layouts/Layout.astro`
