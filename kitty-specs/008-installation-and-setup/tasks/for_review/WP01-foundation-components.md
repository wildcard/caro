---
work_package_id: "WP01"
subtasks:
  - "T001"
  - "T002"
  - "T003"
  - "T004"
  - "T005"
title: "Foundation - Shared Documentation Components"
phase: "Phase 1 - Foundation"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "8158"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-31T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP01 – Foundation - Shared Documentation Components

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ````astro`, ````typescript`, ````bash`

---

## Objectives & Success Criteria

Create 4 reusable Astro components that all documentation pages will use:
1. **CodeBlock.astro** - Syntax-highlighted code with copy button
2. **PlatformTabs.astro** - Tabbed interface for platform-specific instructions
3. **InstallMethod.astro** - Installation method card with badges
4. **ComingSoonBadge.astro** - Badge for planned features

**Success**: Components render correctly in isolation, match existing website design system, and can be imported into any documentation page.

---

## Context & Constraints

**Reference documents**:
- `kitty-specs/008-installation-and-setup/plan.md` (Component Design section, lines 142-167)
- `kitty-specs/008-installation-and-setup/spec.md` (Functional requirements FR-027 through FR-031)
- `kitty-specs/008-installation-and-setup/data-model.md` (InstallationMethod, Platform, SetupOption entities)

**Design constraints**:
- Must match existing website design system (orange gradient #ff8c42 to #ff6b35, existing typography, spacing)
- Responsive behavior required: mobile-friendly layouts
- Accessibility: WCAG 2.1 AA compliance (semantic HTML, keyboard navigation)

**Architectural decisions** (from research.md):
- Use Astro component architecture (existing framework)
- Follow patterns from `website/src/components/Navigation.astro` for styling
- Reuse existing color palette and spacing variables

---

## Subtasks & Detailed Guidance

### Subtask T001 – Create docs component directory
- **Purpose**: Organize new documentation-specific components separately from main website components.
- **Steps**:
  1. Create directory: `mkdir -p website/src/components/docs`
  2. Verify path exists: `ls website/src/components/docs`
- **Files**: `website/src/components/docs/` (new directory)
- **Parallel?**: Must complete before T002-T005.
- **Notes**: Keep docs components separate for better organization; main components (`Navigation.astro`, `Footer.astro`) stay in `website/src/components/`.

### Subtask T002 [P] – Implement CodeBlock.astro with copy button
- **Purpose**: Display syntax-highlighted code snippets with one-click copy functionality.
- **Steps**:
  1. Create `website/src/components/docs/CodeBlock.astro`
  2. Add frontmatter props: `code` (string), `language` (string), `filename?` (optional string)
  3. Use `<pre><code>` for code display with language class (e.g., `class="language-bash"`)
  4. Add copy button with clipboard icon
  5. Implement copy functionality using `navigator.clipboard.writeText()`
  6. Add visual feedback on successful copy (e.g., "Copied!" tooltip or checkmark)
  7. Optional: Add syntax highlighting using Prism.js or existing website highlighting
  8. Style to match website (orange button, rounded corners)
  9. Test copy button in Chrome, Firefox, Safari
- **Files**: `website/src/components/docs/CodeBlock.astro` (create new)
- **Parallel?**: Yes, independent of T003-T005.
- **Notes**:
  - Check if website already uses syntax highlighting library
  - Fallback for browsers without clipboard API: provide "Copy" text feedback only
  - Include filename header if `filename` prop provided

**Example usage**:
```astro
<CodeBlock code="cargo install caro" language="bash" />
<CodeBlock code="npm install" language="bash" filename="Terminal" />
```

### Subtask T003 [P] – Implement PlatformTabs.astro with localStorage persistence
- **Purpose**: Render tabbed interface for platform/shell-specific instructions that remembers user selection.
- **Steps**:
  1. Create `website/src/components/docs/PlatformTabs.astro`
  2. Add props: `platforms` (array of `{name: string, icon: string, content: string}`)
  3. Render tab buttons for each platform (horizontal on desktop, stacked on mobile)
  4. Show active tab content below tabs
  5. Add click handler to switch tabs
  6. Save selected tab to `localStorage.setItem('preferredPlatform', name)`
  7. On component mount, read from localStorage and activate saved tab
  8. Style tabs with orange active indicator
  9. Make keyboard accessible (Tab key navigation, Enter to activate)
- **Files**: `website/src/components/docs/PlatformTabs.astro` (create new)
- **Parallel?**: Yes, independent of T002, T004-T005.
- **Notes**:
  - Use CSS Grid or Flexbox for responsive tab layout
  - On mobile (< 768px): stack tabs vertically
  - Include ARIA attributes: `role="tablist"`, `role="tab"`, `aria-selected`

**Example usage**:
```astro
<PlatformTabs platforms={[
  {name: "bash", icon: "🐚", content: "echo 'source ~/.bashrc'"},
  {name: "zsh", icon: "⚡", content: "echo 'source ~/.zshrc'"},
  {name: "fish", icon: "🐠", content: "echo 'source ~/.config/fish/config.fish'"}
]} />
```

### Subtask T004 [P] – Implement InstallMethod.astro card component
- **Purpose**: Display installation method as styled card with metadata badges.
- **Steps**:
  1. Create `website/src/components/docs/InstallMethod.astro`
  2. Add props matching InstallationMethod entity from data-model.md:
     - `name` (string)
     - `description` (string)
     - `difficulty` ("easy" | "intermediate" | "advanced")
     - `platforms` (array of strings)
     - `status` ("available" | "coming_soon")
     - `installCommand` (string)
     - `verificationCommand` (string)
     - `prerequisites?` (array of strings, optional)
  3. Render card with:
     - Difficulty badge (color-coded: green=easy, yellow=intermediate, red=advanced)
     - Platform icons (macOS, Linux logos)
     - Status badge (if coming_soon, show ComingSoonBadge)
     - Install command in CodeBlock
     - Verification command in CodeBlock
     - Collapsible "Prerequisites" section (if provided)
  4. Style card with border, padding, rounded corners
  5. Add hover effect (subtle shadow increase)
- **Files**: `website/src/components/docs/InstallMethod.astro` (create new)
- **Parallel?**: Yes, but should come after T005 if using ComingSoonBadge internally.
- **Notes**:
  - Reuse CodeBlock component for commands
  - Use `<details>` + `<summary>` for collapsible sections
  - Platform icons: use emoji or SVG (🍎 macOS, 🐧 Linux)

**Example usage**:
```astro
<InstallMethod
  name="Cargo Install"
  description="Install from crates.io using Rust's package manager"
  difficulty="easy"
  platforms={["macOS", "Linux"]}
  status="available"
  installCommand="cargo install caro"
  verificationCommand="caro --version"
  prerequisites={["Rust 1.75+", "Cargo"]}
