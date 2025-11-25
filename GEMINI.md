# Gemini Code Assistant Context: `cmdai` Project

This document provides a comprehensive overview of the `cmdai` project, its architecture, and development conventions to be used as a guide for the Gemini code assistant.

## Project Overview

`cmdai` is a Rust-based Command-Line Interface (CLI) tool designed to convert natural language descriptions into safe, POSIX-compliant shell commands. It leverages local Large Language Models (LLMs) to achieve this, with a strong emphasis on safety, performance, and portability.

-   **Core Functionality**: Translates natural language prompts into shell commands.
-   **Primary Technologies**: Rust
-   **Key Features**:
    -   **Safety-First Design**: A robust validation system with multiple risk levels (Safe, Moderate, High, Critical) to prevent the execution of dangerous commands.
    -   **High Performance**: Targets a sub-100ms startup time and sub-2-second inference time. The release binary is optimized for size (< 50MB).
    -   **Multiple Backends**: A trait-based, modular architecture supports various LLM backends:
        -   **Embedded**: MLX for Apple Silicon and a CPU-based fallback using the Candle framework.
        -   **Remote**: Ollama and vLLM backends are supported.
    -   **Cross-Platform**: Built to run on macOS, Linux, and Windows.
    -   **Single Binary**: Distributed as a self-contained executable with zero external dependencies.
-   **License**: AGPL-3.0

## Development Workflow

The project adheres to a rigorous, spec-first, test-driven development methodology.

### Spec-Driven Development (SDD)

-   All new features begin with a detailed specification located in the `specs/` directory.
-   Each feature specification includes API contracts, implementation plans, user scenarios, and data models.
-   This ensures that development is aligned with clear requirements before any code is written.

### Test-Driven Development (TDD)

-   A strict **Red-Green-Refactor** cycle is followed.
-   The `TDD-WORKFLOW.md` file contains detailed guidelines for this process.
-   Continuous feedback is achieved using `cargo-watch` or the custom `./scripts/test_watch.sh` script.
-   **Contract Testing**: Tests in `tests/contract/` are written first to validate the public API defined in the feature's specification.

## Building and Running

The `Makefile` provides a set of convenience scripts for common development tasks.

-   **Build**:
    -   `make build`: Compile the project in debug mode.
    -   `make release`: Create an optimized release binary.
-   **Testing**:
    -   `make test`: Run the complete test suite.
    -   `make test-watch`: Run tests continuously on file changes.
    -   `make test-nextest`: Use `cargo-nextest` for faster test execution.
    -   `make test-contract`: Run only the API contract tests.
-   **Quality Checks**:
    -   `make check`: A comprehensive check that runs `fmt`, `lint`, `audit`, and `test`.
    -   `make fmt`: Format the code using `rustfmt`.
    -   `make lint`: Run `clippy` with a strict policy (`-D warnings`).
-   **Installation**:
    -   `make install`: Install the `cmdai` binary to `~/.cargo/bin`.
-   **Running the application**:
    -   After building, the binary is located at `./target/release/cmdai`.
    -   Example: `./target/release/cmdai "list all rust files in the src directory"`

## Code Standards & Conventions

-   **Formatting**: Enforced by `rustfmt` using the configuration in `rustfmt.toml`.
-   **Linting**: `clippy` is used to enforce code quality. All warnings are treated as errors.
-   **Error Handling**:
    -   The application must not `panic` in production code.
    -   `thiserror` is used for creating custom error types in library code.
    -   `anyhow` is used for application-level error handling in the binary.
-   **Testing Strategy**:
    -   **Contract Tests**: `tests/contract/` ensures modules adhere to their specified API contracts.
    -   **Integration Tests**: `tests/integration/` validates cross-module workflows.
    -   **Property Tests**: `tests/property/` uses `proptest` to check for invariants.
-   **CI/CD**: The GitHub Actions workflow in `.github/workflows/ci.yml` automates testing across platforms (Linux, macOS, Windows), security audits with `cargo-audit`, and the creation of release builds.
-   **Contribution**: The `CONTRIBUTING.md` file provides detailed guidelines for submitting pull requests, and `AGENTS.md` describes conventions for AI-assisted development.
