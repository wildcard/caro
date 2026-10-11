# Caro

> Caro converts natural language into safe shell commands: POSIX shells (bash, zsh, fish, sh) and, on Windows, PowerShell and cmd. It runs a local LLM on your machine and checks each generated command against more than 52 dangerous-command patterns. A command that a user explicitly allowlists skips those checks, except a floor of catastrophic patterns that always applies.

Caro is a single Rust binary for macOS, Linux and Windows, at version {{version}}. It is open source under the AGPL-3.0 license.

- Install: `cargo install caro`, or `brew install wildcard/tap/caro`. Apple Silicon users can add `--features embedded-mlx`. Details: https://caro.sh/docs/installation
- Use: `caro "list all PDF files in Downloads larger than 10MB"` prints the command and asks before it runs it.
- Safety: deterministic pattern checks rate each command as Safe, Moderate, High or Critical risk, and block or warn. They do not depend on the model's judgement.
- Backends: an embedded local model by default (MLX on Apple Silicon, CPU elsewhere). Ollama, vLLM, Exo, Mesh-LLM and AI-Horde backends are optional.
- Privacy: telemetry is opt-in. Policy: https://caro.sh/telemetry
- Source: https://github.com/wildcard/caro. Crate: https://crates.io/crates/caro
