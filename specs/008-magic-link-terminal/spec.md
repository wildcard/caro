# Feature Specification: Magic Link Terminal

**Feature Branch**: `008-magic-link-terminal`
**Created**: 2026-01-04
**Status**: Draft
**Input**: Enable web-to-terminal command execution via clickable links with comprehensive safety validation

---

## Executive Summary

Magic Link Terminal is a feature that enables users to click specially-formatted links on websites to automatically open their terminal with Caro, analyze the requested command, run comprehensive safety checks, and execute with explicit user permission. This bridges the gap between web-based documentation and local command execution while maintaining Caro's security-first philosophy.

The feature addresses a fundamental problem: developers frequently encounter shell commands on websites (installation guides, tutorials, Stack Overflow answers) that they need to copy, paste, and execute manually. This creates friction and security risks:
- Users may not understand what commands do before running them
- Prerequisites may be missing (e.g., brew not installed)
- Commands may be malicious or inappropriate for the user's system
- The copy-paste workflow is error-prone and tedious

Magic Link Terminal transforms this experience by providing one-click execution with intelligent safety validation, prerequisite detection, and user confirmation.

---

## Goals

### Primary Goals

1. **One-Click Web-to-Terminal**: Users can click a link on any website to open their terminal with Caro and a pre-loaded command
2. **Safety-First Execution**: Every command goes through Caro's full safety validation pipeline before any execution
3. **Explicit User Consent**: Commands never execute without clear user confirmation showing what will happen
4. **Prerequisite Detection**: Caro automatically detects missing dependencies and offers to install them
5. **Cross-Platform Support**: Works on macOS, Linux, and Windows with common terminals
6. **Website Integration**: Provide embeddable components for website authors to create magic links

### Secondary Goals

1. **Browser Extension**: Chrome/Firefox extension to detect code snippets and offer Caro execution
2. **Command Explanation**: Provide clear explanations of what commands will do before execution
3. **Audit Trail**: Log all magic link executions for security review
4. **Offline Safety**: Core safety validation works without network access
5. **Vendor Security Integration**: Optional integration with security vendor APIs for additional threat analysis

### Non-Goals

1. **Automatic Execution**: Commands will never run without user confirmation
2. **Browser Replacement**: This is not a browser-based terminal
3. **Command Modification by Websites**: The website cannot modify commands after the link is clicked
4. **Bypassing System Security**: Will work within OS security constraints, not around them
5. **Universal Terminal Support**: MVP focuses on common terminals (iTerm2, Terminal.app, GNOME Terminal, Windows Terminal)

---

## User Scenarios & Testing

### Primary User Stories

#### Story 1: Documentation Quick-Start
A developer is reading Caro's documentation and sees a "Run in Caro" button next to installation instructions. They click the button, their terminal opens with Caro, they see a clear explanation of what will happen, confirm, and the installation proceeds.

#### Story 2: Unknown Tool Prerequisites
A developer finds a tutorial requiring `jq` for JSON processing. They click the magic link for `jq --version`. Caro detects `jq` is not installed, explains this, and offers to install it via the appropriate package manager (brew on macOS, apt on Ubuntu, etc.).

#### Story 3: Suspicious Command Detection
A developer clicks a magic link from an untrusted source containing `curl | bash`. Caro's safety validation detects the pipe-to-shell pattern, displays a RED warning, explains the risks, and strongly recommends against execution.

#### Story 4: Browser Extension Use
A developer browses Stack Overflow and sees a command snippet. The Caro browser extension adds a "Run safely in Caro" button next to the code block. Clicking opens their terminal with the command pre-loaded and ready for safety validation.

### Acceptance Scenarios

1. **Given** a website has a magic link `caro://run?cmd=brew%20install%20jq`, **When** the user clicks it, **Then** their default terminal opens, Caro starts with the command pre-loaded, displays safety analysis, and awaits user confirmation.

2. **Given** a user clicks a magic link for a command requiring a missing dependency, **When** Caro detects the dependency is missing, **Then** it explains this clearly and offers to install the prerequisite first.

3. **Given** a magic link contains a potentially dangerous command, **When** Caro analyzes it, **Then** it displays appropriate warnings based on risk level (CRITICAL/HIGH/MODERATE) and may refuse execution for CRITICAL risks.

