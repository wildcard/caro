---
work_package_id: "WP05"
subtasks:
  - "T027"
  - "T028"
  - "T029"
  - "T030"
title: "Navigation Integration - Discoverability"
phase: "Phase 2 - Integration (P1)"
lane: "doing"
assignee: ""
agent: "claude"
shell_pid: "40231"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-31T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP05 – Navigation Integration - Discoverability

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately.
- **You must address all feedback** before your work is complete.
- **Mark as acknowledged**: Update `review_status: acknowledged` when you begin addressing feedback.
- **Report progress**: Update Activity Log as you address feedback items.

---

## Review Feedback

*[Empty initially. Reviewers populate this section if work needs changes.]*

---

## Objectives & Success Criteria

Add **Docs dropdown** to Navigation component to make documentation pages discoverable from any page on the website.

**Success**: Users can navigate to Quick Start, Installation, and Setup pages from main navigation. Dropdown works on desktop (hover) and mobile (drawer).

**Acceptance scenarios**:
1. Desktop users see Docs dropdown in main navigation
2. Hovering over Docs shows dropdown with 3 links
3. Mobile users see Documentation section in drawer
4. All links navigate correctly to documentation pages

---

## Context & Constraints

**Reference documents**:
- `kitty-specs/008-installation-and-setup/plan.md` (Navigation contract, lines 193-201)
- `kitty-specs/008-installation-and-setup/quickstart.md` (Phase 5 implementation guide, lines 153-167)
- `website/src/components/Navigation.astro` (existing component to modify)

**Design constraints**:
- Must follow existing dropdown pattern from Resources section (lines 60-97)
- Desktop: Dropdown appears on hover
- Mobile: Section appears in slide-out drawer
- Use icons for visual clarity: 🚀 (Quick Start), 📦 (Installation), ⚙️ (Setup)
- Include brief descriptions for each link

**Reference pattern**: Navigation.astro already has a working dropdown for "Resources" - replicate this pattern for "Docs".

---

## Subtasks & Detailed Guidance

### Subtask T027 – Add "Docs" dropdown trigger to Navigation
- **Purpose**: Add new navigation item between "Resources" and "Support".
- **Steps**:
  1. Read `website/src/components/Navigation.astro` to understand structure
  2. Locate the Resources dropdown section (around lines 60-97)
  3. Add new "Docs" dropdown trigger **between Resources and Support**:
     ```astro
     <!-- After Resources dropdown, before Support -->
     <div class="nav-item dropdown">
       <button class="nav-trigger" aria-haspopup="true" aria-expanded="false">
         Docs
         <span class="dropdown-arrow">▼</span>
       </button>
       <!-- Dropdown panel added in T028 -->
     </div>
     ```
  4. Ensure button has same styling classes as other nav items
  5. Add appropriate ARIA attributes for accessibility
- **Files**: `website/src/components/Navigation.astro` (modify)
- **Parallel?**: Must complete before T028-T030.
- **Notes**: Use exact same pattern as Resources dropdown for consistency

### Subtask T028 – Add dropdown panel with documentation links
- **Purpose**: Create dropdown panel that appears on hover with 3 documentation links.
- **Steps**:
  1. Inside the nav-item div from T027, add dropdown panel:
     ```astro
     <div class="dropdown-panel">
       <a href="/quick-start" class="dropdown-item">
         <span class="item-icon">🚀</span>
         <div class="item-content">
           <div class="item-title">Quick Start</div>
           <div class="item-desc">Get started in 5 minutes</div>
         </div>
       </a>

       <a href="/installation" class="dropdown-item">
         <span class="item-icon">📦</span>
         <div class="item-content">
           <div class="item-title">Installation</div>
           <div class="item-desc">All installation methods</div>
         </div>
       </a>

       <a href="/setup" class="dropdown-item">
         <span class="item-icon">⚙️</span>
         <div class="item-content">
           <div class="item-title">Setup</div>
           <div class="item-desc">Configure your environment</div>
         </div>
       </a>
     </div>
     ```
  2. Ensure dropdown-panel has same styling as Resources dropdown
  3. Verify hover behavior works (dropdown appears on hover)
  4. Test links navigate correctly
- **Files**: `website/src/components/Navigation.astro` (modify)
- **Parallel?**: Depends on T027. Can do in parallel with T029.
- **Notes**: Icons can be emoji or SVG - match existing style

