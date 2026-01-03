/**
 * React Hook for Distro Preferences
 *
 * Provides reactive access to user's OS/shell preferences
 * with automatic updates on changes.
 */

import { useState, useEffect, useCallback } from 'react';
import type { DistroPreferences, OSFamily, LinuxDistro, WindowsEdition, MacOSVersion, ShellType } from '../config/distro-types';
import {
  getPreferences,
  setPreferences as setPrefsInStore,
  setOSFamily as setOSInStore,
  setDistro as setDistroInStore,
  setShell as setShellInStore,
  resetToDetected as resetInStore,
  subscribeToChanges,
  initializePreferences,
} from './distro-preferences';

export interface UseDistroPreferencesReturn {
  /** Current preferences */
  preferences: DistroPreferences | null;
  /** Whether preferences are loading */
  isLoading: boolean;
  /** Set full preferences object */
  setPreferences: (update: Partial<Omit<DistroPreferences, 'isManuallySet' | 'lastUpdated'>>) => void;
  /** Set just the OS family */
  setOSFamily: (osFamily: OSFamily) => void;
  /** Set the specific distro */
  setDistro: (distro: LinuxDistro | WindowsEdition | MacOSVersion | null) => void;
  /** Set shell preference */
  setShell: (shell: ShellType) => void;
  /** Reset to auto-detected values */
  resetToDetected: () => void;
}

/**
 * Hook for accessing and modifying distro preferences
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { preferences, setShell, isLoading } = useDistroPreferences();
 *
 *   if (isLoading) return <Loading />;
 *
 *   return (
 *     <div>
 *       <p>Your OS: {preferences.osFamily}</p>
 *       <button onClick={() => setShell('fish')}>
 *         Switch to Fish
 *       </button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useDistroPreferences(): UseDistroPreferencesReturn {
  const [preferences, setPreferencesState] = useState<DistroPreferences | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize on mount
  useEffect(() => {
    // Only run on client
    if (typeof window === 'undefined') return;

    const prefs = initializePreferences();
    setPreferencesState(prefs);
    setIsLoading(false);

    // Subscribe to changes from other components
    const unsubscribe = subscribeToChanges((event) => {
      setPreferencesState(event.detail.preferences);
    });

    return unsubscribe;
  }, []);

  // Wrapped setters
  const setPreferences = useCallback(
    (update: Partial<Omit<DistroPreferences, 'isManuallySet' | 'lastUpdated'>>) => {
      const newPrefs = setPrefsInStore(update);
      setPreferencesState(newPrefs);
    },
    []
  );

  const setOSFamily = useCallback((osFamily: OSFamily) => {
    const newPrefs = setOSInStore(osFamily);
    setPreferencesState(newPrefs);
  }, []);

  const setDistro = useCallback(
    (distro: LinuxDistro | WindowsEdition | MacOSVersion | null) => {
      const newPrefs = setDistroInStore(distro);
      setPreferencesState(newPrefs);
    },
    []
  );

  const setShell = useCallback((shell: ShellType) => {
    const newPrefs = setShellInStore(shell);
    setPreferencesState(newPrefs);
  }, []);

  const resetToDetected = useCallback(() => {
    const newPrefs = resetInStore();
    setPreferencesState(newPrefs);
  }, []);

  return {
    preferences,
    isLoading,
    setPreferences,
    setOSFamily,
    setDistro,
    setShell,
    resetToDetected,
  };
}

export default useDistroPreferences;
