# Caro: Platform Strategy & Multi-Platform Support

**Version**: 1.0
**Last Updated**: 2026-01-08
**Status**: Complete

---

## Purpose

This document defines the comprehensive platform strategy for Caro, including current platform support (macOS, Linux), future platform expansion (Windows, mobile), and the technical approach to cross-platform compatibility.

**Audience**: Engineering Team, Product Manager, Platform Specialists

---

## Strategic Vision

### Platform Philosophy

**Core Principle**: **UNIX-first, multi-platform eventually**

**Rationale**:
1. **Focus**: Master macOS/Linux first before expanding
2. **Quality**: Excellent experience on 2 platforms > mediocre on 5
3. **Market**: 80% of CLI power users on macOS/Linux
4. **Resources**: Small team, prioritize high-impact platforms

**Long-Term Vision** (2026-2027):
- **v1.x**: macOS + Linux (excellence)
- **v2.0**: Windows support (when resources allow)
- **v2.x**: Mobile companion apps (iOS, Android)
- **v3.0**: Web-based interface (browser extension or web app)

---

## Current Platform Support (v1.1.0)

### Tier 1: First-Class Support

#### macOS
**Architecture**:
- Intel (x86_64)
- Apple Silicon (ARM64 / M1/M2/M3)

**OS Versions**:
- macOS 11 (Big Sur) and later
- Testing: macOS 12 (Monterey), 13 (Ventura), 14 (Sonoma)

**Shell Support**:
- bash (default on older macOS)
- zsh (default on macOS 10.15+)
- fish (community-supported)

**Platform-Specific Features**:
- ✅ BSD command compatibility (BSD ps, ls, find vs GNU)
- ✅ MLX backend (v1.2.0+) for Apple Silicon acceleration
- ✅ Universal binaries (fat binaries for Intel + ARM)
- ✅ Homebrew installation
- ✅ macOS-specific prompts and examples

**Installation Methods**:
```bash
# Homebrew (recommended)
brew install caro

# Cargo
cargo install caro

# Direct download
curl -fsSL https://caro-cli.dev/install.sh | sh
```

---

#### Linux
**Distributions** (Tested):
- Ubuntu 20.04 LTS, 22.04 LTS, 24.04 LTS
- Debian 11, 12
- Fedora 38, 39
- Arch Linux (rolling)
- Rocky Linux 8, 9 (RHEL compatible)

**Architecture**:
- x86_64 (Intel/AMD)
- ARM64 (Raspberry Pi, ARM servers)
- ARMv7 (older Raspberry Pi) - best effort

**Shell Support**:
- bash (default on most distributions)
- zsh
- fish

**Platform-Specific Features**:
- ✅ GNU command compatibility (GNU ps, ls, find vs BSD)
- ✅ Static binaries (musl libc) for maximum portability
- ✅ systemd integration (optional service mode, v1.3+)
- ✅ Package managers (apt, yum, pacman, cargo)
- ✅ Linux-specific prompts and examples

**Installation Methods**:
```bash
# Cargo (all distros)
cargo install caro

# APT (Debian/Ubuntu)
sudo apt install caro

# DNF/YUM (Fedora/RHEL)
sudo dnf install caro

# Pacman (Arch)
sudo pacman -S caro

# Direct download
curl -fsSL https://caro-cli.dev/install.sh | sh
```

---

### Tier 2: Community Support

#### FreeBSD
**Status**: Community-maintained port
- Basic functionality working
- Limited testing by core team
- Community contributors welcome

#### OpenBSD
**Status**: Experimental
- Rust toolchain available
- Command compatibility challenges
- Community contributors needed

---

### Tier 3: Future Support

#### Windows
**Status**: Planned for v2.0.0 (December 2026)
**Target**: Windows 10+ (PowerShell, WSL2)

**Challenges**:
- PowerShell vs bash syntax differences
- Path separators (\ vs /)
- Command ecosystem differences (dir vs ls, tasklist vs ps)
- WSL2 detection (bash in WSL vs PowerShell in Windows)

