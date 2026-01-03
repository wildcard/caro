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
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Use stable Rust from fenix for reproducibility
        toolchain = fenix.packages.${system}.stable.toolchain;

        # Crane for optimized Rust builds with caching
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        # Source filtering - only include Rust-relevant files
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        # Common build arguments shared between dependency and final builds
        commonArgs = {
          inherit src;

          strictDeps = true;

          # Build inputs needed for compilation
          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.libiconv
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            installShellFiles
          ];

          # Environment variables for build
          OPENSSL_NO_VENDOR = "1";
        };

        # Build dependencies only (cached separately for faster rebuilds)
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # Use features that work on all platforms for dependency build
          cargoExtraArgs = "--no-default-features --features embedded-cpu,remote-backends";
        });

        # Platform-specific feature selection
        cargoFeatures =
          if pkgs.stdenv.isDarwin && pkgs.stdenv.isAarch64
          then "--no-default-features --features embedded-cpu,remote-backends"
          else "--no-default-features --features embedded-cpu,remote-backends";

        # Main caro package
        caro = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          cargoExtraArgs = cargoFeatures;

          # Generate and install shell completions
          postInstall = ''
            # Only generate completions if the binary supports it
            if $out/bin/caro completions bash &>/dev/null; then
              installShellCompletion --cmd caro \
                --bash <($out/bin/caro completions bash) \
                --zsh <($out/bin/caro completions zsh) \
                --fish <($out/bin/caro completions fish)
            fi
          '';

          meta = with pkgs.lib; {
            description = "Convert natural language to shell commands using local LLMs";
            homepage = "https://caro.sh";
            repository = "https://github.com/wildcard/caro";
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

        # Development shell with all necessary tools
        devShells.default = craneLib.devShell {
          # Include caro build dependencies
          inputsFrom = [ caro ];

          # Additional development tools
          packages = with pkgs; [
            # Rust toolchain (from fenix)
            toolchain

            # Cargo extensions
            cargo-watch
            cargo-audit
            cargo-edit
            cargo-outdated
            cargo-deny
            cargo-nextest
            cargo-llvm-cov

            # Build tools
            just
            jq

            # Documentation
            mdbook
          ];

          # Development environment variables
          RUST_BACKTRACE = "1";
          RUST_LOG = "info";

          shellHook = ''
            echo "caro development shell"
            echo "  cargo build    - Build the project"
            echo "  cargo test     - Run tests"
            echo "  cargo watch    - Watch for changes"
            echo ""
          '';
        };

        # Formatter for nix files
        formatter = pkgs.nixpkgs-fmt;

        # CI checks
        checks = {
          # Build the package
          inherit caro;

          # Format check
          fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Clippy linting
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets ${cargoFeatures} -- -D warnings";
          });

          # Run tests
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = cargoFeatures;
            # Skip tests that require model downloads in Nix sandbox
            cargoTestExtraArgs = "-- --skip smoke --skip e2e --skip integration";
          });
        };
      }
    ) // {
      # Overlays for use in other flakes
      overlays.default = final: prev: {
        caro = self.packages.${final.system}.caro;
      };

      # NixOS module for system-wide installation
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.caro;
        in {
          options.programs.caro = {
            enable = lib.mkEnableOption "caro - natural language to shell commands";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.caro;
              defaultText = lib.literalExpression "inputs.caro.packages.\${system}.caro";
              description = "The caro package to use.";
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];
          };
        };

      # Home Manager module for per-user configuration
      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.caro;
          tomlFormat = pkgs.formats.toml { };
        in {
          options.programs.caro = {
            enable = lib.mkEnableOption "caro - natural language to shell commands";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.caro;
              defaultText = lib.literalExpression "inputs.caro.packages.\${system}.caro";
              description = "The caro package to use.";
            };

            settings = lib.mkOption {
              type = tomlFormat.type;
              default = { };
              example = lib.literalExpression ''
                {
                  backend = "ollama";
                  model = "qwen2.5-coder:7b";
                  safety = {
                    enabled = true;
                    confirm_dangerous = true;
                  };
                }
              '';
              description = ''
                Configuration written to {file}`~/.config/caro/config.toml`.
                See <https://caro.sh/docs/configuration> for available options.
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            home.packages = [ cfg.package ];

            xdg.configFile."caro/config.toml" = lib.mkIf (cfg.settings != { }) {
              source = tomlFormat.generate "caro-config" cfg.settings;
            };
          };
        };
    };
}
