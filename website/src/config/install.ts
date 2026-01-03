/**
 * Installation Configuration
 * Single source of truth for all installation commands and URLs
 */

// Wizard answer types
export type Platform = 'macos-silicon' | 'macos-intel' | 'linux-amd64' | 'linux-arm64' | 'windows';
export type RustChoice = 'yes' | 'no' | 'install-rust';
export type NetworkEnv = 'normal' | 'proxy' | 'airgapped';

export interface WizardAnswers {
  platform: Platform;
  rust: RustChoice;
  network: NetworkEnv;
}

export const INSTALL_CONFIG = {
  // Installation script URLs
  urls: {
    // Primary installation URL (preferred)
    primary: 'https://setup.caro.sh',
    // Fallback URL (GitHub raw)
    fallback: 'https://raw.githubusercontent.com/wildcard/caro/main/install.sh',
    // GitHub repository
    github: 'https://github.com/wildcard/caro',
  },

  // Installation commands
  commands: {
    // Automated installation (recommended)
    automated: {
      curl: 'bash <(curl --proto \'=https\' --tlsv1.2 -sSfL https://setup.caro.sh)',
      wget: 'bash <(wget -qO- https://setup.caro.sh)',
      // Non-interactive mode (for scripting)
      curlNonInteractive: 'CARO_INTERACTIVE=false bash <(curl --proto \'=https\' --tlsv1.2 -sSfL https://setup.caro.sh)',
      wgetNonInteractive: 'CARO_INTERACTIVE=false bash <(wget -qO- https://setup.caro.sh)',
      // Fallback commands
      curlFallback: 'curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash',
      wgetFallback: 'wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash',
    },

    // Cargo installation variants
    cargo: {
      standard: 'cargo install caro',
      mlx: 'cargo install caro --features embedded-mlx',
      force: 'cargo install caro --force',
      mlxForce: 'cargo install caro --features embedded-mlx --force',
    },

    // Manual package managers
    packageManagers: {
      homebrew: 'brew install caro',
      apt: 'sudo apt install caro',
      aur: 'yay -S caro-bin',
    },

    // Verification
    verify: 'caro --version',
    quickTest: 'caro "list files modified today"',
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

  // Interactive Installation Wizard Configuration
  wizard: {
    // Platform selection options
    platforms: [
      {
        value: 'macos-silicon' as const,
        label: 'macOS Apple Silicon',
        sublabel: 'M1, M2, M3, M4 chips',
        icon: '🍎',
        recommendRust: true,
        mlxAvailable: true,
      },
      {
        value: 'macos-intel' as const,
        label: 'macOS Intel',
        sublabel: 'x86_64 processors',
        icon: '🍎',
        recommendRust: false,
        mlxAvailable: false,
      },
      {
        value: 'linux-amd64' as const,
        label: 'Linux x86_64',
        sublabel: 'Standard Linux desktop/server',
        icon: '🐧',
        recommendRust: false,
        mlxAvailable: false,
      },
      {
        value: 'linux-arm64' as const,
        label: 'Linux ARM64',
        sublabel: 'Raspberry Pi, ARM servers',
        icon: '🐧',
        recommendRust: false,
        mlxAvailable: false,
      },
      {
        value: 'windows' as const,
        label: 'Windows (WSL)',
        sublabel: 'Windows Subsystem for Linux',
        icon: '🪟',
        recommendRust: false,
        mlxAvailable: false,
      },
    ],

    // Rust installation choice
    rustOptions: [
      {
        value: 'yes' as const,
        label: 'Yes, I have Rust',
        sublabel: 'Build from source with cargo',
        recommended: false, // Set dynamically based on platform
      },
      {
        value: 'no' as const,
        label: 'No, download binary',
        sublabel: 'Get pre-built executable',
        recommended: false,
      },
      {
        value: 'install-rust' as const,
        label: 'No, install Rust first',
        sublabel: 'Visit rustup.rs then return',
        recommended: false,
      },
    ],

    // Network environment selection
    networkOptions: [
      {
        value: 'normal' as const,
        label: 'Normal Internet',
        sublabel: 'Standard network access',
        icon: '🌐',
      },
      {
        value: 'proxy' as const,
        label: 'Behind Proxy',
        sublabel: 'Corporate network with proxy',
        icon: '🔐',
      },
      {
        value: 'airgapped' as const,
        label: 'Air-gapped',
        sublabel: 'No internet access',
        icon: '🔒',
      },
    ],
  },
} as const;

/**
 * Generate installation commands based on wizard answers
 */
export function generateInstallCommands(answers: WizardAnswers): {
  curl: string;
  wget: string;
  notes: string[];
  nextSteps: string[];
} {
  const { platform, rust, network } = answers;

  const notes: string[] = [];
  const nextSteps: string[] = [];

  // Generate commands based on rust choice
  let curlCmd = '';
  let wgetCmd = '';

  if (rust === 'yes') {
    // Use cargo
    if (platform === 'macos-silicon') {
      curlCmd = INSTALL_CONFIG.commands.cargo.mlx;
      wgetCmd = INSTALL_CONFIG.commands.cargo.mlx;
      notes.push('💡 Building with MLX optimization for Apple Silicon (faster inference)');
    } else {
      curlCmd = INSTALL_CONFIG.commands.cargo.standard;
      wgetCmd = INSTALL_CONFIG.commands.cargo.standard;
    }
    notes.push('⚙️  Build from source may take 2-5 minutes');
    nextSteps.push('Cargo will install to ~/.cargo/bin');
  } else if (rust === 'no') {
    // Use automated script
    if (network === 'normal') {
      curlCmd = INSTALL_CONFIG.commands.automated.curl;
      wgetCmd = INSTALL_CONFIG.commands.automated.wget;
    } else if (network === 'proxy') {
      curlCmd = `http_proxy=YOUR_PROXY https_proxy=YOUR_PROXY ${INSTALL_CONFIG.commands.automated.curl}`;
      wgetCmd = `http_proxy=YOUR_PROXY https_proxy=YOUR_PROXY ${INSTALL_CONFIG.commands.automated.wget}`;
      notes.push('⚠️  Replace YOUR_PROXY with your proxy URL (e.g., http://proxy.company.com:8080)');
    } else if (network === 'airgapped') {
      curlCmd = '# Download on internet-connected machine, then transfer';
      wgetCmd = '# Download on internet-connected machine, then transfer';
      notes.push('📦 Air-gapped installation requires manual binary download');
      notes.push(`Visit ${INSTALL_CONFIG.urls.github}/releases on a connected machine`);
      notes.push('Download the appropriate binary for your platform');
      notes.push('Transfer to air-gapped machine via USB or secure file transfer');
      nextSteps.push('chmod +x caro');
      nextSteps.push('sudo mv caro /usr/local/bin/');
    }
  } else if (rust === 'install-rust') {
    curlCmd = '# First: curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh';
    wgetCmd = '# First: wget -qO- https://sh.rustup.rs | sh';
    notes.push('🦀 Install Rust first, then re-run this wizard');
    notes.push('Visit https://rustup.rs for installation instructions');
  }

  // Add platform-specific notes
  if (platform === 'macos-silicon' && rust === 'yes') {
    nextSteps.push('MLX backend will be enabled automatically');
  }

  // Add common next steps
  if (rust !== 'install-rust' && network !== 'airgapped') {
    nextSteps.push('Run: caro --version  # Verify installation');
    nextSteps.push('Try: caro "list files modified today"');
  }

  return {
    curl: curlCmd,
    wget: wgetCmd,
    notes,
    nextSteps,
  };
}