4. **Given** the user has not used Caro before, **When** they click their first magic link, **Then** Caro provides a brief onboarding explaining how magic links work and the safety guarantees.

5. **Given** a magic link is clicked while the terminal is already running Caro, **When** this occurs, **Then** the system either opens a new terminal session or queues the command appropriately.

6. **Given** a website attempts to craft a malicious magic link with shell injection, **When** Caro parses the link, **Then** the injection is sanitized and the command is safely handled.

### Edge Cases

- What happens when the user's terminal application is not in the supported list?
- How does the system behave when clicked from a sandboxed browser (e.g., Safari)?
- What occurs if multiple magic links are clicked in rapid succession?
- How does the system handle magic links with very long commands?
- What happens when the command references files that don't exist on the user's system?
- How does offline mode affect vendor security checks?

---

## Requirements

### Functional Requirements

#### URL Protocol & Parsing
- **FR-001**: System MUST register a custom URL protocol (`caro://`) on supported operating systems
- **FR-002**: System MUST parse magic link URLs and extract commands safely, preventing injection attacks
- **FR-003**: System MUST support URL encoding for commands containing special characters
- **FR-004**: System MUST validate URL structure before processing any commands
- **FR-005**: System MUST reject malformed or suspiciously crafted URLs with clear error messages

#### Terminal Integration
- **FR-006**: System MUST detect and open the user's preferred terminal application
- **FR-007**: System MUST support common terminals: iTerm2, Terminal.app, GNOME Terminal, Konsole, Windows Terminal, Alacritty
- **FR-008**: System MUST create a new terminal window/tab for magic link execution
- **FR-009**: System MUST provide fallback behavior when preferred terminal cannot be determined
- **FR-010**: System MUST allow users to configure their preferred terminal for magic links

#### Safety & Security
- **FR-011**: System MUST run all magic link commands through the full Caro safety validation pipeline
- **FR-012**: System MUST display safety analysis results prominently before any user confirmation
- **FR-013**: System MUST require explicit user confirmation for every magic link command
- **FR-014**: System MUST refuse to execute CRITICAL risk commands from magic links by default
- **FR-015**: System MUST provide enhanced scrutiny for magic links (higher suspicion than direct CLI use)
- **FR-016**: System MUST sanitize all command inputs to prevent shell injection
- **FR-017**: System MUST log all magic link executions with source URL (if available) for audit
- **FR-018**: System SHOULD integrate with security vendor APIs for additional threat analysis (optional)

#### Prerequisite Detection
- **FR-019**: System MUST detect when a command references programs not installed on the system
- **FR-020**: System MUST provide clear guidance on how to install missing prerequisites
- **FR-021**: System MAY offer to install common prerequisites automatically (brew, apt packages)
- **FR-022**: System MUST validate prerequisites meet minimum version requirements when specified

#### User Experience
- **FR-023**: System MUST display a clear explanation of what the command will do
- **FR-024**: System MUST show the original source URL (if available) so users know where the link came from
- **FR-025**: System MUST provide first-run onboarding for magic link users
- **FR-026**: System MUST maintain execution time under 3 seconds from click to terminal display
- **FR-027**: System MUST provide clear error messages if terminal launch fails
- **FR-028**: System MUST support cancellation at any point before execution

#### Website Integration
- **FR-029**: System MUST provide documentation for website authors to create magic links
- **FR-030**: System MUST provide embeddable button/component specifications
- **FR-031**: System SHOULD provide a link generator tool for creating properly-formatted magic links
- **FR-032**: System MAY provide copy-paste HTML snippets for common button styles

#### Browser Extension (Phase 2)
- **FR-033**: Extension MUST detect code blocks containing shell commands on web pages
- **FR-034**: Extension MUST add a "Run in Caro" button adjacent to detected code blocks
- **FR-035**: Extension MAY use local WebAssembly model for preliminary command classification
- **FR-036**: Extension MUST respect user privacy and not send code to external servers without consent
- **FR-037**: Extension MUST work offline for basic functionality

### Non-Functional Requirements

- **NFR-001**: Protocol registration MUST work without requiring administrator privileges where possible
- **NFR-002**: Terminal launch latency MUST be under 500ms from link click
- **NFR-003**: Safety validation MUST complete within 2 seconds for typical commands
- **NFR-004**: System MUST gracefully degrade when optional security vendors are unavailable
- **NFR-005**: Browser extension MUST have minimal performance impact (< 50ms page load increase)

