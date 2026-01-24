/**
 * DistroSelector Component
 *
 * An engaging, interactive component for users to select their
 * operating system and shell preferences. Features:
 * - Auto-detection from user agent with visual indicator
 * - Fun icons and colors for each distro
 * - Animated transitions and hover effects
 * - Categorized distro browsing
 * - Shell preference selection
 * - PostHog analytics integration
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import styles from './DistroSelector.module.css';

import type {
  OSFamily,
  LinuxDistro,
  WindowsEdition,
  MacOSVersion,
  ShellType,
  DistroPreferences,
} from '../../config/distro-types';

import {
  LINUX_DISTROS,
  WINDOWS_EDITIONS,
  MACOS_VERSIONS,
  SHELLS,
  DISTRO_CATEGORIES,
} from '../../config/distros';

import {
  getPreferences,
  setPreferences,
  setOSFamily,
  setDistro,
  setShell,
  resetToDetected,
  subscribeToChanges,
} from '../../lib/distro-preferences';

import { detectFromUserAgent } from '../../lib/user-agent-detector';

// ============================================================================
// Types
// ============================================================================

export interface DistroSelectorProps {
  /** Compact mode - just shows icon + current selection */
  compact?: boolean;
  /** Show shell selector */
  showShell?: boolean;
  /** Custom class name */
  className?: string;
  /** Callback when preferences change */
  onPreferencesChange?: (prefs: DistroPreferences) => void;
}

type DistroCategory = keyof typeof DISTRO_CATEGORIES;

// ============================================================================
// OS Family Display Data
// ============================================================================

const OS_FAMILIES: Record<OSFamily, { name: string; icon: string; color: string }> = {
  linux: { name: 'Linux', icon: '🐧', color: '#FCC624' },
  macos: { name: 'macOS', icon: '🍎', color: '#000000' },
  windows: { name: 'Windows', icon: '🪟', color: '#0078D4' },
  bsd: { name: 'BSD', icon: '😈', color: '#AB2B28' },
  unknown: { name: 'Unknown', icon: '🖥️', color: '#666666' },
};

// ============================================================================
// Helper Functions
// ============================================================================

function getDistroDisplayName(
  osFamily: OSFamily,
  distro: LinuxDistro | WindowsEdition | MacOSVersion | null
): string {
  if (!distro) return OS_FAMILIES[osFamily]?.name || 'Unknown';

  if (osFamily === 'linux' || osFamily === 'bsd') {
    return LINUX_DISTROS[distro as LinuxDistro]?.shortName ||
           LINUX_DISTROS[distro as LinuxDistro]?.name || distro;
  }

  if (osFamily === 'windows') {
    return WINDOWS_EDITIONS[distro as WindowsEdition]?.shortName ||
           WINDOWS_EDITIONS[distro as WindowsEdition]?.name || distro;
  }

  if (osFamily === 'macos') {
    return MACOS_VERSIONS[distro as MacOSVersion]?.shortName ||
           MACOS_VERSIONS[distro as MacOSVersion]?.name || distro;
  }

  return distro;
}

function getDistroIcon(
  osFamily: OSFamily,
  distro: LinuxDistro | WindowsEdition | MacOSVersion | null
): string {
  if (!distro) return OS_FAMILIES[osFamily]?.icon || '🖥️';

  if (osFamily === 'linux' || osFamily === 'bsd') {
    return LINUX_DISTROS[distro as LinuxDistro]?.icon || '🐧';
  }

  if (osFamily === 'windows') {
    return WINDOWS_EDITIONS[distro as WindowsEdition]?.icon || '🪟';
  }

  if (osFamily === 'macos') {
    return MACOS_VERSIONS[distro as MacOSVersion]?.icon || '🍎';
  }

  return '🖥️';
}

function getDistroColor(
  osFamily: OSFamily,
  distro: LinuxDistro | WindowsEdition | MacOSVersion | null
): string {
  if (!distro) return OS_FAMILIES[osFamily]?.color || '#666666';

  if (osFamily === 'linux' || osFamily === 'bsd') {
    return LINUX_DISTROS[distro as LinuxDistro]?.color || '#FCC624';
  }

  if (osFamily === 'windows') {
    return WINDOWS_EDITIONS[distro as WindowsEdition]?.color || '#0078D4';
  }

  if (osFamily === 'macos') {
    return MACOS_VERSIONS[distro as MacOSVersion]?.color || '#000000';
  }

  return '#666666';
}

// ============================================================================
// Component
// ============================================================================

