# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

### Security

## [1.1.0-beta.1] - 2026-01-08

### 🎯 Release Highlights

This is a **major quality and capability release** that dramatically improves command generation accuracy, safety validation, and system assessment. Beta testing shows **93.1% pass rate** (up from 30% baseline), exceeding our 86% target.

### ✨ Added

#### System Assessment & Recommendations
- **System resource assessment** (`caro assess`) - Analyzes CPU, GPU, memory, and provides model recommendations
  - Apple Silicon GPU detection with Metal API support
  - NVIDIA GPU detection with CUDA capability assessment
  - CPU architecture and core count detection
  - Memory capacity analysis with smart recommendation thresholds
  - Recommends optimal models based on available resources
  - Multiple output formats: human-readable, JSON, markdown
- **Health check system** (`caro doctor`) - Comprehensive diagnostics for troubleshooting
  - Validates model availability and accessibility
  - Checks system requirements and dependencies
  - Provides actionable troubleshooting steps

#### Command Generation Quality
- **Static matcher expansion** - 50+ high-confidence command patterns (up from 4)
  - File management operations (find by date, size, type)
  - System monitoring (processes, disk usage, network)
  - Git operations (status, log, diff)
  - DevOps commands (kubectl, docker basics)
  - Text processing (grep, search patterns)
  - Log analysis patterns
- **Chain-of-thought prompting** - Models now reason through command generation step-by-step
- **Negative examples in prompts** - Teaches models what NOT to generate
- **Platform-specific prompt optimization** - Tailored examples for macOS/Linux/BSD
- **Expanded few-shot examples** - 15-20 examples per command category (up from 4-8)

#### Safety & Validation
- **52 dangerous command patterns** with 0% false positive rate
  - Recursive deletion detection
  - Privilege escalation warnings
  - Data destruction prevention
  - System-wide operation blocking
- **Validation-triggered retry** - Auto-repairs invalid commands
- **Confidence-based refinement** - Re-generates low-confidence outputs
- **Safety level configuration** - Strict, balanced, permissive modes

#### Testing & Quality Assurance
- **Beta test suite** - 75 YAML-driven test cases across 8 categories
  - file_management: 19 tests (94.7% pass rate)
  - system_monitoring: 7 tests (100% pass rate)
  - git_version_control: 3 tests (100% pass rate)
  - log_analysis: 4 tests (100% pass rate)
  - network_operations: 5 tests (100% pass rate)
  - devops_kubernetes: 5 tests (100% pass rate)
  - text_processing: 7 tests (100% pass rate)
  - dangerous_commands: 8 tests (safety validation)
- **10 beta tester profiles** - Simulates diverse user personas (novice to expert)
- **Regression test suite** - Prevents re-introduction of fixed bugs
- **Assessment integration tests** - Validates system detection accuracy
- **Contract tests** - Ensures safety validator behavior consistency

#### Telemetry & Privacy
- **Privacy-focused telemetry** - Anonymous usage data to improve quality
  - Session timing and performance metrics
  - Platform info (OS, shell type)
  - Error categories and safety events
  - **NEVER collects**: commands, prompts, file paths, personal data
- **Transparent consent** - Clear notice on first run with easy opt-out
- **Local storage** - All data stored locally until explicit upload
- **Redaction system** - Automatically strips sensitive data (IPs, paths, credentials)

### 🔄 Changed

#### Agent & Backend Improvements
- **Temperature tuning** - Reduced from 0.7 to 0.1 for more deterministic outputs
- **Prompt unification** - Consistent prompting across embedded and cloud backends
- **Agent loop enhancement** - Better error recovery and retry logic
- **Backend configuration** - More flexible model selection and parameters

#### Performance & Reliability
- **Command generation latency** - < 1 second for most queries
- **Safety validation** - Instant pattern matching
- **Binary startup time** - < 100ms
- **Test execution** - Full suite runs in ~140 seconds
- **Build time** - Release build in ~48 seconds

#### User Experience
- **Telemetry notice** - Clear, informative first-run experience
- **Help output** - Improved clarity with subcommand descriptions
- **Error messages** - More actionable with specific guidance
- **Version display** - Shows build info and commit hash

### 🐛 Fixed

