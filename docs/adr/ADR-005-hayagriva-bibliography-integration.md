# ADR-005: Hayagriva Bibliography Integration

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | January 2026                        |
| **Authors**    | Caro Maintainers                    |
| **Target**     | Community                           |
| **Supersedes** | N/A                                 |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [About Hayagriva](#about-hayagriva)
4. [Decision Drivers](#decision-drivers)
5. [Potential Integration Scenarios](#potential-integration-scenarios)
6. [Technical Analysis](#technical-analysis)
7. [Alternatives Considered](#alternatives-considered)
8. [Decision](#decision)
9. [Consequences](#consequences)
10. [Implementation Notes](#implementation-notes)
11. [References](#references)

---

## Executive Summary

This ADR evaluates the integration of [Hayagriva](https://github.com/typst/hayagriva), a Rust library for bibliography management and citation formatting, with Caro. After thorough analysis, **we recommend deferring integration** as the use cases do not align with Caro's core mission of converting natural language to shell commands.

**Key Finding:** While Hayagriva is an excellent library for scholarly writing and reference management, the integration cost exceeds the benefit for Caro's primary user base. However, we identify specific future scenarios where integration could become valuable.

---

## Context and Problem Statement

### The Question

During research into Caro's development roadmap, the question arose: **Could Hayagriva enhance Caro's capabilities by providing citation and bibliography features?**

This ADR investigates:
1. What Hayagriva provides
2. How it could integrate with Caro
3. Whether the integration delivers meaningful value

### Current State

Caro is a single-binary CLI tool that:
- Converts natural language descriptions to safe POSIX shell commands
- Uses local LLM inference for privacy-first operation
- Provides safety validation before command execution
- Targets developers and system administrators

Caro does **not** currently:
- Generate or manage bibliographic references
- Track provenance of generated commands
- Produce scholarly or research-oriented output
- Interface with citation management workflows

---

## About Hayagriva

### Overview

[Hayagriva](https://crates.io/crates/hayagriva) (current version: 0.9.1) is a Rust crate developed by the [Typst](https://typst.app/) team that provides:

- **YAML-backed bibliography format**: Native data model for literature entries
- **CSL Processing**: Supports 2,600+ Citation Style Language styles
- **BibTeX/BibLaTeX interop**: Parse and convert existing bibliographies
- **Selector language**: Filter entries by type and metadata
- **Flexible output**: Generate both in-text citations and reference lists

### Key Features

| Feature | Description |
|---------|-------------|
| **CSL Support** | All 2,600+ styles from the official CSL repository |
| **Input Formats** | YAML (native), BibTeX, BibLaTeX |
| **Output** | Formatted citations, reference lists, metadata |
| **Query Language** | Filter by media type, author, date, etc. |
| **Locale Support** | Multi-language formatting |
| **Licensing** | MIT/Apache 2.0 (CSL styles: CC-BY-SA 3.0) |

### Architecture

Hayagriva consists of two crates:

```
┌─────────────────────────────────────────────────┐
│                  Hayagriva                       │
│  - Bibliography driver                          │
│  - Citation formatting                          │
│  - Entry management                             │
└─────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│                 Citationberg                     │
│  - CSL 1.0 parser                               │
│  - Style processing                             │
│  - Locale handling                              │
└─────────────────────────────────────────────────┘
```

### API Example

```rust
use hayagriva::{BibliographyDriver, CitationItem, CitationRequest};
use hayagriva::io::from_yaml_str;

// Parse bibliography
let bibliography = from_yaml_str(yaml_content)?;

// Create citation request
let entry = bibliography.get("entry-key")?;
let citation = CitationItem::with_entry(entry);
let request = CitationRequest::from_items(vec![citation]);

// Format using a style
let driver = BibliographyDriver::new();
let formatted = driver.citation(&request, &style, &locale)?;
```

---

## Decision Drivers

### Primary Drivers (Against Integration)

1. **Mission Alignment**: Caro's core purpose is NL→shell commands, not scholarly writing
2. **Dependency Cost**: Adding ~400KB+ to binary size for limited use cases
3. **Complexity**: Integration would add features orthogonal to primary workflow
4. **User Base**: CLI/DevOps users rarely need citation management

### Secondary Drivers (Potential Value)

1. **Provenance Tracking**: Could cite documentation sources for generated commands
2. **Research Workflows**: Research templates reference citations
3. **Educational Value**: Help users learn command origins
4. **Future Features**: May enable research-oriented command history

---

## Potential Integration Scenarios

### Scenario 1: Command Provenance Tracking

**Concept**: Track and cite the sources that informed command generation.

```
User: "find large files"
Caro: find . -size +100M -type f

      Sources:
      [1] GNU findutils manual, §2.3 "Size Tests"
      [2] POSIX.1-2017 §find(1)
```

**Analysis**:
- **Pros**: Educational value, transparency about knowledge sources
- **Cons**: LLMs don't reliably attribute specific sources; would require separate knowledge base; significant implementation complexity
- **Verdict**: Interesting but impractical with current architecture

### Scenario 2: Research Mode Output

**Concept**: Generate bibliographies for command research sessions.

```bash
caro --research "network diagnostics commands" --output-bib research.yaml
```

**Analysis**:
- **Pros**: Supports systematic learning workflows
- **Cons**: Requires curated command documentation database; low demand from user base
- **Verdict**: Niche use case, better served by external tools

### Scenario 3: Documentation Generation

**Concept**: Create cited documentation for generated scripts.

```bash
caro --explain --cite "backup home directory"
# Output includes BibTeX references to relevant documentation
```

**Analysis**:
- **Pros**: Professional documentation quality
- **Cons**: Requires maintaining extensive citation database; documentation tools already exist
- **Verdict**: Overlaps with existing documentation ecosystem

### Scenario 4: Man Page Reference Integration

**Concept**: Automatically cite man pages and official documentation.

```yaml
# Generated bibliography entry
- id: find-gnu-2023
  type: software-manual
  title: GNU findutils
  version: "4.9"
  url: https://www.gnu.org/software/findutils/manual/
```

**Analysis**:
- **Pros**: Structured reference to authoritative sources
- **Cons**: Man pages are already accessible; citation format adds overhead without clear benefit
- **Verdict**: Over-engineering for the use case

---

## Technical Analysis

### Dependency Impact

```toml
# Estimated additions to Cargo.toml
hayagriva = "0.9"  # Brings in:
                   # - citationberg (CSL parsing)
                   # - unscanny (string handling)
                   # - biblatex (BibTeX parsing, optional)
```

| Metric | Without Hayagriva | With Hayagriva | Delta |
|--------|-------------------|----------------|-------|
| Binary size | ~12MB | ~12.5MB | +500KB |
| Compile time | ~45s | ~55s | +10s |
| Dependencies | 142 | 158 | +16 |

### Integration Points

If integrated, Hayagriva would connect at:

```
┌─────────────────────────────────────────────────────────────┐
│                        Caro CLI                              │
│                            │                                 │
│           ┌────────────────┼────────────────┐               │
│           ▼                ▼                ▼               │
│    ┌──────────┐     ┌──────────┐     ┌──────────┐          │
│    │  Agent   │     │  Output  │     │ Research │ ◄── NEW  │
│    │   Loop   │     │ Formatter│     │  Module  │          │
│    └──────────┘     └──────────┘     └──────────┘          │
│                            │                │               │
│                            └────────┬───────┘               │
│                                     ▼                       │
│                            ┌──────────────┐                 │
│                            │  Hayagriva   │ ◄── NEW        │
│                            │  Integration │                 │
│                            └──────────────┘                 │
└─────────────────────────────────────────────────────────────┘
```

### Feature Gating

If implemented, would require feature gating:

```toml
[features]
default = []
research = ["hayagriva"]
bibliography = ["research", "hayagriva/biblatex"]
```

---

## Alternatives Considered

### Alternative 1: Full Hayagriva Integration

- **Description**: Integrate Hayagriva as a core dependency
- **Pros**: Rich citation capabilities, professional output
- **Cons**: Significant complexity for limited use case, binary bloat
- **Decision**: Not recommended

### Alternative 2: Optional Bibliography CLI Extension

- **Description**: Separate binary `caro-bib` that extends caro with citation features
- **Pros**: No impact on core binary, opt-in for researchers
- **Cons**: Maintenance burden, fragmented user experience
- **Decision**: Possible future consideration

### Alternative 3: External Tool Integration

- **Description**: Recommend external tools (Zotero, JabRef) for citation needs
- **Pros**: Zero implementation cost, leverages mature tools
- **Cons**: Not integrated, requires context switching
- **Decision**: Current recommendation

### Alternative 4: Defer Integration

- **Description**: Document use cases, revisit when demand materializes
- **Pros**: No premature optimization, data-driven decision
- **Cons**: May miss early adopter feedback
- **Decision**: **Recommended approach**

---

## Decision

### Primary Decision

**We recommend deferring Hayagriva integration.**

The analysis reveals:
1. No clear user demand for citation features in a shell command generator
2. Integration cost (complexity, dependencies, maintenance) exceeds identified benefits
3. Use cases are speculative rather than driven by user needs
4. External tools adequately serve researchers who need citation management

### Conditional Triggers for Revisiting

This decision should be revisited if:

1. **User Demand**: >10 user requests for citation/bibliography features
2. **Research Mode**: Caro develops a dedicated research/learning mode
3. **Documentation Generator**: Caro expands to generate documented scripts
4. **Enterprise Features**: Organizations request provenance tracking for compliance

### Recording for Future Reference

The following integration patterns are preserved for future consideration:

```rust
// Potential future API (NOT FOR IMPLEMENTATION)
pub mod research {
    use hayagriva::{Entry, Library};

    /// Track command provenance
    pub struct CommandProvenance {
        pub command: String,
        pub sources: Vec<Entry>,
        pub confidence: f32,
    }

    /// Generate research bibliography
    pub fn generate_bibliography(
        session: &CommandSession,
        style: CitationStyle,
    ) -> Library {
        // Collect sources from command generation
        // Format according to style
        todo!()
    }
}
```

---

## Consequences

### Benefits of Deferral

1. **Maintained Focus**: Core mission remains clear (NL → shell commands)
2. **Binary Size**: No additional ~500KB for unused features
3. **Reduced Complexity**: Simpler architecture, easier maintenance
4. **Faster Builds**: No additional compile-time dependencies

### Trade-offs

1. **Research Gap**: Users interested in command research must use external tools
2. **Provenance Opacity**: No formal tracking of knowledge sources
3. **Future Work**: May need architecture changes if later adopted

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Missing market opportunity | Low | Low | Monitor user feedback for citation requests |
| Architecture lock-in preventing later adoption | Low | Medium | Document integration points (done in this ADR) |
| Competitive disadvantage | Very Low | Low | Core competitors don't offer this either |

---

## Implementation Notes

### If Reconsidered

Should this decision be revisited, implementation should:

1. **Feature-gate strictly**: `cargo build --features bibliography`
2. **Minimal surface area**: Only expose essential citation functions
3. **Optional output formats**: Add `--format=bibtex` without affecting default behavior
4. **Lazy loading**: Don't impact startup time for non-research workflows

### Current Recommendation

For users who need citation management alongside Caro:

```bash
# Generate commands with caro
caro "find all rust files modified this week" > commands.sh

# Manage citations with dedicated tools
# - Zotero (GUI, collaborative)
# - JabRef (Java, BibTeX-focused)
# - papis (Python CLI, integrates with various formats)
# - bibutils (CLI, format conversion)
```

---

## References

### Hayagriva Resources

- [Hayagriva GitHub Repository](https://github.com/typst/hayagriva)
- [Hayagriva Documentation (docs.rs)](https://docs.rs/hayagriva/latest/hayagriva/)
- [Hayagriva on crates.io](https://crates.io/crates/hayagriva)
- [Citationberg CSL Parser](https://docs.rs/citationberg/latest/citationberg/)

### Citation Style Language

- [CSL Official Repository](https://github.com/citation-style-language/styles)
- [CSL Specification](https://docs.citationstyles.org/en/stable/specification.html)

### Related Caro Documents

- [ADR-001: LLM Inference Architecture](./001-llm-inference-architecture.md)
- [Research Templates](../../.kittify/missions/research/templates/)

### Related Tools

- [crate2bib: Citing Rust crates](https://arxiv.org/abs/2511.07468)
- [Typst: Modern typesetting system](https://typst.app/)

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | Caro Maintainers | Initial draft - research evaluation |

---

*This ADR documents the evaluation of Hayagriva integration and recommends deferral. The decision may be revisited based on user demand and evolving product direction.*