---

## Key Entities

### MagicLink
A parsed representation of a `caro://` URL containing:
- **command**: The shell command to potentially execute
- **source_url**: Optional URL of the page containing the link
- **source_domain**: Extracted domain for trust assessment
- **metadata**: Optional additional context (title, description)
- **signature**: Optional cryptographic signature for trusted sources

### ProtocolHandler
System component responsible for:
- Registering the `caro://` URL scheme with the OS
- Receiving and parsing incoming magic link activations
- Launching the appropriate terminal with Caro

### TerminalLauncher
Component that handles:
- Detecting available terminal applications
- Determining user's preferred terminal
- Opening new terminal windows/tabs
- Passing commands to Caro in the new session

### PrerequisiteChecker
Component that:
- Analyzes commands for referenced programs
- Checks if programs are installed and accessible
- Determines appropriate installation methods
- Manages prerequisite installation workflows

### SecurityVendorIntegration
Optional component for:
- Sending command hashes to threat intelligence APIs
- Checking URLs against known malicious sources
- Providing additional confidence scores for unknown commands

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Web Browser                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Website with Magic Link                                     │   │
│  │  <a href="caro://run?cmd=brew%20install%20jq">Install jq</a>│   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              │ User clicks link                      │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Browser Extension (Phase 2)                                 │   │
│  │  • Detects code snippets                                     │   │
│  │  • Adds "Run in Caro" buttons                               │   │
│  │  • WebAssembly local analysis (optional)                    │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               │ caro:// URL activated
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Operating System                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Protocol Handler (registered at install/first-run)         │   │
│  │  • Receives caro:// URLs                                     │   │
│  │  • Validates URL structure                                   │   │
│  │  • Launches Caro with --magic-link flag                     │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               │ Launch terminal with command
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Terminal Application                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  New Terminal Session                                        │   │
│  │  $ caro --magic-link "brew install jq" --source "docs.caro"│   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     Caro Runtime                             │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Magic Link Parser                                    │   │   │
│  │  │  • Decode URL-encoded command                        │   │   │
│  │  │  • Extract metadata                                   │   │   │
│  │  │  • Sanitize inputs                                    │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  │                         │                                    │   │
│  │                         ▼                                    │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Prerequisite Checker                                 │   │   │
│  │  │  • Parse command for program references              │   │   │
│  │  │  • Check PATH for availability                       │   │   │
│  │  │  • Determine installation method                     │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  │                         │                                    │   │
│  │                         ▼                                    │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Safety Validation Pipeline                          │   │   │
│  │  │  • Pattern matching (52+ dangerous patterns)         │   │   │
│  │  │  • Risk level assessment                             │   │   │
│  │  │  • Enhanced scrutiny for magic links                 │   │   │
│  │  │  • Optional: Vendor security API check               │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  │                         │                                    │   │
│  │                         ▼                                    │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  User Confirmation UI                                 │   │   │
│  │  │  • Display command with syntax highlighting          │   │   │
│  │  │  • Show safety analysis results                      │   │   │
│  │  │  • Display source URL/domain                         │   │   │
│  │  │  • Show prerequisite status                          │   │   │
│  │  │  • Require explicit [Y/n] confirmation               │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  │                         │                                    │   │
│  │                         ▼                                    │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Command Executor                                     │   │   │
│  │  │  • Execute validated command                         │   │   │
│  │  │  • Capture output                                     │   │   │
│  │  │  • Handle errors gracefully                          │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### Magic Link Activation Flow

```
User clicks caro:// link on website
        │
        ▼
OS intercepts URL, invokes registered handler
        │
        ▼
Handler parses URL, validates structure
        │
        ├── INVALID: Show error notification, abort
        │
        ▼ VALID
Handler determines terminal application
        │
        ├── NOT FOUND: Show setup instructions
        │
        ▼ FOUND
Launch new terminal window with caro command
        │
        ▼
Caro receives --magic-link flag and encoded command
        │
        ▼
Parse and sanitize command input
        │
        ▼
Check for missing prerequisites
        │
        ├── MISSING: Offer installation workflow
        │   │
        │   ├── USER DECLINES: Show manual install instructions
        │   │
        │   └── USER ACCEPTS: Install prerequisite, continue
        │
        ▼ ALL PRESENT
Run safety validation pipeline
        │
        ├── CRITICAL RISK: Refuse execution, explain why
        │
        ├── HIGH/MODERATE RISK: Show warnings prominently
        │
        ▼
Display confirmation UI with full context
        │
        ├── USER CANCELS: Exit gracefully
        │
        ▼ USER CONFIRMS
Execute command with standard Caro execution
        │
        ▼
Show results, log execution for audit
```

