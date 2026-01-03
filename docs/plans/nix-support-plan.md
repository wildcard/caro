# Complete Nix Support Plan for Caro

## Overview

This plan outlines the implementation of complete Nix support for the caro CLI tool, enabling reproducible builds, development environments, and seamless installation via the Nix package manager.

**Reference Implementation:** [Atuin Nix Support](https://github.com/atuinsh/atuin)

---

## Goals

1. **Reproducible Builds** - Ensure caro builds identically on any Nix-enabled system
2. **Easy Installation** - `nix run github:wildcard/caro` or `nix profile install github:wildcard/caro`
3. **Developer Experience** - Provide a complete dev shell with all dependencies
4. **CI Validation** - Verify Nix builds on every PR
5. **NixOS Integration** - Enable caro as a NixOS module option
6. **Overlay Support** - Allow integration into existing Nix configurations

---

## Implementation Components

### 1. `flake.nix` - Main Nix Flake

**Location:** `/flake.nix`

```nix
{
  description = "caro - Convert natural language to shell commands using local LLMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Use stable Rust from fenix
        toolchain = fenix.packages.${system}.stable.toolchain;

        # Crane for Rust builds with caching
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        # Common arguments for all builds
        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          # Build inputs needed for compilation
          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
            libiconv
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            installShellFiles
          ];

          # Environment variables
          OPENSSL_NO_VENDOR = "1";
        };

        # Build dependencies only (for caching)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Main caro package
        caro = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          # Use minimal features for Nix build (no MLX - requires macOS-specific setup)
          cargoExtraArgs = "--no-default-features --features embedded-cpu,remote-backends";

          # Generate and install shell completions
          postInstall = ''
            installShellCompletion --cmd caro \
              --bash <($out/bin/caro completions bash) \
              --zsh <($out/bin/caro completions zsh) \
              --fish <($out/bin/caro completions fish)
          '';

          meta = with pkgs.lib; {
            description = "Convert natural language to shell commands using local LLMs";
            homepage = "https://caro.sh";
            license = licenses.agpl3Plus;
            maintainers = [ ];
            mainProgram = "caro";
            platforms = platforms.unix;
          };
        });

      in {
        # Packages
        packages = {
          inherit caro;
          default = caro;
        };

        # Apps for `nix run`
        apps = {
          caro = flake-utils.lib.mkApp { drv = caro; };
          default = self.apps.${system}.caro;
        };

        # Development shell
        devShells.default = craneLib.devShell {
          # Include caro in dev shell
          inputsFrom = [ caro ];

          # Additional development tools
          packages = with pkgs; [
            # Rust tools
            cargo-watch
            cargo-audit
            cargo-edit
            cargo-outdated
            cargo-deny

            # Testing
            cargo-nextest
            cargo-llvm-cov

            # Formatting & linting
            rustfmt
            clippy

            # Documentation
            mdbook

            # Utilities
            just
            jq
          ];

          # Development environment variables
          RUST_BACKTRACE = "1";
          RUST_LOG = "info";
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;

        # Checks for CI
        checks = {
          inherit caro;

          # Format check
          fmt = craneLib.cargoFmt { inherit (commonArgs) src; };

          # Clippy
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });

          # Tests
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--no-default-features --features embedded-cpu,remote-backends";
          });

          # Audit
          audit = craneLib.cargoAudit {
            inherit (commonArgs) src;
            advisory-db = pkgs.fetchFromGitHub {
              owner = "rustsec";
              repo = "advisory-db";
              rev = "main";
              sha256 = pkgs.lib.fakeSha256; # Will be replaced on first build
            };
          };
        };
      }
    ) // {
      # Overlays for use in other flakes
      overlays.default = final: prev: {
        caro = self.packages.${final.system}.caro;
      };

      # NixOS module (optional, for system-wide configuration)
      nixosModules.default = { config, lib, pkgs, ... }: {
        options.programs.caro = {
          enable = lib.mkEnableOption "caro - natural language to shell commands";
          package = lib.mkOption {
            type = lib.types.package;
            default = self.packages.${pkgs.system}.caro;
            description = "The caro package to use";
          };
        };

        config = lib.mkIf config.programs.caro.enable {
          environment.systemPackages = [ config.programs.caro.package ];
        };
      };

      # Home Manager module (for per-user configuration)
      homeManagerModules.default = { config, lib, pkgs, ... }: {
        options.programs.caro = {
          enable = lib.mkEnableOption "caro - natural language to shell commands";
          package = lib.mkOption {
            type = lib.types.package;
            default = self.packages.${pkgs.system}.caro;
            description = "The caro package to use";
          };
          settings = lib.mkOption {
            type = lib.types.attrs;
            default = { };
            description = "Configuration for caro (written to ~/.config/caro/config.toml)";
          };
        };

        config = lib.mkIf config.programs.caro.enable {
          home.packages = [ config.programs.caro.package ];

          xdg.configFile."caro/config.toml" = lib.mkIf (config.programs.caro.settings != { }) {
            source = (pkgs.formats.toml { }).generate "caro-config" config.programs.caro.settings;
          };
        };
      };
    };
}
```

---

### 2. `.github/workflows/nix.yml` - CI Workflow

**Location:** `/.github/workflows/nix.yml`

```yaml
# Verify the Nix build is working
# Failures usually occur due to an out of date Rust version
# Update with: nix flake update
name: Nix

on:
  push:
    branches: [main, develop]
    paths-ignore:
      - 'website/**'
      - 'docs-site/**'
      - '*.md'
  pull_request:
    branches: [main]
    paths-ignore:
      - 'website/**'
      - 'docs-site/**'
      - '*.md'

jobs:
  # Validate flake structure and evaluate outputs
  flake-check:
    name: Flake Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install Nix
        uses: cachix/install-nix-action@v31
        with:
          nix_path: nixpkgs=channel:nixpkgs-unstable
          extra_nix_config: |
            experimental-features = nix-command flakes
            access-tokens = github.com=${{ secrets.GITHUB_TOKEN }}

      - name: Check flake
        run: nix flake check --print-build-logs

  # Build the package
  build:
    name: Build (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v6

      - name: Install Nix
        uses: cachix/install-nix-action@v31
        with:
          nix_path: nixpkgs=channel:nixpkgs-unstable
          extra_nix_config: |
            experimental-features = nix-command flakes
            access-tokens = github.com=${{ secrets.GITHUB_TOKEN }}

      - name: Setup Cachix (optional - for caching)
        uses: cachix/cachix-action@v17
        with:
          name: caro
          authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
        continue-on-error: true

      - name: Build caro
        run: nix build --print-build-logs

      - name: Test binary
        run: |
          ./result/bin/caro --version
          ./result/bin/caro --help

  # Run nix flake checks (fmt, clippy, test, audit)
  checks:
    name: Nix Checks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install Nix
        uses: cachix/install-nix-action@v31
        with:
          nix_path: nixpkgs=channel:nixpkgs-unstable
          extra_nix_config: |
            experimental-features = nix-command flakes
            access-tokens = github.com=${{ secrets.GITHUB_TOKEN }}

      - name: Run all checks
        run: |
          nix build .#checks.x86_64-linux.fmt --print-build-logs
          nix build .#checks.x86_64-linux.clippy --print-build-logs
          nix build .#checks.x86_64-linux.test --print-build-logs

  # Development shell validation
  devshell:
    name: Dev Shell
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install Nix
        uses: cachix/install-nix-action@v31
        with:
          nix_path: nixpkgs=channel:nixpkgs-unstable
          extra_nix_config: |
            experimental-features = nix-command flakes
            access-tokens = github.com=${{ secrets.GITHUB_TOKEN }}

      - name: Enter dev shell and verify tools
        run: |
          nix develop --command bash -c "
            echo '=== Verifying dev shell tools ==='
            rustc --version
            cargo --version
            cargo fmt --version
            cargo clippy --version
            echo '=== Dev shell validated ==='
          "
```

---

### 3. `shell.nix` - Backwards Compatibility

**Location:** `/shell.nix`

For users who haven't enabled flakes:

```nix
# Backwards-compatible shell.nix for users without flakes enabled
# Prefer using: nix develop (with flakes)

(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  in fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
    sha256 = lock.nodes.flake-compat.locked.narHash;
  }
) {
  src = ./.;
}).shellNix
```

---

### 4. `default.nix` - Backwards Compatibility

**Location:** `/default.nix`

For `nix-build` compatibility:

```nix
# Backwards-compatible default.nix for users without flakes enabled
# Prefer using: nix build (with flakes)

(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  in fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
    sha256 = lock.nodes.flake-compat.locked.narHash;
  }
) {
  src = ./.;
}).defaultNix
```

---

### 5. `.envrc` - Direnv Integration

**Location:** `/.envrc`

For automatic environment loading with direnv:

```bash
# Automatically enter nix development shell
use flake

# Optional: Add project-specific environment variables
export RUST_BACKTRACE=1
export RUST_LOG=info
```

---

### 6. README Updates

Add to the installation section of README.md:

```markdown
#### Option 5: Nix (Reproducible)

**With Flakes (recommended):**
```bash
# Run directly without installing
nix run github:wildcard/caro -- "list files in current directory"

# Install to your profile
nix profile install github:wildcard/caro

# Enter development shell
nix develop github:wildcard/caro
```

**Traditional Nix:**
```bash
nix-env -iA caro -f https://github.com/wildcard/caro/archive/main.tar.gz
```

**NixOS Configuration:**
```nix
{
  inputs.caro.url = "github:wildcard/caro";

  # In your configuration.nix
  environment.systemPackages = [ inputs.caro.packages.${system}.default ];

  # Or using the NixOS module
  imports = [ inputs.caro.nixosModules.default ];
  programs.caro.enable = true;
}
```

**Home Manager:**
```nix
{
  imports = [ inputs.caro.homeManagerModules.default ];

  programs.caro = {
    enable = true;
    settings = {
      backend = "ollama";
      model = "qwen2.5-coder:7b";
    };
  };
}
```
```

---

## Implementation Phases

### Phase 1: Core Flake (Priority: High)
**Estimated Files:** 3
**Dependencies:** None

1. Create `flake.nix` with basic package definition
2. Create `shell.nix` for backwards compatibility
3. Create `default.nix` for nix-build support
4. Generate initial `flake.lock`

**Validation:**
```bash
nix flake check
nix build
nix develop
```

### Phase 2: CI Integration (Priority: High)
**Estimated Files:** 1
**Dependencies:** Phase 1

1. Create `.github/workflows/nix.yml`
2. Integrate with existing CI matrix
3. Setup Cachix for build caching (optional)

**Validation:**
- PR builds succeed on GitHub Actions
- Build times are reasonable (<10 min)

### Phase 3: Developer Experience (Priority: Medium)
**Estimated Files:** 2
**Dependencies:** Phase 1

1. Add `.envrc` for direnv integration
2. Configure dev shell with all necessary tools
3. Add shell completions generation

**Validation:**
```bash
cd caro && direnv allow
cargo build  # Works in nix shell
```

### Phase 4: Distribution Modules (Priority: Low)
**Estimated Files:** 0 (already in flake.nix)
**Dependencies:** Phase 1

1. NixOS module for system-wide installation
2. Home Manager module for user configuration
3. Overlay for integration with other flakes

**Validation:**
- Test NixOS module in a NixOS VM
- Test Home Manager module

### Phase 5: Documentation (Priority: Medium)
**Estimated Files:** 1
**Dependencies:** Phase 1

1. Update README.md with Nix installation options
2. Add Nix-specific troubleshooting guide
3. Document Home Manager configuration options

---

## File Structure Summary

```
caro/
├── flake.nix              # Main Nix flake (NEW)
├── flake.lock             # Lock file (GENERATED)
├── shell.nix              # Backwards compat (NEW)
├── default.nix            # Backwards compat (NEW)
├── .envrc                 # Direnv integration (NEW)
├── .github/
│   └── workflows/
│       └── nix.yml        # CI workflow (NEW)
├── README.md              # Updated with Nix install
└── docs/
    └── plans/
        └── nix-support-plan.md  # This plan
```

---

## Platform Considerations

### macOS / Apple Silicon
- MLX features require macOS-specific frameworks
- Use `lib.optionals stdenv.isDarwin` for conditional deps
- Consider separate `caro-mlx` package for Apple Silicon

### Linux
- Works with `embedded-cpu` and `remote-backends` features
- Full support for all CPU architectures via Nix cross-compilation

### Feature Flags Mapping

| Cargo Feature | Nix Support | Notes |
|---------------|-------------|-------|
| `embedded-cpu` | Full | Default in Nix build |
| `remote-backends` | Full | Requires network access |
| `embedded-mlx` | macOS only | Requires Apple frameworks |
| `mock-backend` | Dev only | For testing |

---

## Optional Enhancements

### Cachix Binary Cache
Setup at https://app.cachix.io:
1. Create `caro` cache
2. Add `CACHIX_AUTH_TOKEN` to GitHub secrets
3. Uncomment cachix-action in CI

### Cross-Compilation
```nix
# In flake.nix outputs
packages.aarch64-linux = crane.lib.aarch64-linux.buildPackage { ... };
packages.x86_64-darwin = crane.lib.x86_64-darwin.buildPackage { ... };
```

### Nix Package in nixpkgs
After stabilization, submit PR to nixpkgs:
- Follow [nixpkgs contributing guide](https://github.com/NixOS/nixpkgs/blob/master/CONTRIBUTING.md)
- Package goes in `pkgs/by-name/ca/caro/package.nix`

---

## Testing Plan

1. **Local Development**
   ```bash
   nix flake check     # All checks pass
   nix build           # Package builds
   nix develop         # Shell works
   ```

2. **CI Validation**
   - All workflow jobs pass
   - Build time < 10 minutes (with cache)

3. **Installation Testing**
   ```bash
   nix run .             # Direct run works
   nix profile install . # Profile install works
   ./result/bin/caro --version  # Binary works
   ```

4. **Cross-Platform**
   - Test on NixOS (VM)
   - Test on macOS with Nix
   - Test on Ubuntu with Nix

---

## Success Criteria

- [ ] `nix flake check` passes
- [ ] `nix build` produces working binary
- [ ] `nix develop` provides complete dev environment
- [ ] CI workflow runs on all PRs
- [ ] README documents Nix installation
- [ ] Binary includes shell completions
- [ ] Works on both Linux and macOS

---

## References

- [Atuin Nix Implementation](https://github.com/atuinsh/atuin) - Primary reference
- [Crane Documentation](https://crane.dev) - Rust + Nix best practices
- [Nix Flakes](https://nixos.wiki/wiki/Flakes) - Flake specification
- [Cachix](https://cachix.org) - Binary caching
- [Home Manager](https://github.com/nix-community/home-manager) - User configuration