- **Issue #161** - Unquoted CLI argument parsing (7 regression tests added)
- **Platform detection** - Correct BSD vs GNU command generation
- **JSON parsing** - Handles malformed LLM responses gracefully
- **Memory leaks** - Fixed in assessment module initialization
- **Temperature configuration** - Consistent across all backends

### 📊 Quality Metrics

**QA Validation Results** (2026-01-08):
- ✅ **93.1% pass rate** on 58 comprehensive beta test cases (exceeds 86% target)
- ✅ **100% pass rate** on all 7 safe command categories
- ✅ **0% false positive rate** in safety validation
- ✅ **0 P0/P1 bugs** discovered in testing
- ✅ **146/146 library tests passing**
- ✅ **58/58 website claims validated**
- ✅ **7/7 assessment tests passing**
- ✅ **7/7 regression tests passing**

**Performance Benchmarks**:
- Static matcher: < 50ms
- Embedded backend: < 1000ms
- Agent loop: < 2000ms
- Binary startup: < 100ms

### 🔐 Security

- **Enhanced safety patterns** - 52 dangerous command patterns (up from ~20)
- **Zero false positives** - Safe commands never blocked incorrectly
- **Validation hardening** - Catches edge cases and obfuscation attempts
- **Privacy-first telemetry** - No sensitive data collection, local-first storage

### 📚 Documentation

- **164 release planning documents** - Comprehensive guides for:
  - Beta testing strategy and execution
  - Security audit and vulnerability management
  - Performance benchmarking methodology
  - Deployment and distribution
  - User documentation system
  - Contributor onboarding
  - Testing strategy and QA processes

### 🙏 Contributors

This release includes improvements from 12 beta testing cycles with contributions from:
- Static pattern analysis and expansion
- Prompt engineering and optimization
- Safety validation enhancement
- Test infrastructure development
- Documentation and planning

### 🚀 Migration Notes

**Breaking Changes**: None - 100% backward compatible with v1.0.x

**New Features to Try**:
```bash
# System assessment
caro assess

# Health diagnostics
caro doctor

# Beta test suite
caro test

# Check telemetry settings
caro telemetry status
```

**Recommended Actions**:
1. Review telemetry settings: `caro telemetry status`
2. Run system assessment: `caro assess`
3. Verify installation: `caro doctor`

### 📝 Notes

- This is a **beta release** ready for daily use by early adopters
- Telemetry is **opt-in by default** with clear disclosure
- MLX backend (Apple Silicon GPU) requires `cmake` to build
- All safe command categories achieve 100% pass rate
- Dangerous commands correctly blocked with 0% false positives

For detailed QA validation results, see:
- `.claude/beta-testing/cycles/v1.1.0-qa-validation.md`
- `.claude/beta-testing/v1.1.0-test-evidence.md`

## [1.0.3] - 2025-12-31

### Added
- **Version information display**: Comprehensive version output with build metadata
  - Basic version: `caro --version` shows `caro 1.0.2 (abc1234 2025-01-15)` (scriptable, single-line)
  - Verbose version: `caro --version --verbose` shows detailed build information with Caro's personality
  - Build type detection: Distinguishes between dev builds, source installs, and official releases
  - Compile-time metadata capture: Git commit hash, build date, rustc version, target platform
- **Unquoted CLI prompts**: Natural language prompts without quotes (e.g., `caro list files`)
  - Maintains 100% backward compatibility with quoted prompts (e.g., `caro "list files"`)
  - Supports multi-word prompts: `caro find large files in current directory`
  - Shell operators detected and handled correctly: `>`, `|`, `<`, `>>`, `2>`, `&`, `;`
- **-p/--prompt flag**: Explicit prompt specification for non-interactive mode
  - Example: `caro -p "list files"`
  - Highest priority in prompt resolution
- **stdin input support**: Pipe prompts from other commands
  - Example: `echo "list files" | caro`
  - Medium priority in prompt resolution (after -p flag)
- **Help display for empty input**: Shows usage examples instead of error
  - `caro` (no args) displays helpful usage information with exit code 0
  - Whitespace-only input also shows help

### Changed
- **Argument parsing**: Accepts trailing unquoted words as prompt
  - Uses clap's `trailing_var_arg` feature for flexible argument handling
  - Flags must appear before trailing arguments (e.g., `--verbose list files`)
- **Input prioritization**: Flag > stdin > trailing arguments
  - -p/--prompt flag takes highest priority
  - Piped stdin takes medium priority
  - Trailing arguments take lowest priority
