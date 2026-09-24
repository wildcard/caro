# Holiday Configuration Quick Reference

## HolidayEvent Interface

```typescript
interface HolidayEvent {
  id: string;                    // Unique identifier (e.g., 'christmas', 'diwali')
  name: string;                  // Display name
  description: string;           // Brief description
  emoji: string;                 // Representative emoji

  dateRange: DateRange;          // When the holiday is active
  leadTimeDays?: number;         // Days before to start showing (default: 0)

  visibility: 'global' | 'locale-only';  // Who can see it
  locales?: string[];            // ISO country codes (e.g., ['US', 'CA'])

  defaultFor?: DefaultRule[];    // When to auto-apply

  theme: ThemeConfig;            // Visual configuration

  enabled: boolean;              // Master on/off switch
}
```

## DateRange Types

### Fixed (Same date every year)
```typescript
{
  type: 'fixed',
  startMonth: 12,      // 1-12
  startDay: 24,
  endMonth: 12,
  endDay: 26,
}
```

### Calculated (Pre-computed dates)
```typescript
{
  type: 'calculated',
  calculation: 'thanksgiving-us',  // Lookup key
}
```

### Hebrew Calendar
```typescript
{
  type: 'hebrew',
  hebrewHoliday: 'hanukkah',  // Key in hebrew.ts
}
```

## DefaultFor Rules

### By Locale (Highest Priority)
```typescript
{ type: 'locale', values: ['US', 'CA'], priority: 100 }
```

### By Language
```typescript
{ type: 'language', values: ['he', 'yi'], priority: 90 }
```

### By Timezone
```typescript
{ type: 'timezone', values: ['America/New_York'], priority: 80 }
```

### Wildcard (Everyone)
```typescript
{ type: 'locale', values: ['*'], priority: 50 }
```

## Theme Configuration

```typescript
theme: {
  colors: {
    primary: '#c41e3a',      // Main accent color
    secondary: '#228b22',    // Secondary accent
    tertiary: '#ffd700',     // Tertiary accent
    background: '#fff5f5',   // Light mode background tint
    backgroundDark: '#1f1a1a', // Dark mode background tint
  },
  effects: ['snow', 'confetti', 'sparkles'],  // Optional effects
}
```

## Country Codes Reference

| Code | Country | Timezone Examples |
|------|---------|-------------------|
| US | United States | America/New_York, America/Los_Angeles |
| CA | Canada | America/Toronto, America/Vancouver |
| IL | Israel | Asia/Jerusalem |
| IN | India | Asia/Kolkata |
| CN | China | Asia/Shanghai |
| JP | Japan | Asia/Tokyo |
| DE | Germany | Europe/Berlin |
| FR | France | Europe/Paris |
| AE | UAE | Asia/Dubai |
| ID | Indonesia | Asia/Jakarta |
| PH | Philippines | Asia/Manila |

## Example: Complete Holiday Entry

```typescript
{
  id: 'diwali',
  name: 'Diwali',
  description: 'Festival of Lights - Hindu celebration of light over darkness',
  emoji: '🪔',

  dateRange: {
    type: 'calculated',
    calculation: 'diwali',
  },
  leadTimeDays: 5,

  visibility: 'global',
  locales: ['IN', 'NP', 'LK', 'MY', 'SG', 'FJ', 'TT', 'GY', 'SR'],

  defaultFor: [
    { type: 'locale', values: ['IN', 'NP'], priority: 100 },
    { type: 'language', values: ['hi', 'ta', 'te', 'gu', 'mr'], priority: 90 },
    { type: 'locale', values: ['*'], priority: 50 },
  ],

  theme: {
    colors: {
      primary: '#ff9933',
      secondary: '#800080',
      tertiary: '#ffd700',
      background: '#fff8f0',
    },
    effects: ['sparkles', 'diyas'],
  },

  enabled: true,
}
```

## CSS Theme Template

```css
/* ============================================
   [HOLIDAY NAME] THEME
   Colors: [Color palette description]
   ============================================ */
.holiday-id {
  --color-accent: #primary;
  --color-accent-secondary: #secondary;
  --color-accent-tertiary: #tertiary;
  --color-bg-tertiary: #background;
  --holiday-primary: #primary;
  --holiday-secondary: #secondary;
}

.holiday-id.dark {
  --color-bg-tertiary: #backgroundDark;
  /* Adjust colors for dark mode visibility */
}

.holiday-id .cta-button,
.holiday-id .companion-badge {
  background: linear-gradient(135deg, var(--holiday-primary) 0%, var(--holiday-secondary) 100%) !important;
}

.holiday-id .logo {
  background: linear-gradient(135deg, var(--holiday-primary) 0%, var(--holiday-secondary) 100%) !important;
  -webkit-background-clip: text !important;
  background-clip: text !important;
}

.holiday-id .feature {
  border-color: rgba(/* primary RGB */, 0.2) !important;
}

.holiday-id .feature:hover {
  border-color: var(--holiday-primary) !important;
  box-shadow: 0 10px 30px rgba(/* primary RGB */, 0.15) !important;
}

.holiday-id .download-section {
  background: linear-gradient(135deg, var(--holiday-primary) 0%, var(--holiday-secondary) 100%) !important;
}
```

## Hero Decoration Template

```css
.holiday-id .hero::before {
  content: '🎉';           /* Primary emoji */
  position: absolute;
  top: 20px;
  left: 20px;
  font-size: 40px;
  animation: gentleGlow 2s ease-in-out infinite;
  opacity: 0.9;
}

.holiday-id .hero::after {
  content: '✨';           /* Secondary emoji */
  position: absolute;
  top: 20px;
  right: 80px;
  font-size: 36px;
  animation: sparkle 3s ease-in-out infinite;
  opacity: 0.8;
}
```

## Debug Panel Quick Access

- **Hotkey**: `Cmd+Shift+\` (Mac) / `Ctrl+Shift+\` (Windows/Linux)
- **URL**: Add `?holidayDebug=true` to any page

## File Locations

| File | Purpose |
|------|---------|
| `website/src/config/types.ts` | TypeScript interfaces |
| `website/src/config/holidays.ts` | Holiday configurations |
| `website/src/config/seasons.ts` | Seasonal effects |
| `website/src/lib/holiday-engine.ts` | Theme selection logic |
| `website/src/lib/locale-detector.ts` | Locale detection |
| `website/src/lib/calendar/hebrew.ts` | Hebrew calendar dates |
| `website/src/layouts/Layout.astro` | Theme CSS |
| `website/src/components/HolidayDebugPanel.astro` | Debug panel |