/>
```

### Subtask T005 [P] – Implement ComingSoonBadge.astro component
- **Purpose**: Visual indicator for planned features not yet implemented.
- **Steps**:
  1. Create `website/src/components/docs/ComingSoonBadge.astro`
  2. Add props: `feature` (string), `issueUrl?` (optional string), `description?` (optional string)
  3. Render badge with:
     - Orange/yellow background (#FFA500 or similar)
     - "Coming Soon" text with feature name
     - Link to GitHub issue if `issueUrl` provided
     - Tooltip or small text with description if provided
  4. Style as inline badge (not full-width)
  5. Make accessible (proper link semantics, keyboard navigable)
- **Files**: `website/src/components/docs/ComingSoonBadge.astro` (create new)
- **Parallel?**: Yes, independent of T002-T004.
- **Notes**:
  - Use subtle animation (optional): pulse or fade
  - If no issueUrl, render as plain badge (no link)
  - Keep badge small and unobtrusive

**Example usage**:
```astro
<ComingSoonBadge feature="Homebrew" issueUrl="https://github.com/wildcard/caro/issues/123" />
<ComingSoonBadge feature="mise integration" description="Command history sync across projects" />
```

---

## Test Strategy

**Manual testing** (no automated tests requested):
1. Create test page: `website/src/pages/test-components.astro`
2. Import all 4 components
3. Render with various props
4. Test in local dev server (`npm run dev`)
5. Verify:
   - CodeBlock copy button works
   - PlatformTabs switches tabs and persists selection
   - InstallMethod displays all sections correctly
   - ComingSoonBadge renders and links work
6. Test responsive behavior (resize browser to 375px, 768px, 1440px)
7. Test accessibility (keyboard navigation, screen reader)

---

## Risks & Mitigations

**Risk: Copy button doesn't work in older browsers**
- Mitigation: Use feature detection; provide fallback message if clipboard API unavailable

**Risk: Design inconsistency with existing website**
- Mitigation: Reference `website/src/components/Navigation.astro` and homepage for color/spacing patterns

**Risk: localStorage not available (incognito/private mode)**
- Mitigation: Wrap localStorage calls in try-catch; fall back to first tab if storage fails

**Risk: Accessibility failures**
- Mitigation: Use semantic HTML (`<button>`, `<details>`, `<a>`), include ARIA labels, test with keyboard

---

## Definition of Done Checklist

- [ ] T001: `website/src/components/docs/` directory exists
- [ ] T002: CodeBlock.astro renders code with working copy button
- [ ] T003: PlatformTabs.astro switches tabs and persists selection to localStorage
- [ ] T004: InstallMethod.astro displays installation method card with all metadata
- [ ] T005: ComingSoonBadge.astro renders badge with optional GitHub link
- [ ] All components match website design system (colors, typography, spacing)
- [ ] All components are responsive (mobile, tablet, desktop)
- [ ] All components are keyboard accessible
- [ ] `tasks.md` checkboxes for T001-T005 marked complete

---

## Review Guidance

**Key acceptance checkpoints**:
1. **Visual consistency**: Do components match website brand (orange gradient, typography)?
2. **Functionality**: Does copy button work? Do tabs switch? Does localStorage persist?
3. **Responsiveness**: Do components work on mobile, tablet, desktop?
4. **Accessibility**: Can all components be navigated with keyboard? Are ARIA labels present?
5. **Code quality**: Is code clean, well-commented, and using Astro best practices?

**Context for reviewers**:
- Reference existing components (`Navigation.astro`) for design patterns
- Check that components are in `website/src/components/docs/` directory
- Verify components can be imported: `import CodeBlock from '../components/docs/CodeBlock.astro';`

---

## Activity Log

> Append entries when the work package changes lanes. Include timestamp, agent, shell PID, lane, and a short note.

- 2025-12-31T00:00:00Z – system – lane=planned – Prompt created.

---

### Updating Metadata When Changing Lanes

1. Capture your shell PID: `echo $$` (or use helper scripts when available).
2. Update frontmatter (`lane`, `assignee`, `agent`, `shell_pid`).
3. Add an entry to the **Activity Log** describing the transition.
4. Run `.kittify/scripts/bash/tasks-move-to-lane.sh 008-installation-and-setup WP01 <lane>` to move the prompt, update metadata, and append history in one step.
5. Commit or stage the change, preserving history.
- 2025-12-31T09:24:39Z – claude – shell_pid=8158 – lane=doing – Starting implementation of foundation components
- 2025-12-31T09:28:06Z – claude – shell_pid=8158 – lane=for_review – Completed all foundation components (T001-T005)
