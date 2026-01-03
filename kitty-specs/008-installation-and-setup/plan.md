# Implementation Plan: Installation and Setup Documentation

**Branch**: `008-installation-and-setup` | **Date**: 2025-12-31 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/kitty-specs/008-installation-and-setup/spec.md`

## Summary

Create comprehensive installation and setup documentation with three distinct pages to serve different user audiences. The Quick Start Guide targets first-time users with a streamlined path to their first command execution. The Installation Page provides both automated script installation (for simplicity-seekers) and manual methods (for power users). The Setup & Configuration Page documents post-install ergonomics including shell completions, aliases, and backend configuration.

**Technical Approach**: Implement as Astro pages in existing `website/` directory, reusing component architecture and design system. Add new "Docs" dropdown to Navigation component for discoverability. Extract content from README.md and consolidate scattered installation information into authoritative documentation.

## Technical Context

**Language/Version**: TypeScript/JavaScript (Astro 4.x), HTML/CSS for page content
**Primary Dependencies**: Astro (existing static site generator), existing website components (Layout.astro, Navigation.astro, Footer.astro)
**Storage**: Static site generation - no database required; content stored in .astro files
**Testing**: Manual testing of page rendering, responsive design, link validation; automated lighthouse/a11y checks (if existing)
**Target Platform**: Web browsers (desktop and mobile); static hosting (Vercel/Netlify/GitHub Pages)
**Project Type**: Web documentation (static site addition)
**Performance Goals**: Page load < 2s on 3G connection; Lighthouse score > 90; accessibility score 100
**Constraints**: Must maintain existing design system consistency; mobile-responsive mandatory; all code snippets must be copy-pasteable
**Scale/Scope**: 3 new documentation pages; 1 navigation component update; ~15-20 content sections total

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Note**: Constitution file is empty template - no project-specific principles defined yet. Proceeding with standard web documentation best practices:

✅ **Content Quality**: Documentation must be accurate, testable, and maintainable
✅ **Design Consistency**: Must match existing website design system (orange gradient brand, typography, spacing)
✅ **Accessibility**: WCAG 2.1 AA compliance; semantic HTML; keyboard navigable
✅ **Mobile-First**: Responsive design mandatory per specification requirements
✅ **Code Examples**: All snippets must be executable; include platform/shell context
✅ **Maintainability**: Separate content from presentation; use existing component library

**No violations identified** - this is straightforward documentation work using established patterns.

## Project Structure

### Documentation (this feature)

```
kitty-specs/008-installation-and-setup/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file (in progress)
├── research.md          # Phase 0 research decisions (completed)
├── data-model.md        # Entity model (completed)
├── research/
│   ├── evidence-log.csv     # Research evidence tracking (completed)
│   └── source-register.csv  # Source citations (completed)
├── checklists/
│   └── requirements.md  # Spec quality validation (completed)
└── tasks.md             # Phase 2 work packages (created by /spec-kitty.tasks)
```

### Source Code (repository root)

```
website/
├── src/
│   ├── pages/
│   │   ├── quick-start.astro      # NEW: P1 Quick Start Guide
│   │   ├── installation.astro     # NEW: P2 Installation Page
│   │   ├── setup.astro            # NEW: P3 Setup & Configuration Page
│   │   ├── index.astro            # MODIFY: Add links to new docs
│   │   └── [existing pages...]
│   ├── components/
│   │   ├── Navigation.astro       # MODIFY: Add Docs dropdown
│   │   ├── Download.astro         # REFERENCE: Extract install script info
│   │   ├── Footer.astro           # POSSIBLY MODIFY: Add docs links
│   │   └── [docs-specific]/       # NEW: Components for doc pages
│   │       ├── CodeBlock.astro        # Copy button functionality
│   │       ├── PlatformTabs.astro     # Platform-specific instructions
│   │       ├── InstallMethod.astro    # Installation method card
│   │       └── ComingSoonBadge.astro  # "Coming Soon" indicator
│   └── layouts/
│       └── Layout.astro           # EXISTING: Use for documentation pages
├── public/
│   └── [static assets]
└── astro.config.mjs              # EXISTING: No changes needed

