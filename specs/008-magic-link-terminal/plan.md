# Magic Link Terminal - Implementation Plan

**Feature**: 008-magic-link-terminal
**Created**: 2026-01-04
**Status**: Planning

---

## Overview

This document outlines the implementation strategy for the Magic Link Terminal feature. The plan follows a phased approach, starting with a minimal viable product (MVP) and progressively adding capabilities.

---

## Phase 1: MVP - Core Protocol (Target: v1.1)

### Goal
Enable basic `caro://` URL handling on macOS and Linux with terminal launching and safety validation.

### Deliverables

#### 1.1 URL Protocol Handler Module
- Create new module: `src/magic_link/mod.rs`
- Implement URL parsing with security hardening
- Add injection prevention and sanitization
- Create `MagicLink` struct for parsed link data

**Key Files**:
- `src/magic_link/mod.rs` - Module root
- `src/magic_link/parser.rs` - URL parsing logic
- `src/magic_link/sanitizer.rs` - Input sanitization

#### 1.2 macOS Protocol Registration
- Create minimal `.app` bundle for protocol handling
- Implement Info.plist with CFBundleURLTypes
- Create launcher script/binary that invokes `caro` CLI
- Add `caro setup-protocol` command for registration

**Key Files**:
- `packaging/macos/Caro.app/` - App bundle structure
- `packaging/macos/Info.plist` - URL scheme declaration
- `src/cli/setup.rs` - Protocol registration commands

#### 1.3 Linux Protocol Registration
- Create `.desktop` file for XDG registration
- Implement MIME type handler registration
- Add auto-registration during install or first-run

**Key Files**:
- `packaging/linux/caro-handler.desktop` - Desktop entry
- `scripts/register-protocol-linux.sh` - Registration script

#### 1.4 Terminal Launcher
- Implement terminal detection logic
- Create launchers for common terminals:
  - macOS: Terminal.app, iTerm2
  - Linux: GNOME Terminal, Konsole, xterm
- Add fallback behavior when terminal unknown

**Key Files**:
- `src/magic_link/terminal.rs` - Terminal detection
- `src/magic_link/launcher.rs` - Terminal launching logic

#### 1.5 CLI Integration
- Add `--magic-link` flag to CLI parser
- Add `--source` flag for source URL tracking
- Implement magic link entry point in main
- Enhanced safety scrutiny for magic link mode

**Key Changes**:
- `src/main.rs` - Add magic link handling
- `src/cli/mod.rs` - Add new CLI flags

#### 1.6 Magic Link Confirmation UI
- Create dedicated confirmation flow for magic links
- Display source URL prominently
- Show safety validation results
- Require explicit Y/n confirmation

**Key Files**:
- `src/magic_link/confirmation.rs` - Confirmation UI

#### 1.7 Audit Logging
- Log all magic link activations
- Include source URL, command, decision, result
- Store in `~/.config/caro/magic_link_audit.log`

**Key Files**:
- `src/magic_link/audit.rs` - Audit logging

### Exit Criteria
- [ ] `caro://run?cmd=echo%20hello` opens terminal on macOS
- [ ] `caro://run?cmd=echo%20hello` opens terminal on Linux
- [ ] Safety validation runs on magic link commands
- [ ] User confirmation required before execution
- [ ] Audit log captures all magic link usage

---

## Phase 2: Enhanced UX (Target: v1.2)

### Goal
Improve user experience with prerequisite detection, Windows support, and better error handling.

### Deliverables

#### 2.1 Prerequisite Detection
- Parse commands for program references
- Check PATH for program availability
- Build database of common prerequisites
- Implement version checking

**Key Files**:
- `src/magic_link/prerequisites.rs` - Detection logic
- `src/magic_link/prerequisite_db.rs` - Package database

#### 2.2 Prerequisite Installation Offers
- Detect appropriate package manager
- Generate installation commands
- Offer to install missing prerequisites
- Handle installation workflow

**Key Files**:
- `src/magic_link/installer.rs` - Installation logic

#### 2.3 Windows Support
- Implement registry-based protocol handler
- Support Windows Terminal and PowerShell
- Handle Windows-specific path conventions
- Add Windows installer integration

**Key Files**:
- `packaging/windows/` - Windows-specific packaging
- `src/magic_link/windows.rs` - Windows-specific code

#### 2.4 Terminal Configuration
- Add config option for preferred terminal
- Implement terminal preference detection
- Support custom terminal commands

**Config Schema**:
```toml
[magic_link]
preferred_terminal = "iterm2"  # or "auto"
custom_terminal_command = ""
```

#### 2.5 First-Run Onboarding
- Detect first magic link usage
- Show brief explanation of feature
- Explain safety guarantees
- Allow "don't show again"

**Key Files**:
- `src/magic_link/onboarding.rs` - Onboarding flow

#### 2.6 Improved Error Handling
- Better error messages for common failures
- Terminal not found guidance
- Protocol not registered guidance
- Recovery suggestions

### Exit Criteria
- [ ] Missing prerequisites detected and offered for installation
- [ ] Windows protocol handler works with Windows Terminal
- [ ] Users can configure preferred terminal
- [ ] First-time users see onboarding
- [ ] All error states have helpful messages

---

## Phase 3: Web Integration (Target: v1.3)

### Goal
Provide tools for website authors to easily add magic links to their documentation.

### Deliverables

#### 3.1 Link Generator Tool
- Web-based tool on caro website
- Input: shell command, optional metadata
- Output: properly formatted `caro://` URL
- Preview of button appearance

**Location**: Caro website (separate repository)

#### 3.2 Embeddable Button Components
- CSS-only button designs
- Optional JavaScript enhancements
- Multiple style presets (default, minimal, dark)
- Responsive sizing