**Strategy**:
1. **WSL2 First** (easier): Caro in WSL2 is same as Linux
2. **PowerShell Later**: Requires PowerShell-specific prompts and command templates
3. **Market Validation**: Survey Windows user demand before investment

**Decision Criteria for Windows Support**:
- ✅ 20%+ user requests from Windows users
- ✅ Core team has Windows expertise OR hire Windows specialist
- ✅ v1.x feature complete on macOS/Linux
- ✅ Resources available (not blocking higher-priority features)

---

## Cross-Platform Architecture

### Design Principles

1. **Platform Detection**: Runtime platform detection, not compile-time flags
2. **Prompt Specialization**: Platform-specific prompts for command generation
3. **Validation Rules**: Platform-aware safety validation
4. **Graceful Degradation**: Features work everywhere, optimize for specific platforms

---

### Platform Detection

**Implementation**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS { arch: Architecture },
    Linux { distro: Option<LinuxDistro> },
    FreeBSD,
    Windows { shell: WindowsShell },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Armv7,
}

pub fn detect_platform() -> Platform {
    #[cfg(target_os = "macos")]
    {
        let arch = if cfg!(target_arch = "aarch64") {
            Architecture::Aarch64
        } else {
            Architecture::X86_64
        };
        Platform::MacOS { arch }
    }

    #[cfg(target_os = "linux")]
    {
        let distro = detect_linux_distro(); // Read /etc/os-release
        Platform::Linux { distro }
    }

    #[cfg(target_os = "freebsd")]
    Platform::FreeBSD

    #[cfg(target_os = "windows")]
    {
        let shell = detect_shell(); // PowerShell vs WSL bash
        Platform::Windows { shell }
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "windows"
    )))]
    Platform::Unknown
}
```

---

### Platform-Specific Prompts

**Design**: Separate prompt templates for BSD vs GNU commands

**Example** (Process Listing):
```rust
pub fn build_platform_prompt(platform: Platform) -> String {
    match platform {
        Platform::MacOS { .. } | Platform::FreeBSD => {
            // BSD-style commands
            r#"
PLATFORM: macOS/FreeBSD (BSD commands)

Process Management:
- List processes: ps aux (BSD style)
- Top CPU: ps aux | sort -k3 -rn | head
- Top Memory: ps aux | sort -k4 -rn | head
- Find process: ps aux | grep <name>

IMPORTANT: Use BSD flags (no dashes for aux)
WRONG: ps -aux (GNU style)
RIGHT: ps aux (BSD style)
"#
        }

        Platform::Linux { .. } => {
            // GNU-style commands
            r#"
PLATFORM: Linux (GNU commands)

Process Management:
- List processes: ps aux (works, but GNU semantics)
- Top CPU: ps aux --sort=-%cpu | head
- Top Memory: ps aux --sort=-%mem | head
- Find process: ps aux | grep <name>

IMPORTANT: Use GNU flags (--long-options supported)
AVAILABLE: ps --sort, ps --forest, ps --headers
"#
        }

        _ => {
            // Generic POSIX fallback
            r#"
PLATFORM: Generic POSIX

Use only portable POSIX commands:
- Avoid GNU-specific flags (--long-options)
- Avoid BSD-specific flags (aux without dash)
- Test commands for portability
"#
        }
    }
}
```

**Result**: Static matcher and LLM prompts adapt to platform automatically

---

### Platform-Specific Validation

**Design**: Validation rules adapt to platform

**Example** (Command Availability):
```rust
pub fn validate_command_availability(cmd: &str, platform: Platform) -> Result<()> {
    match platform {
        Platform::MacOS { .. } => {
            // macOS-specific validation
            if cmd.contains("apt-get") || cmd.contains("yum") {
                return Err(anyhow!(
                    "apt-get/yum not available on macOS. Use 'brew' instead."
                ));
            }
        }

        Platform::Linux { .. } => {
            // Linux-specific validation
            if cmd.contains("brew") {
                return Err(anyhow!(
                    "Homebrew not standard on Linux. Use apt/yum/pacman instead."
                ));
            }
        }

        _ => {}
    }

    Ok(())
}
```

---

## Platform-Specific Optimizations

### macOS Apple Silicon (v1.2.0+)

**MLX Backend Integration**:
- 10-50x faster inference on M1/M2/M3
- Metal Performance Shaders (MPS) acceleration
- Unified memory architecture optimization

**Universal Binaries**:
- Fat binary with both Intel (x86_64) and ARM (aarch64) code
- Single download works on all macOS machines
- Automatic architecture selection at runtime

**Homebrew Integration**:
- Official Homebrew formula (casks)
- Automatic updates via `brew upgrade`
- Native macOS installation experience

---

### Linux

**Static Binaries** (musl libc):
- No runtime dependencies (no glibc version issues)
- Works on any Linux distribution
- Portable across containers, minimal systems

**ARM Support**:
- Raspberry Pi 4/5 (ARM64)
- ARM servers (AWS Graviton, Ampere)
- ARMv7 (older Raspberry Pi) - best effort

**Package Manager Integration**:
- APT repository (Debian/Ubuntu)
- DNF repository (Fedora/RHEL)
- AUR (Arch User Repository)
- Snap Store (cross-distro)
- Flatpak (cross-distro)

---

## Testing Strategy

### Platform Testing Matrix

**CI/CD Testing** (Every commit):
| Platform | Architecture | Shell | Test Type |
|----------|--------------|-------|-----------|
| macOS 12 | x86_64 | bash, zsh | Unit, integration, safety |
| macOS 13 | ARM64 | bash, zsh | Unit, integration, safety |
| Ubuntu 22.04 | x86_64 | bash | Unit, integration, safety |
| Ubuntu 22.04 | ARM64 | bash | Unit, integration, safety |
| Debian 12 | x86_64 | bash | Integration, safety |
| Fedora 39 | x86_64 | bash | Integration, safety |

**Manual Testing** (Pre-release):
- macOS 14 (Sonoma) - Intel and ARM
- Arch Linux (rolling)
- Rocky Linux 9 (RHEL clone)
- FreeBSD 14 (community validation)

**Beta Testing** (v1.1.0 and beyond):
- 20-40 platform-specific testers
- Diverse hardware (old MacBooks, new M3, various Linux)
- Diverse shells (bash, zsh, fish)

---

### Platform-Specific Test Cases

**macOS-Specific Tests**:
- BSD command generation (ps, ls, find)
- MLX backend performance (v1.2.0+)
- Universal binary runs on both Intel and ARM
- Homebrew installation and update

**Linux-Specific Tests**:
- GNU command generation (ps, ls, find)
- Static binary runs on multiple distros
- APT/DNF/Pacman installation
- systemd integration (v1.3.0+)

**Cross-Platform Tests**:
- Same query generates platform-appropriate commands
- Validation blocks platform-incompatible commands
- Installation scripts work on all platforms
- Binary size <10MB compressed on all platforms

---

## Distribution Strategy

### Binary Distribution

**Release Artifacts** (per platform):
```
caro-1.1.0-x86_64-apple-darwin.tar.gz         # macOS Intel
caro-1.1.0-aarch64-apple-darwin.tar.gz        # macOS ARM
caro-1.1.0-universal-apple-darwin.tar.gz      # macOS Universal (Intel+ARM)
caro-1.1.0-x86_64-unknown-linux-musl.tar.gz   # Linux x86 (static)
caro-1.1.0-aarch64-unknown-linux-musl.tar.gz  # Linux ARM (static)
caro-1.1.0-x86_64-unknown-freebsd.tar.gz      # FreeBSD (community)
```

**Checksums**:
```
SHA256SUMS              # SHA-256 checksums for all artifacts
SHA256SUMS.asc          # GPG signature of checksums
```

---

### Package Manager Integration

**Homebrew (macOS)**:
```ruby
class Caro < Formula
  desc "Natural language shell commands with local AI"
  homepage "https://caro-cli.dev"
  url "https://github.com/username/caro/archive/v1.1.0.tar.gz"
  sha256 "abc123..."

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "caro 1.1.0", shell_output("#{bin}/caro --version")
  end
