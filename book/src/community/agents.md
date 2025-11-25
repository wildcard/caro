# Repository Guidelines

## Project Structure & Module Organization
- `src/` hosts the Rust crate: CLI wiring in `main.rs`, backend adapters in `src/backends/`, safety logic in `src/safety/`, and supporting layers (`cache/`, `config/`, `logging/`, `execution/`).
- `tests/` is contract-first: use `tests/contract/`, `tests/property/`, `tests/integration/`, and matching `*_tests.rs` entry points; scenario walk-throughs live in `tests/quickstart_scenarios.rs`.
- `benches/` contains Criterion benchmarks for performance envelopes, `specs/` captures product/architecture contracts, and `exports/` stores generated artifacts for reviews.

## Build, Test, and Development Commands
- `make build` / `make release` compile debug or optimized binaries via Cargo.
- `make test` runs the full suite (`cargo test -q --all-features`); `make test-contract`, `make test-integration`, and `make test-property` narrow focus. Use `make test-nextest` when `cargo-nextest` is installed.
- `make fmt`, `make lint`, and `make audit` enforce formatting (rustfmt), Clippy linting as errors, and advisory scanning; `make check` chains them with tests for pre-PR validation.
- `make bench` runs Criterion benches; `RUST_LOG=debug cargo run -- "<prompt>"` exercises the CLI during development.

## Coding Style & Naming Conventions
- Run `cargo fmt --all` before committing; formatting is pinned to 4-space indentation, 100-column line width, reordered imports, and 2021 edition idioms (`rustfmt.toml`).
- Keep types in UpperCamelCase, modules/functions in snake_case, and constants in SCREAMING_SNAKE_CASE; prefer descriptive enum variants (Clippy threshold is 3).
- Treat lint warnings as fatal (`make lint`); any allow attributes must be local with an explanation comment.

## Testing Guidelines
- Default to `make test`; add contract and property cases under their respective subdirectories using the `<suite>_tests.rs` pattern.
- Integration flows belong in `tests/integration/` and should assert prompt → command behavior, including safety validator outcomes.
- Document new scenarios in `specs/` when extending safety rules, and keep logs quiet by inheriting the provided `RUST_LOG` levels.

## Commit & Pull Request Guidelines
- Follow the existing history: concise, Title Case subjects, optional leading emoji, and PR references in parentheses (e.g., `(#12)`); keep the subject under 72 chars and use imperative mood when possible.
- Squash noisy work-in-progress commits before requesting review.
- PRs should describe intent, list touched modules, link to relevant specs/issues, and attach screenshots or command transcripts for user-visible changes.

## Security & Dependency Checks
- Run `make audit` (cargo-audit) before merging and address findings or document mitigations in the PR.
- For new third-party integrations, update `deny.toml` gating rules and capture required secrets or sandbox notes in `README.md` and `specs/`.