### Subtask T029 – Add Documentation section to mobile drawer
- **Purpose**: Make documentation accessible in mobile navigation drawer.
- **Steps**:
  1. Locate mobile drawer section in Navigation.astro (around lines 226-247)
  2. Add new "Documentation" section after existing sections:
     ```astro
     <!-- Mobile drawer section -->
     <div class="drawer-section">
       <div class="section-title">Documentation</div>
       <a href="/quick-start" class="drawer-item">
         <span class="item-icon">🚀</span>
         Quick Start
       </a>
       <a href="/installation" class="drawer-item">
         <span class="item-icon">📦</span>
         Installation
       </a>
       <a href="/setup" class="drawer-item">
         <span class="item-icon">⚙️</span>
         Setup
       </a>
     </div>
     ```
  3. Ensure styling matches existing drawer sections
  4. Verify drawer opens and links work on mobile
- **Files**: `website/src/components/Navigation.astro` (modify)
- **Parallel?**: Can do in parallel with T028 after T027.
- **Notes**: Test on actual mobile device or in browser dev tools mobile view

### Subtask T030 – Test dropdown and mobile navigation
- **Purpose**: Verify navigation integration works correctly on all devices.
- **Steps**:
  1. Start dev server: `cd website && npm run dev`
  2. **Desktop testing**:
     - Hover over "Docs" in navigation
     - Verify dropdown appears with 3 items
     - Click each link (Quick Start, Installation, Setup)
     - Verify navigation to correct pages
     - Test keyboard navigation (Tab to Docs, Enter to open)
  3. **Mobile testing**:
     - Resize browser to 375px width
     - Open mobile drawer (hamburger menu)
     - Verify "Documentation" section appears
     - Click each link
     - Verify navigation works
  4. **Cross-browser testing**:
     - Test in Chrome, Firefox, Safari
     - Verify dropdown works consistently
  5. **Accessibility testing**:
     - Tab through navigation with keyboard
     - Verify screen reader announces "Docs" correctly
     - Check ARIA attributes are correct
- **Files**: `website/src/components/Navigation.astro` (testing only)
- **Parallel?**: Must complete after T027-T029.
- **Notes**: Document any browser-specific issues found

---

## Test Strategy

**Manual testing**:
1. Desktop hover behavior:
   - Hover over "Docs" → dropdown appears
   - Move mouse away → dropdown disappears
   - Click links → navigate to correct pages
2. Mobile drawer:
   - Open drawer → see "Documentation" section
   - Tap links → navigate to correct pages
   - Close drawer → drawer closes cleanly
3. Keyboard navigation:
   - Tab to "Docs" button
   - Enter key → opens dropdown
   - Tab through links
   - Enter on link → navigates
4. Responsive breakpoints:
   - Test at 320px, 375px, 768px, 1024px, 1440px
   - Verify layout looks correct at all sizes
5. Cross-browser:
   - Chrome, Firefox, Safari, Edge
   - Verify consistent behavior

---

## Risks & Mitigations

**Risk: Dropdown conflicts with existing JavaScript**
- Mitigation: Follow exact pattern from Resources dropdown; test thoroughly

**Risk: Mobile drawer layout breaks**
- Mitigation: Use same styling classes as existing drawer sections

**Risk: Hover behavior doesn't work on touch devices**
- Mitigation: Ensure dropdown also opens on tap/click, not just hover

**Risk: Z-index issues with dropdown**
- Mitigation: Match z-index from Resources dropdown panel

---

## Definition of Done Checklist

- [ ] T027: "Docs" dropdown trigger added to Navigation.astro
- [ ] T028: Dropdown panel created with 3 documentation links
- [ ] T029: "Documentation" section added to mobile drawer
- [ ] T030: Desktop dropdown tested and working
- [ ] T030: Mobile drawer tested and working
- [ ] Desktop hover behavior works correctly
- [ ] Mobile tap/click behavior works correctly
- [ ] All links navigate to correct pages
- [ ] Keyboard navigation works (Tab, Enter)
- [ ] ARIA attributes correct for accessibility
- [ ] Cross-browser testing passed (Chrome, Firefox, Safari)
- [ ] Responsive design works (mobile, tablet, desktop)
- [ ] `tasks.md` checkboxes for T027-T030 marked complete

---

## Review Guidance

**Acceptance checkpoints**:
1. Can users discover documentation from navigation?
2. Does dropdown appear on hover (desktop)?
3. Do links navigate correctly?
4. Does mobile drawer show documentation section?
5. Is keyboard navigation working?
6. Does it work in all major browsers?

---

## Activity Log

- 2025-12-31T00:00:00Z – system – lane=planned – Prompt created.
- 2025-12-31T09:43:06Z – claude – shell_pid=40231 – lane=doing – Starting Navigation Integration implementation
