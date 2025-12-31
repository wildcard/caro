/**
 * Installation Configuration
 * Single source of truth for all installation commands and URLs
 */

export const INSTALL_CONFIG = {
  // Installation script URLs
  urls: {
    // Primary installation URL (preferred)
    primary: 'https://setup.caro.sh',
    // Fallback URL (GitHub raw)
    fallback: 'https://raw.githubusercontent.com/wildcard/caro/main/install.sh',
  },

  // Installation commands
  commands: {
    // Automated installation (recommended)
    automated: {
      curl: 'bash <(curl --proto \'=https\' --tlsv1.2 -sSfL https://setup.caro.sh)',
      wget: 'bash <(wget -qO- https://setup.caro.sh)',
      // Fallback commands
      curlFallback: 'curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash',
      wgetFallback: 'wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash',
    },

    // Manual package managers
    cargo: 'cargo install caro',
    homebrew: 'brew install caro',
    apt: 'sudo apt install caro',
    aur: 'yay -S caro-bin',

    // Verification
    verify: 'caro --version',
  },

  // Installation script features
  features: [
    {
      title: 'Interactive Detection',
      description: 'Detects your OS, architecture, and shell automatically',
      icon: '🔍',
    },
    {
      title: 'Smart Installation',
      description: 'With Rust/Cargo: Builds with MLX optimization on Apple Silicon. Without Rust: Downloads pre-built binary',
      icon: '🎯',
    },
    {
      title: 'Security First',
      description: 'Verifies SHA256 checksums automatically',
      icon: '🔒',
    },
    {
      title: 'PATH Configuration',
      description: 'Configures your PATH and shell completion automatically',
      icon: '⚙️',
    },
  ],

  // Supported platforms
  platforms: {
    supported: ['macOS', 'Linux', 'Windows'],
    architectures: {
      macos: ['Apple Silicon (M1/M2/M3)', 'Intel (x86_64)'],
      linux: ['x86_64', 'ARM64'],
      windows: ['x86_64'],
    },
  },
} as const;
