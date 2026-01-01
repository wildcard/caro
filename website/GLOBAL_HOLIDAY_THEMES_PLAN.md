# Global Holiday Themes Plan

## Overview

This document outlines a culturally inclusive approach to holiday themes for the Caro website. Rather than defaulting to any single holiday, all themes are **opt-in**, respecting users from all backgrounds and cultures.

## Current Implementation (v1.0)

- **Christmas** (December) - Western Christian tradition
- **Hanukkah** (December, varies) - Jewish tradition
- **New Year** (January 1) - Gregorian calendar celebration

All themes are now **opt-in by default** as of the latest update.

---

## Proposed Global Holiday Themes

### Tier 1: Major Global Celebrations (Billions of Participants)

These holidays are celebrated globally at a scale comparable to Christmas:

| Holiday | Date(s) | Region/Culture | Participants | Priority |
|---------|---------|----------------|--------------|----------|
| **Lunar New Year / Chinese New Year** | Jan/Feb (varies) | East Asia, SE Asia, Global diaspora | ~2 billion | HIGH |
| **Diwali** (Festival of Lights) | Oct/Nov (varies) | South Asia, Global diaspora | ~1.5 billion | HIGH |
| **Eid al-Fitr** (End of Ramadan) | Varies (lunar) | Global Islamic community | ~1.8 billion | HIGH |
| **Eid al-Adha** (Festival of Sacrifice) | Varies (lunar) | Global Islamic community | ~1.8 billion | HIGH |

### Tier 2: Major Regional/Cultural Celebrations

| Holiday | Date(s) | Region/Culture | Notes |
|---------|---------|----------------|-------|
| **Nowruz** (Persian New Year) | March 20-21 | Iran, Central Asia, Caucasus | Spring equinox, ~300M |
| **Holi** (Festival of Colors) | Feb/Mar (varies) | South Asia | Spring festival |
| **Easter** | Mar/Apr (varies) | Christian traditions | Already covered by spring |
| **Vesak** (Buddha Day) | May (varies) | Buddhist communities | Buddha's birthday |
| **Songkran** | April 13-15 | Thailand, SE Asia | Thai New Year |
| **Obon** | August | Japan | Ancestor remembrance |
| **Mid-Autumn Festival** | Sep/Oct (varies) | East Asia | Moon festival |

### Tier 3: Regional Celebrations

| Holiday | Date(s) | Region |
|---------|---------|--------|
| **Day of the Dead** | Nov 1-2 | Mexico, Latin America |
| **Carnival** | Feb/Mar (varies) | Brazil, Caribbean |
| **Thanksgiving** | November | USA, Canada |
| **Kwanzaa** | Dec 26 - Jan 1 | African diaspora |

---

## Implementation Priority

### Phase 1: Immediate (Next Release)
1. **Lunar New Year** - Largest celebration not yet implemented
   - Date range: Variable (Chinese lunar calendar), typically late Jan - mid Feb
   - Colors: Red, Gold
   - Symbols: Dragon, Lanterns, Red envelopes, Fireworks
   - Effects: Lantern glow, confetti, fireworks

2. **Diwali** - Festival of Lights
   - Date: Variable (Hindu lunar calendar), typically Oct/Nov
   - Colors: Gold, Orange, Deep Purple, Magenta
   - Symbols: Diyas (oil lamps), Rangoli patterns, Fireworks
   - Effects: Lamp glow, sparkle effects

### Phase 2: Cultural Expansion
3. **Eid al-Fitr** - End of Ramadan celebration
   - Date: Variable (Islamic lunar calendar)
   - Colors: Green, Gold, White
   - Symbols: Crescent moon, Stars, Lanterns
   - Effects: Moon glow, star sparkle

4. **Nowruz** - Persian New Year
   - Date: March 20-21 (Spring Equinox)
   - Colors: Green (rebirth), Gold, Sky Blue
   - Symbols: Haft-sin table elements, Goldfish, Flowers
   - Effects: Spring bloom, growth animation

### Phase 3: Expanded Regional Support
- Holi (colors/paint splash effects)
- Mid-Autumn Festival (moon and lantern theme)
- Day of the Dead (marigold and calavera theme)

---

## Technical Implementation Guidelines

### Date Detection
```javascript
// Example for Lunar New Year (simplified)
function isLunarNewYearPeriod() {
  const now = new Date();
  // Lunar New Year varies between Jan 21 - Feb 20
  // Use lunar calendar library for accurate dates
  // Show theme for 15 days (through Lantern Festival)
}
```

### Theme Structure (per holiday)
```css
.lunar-new-year {
  --holiday-primary: #de2910;    /* Chinese red */
  --holiday-secondary: #ffde00;  /* Gold */
  --holiday-accent: #ffffff;     /* White */
}
```

### Effect Components Needed
1. Lantern effect (floating lanterns)
2. Firework effect (shared with New Year)
3. Sparkle/glow effect (shared across themes)
4. Confetti effect (configurable colors)

---

## User Experience Principles

1. **Opt-in Only**: Never auto-apply any holiday theme
2. **Easy Discovery**: Show seasonal toggle when holidays are approaching
3. **Respectful Representation**: Research each holiday's significance with cultural consultants
4. **Performance**: Effects should be toggleable and respect reduced-motion preferences
5. **Accessibility**: Ensure color contrast ratios meet WCAG standards

---

## Cultural Sensitivity Guidelines

1. **Research**: Consult with people from each culture before implementation
2. **Avoid Stereotypes**: Use authentic symbols and representations
3. **Religious Respect**: Distinguish between religious and cultural celebrations
4. **Naming**: Use proper names (e.g., "Lunar New Year" not just "Chinese New Year")
5. **Timing**: Respect the actual celebration dates, not approximations

---

## Detection Strategy

### Option A: Date-Based (Recommended)
- Detect based on calendar dates
- Show theme toggle 1-2 weeks before and during the holiday
- Use accurate lunar calendar calculations where needed

### Option B: Location-Based (Privacy Concern)
- Detect timezone to suggest relevant themes
- Privacy-respecting: Use timezone only, not IP geolocation
- Always remain opt-in

### Option C: User Preference
- Allow users to set preferred holidays in settings
- Remember preferences across sessions

---

## Implementation Checklist

For each new holiday theme:

- [ ] Research cultural significance and symbolism
- [ ] Design color palette with cultural accuracy
- [ ] Create appropriate visual effects
- [ ] Implement accurate date detection
- [ ] Add theme toggle option
- [ ] Test with users from that culture
- [ ] Write documentation
- [ ] Add to theme selector menu

---

## References

- Lunar Calendar: https://www.timeanddate.com/calendar/
- Islamic Calendar: https://www.islamicfinder.org/
- Hindu Calendar: https://www.drikpanchang.com/
- Persian Calendar: https://www.time.ir/

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-01 | Initial plan, made all themes opt-in |
