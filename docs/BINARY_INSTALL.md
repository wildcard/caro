# Binary Installation Guide

This guide covers installing caro from pre-built binaries. Binary installation is the fastest way to get started - no compilation required.

## Quick Install (Recommended)

Use the setup script to automatically download and install the correct binary for your platform:

```bash
# Using curl (binary-only, no compilation)
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary

# Using wget
wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary
```

The script will:
- Detect your operating system and architecture
- Download the appropriate pre-built binary
- Verify SHA256 checksum for security
- Install to `~/.local/bin` (configurable via `CARO_INSTALL_DIR`)
- Make the binary executable

### Command-line Options

| Option | Description |
|--------|-------------|
| `--binary` | Force binary download (skip cargo even if available) |
| `--cargo` | Force cargo install |
| `--non-interactive` | Skip all interactive prompts |
| `--no-modify-path` | Don't add install directory to PATH |
| `--help` | Show help message |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CARO_INSTALL_DIR` | `~/.local/bin` | Installation directory |
| `CARO_INTERACTIVE` | `true` | Set to `false` for non-interactive mode |
| `CARO_INSTALL_METHOD` | auto | Force `binary` or `cargo` |
| `CARO_SHELL_COMPLETION` | `true` | Enable shell completion setup |
| `CARO_PATH_AUTO` | `true` | Auto-add install dir to PATH |
| `CARO_SAFETY_CONFIG` | `true` | Configure safety level |

**Examples:**

```bash
# Quick binary-only install
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary

# Non-interactive binary install (no prompts)
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary --non-interactive

# Install to /usr/local/bin
CARO_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary

# Install to home bin directory
CARO_INSTALL_DIR=~/bin curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary

# Minimal install (no PATH or shell modifications)
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash -s -- --binary --non-interactive --no-modify-path
```

## Manual Installation

### Step 1: Download the Binary

Download the appropriate binary for your platform from the [latest release](https://github.com/wildcard/caro/releases/latest):

| Platform | Architecture | Binary Name |
|----------|--------------|-------------|
| Linux | x86_64 (Intel/AMD) | `caro-VERSION-linux-amd64` |
| Linux | ARM64 (Raspberry Pi, etc.) | `caro-VERSION-linux-arm64` |
| macOS | Intel | `caro-VERSION-macos-intel` |
| macOS | Apple Silicon (M1/M2/M3) | `caro-VERSION-macos-silicon` |
| Windows | x86_64 | `caro-VERSION-windows-amd64.exe` |

**Example download commands (replace VERSION with latest, e.g., `1.0.2`):**

```bash
# Linux x86_64
curl -fsSL https://github.com/wildcard/caro/releases/download/v1.0.2/caro-1.0.2-linux-amd64 -o caro

# Linux ARM64
curl -fsSL https://github.com/wildcard/caro/releases/download/v1.0.2/caro-1.0.2-linux-arm64 -o caro

# macOS Intel
curl -fsSL https://github.com/wildcard/caro/releases/download/v1.0.2/caro-1.0.2-macos-intel -o caro

# macOS Apple Silicon
curl -fsSL https://github.com/wildcard/caro/releases/download/v1.0.2/caro-1.0.2-macos-silicon -o caro

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/wildcard/caro/releases/download/v1.0.2/caro-1.0.2-windows-amd64.exe" -OutFile "caro.exe"
```

### Step 2: Verify Checksum (Recommended)

Each binary has a corresponding `.sha256` checksum file. Always verify before installing:

```bash
# Download the checksum file
curl -fsSL https://github.com/wildcard/caro/releases/download/v1.0.2/caro-1.0.2-linux-amd64.sha256 -o caro.sha256

# Verify (Linux)
sha256sum -c caro.sha256

# Verify (macOS)
shasum -a 256 -c caro.sha256
```

Expected output: `caro: OK`

### Step 3: Install the Binary

**Linux/macOS:**
```bash
# Make executable
chmod +x caro

# Move to a directory in your PATH
sudo mv caro /usr/local/bin/

# Or install to user directory (no sudo required)
mkdir -p ~/.local/bin
mv caro ~/.local/bin/
```

**Windows:**
```powershell
# Move to a directory in your PATH, e.g., C:\Users\YourName\bin
Move-Item caro.exe $env:USERPROFILE\bin\caro.exe