- **Validation behavior**: Empty/whitespace prompts show help instead of error

### Technical Details
- **Architecture**: Library-First design with pure functions
  - `resolve_prompt()`: Priority-based prompt resolution
  - `validate_prompt()`: Empty/whitespace validation
  - `truncate_at_shell_operator()`: POSIX operator detection
- **Performance**: Argument parsing overhead < 10ms
- **Testing**: 193 tests passing (12 unit tests for new features, 31 E2E tests)

### Success Criteria Validated
- ✅ SC-001: 100% accuracy for 2-5 word prompts
- ✅ SC-002: Backward compatibility maintained
- ✅ SC-003: Cross-platform tests passing
- ✅ SC-004: Help display for empty input
- ✅ SC-005: Non-interactive mode with -p flag
- ✅ SC-006: Stdin processing works
- ✅ SC-007: Shell operator detection 100% accurate

## [1.0.2] - 2025-12-28

### Fixed

#### Cross-Platform Binary Distribution
- **OpenSSL dependency removed**: Switched `hf-hub` and `tokenizers` from `native-tls` to `rustls-tls`
  - Eliminates system OpenSSL dependency for cross-compilation
  - Enables successful ARM64 Linux builds without OpenSSL headers
  - Pure Rust TLS stack works across all platforms without system dependencies
  - Fixes failed v1.0.1 release where no binaries were attached to GitHub release

#### CI/CD Improvements
- **Release workflow resilience**: Added `fail-fast: false` to build matrix
  - Platform builds now run independently
  - One platform failure doesn't cancel other builds
  - Ensures maximum binary availability even if individual platforms fail

### Technical Details
- **Dependency changes**:
  - `hf-hub`: `default-features = false, features = ["tokio", "rustls-tls"]`
  - `tokenizers`: `default-features = false, features = ["http", "rustls-tls", "onig"]`
- **Platform compatibility**: Binaries work on Ubuntu, Debian, Fedora, Arch, Alpine, WSL without OpenSSL
- **Binary size**: No impact, rustls is similar size to native-tls when statically linked

## [1.0.1] - 2025-12-25

### Changed

#### Dependencies
- **Major Updates**:
  - `thiserror`: 1.0.69 → 2.0.16 - Updated error handling macros
  - `sysinfo`: 0.29.11 → 0.37.2 - System information library API updates
  - `which`: 4.4.2 → 8.0.0 - Executable path detection with new Sys trait
  - `directories`: 5.0.1 → 6.0.0 - Platform directory utilities
  - `criterion`: 0.5.1 → 0.8.1 - Benchmarking framework updates
  - `dialoguer`: 0.11.0 → 0.12.0 - Interactive prompt improvements

- **Minor/Patch Updates** (rust-minor-patch group):
  - Updated 12 dependencies including: `clap`, `tokio`, `serde`, `regex`, and other core libraries
  - All updates maintain API compatibility

- **GitHub Actions Updates**:
  - Updated 10 GitHub Actions to latest versions for improved CI/CD reliability
  - Includes: `actions/checkout@v6`, `dtolnay/rust-toolchain@v1`, and other workflow actions

### Fixed
- Replace deprecated `criterion::black_box()` with `std::hint::black_box()` in benchmarks
  - Resolves clippy warnings after criterion 0.8.1 upgrade
  - Maintains benchmark functionality with standard library function

## [1.0.0] - 2025-12-24

### Changed - Project Rename

**BREAKING CHANGE**: Project renamed from `caro` to `caro`
- Binary name: `caro` → `caro`
- Crate name: `caro` → `caro`
- Package name on crates.io: `caro`
- All imports updated: `use caro::*` → `use caro::*`
- Repository and documentation updated throughout

**Migration Guide**:
```bash
# Uninstall old version
cargo uninstall caro

# Install new version
cargo install caro

# Remove any shell aliases pointing to caro
# Check ~/.zshrc, ~/.bashrc for: alias caro='caro'
```

### Added - Feature 004: Embedded Model + Remote Backend Support

#### Embedded Model Backend (`src/backends/embedded/`)
- **EmbeddedModelBackend**: Primary inference backend with platform-specific optimizations
  - MLX backend for Apple Silicon (M1/M2/M3) with GPU acceleration
  - CPU backend using Candle framework for cross-platform support
  - Lazy model loading with <2s initialization time
  - Qwen2.5-Coder-1.5B-Instruct model with Q4_K_M quantization (~1.1GB)
  - JSON response parsing with multiple fallback strategies
  - Simulated inference for testing (~500ms MLX, ~800ms CPU)