---

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Malicious magic links | Full safety validation, user confirmation required |
| Shell injection via URL | Strict URL parsing, input sanitization, no shell expansion on URL params |
| Phishing via fake Caro sites | Clear source display, domain trust indicators |
| Privilege escalation | Sudo detection, enhanced warnings, optional blocking |
| Pipe-to-shell patterns | Specific detection, CRITICAL risk classification |
| Social engineering | Education in UI, "don't trust blindly" messaging |

### Trust Levels

1. **Untrusted (Default)**: All magic links start here; maximum scrutiny
2. **Known Source**: Links from domains the user has previously trusted
3. **Verified Publisher**: Links with valid cryptographic signatures (Phase 2+)
4. **Official Caro**: Links from caro's official domains with special validation

### Audit Logging

All magic link executions are logged with:
- Timestamp
- Source URL (if available)
- Command (redacted for sensitive content)
- Safety validation results
- User decision (executed/cancelled)
- Execution result (success/failure)

---

## UX Principles

1. **Transparency First**: Users always see exactly what will run before it runs
2. **No Surprises**: Commands behave exactly as shown in the confirmation UI
3. **Easy Escape**: Cancel is always available and prominently displayed
4. **Education**: Every warning explains why, not just what
5. **Progressive Trust**: Repeat use of trusted sources can streamline (with user opt-in)
6. **Graceful Degradation**: When features fail, users can still accomplish their goal manually

---

## Version Roadmap

### Phase 1: MVP - Core Protocol (v1.1)
- `caro://` URL protocol registration (macOS, Linux)
- Basic terminal detection and launching
- Command parsing and sanitization
- Safety validation with enhanced scrutiny
- Simple TUI confirmation flow
- Execution with standard Caro pipeline
- Basic audit logging

### Phase 2: Enhanced UX (v1.2)
- Windows Terminal support
- Prerequisite detection and installation offers
- Source URL display and trust indicators
- Improved error messages and recovery
- Configuration for preferred terminal
- First-run onboarding flow

### Phase 3: Web Integration (v1.3)
- Official embeddable button components
- Link generator tool on caro website
- Documentation for website authors
- Copy-paste HTML snippets
- Integration examples for popular doc platforms

### Phase 4: Browser Extension (v2.0)
- Chrome extension for code snippet detection
- Firefox extension
- "Run in Caro" buttons on detected snippets
- Optional WebAssembly local analysis
- Privacy-preserving design

### Phase 5: Advanced Security (v2.1)
- Vendor security API integration
- Cryptographic link signing for trusted publishers
- Domain allowlist/blocklist
- Team policy controls
- Enhanced audit and reporting

---

## Success Metrics

- **Activation Rate**: % of magic link clicks that complete execution
- **Safety Catches**: Number of dangerous commands blocked or warned
- **User Retention**: Users returning via magic links after first use
- **Time to Execution**: Average seconds from click to command complete
- **Zero Incidents**: No security breaches via magic link vector

---

## Dependencies & Assumptions

### Dependencies
- OS-level URL protocol registration APIs
- Terminal application launch capabilities
- Existing Caro safety validation pipeline

### Assumptions
- Users have Caro installed before clicking magic links
- Users have a supported terminal application
- Browsers allow custom protocol handlers (may require user permission)

---

## Open Questions

1. **[NEEDS CLARIFICATION]**: Should magic links work if Caro is not installed? (Install prompt flow?)
2. **[NEEDS CLARIFICATION]**: What is the maximum command length we should support?
3. **[NEEDS CLARIFICATION]**: Should we support batch/multiple commands in a single magic link?
4. **[NEEDS CLARIFICATION]**: How should we handle magic links clicked on mobile devices?
5. **[NEEDS CLARIFICATION]**: Should there be a "remember trust for this domain" feature?

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [ ] Review checklist passed (pending clarification resolution)