# Or add to system PATH via Settings > System > Environment Variables
```

### Step 4: Add to PATH (if needed)

If you installed to `~/.local/bin`, ensure it's in your PATH:

**Bash (~/.bashrc or ~/.bash_profile):**
```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Zsh (~/.zshrc):**
```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Fish (~/.config/fish/config.fish):**
```fish
set -gx PATH $HOME/.local/bin $PATH
```

**PowerShell ($PROFILE):**
```powershell
$env:PATH = "$env:USERPROFILE\bin;$env:PATH"
```

Reload your shell configuration:
```bash
# Bash/Zsh
source ~/.bashrc  # or ~/.zshrc

# Fish
source ~/.config/fish/config.fish
```

### Step 5: Verify Installation

```bash
caro --version
```

Expected output: `caro 1.0.2` (or your installed version)

## Platform-Specific Notes

### macOS Apple Silicon

Pre-built binaries work immediately on Apple Silicon. However, for **maximum performance** with MLX GPU acceleration, build from source:

```bash
# Install Rust if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install with MLX support
cargo install caro --features embedded-mlx
```

See [macOS Setup Guide](MACOS_SETUP.md) for detailed GPU acceleration instructions.

### macOS Security (Gatekeeper)

If macOS blocks the binary with "cannot be opened because the developer cannot be verified":

```bash
# Option 1: Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/caro

# Option 2: Allow in System Preferences
# Go to: System Preferences > Security & Privacy > General
# Click "Allow Anyway" next to the caro message
```

### Linux ARM64 (Raspberry Pi, etc.)

The ARM64 binary is cross-compiled for aarch64 Linux. If you encounter GLIBC compatibility issues:

```bash
# Check your GLIBC version
ldd --version

# If too old, build from source instead
cargo install caro
```

### Windows

The Windows binary requires no special setup. If Windows Defender blocks it:
1. Click "More info"
2. Click "Run anyway"

Or add an exclusion in Windows Security settings.

## Shell Completion

Enable tab completion for caro commands:

**Bash:**
```bash
echo 'eval "$(caro --completion bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
echo 'eval "$(caro --completion zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Fish:**
```bash
echo 'caro --completion fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

**PowerShell:**
```powershell
Add-Content $PROFILE 'Invoke-Expression (& caro --completion powershell)'
. $PROFILE
```

## Updating

To update to a new version, simply repeat the download and installation steps with the new version number.

**Using the install script:**
```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | CARO_INTERACTIVE=false bash
```

**Manual update:**
```bash
# Download new version
curl -fsSL https://github.com/wildcard/caro/releases/download/vNEW_VERSION/caro-NEW_VERSION-PLATFORM -o caro

# Replace existing binary
chmod +x caro
sudo mv caro /usr/local/bin/
```

## Uninstallation

**Remove the binary:**
```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/caro

# If installed to ~/.local/bin
rm ~/.local/bin/caro
```

**Remove configuration (optional):**
```bash
rm -rf ~/.config/caro
rm -rf ~/.cache/caro
```

**Remove shell completion (optional):**

Edit your shell config file and remove the caro completion line.

## Troubleshooting

### "command not found: caro"

Your installation directory isn't in PATH. Either:
1. Add the directory to PATH (see Step 4 above)
2. Use the full path: `~/.local/bin/caro "your command"`

### "Permission denied"

Make the binary executable:
```bash
chmod +x /path/to/caro
```

### "cannot execute binary file"

You downloaded the wrong binary for your platform. Check:
```bash
# Your OS
uname -s

# Your architecture
uname -m
```

Download the matching binary from the table above.

### Checksum mismatch

Re-download both the binary and checksum file. If the issue persists, check:
1. The release page for any known issues
2. Open an issue on GitHub

## Related Documentation

- [README](../README.md) - Full project documentation
- [macOS Setup Guide](MACOS_SETUP.md) - Detailed macOS and GPU acceleration setup
- [Release Process](RELEASE_PROCESS.md) - How releases are created

## Getting Help

- [GitHub Issues](https://github.com/wildcard/caro/issues) - Bug reports
- [GitHub Discussions](https://github.com/wildcard/caro/discussions) - Questions and community