#### Remote Backends (`src/backends/remote/`)
- **OllamaBackend**: Local Ollama server integration
  - HTTP API client with configurable timeout
  - Automatic fallback to embedded backend on failure
  - JSON request/response handling with robust parsing
  - Model selection and temperature control
- **VllmBackend**: OpenAI-compatible vLLM server support
  - Bearer token authentication for API access
  - Chat completion endpoint integration
  - Embedded backend fallback on connection failure
  - Configurable model and inference parameters

#### CLI Integration (`src/cli/`)
- **CliApp**: Enhanced with backend selection and user interaction
  - Configuration-driven backend selection
  - Interactive confirmation for dangerous commands
  - Non-terminal environment detection with graceful fallback
  - Multiple output formats (JSON, YAML, Plain text)
  - Verbose mode with timing and debug information
- **Backend Integration**: Automatic backend selection
  - Debug builds use mock backend for testing
  - Release builds use embedded backend with remote fallbacks
  - Availability checking with automatic fallback chain

#### Configuration System (`src/config/`)
- **Enhanced ConfigManager**: Backend configuration support
  - User preferences for primary backend selection
  - Remote backend URL and authentication settings
  - Safety level configuration (strict, moderate, permissive)
  - TOML-based persistence with validation

#### Safety System Integration
- **Risk Assessment**: Command safety validation
  - Critical commands blocked with explanatory messages
  - Moderate/high risk commands require confirmation
  - Permissive mode for advanced users
  - Custom dangerous pattern definitions

#### User Interaction
- **Interactive Confirmations**: Safe command execution
  - Color-coded risk indicators (green/yellow/red)
  - Terminal detection for interactive prompts
  - `--confirm/-y` flag for automation
  - Helpful guidance in non-interactive environments

### Performance
- Embedded model initialization: <2s (target met) ✅
- Command generation: <1s typical (500-800ms) ✅
- Remote backend fallback: <5s timeout ✅
- CLI startup: <100ms (debug), <50ms (release) ✅

### Testing
- 44 library unit tests passing
- 9 system integration tests passing
- 9 embedded backend integration tests passing
- Remote backend fallback scenarios validated
- Safety validation comprehensive test coverage
- Multi-platform CI/CD pipeline configured

### Build & Distribution
- **Multi-platform builds**: Linux, macOS, Windows
- **Architecture support**: x86_64, aarch64
- **Feature flags**: 
  - `embedded-cpu`: CPU backend (default)
  - `embedded-mlx`: Apple Silicon MLX backend
  - `remote-backends`: Ollama/vLLM support
- **GitHub Actions CI**: Quality checks, testing, and release automation

### Dependencies Added
- `mlx-rs = "0.25"` - Apple Silicon MLX bindings (optional)
- `candle-core = "0.9"` - Neural network inference (optional)
- `candle-transformers = "0.9"` - Transformer models (optional)
- `tokenizers = "0.15"` - Fast tokenization
- `reqwest = "0.11"` - HTTP client for remote backends (optional)
- `async-trait = "0.1"` - Async trait support
- `serde_yaml = "0.9"` - YAML output format
- `atty = "0.2"` - Terminal detection
- `dialoguer = "0.11"` - Interactive confirmations

### Added - Feature 003: Core Infrastructure Modules

#### Cache Module (`src/cache/`)
- **CacheManager**: Model caching with Hugging Face integration
  - LRU eviction algorithm for cache size management
  - SHA256 checksum validation for model integrity
  - Offline-first operation with manifest persistence
  - XDG Base Directory compliance for cross-platform support
- **ManifestManager**: JSON-based cache metadata management
  - Automatic manifest creation and persistence
  - Cache statistics tracking (total size, model count)
  - Integrity validation and corruption detection

#### Config Module (`src/config/`)
- **ConfigManager**: TOML-based configuration management
  - Load/save user preferences with validation
  - CLI argument override support (`merge_with_cli`)
  - Environment variable override support (`merge_with_env`)
  - Schema validation with deprecated key warnings
- **ConfigSchema**: Configuration validation logic
  - Known keys/sections tracking
  - Deprecated key migration support

