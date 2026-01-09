# ⚠️ v1.1.0-beta.1 - Beta Release (NOT FOR GENERAL AVAILABILITY)

**Release Date**: 2026-01-08
**Status**: 🧪 Beta Testing Only
**Version**: 1.1.0-beta.1

---

## 🚨 IMPORTANT: This is a Beta Release

This is a **beta release** for early testing and feedback. **Not recommended for general availability yet.**

- ✅ Stable and safe for daily use by early adopters
- ✅ Ready for beta testers (5-day testing cycle)
- ❌ NOT for production systems yet
- ❌ NOT for general audience

**Beta Testing Period**: 5 days with 3-5 testers
**GA Release**: ONLY if explicitly decided after successful beta

---

## 🎯 Release Highlights

This is a **major quality and capability release** that dramatically improves command generation accuracy, safety validation, and system assessment.

**Key Metrics**:
- 🎯 **93.1% pass rate** (up from 30% baseline, exceeding 86% target)
- 🛡️ **0% false positive rate** (52 safety patterns)
- ⚡ **<1s command generation** (50+ static patterns)
- 🧪 **146/146 library tests passing**

---

## ✨ What's New in v1.1.0-beta.1

### System Assessment & Recommendations
**New Command**: `caro assess`

Analyzes your system and provides intelligent model recommendations:
- Apple Silicon GPU detection with Metal API support
- NVIDIA GPU detection with CUDA capability assessment
- CPU analysis (cores, architecture)
- Memory analysis with recommendations
- Intelligent backend recommendations

```bash
$ caro assess

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  System Assessment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CPU: Apple M3 Max (16 cores)
GPU: Apple M3 Max GPU (Metal)
Memory: 128 GB

💡 Recommendation: MLX backend (optimized for Apple Silicon)
```

### Health Diagnostics
**New Command**: `caro doctor`

Troubleshooting and health check:
- Platform detection verification
- Backend availability check
- Configuration validation
- Model download status
- Common issue diagnostics

### Enhanced Command Generation Quality

**50+ Static Patterns**:
- File management commands (19 patterns)
- System monitoring (7 patterns)
- Git operations (3 patterns)
- Network diagnostics (5 patterns)
- Text processing (7 patterns)
- DevOps/Kubernetes (5 patterns)

**Quality Improvements**:
- Temperature fixed from 0.7 to 0.1 (more deterministic)
- Chain-of-thought prompting added
- Validation-triggered retry loop
- Platform-specific command generation (macOS BSD vs Linux GNU)

### Safety Validation System

**52 Dangerous Command Patterns**:
- Recursive deletion protection
- System-wide file operations
- Permission changes
- Process termination
- Database operations
- Network security

**0% False Positive Rate**: Safe commands pass through without warnings.

### Privacy-First Telemetry

**Beta Default**: Telemetry **enabled** by default for quality data collection.

**What We Collect** (Anonymous Metadata Only):
- ✅ Session timing and performance metrics
- ✅ Platform info (OS, shell type)
- ✅ Backend usage statistics
- ✅ Error categories and safety events

**What We DON'T Collect** (Zero PII):
- ❌ Your commands or natural language input
- ❌ File paths or directory structures
- ❌ Email addresses or IP addresses
- ❌ Environment variables or API keys
- ❌ Any personally identifiable information

**Privacy Controls**:
```bash
caro telemetry status    # Check settings
caro telemetry show      # View collected events
caro telemetry export    # Export to JSON
caro telemetry clear     # Clear all events
caro telemetry disable   # Disable telemetry
```

---

## 📦 Installation

### Method 1: Direct Binary Download (Recommended)

**macOS Apple Silicon (M1/M2/M3)**:
```bash
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-macos-aarch64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
caro --version
```

**macOS Intel (x86_64)**:
```bash
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-macos-x86_64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
```

**Linux x86_64**:
```bash
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-linux-x86_64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
```

**Linux ARM64**:
```bash
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.1/caro-linux-aarch64 -o caro
chmod +x caro
sudo mv caro /usr/local/bin/caro
```

### Method 2: Build from Source

```bash
git clone https://github.com/wildcard/caro
cd caro
git checkout v1.1.0-beta.1
cargo build --release
sudo cp target/release/caro /usr/local/bin/
```

**Requirements**: Rust 1.83+, optional cmake for MLX backend

### Method 3: Cargo Install

```bash
cargo install --git https://github.com/wildcard/caro --tag v1.1.0-beta.1
```

