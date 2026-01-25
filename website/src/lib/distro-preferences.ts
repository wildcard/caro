/**
 * Distro Preferences Store
 *
 * Manages user OS/shell preferences with:
 * - localStorage persistence
 * - PostHog analytics integration
 * - Auto-detection fallback
 * - Event-based updates for cross-component sync
 */

import type {
  DistroPreferences,
  DetectedSystem,
  OSFamily,
  LinuxDistro,
  WindowsEdition,
  MacOSVersion,
  ShellType,
} from '../config/distro-types';
import { detectFromUserAgent } from './user-agent-detector';

// ============================================================================
// Constants
// ============================================================================

const STORAGE_KEY = 'caro-distro-preferences';
const PREFERENCES_CHANGE_EVENT = 'caro:distro-preferences-change';

// ============================================================================
// Types
// ============================================================================

export interface PreferencesChangeEvent extends CustomEvent {
  detail: {
    preferences: DistroPreferences;
    source: 'user' | 'auto' | 'init';
  };
}

// ============================================================================
// PostHog Integration
// ============================================================================

/**
 * Track preference change with PostHog
 */
function trackPreferenceChange(
  preferences: DistroPreferences,
  previousPreferences: DistroPreferences | null,
  source: 'user' | 'auto' | 'init'
): void {
  if (typeof window === 'undefined' || !window.posthog) return;

  // Set user properties for filtering
  window.posthog.register({
    os_family: preferences.osFamily,
    distro: preferences.distro || 'unknown',
    shell: preferences.shell,
    preferences_manually_set: preferences.isManuallySet,
  });

  // Track change event if user manually changed
  if (source === 'user') {
    window.posthog.capture('distro_preferences_changed', {
      os_family: preferences.osFamily,
      distro: preferences.distro,
      shell: preferences.shell,
      terminal: preferences.terminal,
      previous_os_family: previousPreferences?.osFamily,
      previous_distro: previousPreferences?.distro,
      previous_shell: previousPreferences?.shell,
    });
  }

  // Track initial detection
  if (source === 'init' && !preferences.isManuallySet) {
    window.posthog.capture('distro_auto_detected', {
      os_family: preferences.osFamily,
      distro: preferences.distro,
      shell: preferences.shell,
    });
  }
}

// ============================================================================
// Storage Functions
// ============================================================================

/**
 * Load preferences from localStorage
 */
function loadFromStorage(): DistroPreferences | null {
  if (typeof window === 'undefined') return null;

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return null;

    const parsed = JSON.parse(stored);

    // Validate the structure
    if (
      parsed &&
      typeof parsed.osFamily === 'string' &&
      typeof parsed.shell === 'string'
    ) {
      return parsed as DistroPreferences;
    }
  } catch {
    // Invalid data, ignore
  }

  return null;
}

/**
 * Save preferences to localStorage
 */
function saveToStorage(preferences: DistroPreferences): void {
  if (typeof window === 'undefined') return;

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
  } catch {
    // Storage full or disabled, ignore
  }
}

// ============================================================================
// Event Dispatch
// ============================================================================

/**
 * Dispatch preference change event for cross-component sync
 */
function dispatchChangeEvent(
  preferences: DistroPreferences,
  source: 'user' | 'auto' | 'init'
): void {
  if (typeof window === 'undefined') return;

  const event = new CustomEvent(PREFERENCES_CHANGE_EVENT, {
    detail: { preferences, source },
  });
  window.dispatchEvent(event);
}

// ============================================================================
// Main API
// ============================================================================

/**
 * Initialize preferences - loads from storage or auto-detects
 */
export function initializePreferences(): DistroPreferences {
  // Try loading from storage first
  const stored = loadFromStorage();

  if (stored) {
    // Dispatch init event for PostHog
    trackPreferenceChange(stored, null, 'init');
    return stored;
  }

  // Auto-detect from user agent
  const detected = detectFromUserAgent();
  const preferences = createFromDetected(detected);

  // Save for future sessions
  saveToStorage(preferences);

  // Track the detection
  trackPreferenceChange(preferences, null, 'init');

  return preferences;
}

/**
 * Create preferences from detected system info
 */