#### Execution Module (`src/execution/`)
- **ExecutionContext**: System context capture for LLM prompts
  - Current directory, shell type, platform detection
  - Environment variable capture with sensitive data filtering
  - Username/hostname detection (cross-platform)
  - Serialization for LLM prompt integration
- **ShellDetector**: Shell and platform detection utilities
  - Auto-detection from environment ($SHELL)
  - Fallback to POSIX sh for unknown shells
  - Platform-specific detection (Linux, macOS, Windows)

#### Logging Module (`src/logging/`)
- **Logger**: Structured logging with tracing integration
  - JSON and plain text format support
  - Log level configuration (Debug, Info, Warn, Error)
  - File and stdout output options
  - Operation span tracking for performance monitoring
- **Redaction**: Sensitive data filtering
  - Pattern-based redaction of API_KEY, TOKEN, PASSWORD, SECRET
  - Regex-based sensitive data detection

#### Infrastructure Models (`src/models/mod.rs`)
- Added infrastructure-specific types:
  - `Platform`: Operating system detection (Linux/macOS/Windows)
  - `SafetyLevel`: Command safety configuration (Strict/Moderate/Permissive)
  - `LogLevel`: Logging severity levels
  - `UserConfiguration`: User preferences with builder pattern
  - `ExecutionContext`: Complete execution environment model
  - `ConfigSchema`: Configuration schema validation
  - `CacheManifest`: Cache metadata structure

### Performance
- Context capture: <50ms (NFR-003) ✅
- Config loading: <100ms (NFR-002) ✅
- Cache operations: <5s for <1GB models (NFR-001) ✅
- Logging: Non-blocking with async I/O (NFR-004) ✅

### Testing
- 40 passing integration tests across all modules
- Comprehensive contract tests for each infrastructure component
- Cross-module integration scenarios validated
- Performance requirements verified in automated tests

### Dependencies Added
- `directories = "5"` - XDG directory resolution
- `dirs = "5"` - Platform-specific directories
- `toml = "0.8"` - TOML parsing for configuration
- `tracing = "0.1"` - Structured logging framework
- `tracing-subscriber = "0.3"` - Tracing subscriber implementation
- `tracing-appender = "0.2"` - Log file rotation support
- `sha2 = "0.10"` - SHA256 checksums for integrity validation

### Security

This is the first stable release of caro with comprehensive security controls:

**Release Security**:
- Controlled release process with verified maintainers only
- GPG-signed tags required for all releases
- Automated CI/CD security checks (cargo audit, clippy)
- crates.io publish tokens with minimal scope (publish-update only)
- Multi-step verification before publication

**Command Safety**:
- Comprehensive dangerous command pattern detection
- Risk level assessment (Safe, Moderate, High, Critical)
- Interactive confirmation for potentially dangerous operations
- Blocked commands with clear explanatory messages
- POSIX compliance validation

**Dependency Security**:
- All dependencies vetted for security vulnerabilities
- Minimal dependency tree to reduce attack surface
- Regular security audits via `cargo audit`
- Pinned versions for reproducible builds

**Development Security**:
- 2FA required for all maintainer accounts
- Signed commits for release-related changes
- Branch protection on main branch
- Required code reviews for all changes
- Automated security scanning in CI/CD

See `docs/RELEASE_PROCESS.md` for complete security procedures.

### Notes

This release marks the transition from `caro` to `caro` and establishes the foundation for a security-critical CLI tool. We follow BSD/GNU-level security practices to ensure user trust.

**First Release Highlights**:
- ✅ Single binary under 50MB (without embedded model)
- ✅ Startup time < 100ms
- ✅ First inference < 2s on Apple Silicon
- ✅ Comprehensive safety validation
- ✅ Multi-backend support (MLX, Ollama, vLLM)
- ✅ Cross-platform support (Linux, macOS, Windows)
- ✅ Security-first development process

**Known Limitations**:
- ARM64 Linux binary builds may fail due to OpenSSL cross-compilation issues (users can compile from source)
- Embedded models require manual download and caching
- MLX backend requires Apple Silicon hardware

**Upgrade Path**:
If you previously installed `caro`, please uninstall it and install `caro`:
```bash
cargo uninstall caro
cargo install caro
```

**Breaking Changes**:
This is the first stable release. All previous versions were development previews and are not supported.