**Complete Installation Guide**: See [INSTALL-BETA.md](https://github.com/wildcard/caro/blob/release/v1.1.0/INSTALL-BETA.md)

---

## ✅ Verification

After installation:

```bash
# Check version (should show beta marker)
caro --version
# Expected: caro 1.1.0-beta.1 (1e8ca84 2026-01-08)

# Test command generation
caro "list files"

# Check system assessment (new feature)
caro assess

# Check health diagnostics (new feature)
caro doctor
```

---

## 🧪 Beta Testing Information

### What We're Testing
- Command generation quality across diverse use cases
- Safety validation in real-world scenarios
- Privacy guarantees (zero PII in practice)
- System assessment accuracy
- Performance and resource usage
- Telemetry collection reliability

### How to Provide Feedback

**GitHub Issues**: [Report bugs](https://github.com/wildcard/caro/issues)

**Include in Bug Reports**:
- OS and shell type
- Caro version (`caro --version`)
- Command that failed
- Expected vs actual behavior
- Telemetry export (if relevant): `caro telemetry export feedback.json`

**Feedback Channels**:
- GitHub Issues for bugs
- GitHub Discussions for questions
- [Beta tester feedback survey - link TBD]

### Beta Testing Timeline
- **Duration**: 5 days
- **Target Testers**: 3-5 early adopters
- **Daily Check-ins**: Expected
- **Bug Triage**: Continuous
- **GA Decision**: After beta completion (user approval required)

---

## 📊 Performance Benchmarks

**Command Generation Speed**:
- Static matcher: <50ms (instant)
- Embedded backend: <1000ms (local inference)
- Agent loop: <2000ms (with validation retry)

**Resource Usage**:
- Binary size: 45MB (optimized)
- Memory footprint: ~100MB (efficient)
- No external dependencies (single binary)

**Quality Metrics**:
- Overall pass rate: **93.1%** (54/58 tests)
- Safe command categories: **100%** (7/7 categories)
- False positive rate: **0%** (52 safety patterns)

---

## 🐛 Known Limitations

**Beta Constraints**:
- MLX backend requires cmake to build from source
- Some E2E tests fail due to JSON parsing (P2 bug, doesn't affect users)
- Minor command variations possible (e.g., `ls -la` vs `ls -l`)
- Telemetry upload endpoint not yet deployed (queues locally)

**Platform Support**:
- ✅ macOS (Apple Silicon and Intel)
- ✅ Linux (x86_64 and ARM64)
- ⚠️ Windows: Experimental (not fully tested)

---

## 🔄 Changes Since Last Release

### Added
- System resource assessment (`caro assess`)
- Health diagnostics (`caro doctor`)
- Beta test suite (`caro test`)
- Privacy-first telemetry system
- 50+ static command patterns
- Validation-triggered retry loop
- Chain-of-thought prompting

### Changed
- Temperature: 0.7 → 0.1 (more deterministic)
- Pass rate: 30% → 93.1% (quality improvement)
- Safety patterns: 4 → 52 (comprehensive coverage)
- Binary size: Optimized (<50MB target)

### Fixed
- Platform detection (BSD vs GNU commands)
- JSON parsing edge cases
- Command validation accuracy
- Prompt consistency across backends

**Full Details**: See [CHANGELOG.md](https://github.com/wildcard/caro/blob/release/v1.1.0/CHANGELOG.md)

---

## 📚 Documentation

**Installation**: [INSTALL-BETA.md](https://github.com/wildcard/caro/blob/release/v1.1.0/INSTALL-BETA.md)
**Telemetry**: [docs/TELEMETRY.md](https://github.com/wildcard/caro/blob/release/v1.1.0/docs/TELEMETRY.md)
**Changelog**: [CHANGELOG.md](https://github.com/wildcard/caro/blob/release/v1.1.0/CHANGELOG.md)
**README**: [README.md](https://github.com/wildcard/caro/blob/release/v1.1.0/README.md)

---

## ⚠️ Beta Release Disclaimer

**This is a BETA release**:
- Intended for early adopters and beta testers only
- Not recommended for production-critical workflows
- Telemetry enabled by default for quality data (can be disabled)
- 5-day testing period before GA decision
- GA release ONLY if explicitly approved after successful beta

**Beta Testing Agreement**:
- By using this beta, you agree to provide feedback
- Telemetry helps improve quality (privacy-first, no PII)
- You can disable telemetry anytime: `caro telemetry disable`
- Report issues on GitHub

---

## 🙏 Thank You

Thank you for participating in the caro v1.1.0 beta test!

Your feedback directly shapes the product. We're committed to building the best natural language to shell command tool with:
- Privacy first (zero PII)
- Safety first (comprehensive validation)
- Quality first (93%+ pass rate)

**Questions?** Open a [GitHub Discussion](https://github.com/wildcard/caro/discussions)
**Found a bug?** File an [Issue](https://github.com/wildcard/caro/issues)
**Want to contribute?** See [CONTRIBUTING.md](https://github.com/wildcard/caro/blob/release/v1.1.0/CONTRIBUTING.md)

---

**Download**: [caro-v1.1.0-beta.1 binaries](https://github.com/wildcard/caro/releases/tag/v1.1.0-beta.1)
**Commit**: 1e8ca84
**Released**: 2026-01-08
