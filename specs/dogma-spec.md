# Feature Specification: Dogma Rule Engine

**Feature Branch**: `001-dogma-rule-engine`
**Created**: 2024-12-24
**Status**: Draft
**Input**: User description: "Dogma: Distributed rule engine crate with standalone capability, shellfirm integration, and feature-flagged Caro integration"

---

## Quick Guidelines
- Focus on WHAT users need and WHY
- Dogma works independently AND integrates with Karo
- Maintains backward compatibility with existing Caro safety behavior

---

## User Scenarios & Testing *(mandatory)*

### Primary User Stories

#### Story 1: Standalone Dogma User
As a security-conscious developer, I want to validate shell commands against a comprehensive rule database before execution, so that I can prevent dangerous operations from harming my system.

#### Story 2: Karo User with Dogma Backend
As a Karo user, I want to optionally use Dogma as the safety validation backend, so that I can benefit from community-contributed rules and shellfirm patterns while maintaining the same safety guarantees.

#### Story 3: Rule Contributor
As a community member, I want to contribute new dangerous command patterns via YAML files, so that the community benefits from collective security knowledge without requiring Rust expertise.

### Acceptance Scenarios

#### Standalone Usage
1. **Given** Dogma is installed as a standalone binary, **When** I run `dogma validate "rm -rf /"`, **Then** I receive a CRITICAL risk assessment with explanation
2. **Given** Dogma is installed, **When** I run `dogma validate "ls -la"`, **Then** I receive a SAFE risk assessment
3. **Given** shellfirm vendor is enabled, **When** I validate a git force-push command, **Then** rules from both Dogma and shellfirm are applied

#### Library Integration
4. **Given** Dogma is used as a library, **When** I call `dogma.validate("command")`, **Then** I get a ValidationResult with risk_level, allowed, and matched_patterns
5. **Given** local rules exist in `~/.config/karo/rules/`, **When** validation runs, **Then** local rules are checked in addition to embedded rules

#### Caro Integration
6. **Given** Caro is built with `--features dogma`, **When** Caro validates a command, **Then** Dogma is used instead of native safety module
7. **Given** Caro runs the existing test suite, **When** using Dogma backend, **Then** all tests pass with equivalent behavior
8. **Given** a command that native safety blocks, **When** validated by Dogma, **Then** same risk level is returned

### Edge Cases
- What happens when a rule pattern is invalid regex? **System logs warning and skips that rule**
- What happens when YAML rule file has syntax errors? **System returns parse error with file location**
- What happens when local and embedded rules conflict? **Local rules take priority**
- How does system handle empty command input? **Returns Safe with "empty command" note**
- What happens with shell-specific rules on wrong shell? **Rules are filtered by shell type**

---

## Requirements *(mandatory)*

### Functional Requirements - Core Engine

- **FR-001**: Dogma MUST work as a standalone CLI binary (`dogma validate "command"`)
- **FR-002**: Dogma MUST work as a Rust library crate (`dogma::Dogma::validate()`)
- **FR-003**: Dogma MUST load rules from multiple sources with defined priority order
- **FR-004**: Dogma MUST support YAML rule format with regex patterns
- **FR-005**: Dogma MUST return validation results with: risk_level, allowed, explanation, matched_patterns
- **FR-006**: Dogma MUST compile and cache regex patterns for performance

### Functional Requirements - Rule Sources

- **FR-010**: Dogma MUST embed community rules at compile time (zero-cost loading)
- **FR-011**: Dogma MUST support local user rules from configuration directory
- **FR-012**: Dogma MUST support vendor rules (shellfirm integration)
- **FR-013**: Dogma SHOULD support remote rule repositories (opt-in, future)
- **FR-014**: Dogma MUST provide rule attribution (source tracking)

### Functional Requirements - Shellfirm Integration

- **FR-020**: Dogma MUST vendor shellfirm YAML check files
- **FR-021**: Dogma MUST translate shellfirm rule format to Dogma format
- **FR-022**: Dogma MUST support shellfirm filter types (IsExists, NotContains)
- **FR-023**: Dogma MUST attribute shellfirm rules with vendor source

### Functional Requirements - Caro Integration

- **FR-030**: Caro MUST support feature flag `--features dogma` for Dogma backend
- **FR-031**: Caro MUST maintain `--features native-safety` for existing implementation
- **FR-032**: Caro MUST produce identical results with either backend for existing patterns
- **FR-033**: Comparison tests MUST validate behavior parity between backends

### Functional Requirements - Rule Format

- **FR-040**: Rules MUST have unique IDs (format: `source:category:name`)
- **FR-041**: Rules MUST specify severity level (Critical, High, Moderate, Low, Info)
- **FR-042**: Rules MUST include regex pattern for command matching
- **FR-043**: Rules MUST include human-readable description
- **FR-044**: Rules MAY specify shell types (bash, zsh, sh, fish, powershell)
- **FR-045**: Rules MAY include filters for contextual application

### Non-Functional Requirements

- **NFR-001**: Validation latency MUST be < 10ms for typical commands
- **NFR-002**: Rule loading MUST complete in < 100ms at startup
- **NFR-003**: Binary size increase MUST be < 5MB with all rules
- **NFR-004**: All existing Caro safety tests MUST pass with Dogma backend

---

### Key Entities

- **Rule**: A single validation pattern with ID, severity, regex, description, and metadata
- **RuleSet**: Collection of rules from one or more sources, deduplicated by priority
- **RuleSource**: Origin of rules (Embedded, Local, Vendor, Remote)
- **ValidationResult**: Outcome of command validation with risk assessment
- **Filter**: Contextual condition for rule application (PathExists, NotContains, etc.)
- **Provider**: Rule loader implementation for a specific source type

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks, APIs) - *Spec describes WHAT not HOW*
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Dependencies & Assumptions

### Dependencies
- Existing Caro safety module patterns (source for initial Dogma rules)
- Shellfirm repository (vendor rules source)
- Cargo workspace support

### Assumptions
- Users have `~/.config/karo/` directory for local configuration
- YAML is acceptable format for rule definitions
- Shellfirm rules are compatible with Dogma's safety model

---

## Out of Scope

- Real-time remote rule synchronization (future feature)
- Rule signing and verification (future feature)
- GUI rule editor (future feature)
- Rule analytics/telemetry (future feature)

---

## Success Metrics

1. **Behavioral Parity**: 100% of existing Caro safety tests pass with Dogma backend
2. **Performance**: No measurable regression in command validation latency
3. **Coverage**: All 48+ existing patterns migrated to Dogma YAML format
4. **Extensibility**: At least 50 additional rules from shellfirm integration

---

## Execution Status

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked (none remaining)
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed
