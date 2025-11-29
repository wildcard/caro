# Installation Guide

This guide covers all available methods for installing cmdai on your system.

## Quick Install (Recommended)

### macOS / Linux

Install with a single command using our install script:

```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash
```

**Custom installation directory:**
```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash -s -- --install-dir ~/.local/bin
```

The script will:
- Automatically detect your platform and architecture
- Download the appropriate binary
- Verify checksums for security
- Install to `/usr/local/bin` by default
- Check if the install directory is in your PATH

## Package Managers

### Homebrew (macOS / Linux)

```bash
# Add the cmdai tap
brew tap wildcard/tap

# Install cmdai
brew install cmdai
```

Or in one command:
```bash
brew install wildcard/tap/cmdai
```

**Update cmdai:**
```bash
brew upgrade cmdai
```

### Cargo (All Platforms)

If you have Rust installed:

```bash
cargo install cmdai
```

**Update cmdai:**
```bash
cargo install cmdai --force
```

### Scoop (Windows)

```powershell
# Add the cmdai bucket
scoop bucket add wildcard https://github.com/wildcard/scoop-bucket

# Install cmdai
scoop install cmdai
```

**Update cmdai:**
```powershell
scoop update cmdai
```

### Chocolatey (Windows)

```powershell
choco install cmdai
```

**Update cmdai:**
```powershell
choco upgrade cmdai
```

### Winget (Windows)

```powershell
winget install wildcard.cmdai
```

**Update cmdai:**
```powershell
winget upgrade wildcard.cmdai
```

## Manual Installation

### Download Pre-built Binaries

1. Visit the [latest release page](https://github.com/wildcard/cmdai/releases/latest)
2. Download the appropriate binary for your platform:
   - **Linux (x86_64)**: `cmdai-linux-amd64`
   - **Linux (ARM64)**: `cmdai-linux-arm64`
   - **macOS (Intel)**: `cmdai-macos-intel`
   - **macOS (Apple Silicon)**: `cmdai-macos-silicon`
   - **Windows (x86_64)**: `cmdai-windows-amd64.exe`

3. Download the corresponding `.sha256` checksum file

### Verify the Download

**macOS / Linux:**
```bash
shasum -a 256 -c cmdai-*.sha256
```

**Windows PowerShell:**
```powershell
$hash = (Get-FileHash cmdai-windows-amd64.exe -Algorithm SHA256).Hash
$expected = (Get-Content cmdai-windows-amd64.exe.sha256).Split()[0]
if ($hash -eq $expected) { Write-Host "✓ Checksum verified" } else { Write-Host "✗ Checksum mismatch" }
```

### Install the Binary

**macOS / Linux:**
```bash
# Make it executable
chmod +x cmdai-*

# Move to a directory in your PATH
sudo mv cmdai-* /usr/local/bin/cmdai

# Or to your local bin directory (no sudo needed)
mkdir -p ~/.local/bin
mv cmdai-* ~/.local/bin/cmdai
export PATH="$HOME/.local/bin:$PATH"  # Add to ~/.bashrc or ~/.zshrc
```

**Windows:**
1. Rename `cmdai-windows-amd64.exe` to `cmdai.exe`
2. Move it to a directory in your PATH, such as:
   - `C:\Program Files\cmdai\`
   - Or add the current directory to your PATH

## Building from Source

### Prerequisites
- Rust 1.75 or later
- Cargo

### Build and Install

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build the release binary
cargo build --release

# The binary will be at: target/release/cmdai

# Install to Cargo's bin directory
cargo install --path .
```

## Verification

After installation, verify that cmdai is working:

```bash
cmdai --version
```

You should see output like:
```
cmdai 0.1.0
```

## First Steps

### Run Your First Command

```bash
cmdai "list all PDF files larger than 10MB"
```

### View Help

```bash
cmdai --help
```

### Configure cmdai

Create a configuration file at `~/.config/cmdai/config.toml`:

```toml
[backend]
primary = "embedded"  # or "ollama", "vllm"
enable_fallback = true

[safety]
enabled = true
level = "moderate"  # strict, moderate, or permissive

[output]
format = "plain"  # json, yaml, or plain
verbose = false
```

## Platform-Specific Notes

### macOS

**Gatekeeper Warning:**
If you see a security warning when running cmdai:
```bash
xattr -d com.apple.quarantine /usr/local/bin/cmdai
```

**Apple Silicon:**
Make sure to download the `cmdai-macos-silicon` binary for M1/M2/M3 Macs.

### Linux

**AppArmor/SELinux:**
Some systems may require additional permissions. If you encounter issues:
```bash
# For systems using AppArmor
sudo aa-complain /usr/local/bin/cmdai

# For systems using SELinux
sudo setenforce 0
```

### Windows

**Antivirus:**
Some antivirus software may flag the binary. Add an exception if needed.

**PowerShell Execution Policy:**
If you encounter execution policy issues:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Updating cmdai

### Using Package Managers

- **Homebrew**: `brew upgrade cmdai`
- **Cargo**: `cargo install cmdai --force`
- **Scoop**: `scoop update cmdai`
- **Chocolatey**: `choco upgrade cmdai`
- **Winget**: `winget upgrade wildcard.cmdai`

### Manual Update

1. Download the latest binary from [GitHub Releases](https://github.com/wildcard/cmdai/releases/latest)
2. Replace your existing binary
3. Verify the new version: `cmdai --version`

## Uninstalling

### Package Managers

- **Homebrew**: `brew uninstall cmdai`
- **Cargo**: `cargo uninstall cmdai`
- **Scoop**: `scoop uninstall cmdai`
- **Chocolatey**: `choco uninstall cmdai`
- **Winget**: `winget uninstall wildcard.cmdai`

### Manual Uninstall

```bash
# Remove the binary
sudo rm /usr/local/bin/cmdai

# Remove configuration (optional)
rm -rf ~/.config/cmdai
```

## Troubleshooting

### Command Not Found

If you get "command not found" after installation:

1. Check if the install directory is in your PATH:
   ```bash
   echo $PATH
   ```

2. Add the directory to your PATH:
   ```bash
   # For ~/.local/bin
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc

   # For zsh users
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

### Permission Denied

If you get permission errors:
```bash
chmod +x /usr/local/bin/cmdai
```

### Installation Script Fails

If the installation script fails:
1. Check your internet connection
2. Ensure `curl` is installed: `which curl`
3. Try manual installation instead
4. Report issues at: https://github.com/wildcard/cmdai/issues

## Getting Help

- **Documentation**: https://github.com/wildcard/cmdai
- **Issues**: https://github.com/wildcard/cmdai/issues
- **Discussions**: https://github.com/wildcard/cmdai/discussions

## Next Steps

Once installed, check out:
- [README.md](README.md) - Project overview and usage examples
- [Configuration Guide](docs/configuration.md) - Detailed configuration options
- [Safety Guide](docs/safety.md) - Understanding cmdai's safety features

---

**Installation Methods Quick Reference:**

| Platform | Method | Command |
|----------|--------|---------|
| macOS/Linux | Quick Install | `curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh \| bash` |
| macOS/Linux | Homebrew | `brew install wildcard/tap/cmdai` |
| All | Cargo | `cargo install cmdai` |
| Windows | Scoop | `scoop install cmdai` |
| Windows | Chocolatey | `choco install cmdai` |
| Windows | Winget | `winget install wildcard.cmdai` |
| All | Manual | Download from [releases](https://github.com/wildcard/cmdai/releases) |
