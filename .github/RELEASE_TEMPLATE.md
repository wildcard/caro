## Installation

### Pre-built Binaries

Download the appropriate binary for your platform from the release assets below:

| Platform | Binary (versioned) |
|----------|-------------------|
| Linux x86_64 | `caro-VERSION-linux-amd64` |
| Linux ARM64 | `caro-VERSION-linux-arm64` |
| macOS Intel | `caro-VERSION-macos-intel` |
| macOS Apple Silicon | `caro-VERSION-macos-silicon` |
| Windows x64 | `caro-VERSION-windows-amd64.exe` |

All binaries include SHA256 checksum files (`.sha256`) for verification.

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
```

Or use the setup script:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/setup.sh)
```

### Via cargo

```bash
cargo install caro
```

For Apple Silicon with MLX optimization:

```bash
cargo install caro --features embedded-mlx
```

## Documentation

- **Website**: https://caro.sh
- **GitHub**: https://github.com/wildcard/caro
- **Getting Started**: Run `caro --help`

## Support

If you encounter any issues, please report them on [GitHub Issues](https://github.com/wildcard/caro/issues).
