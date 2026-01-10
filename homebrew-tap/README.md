# Homebrew Tap for Caro

This is the official [Homebrew](https://brew.sh/) tap for [caro](https://github.com/wildcard/caro), an AI-powered shell command assistant.

## Installation

```bash
# Add the tap
brew tap wildcard/caro https://github.com/wildcard/caro.git --force-auto-update

# Install caro
brew install wildcard/caro/caro
```

Or install directly without adding the tap:

```bash
brew install wildcard/caro/caro
```

## Upgrading

```bash
brew update
brew upgrade caro
```

## Uninstalling

```bash
brew uninstall caro
brew untap wildcard/caro
```

## Shell Completions

After installation, you can set up shell completions:

```bash
# Bash
caro --completion bash > $(brew --prefix)/etc/bash_completion.d/caro

# Zsh
caro --completion zsh > $(brew --prefix)/share/zsh/site-functions/_caro

# Fish
caro --completion fish > $(brew --prefix)/share/fish/vendor_completions.d/caro.fish
```

## Supported Platforms

| Platform | Architecture | Support |
|----------|--------------|---------|
| macOS | Apple Silicon (arm64) | Full support with MLX optimization |
| macOS | Intel (x86_64) | Full support |
| Linux | x86_64 | Full support |
| Linux | ARM64 | Full support |

## Troubleshooting

### Formula not found

If you get an error about the formula not being found, try:

```bash
brew tap --repair
brew update
```

### SHA256 mismatch

If you encounter a checksum error, it may indicate a release is being updated. Try again after a few minutes, or report the issue.

## For Maintainers

### Updating the Formula

When releasing a new version of caro:

1. Run the `scripts/update-homebrew-formula.sh` script after the release workflow completes
2. Or manually update the formula:
   - Update the `version` in `Formula/caro.rb`
   - Download the new binaries and compute their SHA256 checksums
   - Update the `sha256` values for each platform

### Computing SHA256 Checksums

```bash
# Download and compute checksums for a release
VERSION=1.1.0
for platform in macos-silicon macos-intel linux-amd64 linux-arm64; do
  curl -sL "https://github.com/wildcard/caro/releases/download/v${VERSION}/caro-${VERSION}-${platform}" | sha256sum
done
```

## License

This formula is part of the caro project, licensed under AGPL-3.0.
