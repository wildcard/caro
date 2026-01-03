/**
 * User Agent Detection
 *
 * Parses browser user agent to detect operating system, distribution,
 * and make educated guesses about shell preferences.
 *
 * Privacy-respecting: Only uses client-side user agent, no fingerprinting.
 */

import type {
  OSFamily,
  LinuxDistro,
  WindowsEdition,
  MacOSVersion,
  ShellType,
  DetectedSystem,
} from '../config/distro-types';

// ============================================================================
// User Agent Patterns
// ============================================================================

const UA_PATTERNS = {
  // Operating Systems
  windows: /Windows NT (\d+\.\d+)/i,
  macOS: /Mac OS X (\d+)[._](\d+)/i,
  linux: /Linux/i,
  android: /Android/i,
  iOS: /iPhone|iPad|iPod/i,
  chromeos: /CrOS/i,
  freebsd: /FreeBSD/i,
  openbsd: /OpenBSD/i,
  netbsd: /NetBSD/i,

  // Linux Distributions (some embed in UA)
  ubuntu: /Ubuntu/i,
  fedora: /Fedora/i,
  debian: /Debian/i,
  arch: /Arch/i,
  gentoo: /Gentoo/i,
  suse: /SUSE|openSUSE/i,
  redhat: /Red Hat|RHEL/i,
  centos: /CentOS/i,
  mint: /Linux Mint/i,

  // Browsers can give hints
  firefox: /Firefox\/(\d+)/i,
  chrome: /Chrome\/(\d+)/i,
  chromium: /Chromium\/(\d+)/i,
  safari: /Safari\/(\d+)/i,
  edge: /Edg\/(\d+)/i,

  // WSL detection (browser won't show this, but helpful for reference)
  wsl: /WSL|Microsoft/i,
};

// ============================================================================
// Windows Version Mapping
// ============================================================================

const WINDOWS_VERSION_MAP: Record<string, WindowsEdition> = {
  '10.0': 'windows-10', // Could be 10 or 11, need more context
  '6.3': 'windows-10',  // Windows 8.1 - map to 10 for simplicity
  '6.2': 'windows-10',  // Windows 8
  '6.1': 'windows-10',  // Windows 7
};

// ============================================================================
// macOS Version Mapping
// ============================================================================

const MACOS_VERSION_MAP: Record<string, MacOSVersion> = {
  '15': 'macos-sequoia',
  '14': 'macos-sonoma',
  '13': 'macos-ventura',
  '12': 'macos-monterey',
  '11': 'macos-bigsur',
  '10_15': 'macos-older',
  '10_14': 'macos-older',
};

// ============================================================================
// Default Shell by OS
// ============================================================================

const DEFAULT_SHELLS: Record<OSFamily, ShellType> = {
  windows: 'powershell',
  macos: 'zsh',
  linux: 'bash',
  bsd: 'sh',
  unknown: 'bash',
};

// ============================================================================
// Detection Functions
// ============================================================================

/**
 * Detect operating system family from user agent
 */
function detectOSFamily(ua: string): OSFamily {
  if (UA_PATTERNS.windows.test(ua)) return 'windows';
  if (UA_PATTERNS.macOS.test(ua) || UA_PATTERNS.iOS.test(ua)) return 'macos';
  if (UA_PATTERNS.chromeos.test(ua)) return 'linux';
  if (UA_PATTERNS.android.test(ua)) return 'linux';
  if (UA_PATTERNS.linux.test(ua)) return 'linux';
  if (UA_PATTERNS.freebsd.test(ua) || UA_PATTERNS.openbsd.test(ua) || UA_PATTERNS.netbsd.test(ua)) return 'bsd';
  return 'unknown';
}

/**
 * Detect specific Windows edition
 */
function detectWindowsEdition(ua: string): WindowsEdition | null {
  const match = ua.match(UA_PATTERNS.windows);
  if (!match) return null;

  const version = match[1];

  // Windows 11 has same NT version as 10, but we can detect via Chrome version
  // Chrome 95+ on Windows 11 has special handling
  if (version === '10.0') {
    // Check for Windows 11 hints
    const chromeMatch = ua.match(UA_PATTERNS.chrome);
    if (chromeMatch) {
      const chromeVersion = parseInt(chromeMatch[1], 10);
      // Windows 11 was released around Chrome 95, newer Chrome more likely Win 11
      // This is a heuristic, not definitive
      if (chromeVersion >= 95) {
        return 'windows-11';
      }
    }
    return 'windows-10';
  }

  return WINDOWS_VERSION_MAP[version] || 'windows-other';
}

/**
 * Detect specific macOS version
 */
