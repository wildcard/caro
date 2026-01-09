# v1.1.0-beta.1 Installation Guide

⚠️ **This is a BETA release** - Not for general availability yet. For beta testers only.

---

## Prerequisites

- macOS (Apple Silicon or Intel) or Linux (x86_64 or ARM64)
- No external dependencies required (single binary)
- Optional: Rust toolchain for building from source

---

## Installation Methods

### Method 1: Direct Binary Download (Recommended for Beta Testers)

**macOS Apple Silicon (M1/M2/M3)**:
```bash
# Download beta binary
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-macos-aarch64 -o caro

# Make executable
chmod +x caro

# Move to PATH
sudo mv caro /usr/local/bin/caro

# Verify installation
caro --version
# Should show: caro 1.1.0-beta.1 (...)
```

**macOS Intel (x86_64)**:
```bash
# Download beta binary
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-macos-x86_64 -o caro

# Make executable and install
chmod +x caro
sudo mv caro /usr/local/bin/caro

# Verify
caro --version
```

**Linux x86_64**:
```bash
# Download beta binary
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-linux-x86_64 -o caro

# Make executable and install
chmod +x caro
sudo mv caro /usr/local/bin/caro

# Verify
caro --version
```

**Linux ARM64**:
```bash
# Download beta binary
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-linux-aarch64 -o caro

# Make executable and install
chmod +x caro
sudo mv caro /usr/local/bin/caro

# Verify
caro --version
```

---

### Method 2: Install Script (When Available)

```bash
# Install beta version using install script
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/release/v1.1.0/install.sh | bash
```

The script will:
- Detect your platform automatically
- Download the appropriate beta binary
- Install to `/usr/local/bin` or `~/.local/bin`
- Verify the installation

---

### Method 3: Build from Source

For developers or if you want to build from source:

```bash
# Clone repository
git clone https://github.com/wildcard/caro
cd caro

# Checkout beta tag
git checkout v1.1.0-beta.1

# Build release binary
cargo build --release

# The binary will be at: ./target/release/caro
./target/release/caro --version
# Should show: caro 1.1.0-beta.1

# Optional: Install to PATH
cargo install --path .
```

**Build Requirements**:
- Rust 1.83+ (install via rustup.rs)
- No other dependencies for basic build
- Optional: cmake (for MLX backend on Apple Silicon)

---

### Method 4: Cargo Install (For Beta Testing)

```bash
# Install beta from git repository
cargo install --git https://github.com/wildcard/caro --tag v1.1.0-beta.1

# Verify
caro --version
```

---

## Verification

After installation, verify the beta version:

```bash
# Check version
caro --version
# Expected output: caro 1.1.0-beta.1 (1e8ca84 2026-01-08)

# Run help
caro --help

# Test command generation
caro "list files"
# Should show telemetry notice and generate: ls -la

# Check system assessment (new feature)
caro assess

# Check health diagnostics (new feature)
caro doctor
```

---

## First Run Experience

On first run, you'll see:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  Telemetry & Privacy
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Caro is in beta and collects anonymous usage data to improve the product.

We collect:
  ✓ Session timing and performance metrics
  ✓ Platform info (OS, shell type)
  ✓ Error categories and safety events

We NEVER collect:
  ✗ Your commands or natural language input
  ✗ File paths or environment variables
  ✗ Any personally identifiable information

Learn more: https://caro.sh/telemetry
You can disable telemetry anytime with:
  caro config set telemetry.enabled false

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Telemetry is ON by default** for beta testing to collect quality data. You can opt-out anytime:
```bash
caro telemetry disable
```

---

## Troubleshooting

### Binary Not Executable
```bash
# Fix permissions
chmod +x /usr/local/bin/caro
```

### Command Not Found
```bash
# Check if in PATH
which caro

# If not, add to PATH (bash/zsh)
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Version Shows Wrong Number
```bash
# Remove old version
which caro  # Find location
sudo rm $(which caro)

# Reinstall beta version (see methods above)
```

### Platform Detection Issues
```bash
# Check your platform
uname -s  # Should be: Linux or Darwin
uname -m  # Should be: x86_64, aarch64, or arm64

# Download the correct binary for your platform
```

### Build from Source Fails
```bash
# Update Rust toolchain
rustup update

# Check Rust version
rustc --version  # Should be 1.83 or newer

# Clean and rebuild
cargo clean
cargo build --release
```

---

## What's New in v1.1.0-beta.1

**Major Improvements**:
- 🎯 93.1% pass rate on comprehensive test suite (up from 30% baseline)
- 🛡️ 52 safety patterns with 0% false positives
- 💻 System assessment (`caro assess`) - Analyzes CPU, GPU, memory
- 🏥 Health diagnostics (`caro doctor`) - Troubleshooting help
- 🚀 50+ static command patterns (instant generation)
- 📊 Privacy-first telemetry (anonymous, no PII)

**New Commands**:
```bash
caro assess           # System resource assessment
caro doctor           # Health check diagnostics
caro test             # Run beta test suite
caro telemetry status # Check telemetry settings
```

See [CHANGELOG.md](CHANGELOG.md) for complete details.

---

## Beta Testing Expectations

**What's Working**:
- ✅ Command generation (93.1% pass rate)
- ✅ Safety validation (0% false positives)
- ✅ System assessment and recommendations
- ✅ Health diagnostics
- ✅ Telemetry collection (privacy-first)

**Known Limitations**:
- ⚠️ MLX backend requires cmake to build
- ⚠️ Some E2E tests fail due to JSON parsing (P2 bug, doesn't affect users)
- ⚠️ Minor command variations (e.g., `ls -la` vs `ls -l`)

**Beta Testing Period**: 5 days with 3-5 testers

**Feedback**: Please report issues at [GitHub Issues](https://github.com/wildcard/caro/issues)

---

## Uninstallation

To remove caro:

```bash
# Remove binary
sudo rm /usr/local/bin/caro
# Or if installed to ~/.local/bin:
rm ~/.local/bin/caro

# Remove configuration and data
rm -rf ~/.config/caro
rm -rf ~/.local/share/caro

# If installed via cargo
cargo uninstall caro
```

---

## Support

**For Beta Testers**:
- Report issues: [GitHub Issues](https://github.com/wildcard/caro/issues)
- Ask questions: [GitHub Discussions](https://github.com/wildcard/caro/discussions)
- Email: [beta tester email if available]

**Documentation**:
- README: [README.md](README.md)
- Telemetry: [docs/TELEMETRY.md](docs/TELEMETRY.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md)

---

**Last Updated**: 2026-01-08
**Beta Version**: v1.1.0-beta.1
**Status**: Active Beta Testing
