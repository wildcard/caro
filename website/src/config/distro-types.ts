/**
 * Operating System & Shell Preference Types
 *
 * Comprehensive type definitions for OS detection and user preferences.
 * Supports Windows, macOS, and an exhaustive list of Linux distributions
 * along with popular shell environments.
 */

// ============================================================================
// Operating System Family
// ============================================================================

export type OSFamily = 'windows' | 'macos' | 'linux' | 'bsd' | 'unknown';

// ============================================================================
// Windows Editions
// ============================================================================

export type WindowsEdition =
  | 'windows-11'
  | 'windows-10'
  | 'windows-server'
  | 'windows-wsl'
  | 'windows-other';

// ============================================================================
// macOS Versions
// ============================================================================

export type MacOSVersion =
  | 'macos-sequoia'    // 15
  | 'macos-sonoma'     // 14
  | 'macos-ventura'    // 13
  | 'macos-monterey'   // 12
  | 'macos-bigsur'     // 11
  | 'macos-older';

// ============================================================================
// Linux Distributions - Exhaustive List
// ============================================================================

/**
 * Major Linux distribution families
 */
export type LinuxFamily =
  | 'debian'      // Debian-based (Ubuntu, Mint, etc.)
  | 'redhat'      // Red Hat-based (Fedora, RHEL, CentOS, etc.)
  | 'arch'        // Arch-based (Arch, Manjaro, EndeavourOS, etc.)
  | 'suse'        // SUSE-based (openSUSE, SLES)
  | 'gentoo'      // Gentoo-based
  | 'slackware'   // Slackware-based
  | 'alpine'      // Alpine Linux
  | 'void'        // Void Linux
  | 'nixos'       // NixOS
  | 'independent' // Independent distros
  | 'other';

/**
 * Specific Linux distributions - comprehensive list
 */
export type LinuxDistro =
  // === Debian Family ===
  | 'debian'
  | 'ubuntu'
  | 'ubuntu-lts'
  | 'kubuntu'
  | 'xubuntu'
  | 'lubuntu'
  | 'ubuntu-mate'
  | 'ubuntu-studio'
  | 'ubuntu-server'
  | 'linux-mint'
  | 'linux-mint-lmde'
  | 'pop-os'
  | 'elementary-os'
  | 'zorin-os'
  | 'mx-linux'
  | 'antiX'
  | 'kali-linux'
  | 'parrot-os'
  | 'raspbian'
  | 'raspberry-pi-os'
  | 'deepin'
  | 'bodhi-linux'
  | 'peppermint-os'
  | 'sparky-linux'
  | 'lmde'
  | 'devuan'
  | 'bunsenlabs'
  | 'q4os'
  | 'kde-neon'
  | 'tails'
  | 'pureos'
  | 'steamos'

  // === Red Hat Family ===
  | 'fedora'
  | 'fedora-silverblue'
  | 'fedora-kinoite'
  | 'rhel'
  | 'centos'
  | 'centos-stream'
  | 'rocky-linux'
  | 'alma-linux'
  | 'oracle-linux'
  | 'amazon-linux'
  | 'nobara'
  | 'ultramarine'
  | 'qubes-os'
  | 'scientific-linux'
  | 'clearos'

  // === Arch Family ===
  | 'arch-linux'
  | 'manjaro'
  | 'endeavouros'
  | 'garuda-linux'
  | 'arco-linux'
  | 'artix-linux'
  | 'blackarch'
  | 'archcraft'
  | 'cachyos'
  | 'crystal-linux'
  | 'rebornos'
  | 'archbang'
  | 'hyperbola'
  | 'parabola'

  // === SUSE Family ===
  | 'opensuse-tumbleweed'
  | 'opensuse-leap'
  | 'sles'
  | 'gecko-linux'

  // === Gentoo Family ===
  | 'gentoo'
  | 'funtoo'
  | 'calculate-linux'
  | 'sabayon'
  | 'redcore'

  // === Independent / Other ===
  | 'nixos'
  | 'void-linux'
  | 'alpine-linux'
  | 'slackware'
  | 'solus'
  | 'pclinuxos'
  | 'mageia'
  | 'openmandriva'
  | 'puppy-linux'
  | 'tiny-core'
  | 'porteus'
  | 'slitaz'
  | 'clear-linux'
  | 'chrome-os'
  | 'chromium-os'
  | 'guix'
  | 'bedrock-linux'
  | 'kiss-linux'
  | 'gobo-linux'

  // === BSD Family (included for completeness) ===
  | 'freebsd'
  | 'openbsd'
  | 'netbsd'
  | 'dragonflybsd'
  | 'ghostbsd'
  | 'trueos'
  | 'midnightbsd'

  // Fallback
  | 'linux-other';