function detectMacOSVersion(ua: string): MacOSVersion | null {
  const match = ua.match(UA_PATTERNS.macOS);
  if (!match) return null;

  const major = match[1];
  const minor = match[2];

  // macOS 11+ uses just major version
  if (parseInt(major, 10) >= 11) {
    return MACOS_VERSION_MAP[major] || 'macos-older';
  }

  // macOS 10.x uses major_minor
  return MACOS_VERSION_MAP[`${major}_${minor}`] || 'macos-older';
}

/**
 * Detect Linux distribution from user agent
 * Note: Most browsers don't include distro info, so this is limited
 */
function detectLinuxDistro(ua: string): LinuxDistro | null {
  // ChromeOS is special
  if (UA_PATTERNS.chromeos.test(ua)) {
    return 'chrome-os';
  }

  // Check for embedded distro info (rare but some browsers do this)
  if (UA_PATTERNS.ubuntu.test(ua)) return 'ubuntu';
  if (UA_PATTERNS.fedora.test(ua)) return 'fedora';
  if (UA_PATTERNS.debian.test(ua)) return 'debian';
  if (UA_PATTERNS.arch.test(ua)) return 'arch-linux';
  if (UA_PATTERNS.gentoo.test(ua)) return 'gentoo';
  if (UA_PATTERNS.suse.test(ua)) return 'opensuse-tumbleweed';
  if (UA_PATTERNS.redhat.test(ua)) return 'rhel';
  if (UA_PATTERNS.centos.test(ua)) return 'centos-stream';
  if (UA_PATTERNS.mint.test(ua)) return 'linux-mint';

  // No specific distro detected
  return null;
}

/**
 * Detect BSD variant
 */
function detectBSD(ua: string): LinuxDistro | null {
  if (UA_PATTERNS.freebsd.test(ua)) return 'freebsd';
  if (UA_PATTERNS.openbsd.test(ua)) return 'openbsd';
  if (UA_PATTERNS.netbsd.test(ua)) return 'netbsd';
  return null;
}

/**
 * Determine confidence level of detection
 */
function getConfidence(osFamily: OSFamily, distro: string | null): 'high' | 'medium' | 'low' {
  if (osFamily === 'windows' || osFamily === 'macos') {
    return 'high'; // UA reliably shows these
  }

  if (osFamily === 'linux' && distro) {
    return 'medium'; // If we detected a distro, that's decent
  }

  if (osFamily === 'linux') {
    return 'low'; // Just "Linux" without distro info
  }

  return 'low';
}

// ============================================================================
// Main Detection Function
// ============================================================================

/**
 * Detect system information from user agent string
 */
export function detectFromUserAgent(userAgent?: string): DetectedSystem {
  // Get user agent from browser if not provided
  const ua = userAgent || (typeof navigator !== 'undefined' ? navigator.userAgent : '');

  // Detect OS family
  const osFamily = detectOSFamily(ua);

  // Detect specific distro/edition
  let distro: LinuxDistro | WindowsEdition | MacOSVersion | null = null;

  switch (osFamily) {
    case 'windows':
      distro = detectWindowsEdition(ua);
      break;
    case 'macos':
      distro = detectMacOSVersion(ua);
      break;
    case 'linux':
      distro = detectLinuxDistro(ua);
      break;
    case 'bsd':
      distro = detectBSD(ua);
      break;
  }

  // Determine default shell based on OS
  const shell = DEFAULT_SHELLS[osFamily];

  // Confidence level
  const confidence = getConfidence(osFamily, distro);

  return {
    osFamily,
    distro,
    shell,
    confidence,
    rawUserAgent: ua,
  };
}

/**
 * Get a human-readable description of the detected system
 */
export function getDetectionSummary(detected: DetectedSystem): string {
  const parts: string[] = [];

  switch (detected.osFamily) {
    case 'windows':
      parts.push(detected.distro === 'windows-11' ? 'Windows 11' : 'Windows');
      break;
    case 'macos':
      parts.push('macOS');
      break;
    case 'linux':
      parts.push(detected.distro ? 'Linux' : 'Linux (unknown distro)');
      break;
    case 'bsd':
      parts.push('BSD');
      break;
    default:
      parts.push('Unknown OS');
  }

  return parts.join(' ');
}

/**
 * Check if user agent suggests developer/power user
 * (e.g., using Firefox, specific Linux distro mentioned)
 */
export function isPowerUser(detected: DetectedSystem): boolean {
  const ua = detected.rawUserAgent.toLowerCase();

  // Firefox users tend to be more technical
  if (UA_PATTERNS.firefox.test(detected.rawUserAgent)) return true;

  // Any Linux user
  if (detected.osFamily === 'linux') return true;

  // BSD users
  if (detected.osFamily === 'bsd') return true;

  // Chromium (not Chrome) users
  if (UA_PATTERNS.chromium.test(detected.rawUserAgent) && !UA_PATTERNS.chrome.test(detected.rawUserAgent)) {
    return true;
  }

  return false;
}