README.md                         # REFERENCE: Extract installation content
```

**Structure Decision**: Using existing Astro web application structure (Option 2 pattern). New documentation pages go in `website/src/pages/` alongside existing marketing pages. Reuse existing `Layout.astro` for consistent page structure. Create new doc-specific components in `website/src/components/docs-specific/` subdirectory to keep them organized.

## Complexity Tracking

*No violations - this section is empty as no constitutional violations were identified.*

## Parallel Work Analysis

*This feature does not require parallel development - single developer/agent can complete sequentially.*

### Sequential Implementation Order

1. **Foundation** (Create shared components):
   - CodeBlock.astro with copy button
   - PlatformTabs.astro for platform-specific content
   - InstallMethod.astro card component
   - ComingSoonBadge.astro indicator

2. **Page Creation** (Build documentation pages):
   - quick-start.astro (P1 - highest value, smallest scope)
   - installation.astro (P2 - core content, larger scope)
   - setup.astro (P3 - optional enhancements)

3. **Navigation Integration** (Make pages discoverable):
   - Add Docs dropdown to Navigation.astro
   - Update mobile drawer navigation
   - Add links from homepage/README

4. **Content Population** (Fill pages with actual content):
   - Extract installation methods from README.md
   - Document shell completion setup
   - Create example configurations

5. **Polish & Testing** (Ensure quality):
   - Verify all links work
   - Test responsive behavior
   - Validate copy buttons function
   - Check accessibility

## Phase 0: Research Outcomes

✅ **Completed** - See `research.md` for full details.

**Key Decisions**:
- Confirmed Astro framework usage (existing infrastructure)
- Navigation integration via dropdown pattern (matches existing Compare/Resources dropdowns)
- Three separate pages for audience segmentation (novices vs power users)
- Component reuse from existing website architecture

**Evidence Sources**:
- Codebase exploration: `website/src/components/Navigation.astro`
- Codebase exploration: `website/src/pages/*.astro`
- Astro documentation: https://docs.astro.build

## Phase 1: Design & Contracts

### Component Design

**CodeBlock.astro**:
- Props: `code` (string), `language` (string), `filename` (optional string)
- Displays syntax-highlighted code block
- Includes copy-to-clipboard button
- Shows filename header if provided
- Emits "copied" event for analytics

**PlatformTabs.astro**:
- Props: `platforms` (array of {name, icon, content})
- Renders tabbed interface for platform-specific instructions
- Persists selected tab to localStorage
- Responsive: stacked on mobile, tabs on desktop

**InstallMethod.astro**:
- Props: `method` (InstallationMethod entity)
- Displays installation method as card
- Shows difficulty badge, platform icons, status badge
- Includes collapsible "Prerequisites" and "Verification" sections

**ComingSoonBadge.astro**:
- Props: `feature` (string), `issueUrl` (optional string)
- Displays orange/yellow badge with "Coming Soon" text
- Links to GitHub issue if URL provided
- Includes brief description of planned feature

### Page Contracts

**quick-start.astro**:
- Input: None (static content)
- Output: HTML page with 3-4 sequential steps
- Sections: Install → Verify → Generate First Command → Next Steps
- Links to: installation.astro, setup.astro
- Success: User can complete in < 5 minutes

**installation.astro**:
- Input: None (static content, possibly dynamic status from build-time data)
- Output: HTML page with all installation methods
- Sections: Automated Script (top) → Manual Methods (cargo, binaries, source, package managers)
- Each method includes: description, requirements, commands, verification, troubleshooting
- "Coming Soon" badges on unsupported methods

**setup.astro**:
- Input: None (static content)
- Output: HTML page with post-install configuration options
- Sections: Shell Completions → Aliases → Backend Config → Tool Integrations → Environment Variables
- Shell-specific tabs for completion/alias examples
- "Coming Soon" badges on planned integrations (mise, direnv)

### Navigation Contract

**Navigation.astro modifications**:
- Add new dropdown trigger: "Docs" (between "Resources" and "Support")
- Dropdown panel items:
  1. Quick Start (icon: 🚀, desc: "Get started in 5 minutes")
  2. Installation (icon: 📦, desc: "All installation methods")
  3. Setup (icon: ⚙️, desc: "Configure your environment")
- Mobile drawer: Add new "Documentation" section with same 3 links

## Quickstart Guide

*For developers implementing this feature - see `quickstart.md` (to be created in Phase 1)*

## Next Steps

1. ✅ Phase 0 Research: Complete (research.md, data-model.md, CSV logs created)
2. **Current**: Phase 1 Design (this plan.md document)
3. **Next**: Run `/spec-kitty.tasks` to generate work packages from this plan
4. **Then**: Run `/spec-kitty.implement` to execute implementation tasks
5. **Finally**: Run `/spec-kitty.review` → `/spec-kitty.accept` → `/spec-kitty.merge`

## Open Questions

1. **Automated Install Script**: Does an automated installation script currently exist? If not, should we create it or mark as "Coming Soon"?
   - **Impact**: Affects quick-start.astro and installation.astro content
   - **Resolution**: Check project scripts, CI/CD, or create issue to track

2. **Shell Completion Generation**: Can Caro CLI generate shell completions (e.g., via clap's `completions` feature)?
   - **Impact**: Affects setup.astro shell completions section
   - **Resolution**: Check `src/main.rs` for clap completion code

3. **Configuration File Format**: Does Caro use a config file? If so, what format (TOML, YAML, JSON)?
   - **Impact**: Affects setup.astro backend configuration examples
   - **Resolution**: Check codebase for config loading code

**Action**: These questions should be resolved during implementation by reading codebase and documenting actual capabilities. Mark features as "Coming Soon" if not yet implemented.
