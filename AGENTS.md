# Repository Guidelines

## Project Structure & Module Organization
The Rust crate lives in `src/`, with CLI wiring in `src/main.rs`, adapters in `src/backends/`, safety evaluators in `src/safety/`, and shared layers under `src/{cache,config,execution,logging}/`. Tests are contract-first inside `tests/`, grouped by `tests/{contract,property,integration}/` with `*_tests.rs` entrypoints; walkthrough scenarios sit in `tests/quickstart_scenarios.rs`. Benchmarks land in `benches/`, formal expectations in `specs/`, generated artifacts in `exports/`, docs in `docs/`, and helper utilities in `scripts/`.

## Build, Test, and Development Commands
Use `make build` for a debug binary and `make release` for optimized output. `make test` runs `cargo test -q --all-features`, while `make test-contract`, `make test-integration`, and `make test-property` scope to individual suites. Run `make fmt`, `make lint`, and `make audit` to enforce rustfmt, Clippy-as-errors, and cargo-audit; `make check` chains them. For manual drills, invoke `RUST_LOG=debug cargo run -- "<prompt>"`, and profile hotspots with `make bench`.

## Coding Style & Naming Conventions
Format with `cargo fmt --all` (4-space indent, 100-column width, ordered imports per `rustfmt.toml`). Favor UpperCamelCase for types, snake_case for functions and modules, SCREAMING_SNAKE_CASE for constants, and descriptive enum variants (Clippy warns below three characters). Treat every lint warning as a failure; if you must `allow`, scope it tightly and document the risk.

## Testing Guidelines
Default to `make test`; add focused cases near the code under test. Contract and property suites belong in their respective directories, while integration tests validate prompt → command flows including safety decisions. Keep automation logs quiet by inheriting default `RUST_LOG`. Capture new safety rules or test expectations in `specs/` alongside code changes.

## Commit & Pull Request Guidelines
Use concise, Title Case commit subjects with optional leading emoji and trailing PR references such as `(#42)`. Squash noisy WIP commits. Pull requests should state intent, list touched modules, link relevant issues or specs, and include transcripts or screenshots for user-visible changes.

## Security & Configuration Tips
Run `make audit` before merging and note persisting findings in the PR description. Update `deny.toml`, `README.md`, and `specs/` when dependencies, sandbox assumptions, or secret workflows change so operators can follow the latest guardrails.