// ============================================================================
// Shell Types
// ============================================================================

export type ShellType =
  // POSIX / Unix shells
  | 'bash'
  | 'zsh'
  | 'fish'
  | 'sh'
  | 'dash'
  | 'ksh'
  | 'tcsh'
  | 'csh'
  | 'ash'

  // Modern / Alternative shells
  | 'nushell'
  | 'elvish'
  | 'ion'
  | 'xonsh'
  | 'oil'
  | 'murex'
  | 'yash'
  | 'rc'

  // Windows shells
  | 'powershell'
  | 'pwsh'  // PowerShell Core (cross-platform)
  | 'cmd'

  // Fallback
  | 'other';

// ============================================================================
// Terminal Emulators (for fun theming)
// ============================================================================

export type TerminalEmulator =
  // Cross-platform
  | 'alacritty'
  | 'kitty'
  | 'wezterm'
  | 'hyper'
  | 'tabby'
  | 'warp'
  | 'rio'

  // macOS
  | 'terminal-app'  // Apple Terminal
  | 'iterm2'

  // Linux
  | 'gnome-terminal'
  | 'konsole'
  | 'xfce4-terminal'
  | 'tilix'
  | 'terminator'
  | 'guake'
  | 'yakuake'
  | 'urxvt'
  | 'st'
  | 'xterm'
  | 'foot'
  | 'cool-retro-term'

  // Windows
  | 'windows-terminal'
  | 'conemu'
  | 'cmder'
  | 'fluent-terminal'

  | 'other';

// ============================================================================
// User Preferences
// ============================================================================

/**
 * Complete user preferences for OS/shell customization
 */
export interface DistroPreferences {
  /** Operating system family */
  osFamily: OSFamily;

  /** Specific distribution/edition (null for auto-detect) */
  distro: LinuxDistro | WindowsEdition | MacOSVersion | null;

  /** Preferred shell */
  shell: ShellType;

  /** Terminal emulator (optional, for fun theming) */
  terminal?: TerminalEmulator;

  /** Whether preferences were manually set (vs auto-detected) */
  isManuallySet: boolean;

  /** Timestamp of last update */
  lastUpdated: number;
}

/**
 * Detected system info from user agent
 */
export interface DetectedSystem {
  osFamily: OSFamily;
  distro: LinuxDistro | WindowsEdition | MacOSVersion | null;
  shell: ShellType;
  confidence: 'high' | 'medium' | 'low';
  rawUserAgent: string;
}

// ============================================================================
// Display Metadata
// ============================================================================

export interface DistroInfo {
  id: LinuxDistro | WindowsEdition | MacOSVersion;
  name: string;
  shortName?: string;
  family: OSFamily | LinuxFamily;
  icon: string;  // emoji or icon identifier
  color: string; // brand color (hex)
  website?: string;
  packageManager?: string;
  description?: string;
}

export interface ShellInfo {
  id: ShellType;
  name: string;
  icon: string;
  color: string;
  website?: string;
  description?: string;
}

// ============================================================================
// Theme Customization
// ============================================================================

/**
 * Visual theme adjustments based on preferences
 */
export interface DistroTheme {
  /** Primary accent color */
  accentColor: string;
  /** Secondary color */
  secondaryColor: string;
  /** Terminal-style background (for code blocks) */
  terminalBg: string;
  /** Terminal text color */
  terminalText: string;
  /** Prompt style identifier */
  promptStyle: 'bash' | 'zsh' | 'fish' | 'powershell' | 'cmd' | 'nushell' | 'default';
  /** CSS class to apply to root */
  cssClass: string;
}