end
```

**APT (Debian/Ubuntu)**:
```bash
# Add repository
echo "deb [signed-by=/usr/share/keyrings/caro-archive-keyring.gpg] \
  https://apt.caro-cli.dev stable main" | \
  sudo tee /etc/apt/sources.list.d/caro.list

# Install
sudo apt update
sudo apt install caro
```

**DNF (Fedora/RHEL)**:
```bash
# Add repository
sudo dnf config-manager --add-repo https://yum.caro-cli.dev/caro.repo

# Install
sudo dnf install caro
```

**Cargo (All platforms)**:
```bash
cargo install caro
```

---

## Future Platform Expansion

### Windows Support (v2.0.0, Q4 2026)

**Phase 1: WSL2 Support** (Easiest)
- Caro runs in WSL2 (Windows Subsystem for Linux)
- Same Linux binary works in WSL2
- No additional work needed beyond documenting WSL2 setup

**Phase 2: Native PowerShell Support** (Hard)
- Requires PowerShell-specific command templates
- Different command ecosystem (dir, tasklist, Get-*, etc.)
- Path handling differences (\ vs /)
- Registry vs file-based config

**Decision Criteria**:
- 20%+ of user requests from Windows users
- Core team Windows expertise OR hire specialist
- Community contributor volunteers for PowerShell prompts

**RICE Score Estimate**:
```
Reach: 4 (20% of potential users on Windows)
Impact: 1.0 (New platform, but no unique features)
Confidence: 60% (uncertain demand)
Effort: 8 weeks (PowerShell prompts + testing)