export function DistroSelector({
  compact = false,
  showShell = true,
  className = '',
  onPreferencesChange,
}: DistroSelectorProps) {
  // State
  const [preferences, setPreferencesState] = useState<DistroPreferences | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'os' | 'distro' | 'shell'>('os');
  const [selectedCategory, setSelectedCategory] = useState<DistroCategory>('popular');
  const [searchQuery, setSearchQuery] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);

  // Detected system (for comparison)
  const [detected, setDetected] = useState(() => detectFromUserAgent());

  // Initialize preferences
  useEffect(() => {
    const prefs = getPreferences();
    setPreferencesState(prefs);

    // Subscribe to changes
    const unsubscribe = subscribeToChanges((event) => {
      setPreferencesState(event.detail.preferences);
      onPreferencesChange?.(event.detail.preferences);
    });

    return unsubscribe;
  }, [onPreferencesChange]);

  // Close on click outside
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [isOpen]);

  // Handlers
  const handleOSSelect = useCallback((os: OSFamily) => {
    setOSFamily(os);
    if (os === 'linux') {
      setActiveTab('distro');
    } else {
      setIsOpen(false);
    }
  }, []);

  const handleDistroSelect = useCallback((distro: LinuxDistro | WindowsEdition | MacOSVersion) => {
    setDistro(distro);
    if (showShell) {
      setActiveTab('shell');
    } else {
      setIsOpen(false);
    }
  }, [showShell]);

  const handleShellSelect = useCallback((shell: ShellType) => {
    setShell(shell);
    setIsOpen(false);
  }, []);

  const handleReset = useCallback(() => {
    resetToDetected();
    setIsOpen(false);
  }, []);

  // Filter distros by search
  const getFilteredDistros = useCallback((category: DistroCategory): LinuxDistro[] => {
    const distros = DISTRO_CATEGORIES[category];
    if (!searchQuery) return distros;

    const query = searchQuery.toLowerCase();
    return distros.filter((d) => {
      const info = LINUX_DISTROS[d];
      return (
        info.name.toLowerCase().includes(query) ||
        info.description?.toLowerCase().includes(query) ||
        d.toLowerCase().includes(query)
      );
    });
  }, [searchQuery]);

  // Loading state
  if (!preferences) {
    return (
      <div className={`${styles.container} ${styles.loading} ${className}`}>
        <div className={styles.skeleton} />
      </div>
    );
  }

  const currentIcon = getDistroIcon(preferences.osFamily, preferences.distro);
  const currentName = getDistroDisplayName(preferences.osFamily, preferences.distro);
  const currentColor = getDistroColor(preferences.osFamily, preferences.distro);
  const isDetected = !preferences.isManuallySet;

  return (
    <div
      ref={containerRef}
      className={`${styles.container} ${compact ? styles.compact : ''} ${className}`}
    >
      {/* Main Trigger Button */}
      <button
        className={styles.trigger}
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-haspopup="true"
        style={{ '--distro-color': currentColor } as React.CSSProperties}
      >
        <span className={styles.triggerIcon}>{currentIcon}</span>
        {!compact && (
          <>
            <span className={styles.triggerText}>
              <span className={styles.triggerName}>{currentName}</span>
              {preferences.shell && showShell && (
                <span className={styles.triggerShell}>
                  {SHELLS[preferences.shell]?.icon} {SHELLS[preferences.shell]?.name}
                </span>
              )}
            </span>
            {isDetected && (
              <span className={styles.detectedBadge} title="Auto-detected from your browser">
                detected
              </span>
            )}
          </>
        )}
        <svg
          className={`${styles.chevron} ${isOpen ? styles.chevronOpen : ''}`}
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
        >
          <path
            d="M4 6L8 10L12 6"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>

      {/* Dropdown Panel */}
      {isOpen && (
        <div className={styles.panel}>
          {/* Header */}
          <div className={styles.header}>
            <h3 className={styles.headerTitle}>Your Setup</h3>
            {preferences.isManuallySet && (
              <button className={styles.resetButton} onClick={handleReset}>
                Reset to detected
              </button>
            )}
          </div>

          {/* Tabs */}
          <div className={styles.tabs}>
            <button
              className={`${styles.tab} ${activeTab === 'os' ? styles.tabActive : ''}`}
              onClick={() => setActiveTab('os')}
            >
              OS
            </button>
            <button
              className={`${styles.tab} ${activeTab === 'distro' ? styles.tabActive : ''}`}
              onClick={() => setActiveTab('distro')}
              disabled={preferences.osFamily !== 'linux' && preferences.osFamily !== 'bsd'}
            >
              Distro
            </button>
            {showShell && (
              <button
                className={`${styles.tab} ${activeTab === 'shell' ? styles.tabActive : ''}`}
                onClick={() => setActiveTab('shell')}
              >
                Shell
              </button>
            )}
          </div>

          {/* OS Selection */}
          {activeTab === 'os' && (
            <div className={styles.grid}>
              {(Object.entries(OS_FAMILIES) as [OSFamily, typeof OS_FAMILIES[OSFamily]][])
                .filter(([key]) => key !== 'unknown')
                .map(([key, info]) => (
                  <button
                    key={key}
                    className={`${styles.osCard} ${preferences.osFamily === key ? styles.selected : ''}`}
                    onClick={() => handleOSSelect(key)}
                    style={{ '--card-color': info.color } as React.CSSProperties}
                  >
                    <span className={styles.cardIcon}>{info.icon}</span>
                    <span className={styles.cardName}>{info.name}</span>
                    {detected.osFamily === key && !preferences.isManuallySet && (
                      <span className={styles.cardDetected}>detected</span>
                    )}
                  </button>
                ))}
            </div>
          )}

          {/* Linux Distro Selection */}
          {activeTab === 'distro' && (
            <div className={styles.distroPanel}>
              {/* Search */}
              <div className={styles.search}>
                <input
                  type="text"
                  placeholder="Search distros..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className={styles.searchInput}
                />
              </div>

              {/* Category Tabs */}
              <div className={styles.categoryTabs}>
                {(Object.keys(DISTRO_CATEGORIES) as DistroCategory[]).map((cat) => (
                  <button
                    key={cat}
                    className={`${styles.categoryTab} ${selectedCategory === cat ? styles.categoryActive : ''}`}
                    onClick={() => setSelectedCategory(cat)}
                  >
                    {cat.charAt(0).toUpperCase() + cat.slice(1)}
                  </button>
                ))}
              </div>

              {/* Distro Grid */}
              <div className={styles.distroGrid}>
                {getFilteredDistros(selectedCategory).map((distroId) => {
                  const info = LINUX_DISTROS[distroId];
                  if (!info) return null;

                  return (
                    <button
                      key={distroId}
                      className={`${styles.distroCard} ${preferences.distro === distroId ? styles.selected : ''}`}
                      onClick={() => handleDistroSelect(distroId)}
                      style={{ '--card-color': info.color } as React.CSSProperties}
                      title={info.description}
                    >
                      <span className={styles.cardIcon}>{info.icon}</span>
                      <span className={styles.cardName}>{info.shortName || info.name}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {/* Shell Selection */}
          {activeTab === 'shell' && showShell && (
            <div className={styles.shellPanel}>
              <div className={styles.shellGrid}>
                {/* POSIX Shells */}
                <div className={styles.shellGroup}>
                  <h4 className={styles.shellGroupTitle}>POSIX Shells</h4>
                  <div className={styles.shellOptions}>
                    {(['bash', 'zsh', 'fish', 'sh', 'dash', 'ksh'] as ShellType[]).map((shellId) => {
                      const info = SHELLS[shellId];
                      return (
                        <button
                          key={shellId}
                          className={`${styles.shellCard} ${preferences.shell === shellId ? styles.selected : ''}`}
                          onClick={() => handleShellSelect(shellId)}
                          style={{ '--card-color': info.color } as React.CSSProperties}
                        >
                          <span className={styles.cardIcon}>{info.icon}</span>
                          <span className={styles.cardName}>{info.name}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>

                {/* Modern Shells */}
                <div className={styles.shellGroup}>
                  <h4 className={styles.shellGroupTitle}>Modern Shells</h4>
                  <div className={styles.shellOptions}>
                    {(['nushell', 'elvish', 'xonsh', 'oil'] as ShellType[]).map((shellId) => {
                      const info = SHELLS[shellId];
                      return (
                        <button
                          key={shellId}
                          className={`${styles.shellCard} ${preferences.shell === shellId ? styles.selected : ''}`}
                          onClick={() => handleShellSelect(shellId)}
                          style={{ '--card-color': info.color } as React.CSSProperties}
                        >
                          <span className={styles.cardIcon}>{info.icon}</span>
                          <span className={styles.cardName}>{info.name}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>

                {/* Windows Shells */}
                {(preferences.osFamily === 'windows' || preferences.distro === 'windows-wsl') && (
                  <div className={styles.shellGroup}>
                    <h4 className={styles.shellGroupTitle}>Windows Shells</h4>
                    <div className={styles.shellOptions}>
                      {(['powershell', 'pwsh', 'cmd'] as ShellType[]).map((shellId) => {
                        const info = SHELLS[shellId];
                        return (
                          <button
                            key={shellId}
                            className={`${styles.shellCard} ${preferences.shell === shellId ? styles.selected : ''}`}
                            onClick={() => handleShellSelect(shellId)}
                            style={{ '--card-color': info.color } as React.CSSProperties}
                          >
                            <span className={styles.cardIcon}>{info.icon}</span>
                            <span className={styles.cardName}>{info.name}</span>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Footer */}
          <div className={styles.footer}>
            <p className={styles.footerText}>
              {preferences.isManuallySet
                ? 'Preferences saved for future visits'
                : 'We detected this from your browser'}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

export default DistroSelector;