**Deliverables**:
- Button CSS
- Usage documentation
- Code snippets

#### 3.3 Documentation for Website Authors
- How magic links work
- URL format specification
- Best practices for security
- Accessibility considerations

**Location**: `docs/magic-link-integration.md`

#### 3.4 Platform Integration Guides
- GitHub Pages
- Docusaurus
- MkDocs
- ReadTheDocs
- Notion (limitations)

**Location**: `docs/integrations/`

#### 3.5 Safety Indicators
- Badge indicating link goes to Caro
- Clear visual distinction from regular links
- "Verified by Caro" styling

### Exit Criteria
- [ ] Link generator available on website
- [ ] Embeddable buttons documented
- [ ] At least 3 platform integration guides
- [ ] Caro's own docs use magic links

---

## Phase 4: Browser Extension (Target: v2.0)

### Goal
Automatically detect shell commands on web pages and offer Caro execution.

### Deliverables

#### 4.1 Chrome Extension
- Manifest V3 compliant
- Content script for code detection
- "Run in Caro" button injection
- Extension popup for settings

**Repository**: `caro-browser-extension/`

#### 4.2 Firefox Extension
- WebExtension compatible
- Same functionality as Chrome
- Firefox Add-ons store submission

#### 4.3 Code Detection Engine
- Heuristic-based shell command detection
- Support for common code block formats
- Configurable sensitivity

#### 4.4 WebAssembly Safety Preview (Optional)
- Compile safety classifier to WASM
- Run preliminary analysis in browser
- Display risk indicators before click
- Privacy-preserving (no external calls)

#### 4.5 Extension Settings
- Enable/disable per-domain
- Sensitivity configuration
- Theme matching (light/dark)
- Keyboard shortcut configuration

### Exit Criteria
- [ ] Chrome extension published
- [ ] Firefox extension published
- [ ] Detection works on major sites (GitHub, Stack Overflow)
- [ ] User can disable per-site
- [ ] No privacy violations

---

## Phase 5: Advanced Security (Target: v2.1)

### Goal
Integrate with security vendors and enable enterprise features.

### Deliverables

#### 5.1 Vendor Security Integration
- Define `SecurityVendor` trait
- Implement VirusTotal integration
- Implement Safe Browsing integration
- Add caching layer

**Key Files**:
- `src/magic_link/vendors/mod.rs`
- `src/magic_link/vendors/virustotal.rs`
- `src/magic_link/vendors/safe_browsing.rs`

#### 5.2 Cryptographic Link Signing
- Define signature format
- Implement verification
- Trusted publisher registry
- Signature generation tool for publishers

#### 5.3 Domain Trust Management
- User-configurable allowlist/blocklist
- Persist trust decisions
- "Remember this domain" option
- Enterprise policy integration

**Config Schema**:
```toml
[magic_link.trust]
allowed_domains = ["docs.caro.dev", "github.com"]
blocked_domains = ["suspicious-site.com"]
remember_trust = true
```

#### 5.4 Team Policy Controls
- Define organization policies
- Enforce restrictions (block categories, require signing)
- Central policy server integration
- Compliance logging

#### 5.5 Enhanced Audit & Reporting
- Structured audit log format
- Export to SIEM systems
- Summary reports
- Alert on suspicious patterns

### Exit Criteria
- [ ] At least one security vendor integrated
- [ ] Signed links can be verified
- [ ] Domain trust persists across sessions
- [ ] Audit logs exportable

---

## Testing Strategy

### Unit Tests
- URL parsing with malicious inputs
- Sanitization edge cases
- Terminal detection mocking
- Prerequisite detection

### Integration Tests
- End-to-end magic link flow
- Cross-platform terminal launching
- Protocol registration verification
- Safety pipeline integration

### Security Tests
- Fuzzing URL parser
- Shell injection attempts
- Boundary testing (long URLs, special chars)
- Cryptographic verification

### Manual Testing Matrix
- [ ] macOS + Terminal.app
- [ ] macOS + iTerm2
- [ ] Ubuntu + GNOME Terminal
- [ ] Fedora + Konsole
- [ ] Windows + Windows Terminal
- [ ] Windows + PowerShell

---

## Risk Mitigation

| Risk | Mitigation | Phase |
|------|------------|-------|
| Protocol handler security | Extensive input validation | 1 |
| Terminal compatibility | Fallback chain, user override | 1-2 |
| User confusion | Clear onboarding, documentation | 2 |
| Extension rejection | Follow store guidelines, privacy focus | 4 |
| Vendor API costs | Caching, rate limiting, optional | 5 |

---

## Dependencies

### Internal
- Existing safety validation pipeline (`src/safety/`)
- CLI argument parsing (`src/cli/`)
- Configuration system (`src/config/`)
- Execution context (`src/execution/`)

### External
- Platform-specific APIs (macOS Launch Services, XDG, Windows Registry)
- Browser extension APIs (Chrome/Firefox WebExtension)
- Optional: VirusTotal API, Safe Browsing API

---

## Success Metrics

### Phase 1
- Magic links work on 90%+ of macOS/Linux systems tested
- Zero security incidents from URL parsing

### Phase 2
- 80%+ of missing prerequisites correctly detected
- Windows support reaches parity with macOS/Linux

### Phase 3
- 10+ external sites integrate magic links
- Documentation receives positive feedback

### Phase 4
- 1000+ extension users
- Detection accuracy > 90% on target sites

### Phase 5
- Enterprise customer adoption
- Security audit passed

---

## Timeline Considerations

This plan does not specify fixed timelines. Implementation priority should be based on:

1. User demand and feedback
2. Available development resources
3. Strategic alignment with Caro roadmap
4. Security considerations (Phase 1 security must be solid before Phase 4)

Each phase should be considered complete when all exit criteria are met and security review is passed.