export function createFromDetected(detected: DetectedSystem): DistroPreferences {
  return {
    osFamily: detected.osFamily,
    distro: detected.distro,
    shell: detected.shell,
    isManuallySet: false,
    lastUpdated: Date.now(),
  };
}

/**
 * Get current preferences (from storage or auto-detect)
 */
export function getPreferences(): DistroPreferences {
  const stored = loadFromStorage();
  if (stored) return stored;

  return initializePreferences();
}

/**
 * Update preferences (marks as manually set)
 */
export function setPreferences(
  update: Partial<Omit<DistroPreferences, 'isManuallySet' | 'lastUpdated'>>
): DistroPreferences {
  const previous = getPreferences();

  const newPreferences: DistroPreferences = {
    ...previous,
    ...update,
    isManuallySet: true,
    lastUpdated: Date.now(),
  };

  // Save to storage
  saveToStorage(newPreferences);

  // Track with PostHog
  trackPreferenceChange(newPreferences, previous, 'user');

  // Dispatch change event
  dispatchChangeEvent(newPreferences, 'user');

  return newPreferences;
}

/**
 * Update just the OS family (and reset distro if family changed)
 */
export function setOSFamily(osFamily: OSFamily): DistroPreferences {
  const previous = getPreferences();

  // If family changed, reset distro
  const distro = previous.osFamily === osFamily ? previous.distro : null;

  // Set appropriate default shell for the OS
  let shell = previous.shell;
  if (previous.osFamily !== osFamily) {
    shell = getDefaultShellForOS(osFamily);
  }

  return setPreferences({ osFamily, distro, shell });
}

/**
 * Update the specific distro/edition
 */
export function setDistro(
  distro: LinuxDistro | WindowsEdition | MacOSVersion | null
): DistroPreferences {
  return setPreferences({ distro });
}

/**
 * Update shell preference
 */
export function setShell(shell: ShellType): DistroPreferences {
  return setPreferences({ shell });
}

/**
 * Reset to auto-detected values
 */
export function resetToDetected(): DistroPreferences {
  const detected = detectFromUserAgent();
  const preferences: DistroPreferences = {
    osFamily: detected.osFamily,
    distro: detected.distro,
    shell: detected.shell,
    isManuallySet: false,
    lastUpdated: Date.now(),
  };

  // Save to storage
  saveToStorage(preferences);

  // Track with PostHog
  trackPreferenceChange(preferences, getPreferences(), 'auto');

  // Dispatch change event
  dispatchChangeEvent(preferences, 'auto');

  return preferences;
}

/**
 * Clear all preferences (useful for debugging/testing)
 */
export function clearPreferences(): void {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(STORAGE_KEY);
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get default shell for an OS family
 */
export function getDefaultShellForOS(osFamily: OSFamily): ShellType {
  switch (osFamily) {
    case 'windows':
      return 'powershell';
    case 'macos':
      return 'zsh';
    case 'linux':
      return 'bash';
    case 'bsd':
      return 'sh';
    default:
      return 'bash';
  }
}

/**
 * Subscribe to preference changes
 */
export function subscribeToChanges(
  callback: (event: PreferencesChangeEvent) => void
): () => void {
  if (typeof window === 'undefined') {
    return () => {};
  }

  const handler = (e: Event) => callback(e as PreferencesChangeEvent);
  window.addEventListener(PREFERENCES_CHANGE_EVENT, handler);

  return () => {
    window.removeEventListener(PREFERENCES_CHANGE_EVENT, handler);
  };
}

/**
 * Check if preferences have been manually set
 */
export function hasManualPreferences(): boolean {
  const prefs = loadFromStorage();
  return prefs?.isManuallySet ?? false;
}

// ============================================================================
// React Hook Support
// ============================================================================

/**
 * Get a reactive version of preferences (for use with React)
 * Returns current preferences and a setter
 */
export function useDistroPreferences(): [DistroPreferences, typeof setPreferences] {
  // This is a simple export for React hooks to consume
  // The actual React hook will be in the component file
  return [getPreferences(), setPreferences];
}

// ============================================================================
// Type Declarations for Window
// ============================================================================

declare global {
  interface Window {
    posthog?: {
      capture: (event: string, properties?: Record<string, unknown>) => void;
      register: (properties: Record<string, unknown>) => void;
      identify: (distinctId: string, properties?: Record<string, unknown>) => void;
    };
  }
}
