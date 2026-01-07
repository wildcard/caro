/**
 * Seasonal Effects Configuration
 *
 * Seasonal effects are independent of holidays. They provide ambient
 * visual effects based on the time of year, optionally adjusting for
 * hemisphere differences.
 *
 * Currently supported:
 * - Winter: Snow effect (Dec-Feb northern, Jun-Aug southern)
 *
 * Future possibilities:
 * - Spring: Cherry blossom petals
 * - Summer: Sunshine/warmth effects
 * - Autumn: Falling leaves
 */

import type { SeasonalEffect } from './types';

// ============================================================================
// Seasonal Effect Definitions
// ============================================================================

export const SEASONS: SeasonalEffect[] = [
  {
    id: 'winter-snow',
    name: 'Winter Snow',
    effect: 'snow',
    dateRange: {
      type: 'fixed',
      startMonth: 12,
      startDay: 1,
      endMonth: 2,
      endDay: 28,
    },
    hemisphereAware: true,
    defaultEnabled: true,
    icon: '❄️',
  },

  // Future seasonal effects (disabled for now)
  {
    id: 'autumn-leaves',
    name: 'Autumn Leaves',
    effect: 'leaves',
    dateRange: {
      type: 'fixed',
      startMonth: 9,
      startDay: 21,
      endMonth: 11,
      endDay: 30,
    },
    hemisphereAware: true,
    defaultEnabled: false, // Not yet implemented
    icon: '🍂',
  },

  {
    id: 'spring-petals',
    name: 'Spring Blossoms',
    effect: 'petals',
    dateRange: {
      type: 'fixed',
      startMonth: 3,
      startDay: 20,
      endMonth: 5,
      endDay: 31,
    },
    hemisphereAware: true,
    defaultEnabled: false, // Not yet implemented
    icon: '🌸',
  },
];

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get a seasonal effect by ID
 */
export function getSeasonById(id: string): SeasonalEffect | undefined {
  return SEASONS.find((s) => s.id === id);
}

/**
 * Get all enabled seasonal effects
 */
export function getEnabledSeasons(): SeasonalEffect[] {
  return SEASONS.filter((s) => s.defaultEnabled);
}

/**
 * Flip date range for southern hemisphere
 * Winter (Dec-Feb) becomes (Jun-Aug), etc.
 */
export function flipForSouthernHemisphere(
  startMonth: number,
  endMonth: number
): { startMonth: number; endMonth: number } {
  // Add 6 months and wrap around
  const flip = (month: number) => ((month + 5) % 12) + 1;
  return {
    startMonth: flip(startMonth),
    endMonth: flip(endMonth),
  };
}