RICE = (4 × 1.0 × 0.6) / 8 = 0.3 (LOWER PRIORITY than MLX, i18n, teams)
```

---

### Mobile Apps (v2.0.0+, Q4 2026)

**iOS App**:
- SwiftUI frontend
- Rust core via FFI (Foreign Function Interface)
- On-device model inference (CoreML or MLX)
- iCloud sync with E2EE
- SSH tunnel to desktop machines (optional)

**Android App**:
- Kotlin/Jetpack Compose frontend
- Rust core via JNI (Java Native Interface)
- On-device model inference (ONNX Runtime or TensorFlow Lite)
- Google Drive sync with E2EE (optional)
- SSH tunnel to desktop machines (optional)

**Mobile-Specific Challenges**:
- Model size constraints (100-200MB max for mobile)
- Battery usage (inference is power-hungry)
- Limited screen space (CLI output formatting)
- Mobile vs desktop command differences

**Use Cases**:
- Quick command lookup on the go
- Copy command to clipboard, paste on desktop
- Learn CLI while commuting
- View command history synced from desktop

**Not a Replacement**: Mobile is a companion, not primary experience

---

### Web Interface (v3.0.0+, 2027+)

**Browser Extension**:
- Chrome, Firefox, Safari, Edge
- Generate commands within web-based terminals (Google Cloud Shell, AWS CloudShell)
- Sync history with desktop app
- Privacy-first (no cloud inference by default)

**Web App**:
- Progressive Web App (PWA)
- Works offline
- Educational tool (learn CLI commands)
- Not a full replacement (security limitations in browser)

---

## Platform Deprecation Policy

### When to Drop Platform Support

**Criteria** (ALL must be true):
1. Platform usage <1% of users (measured via opt-in telemetry)
2. Platform maintenance burden >20% of development time
3. Platform blocks critical features (e.g., performance, security)
4. Community offers no maintainers for platform

**Process**:
1. **Announce** deprecation 6 months in advance
2. **Explain** rationale transparently (usage, maintenance burden)
3. **Offer** community maintenance if volunteers exist
4. **Document** deprecation in release notes
5. **Archive** last working version for platform

**Example**: If FreeBSD usage remains <0.5% and no community maintainers, we may drop official support in v2.0+. Community forks are encouraged.

---

## Platform Roadmap

### 2026 Roadmap

**Q1 (Jan-Mar)**:
- ✅ v1.1.0: macOS + Linux (Intel + ARM) first-class support
- ✅ v1.2.0: MLX backend for Apple Silicon (10-50x speedup)

**Q2 (Apr-Jun)**:
- v1.3.0: Universal binaries for macOS (Intel + ARM in single binary)
- v1.3.0: Improved Linux package manager integration (APT, DNF, Snap)

**Q3 (Jul-Sep)**:
- v1.4.0: systemd integration for Linux (optional service mode)
- v1.4.0: FreeBSD community testing program

**Q4 (Oct-Dec)**:
- v2.0.0: Windows WSL2 support (documentation + testing)
- v2.0.0: Native Windows PowerShell support (if resources allow)
- v2.0.0: Mobile app planning (iOS/Android)

---

### 2027 Roadmap

**Q1 (Jan-Mar)**:
- v2.1.0: iOS companion app (beta)
- v2.1.0: Android companion app (beta)

**Q2 (Apr-Jun)**:
- v2.2.0: Mobile apps (general availability)
- v2.2.0: Cross-device sync with E2EE

**Q3 (Jul-Sep)**:
- v2.3.0: Browser extension (Chrome, Firefox)
- v2.3.0: Web-based terminal integration

**Q4 (Oct-Dec)**:
- v3.0.0: Web app (PWA)
- v3.0.0: Multi-platform sync maturity

---

## Success Metrics

### Platform Distribution (Target)

| Platform | v1.1.0 (Launch) | v1.4.0 (Sep 2026) | v2.0.0 (Dec 2026) |
|----------|-----------------|-------------------|-------------------|
| macOS Intel | 30% | 20% | 15% |
| macOS ARM | 30% | 40% | 45% |
| Linux x86 | 35% | 30% | 25% |
| Linux ARM | 5% | 5% | 5% |
| Windows | 0% | 0% | 5% |
| Mobile | 0% | 0% | 3% |
| Other | 0% | 5% | 2% |

**Trend**: macOS ARM growing (Apple Silicon adoption), Windows entering in v2.0

---

### Platform Performance Metrics

| Metric | macOS Intel | macOS ARM (MLX) | Linux x86 | Linux ARM |
|--------|-------------|-----------------|-----------|-----------|
| Static Matcher | <50ms | <50ms | <50ms | <50ms |
| Embedded Backend | 300-600ms | 300-600ms | 300-600ms | 400-800ms |
| MLX Backend | N/A | 10-30ms | N/A | N/A |
| Binary Size | <10MB | <10MB | <8MB (musl) | <8MB (musl) |
| Memory Usage | 100-200MB | 80-150MB | 100-200MB | 100-250MB |

**Target**: Performance parity across platforms, with Apple Silicon advantage via MLX

---

## Conclusion

**Platform strategy prioritizes quality over quantity.**

**Key Principles**:
1. **UNIX First**: Master macOS/Linux before expanding
2. **Platform-Aware**: Commands adapt to platform automatically
3. **Testing Rigor**: CI/CD tests every platform/architecture combo
4. **Community-Driven**: Community ports (FreeBSD) are valued
5. **Future-Ready**: Mobile and web planned for v2.0+

**Current Support**:
- ✅ macOS (Intel + ARM)
- ✅ Linux (x86 + ARM)
- 🔶 FreeBSD (community)

**Future Support** (as resources allow):
- v2.0: Windows (WSL2 + PowerShell)
- v2.0: Mobile (iOS + Android)
- v3.0: Web (browser extension + PWA)

**By focusing on excellence on core platforms, we build trust before expanding.**

---

**Last Updated**: 2026-01-08
**Next Review**: 2026-04-01 (v1.2.0 retrospective)
**Version**: 1.0
