---
work_package_id: "WP02"
subtasks:
  - "T006"
  - "T007"
  - "T008"
  - "T009"
  - "T010"
title: "Quick Start Page - MVP"
phase: "Phase 2 - User Story 1 (P1)"
lane: "doing"
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

# Work Package Prompt: WP02 – Quick Start Page - MVP

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

Create **Quick Start Guide** page enabling first-time users to install Caro and generate their first command in under 5 minutes.

**Success**: New user on clean macOS/Linux system can follow guide and successfully run first `caro` command within 5 minutes. Page is accessible via `/quick-start` URL.

**Acceptance scenarios** (from spec.md, User Story 1):
1. User installs Caro and generates first command within 5 minutes
2. User can verify installation works with example command
3. User understands which backend they're using

---

## Context & Constraints

**Reference documents**:
- `kitty-specs/008-installation-and-setup/spec.md` (User Story 1, lines 10-23)
- `kitty-specs/008-installation-and-setup/plan.md` (Page contract for quick-start.astro, lines 171-176)
- `kitty-specs/008-installation-and-setup/quickstart.md` (Phase 2 implementation guide, lines 57-88)

**Design constraints**:
- Reading time must be < 3 minutes
- Sequential numbered steps (1-4 maximum)
- Minimal explanations - just commands and expected output
- Must use CodeBlock component from WP01

**Content priority** (from plan.md):
- Step 1: Install (automated script OR cargo install - verify which exists)
- Step 2: Verify (`caro --version`)
- Step 3: Generate first command (example: "list all files")
- Step 4: Next steps (links to installation.astro, setup.astro)

---

## Subtasks & Detailed Guidance

### Subtask T006 – Create quick-start.astro page structure
- **Purpose**: Set up page foundation using existing Layout.astro pattern.
- **Steps**:
  1. Create `website/src/pages/quick-start.astro`
  2. Import components:
     ```astro
     import Layout from '../layouts/Layout.astro';
     import Navigation from '../components/Navigation.astro';
     import Footer from '../components/Footer.astro';
     import CodeBlock from '../components/docs/CodeBlock.astro';
     ```
  3. Add Layout wrapper with title and meta description:
     ```astro
     <Layout
       title="Quick Start Guide | Caro"
       description="Get started with Caro in under 5 minutes. Install, verify, and generate your first shell command."
     >
       <Navigation />
       <!-- Content sections go here (T007-T010) -->
       <Footer />
     </Layout>
     ```
  4. Add hero section with title "Quick Start Guide" and subtitle
  5. Create container div for content (max-width: 800px, centered)
- **Files**: `website/src/pages/quick-start.astro` (create new)
- **Parallel?**: Must complete before T007-T010.

### Subtask T007 – Write Step 1: Install section
- **Purpose**: Provide simplest installation method for new users.
- **Steps**:
  1. Add section heading: "Step 1: Install Caro"
  2. Check if automated install script exists (see Open Questions in plan.md line 216)
  3. **Option A** (if automated script exists):
     ```astro
     <h2>Step 1: Install Caro</h2>
     <p>Run this command in your terminal:</p>
     <CodeBlock
       code="curl -fsSL https://install.caro.sh | sh"
       language="bash"
     />
     ```
  4. **Option B** (if no automated script - use cargo):
     ```astro
     <h2>Step 1: Install Caro</h2>
     <p>Install using Cargo (requires Rust):</p>
     <CodeBlock
       code="cargo install caro"
       language="bash"
     />
     ```
  5. Add brief note about system requirements (macOS/Linux only if applicable)
- **Files**: `website/src/pages/quick-start.astro` (modify)
- **Parallel?**: Can draft in parallel with T008-T010 after T006.
- **Notes**: Verify which installation method is simpler - check README.md or crates.io

### Subtask T008 – Write Step 2: Verify installation
- **Purpose**: Help user confirm Caro installed successfully.
- **Steps**:
  1. Add section heading: "Step 2: Verify Installation"
  2. Show verification command:
     ```astro
     <h2>Step 2: Verify Installation</h2>
     <p>Check that Caro is installed:</p>
     <CodeBlock
       code="caro --version"
       language="bash"
     />
     ```
  3. Add expected output example:
     ```astro
     <p>You should see output like:</p>
     <CodeBlock
       code="caro 0.1.0"
       language="text"
     />
     ```
- **Files**: `website/src/pages/quick-start.astro` (modify)
- **Parallel?**: Can draft in parallel with T007, T009-T010.
- **Notes**: Update version number to match actual latest version

### Subtask T009 – Write Step 3: Generate first command
- **Purpose**: Give user immediate hands-on experience with Caro's core functionality.
- **Steps**:
  1. Add section heading: "Step 3: Generate Your First Command"
  2. Show example natural language prompt:
     ```astro
     <h2>Step 3: Generate Your First Command</h2>
     <p>Try generating a shell command:</p>
     <CodeBlock
       code='caro "list all files in this directory"'
       language="bash"
     />
     ```
  3. Add expected behavior:
     ```astro
     <p>Caro will generate and show you a safe shell command. Review it, then execute if it looks correct.</p>
     ```
  4. Briefly mention backend being used (if known): "Using [Ollama/vLLM/MLX] for inference"
- **Files**: `website/src/pages/quick-start.astro` (modify)
- **Parallel?**: Can draft in parallel with T007-T008, T010.
- **Notes**: Use simple, safe example - avoid anything destructive or complex

### Subtask T010 – Write Step 4: Next steps
- **Purpose**: Guide users to more advanced features and documentation.
- **Steps**:
  1. Add section heading: "Next Steps"
  2. Add links to other documentation pages:
     ```astro
     <h2>Next Steps</h2>
     <ul>
       <li><a href="/installation">Explore all installation methods</a> - Cargo, binaries, build from source</li>
       <li><a href="/setup">Configure your environment</a> - Shell completions, aliases, backend selection</li>
       <li><a href="https://github.com/wildcard/caro">View on GitHub</a> - Source code and documentation</li>
     </ul>
     ```
  3. Add call-to-action: "Need help? Check our support page or open an issue."
- **Files**: `website/src/pages/quick-start.astro` (modify)
- **Parallel?**: Can draft in parallel with T007-T009.
- **Notes**: Verify GitHub URL is correct

---

## Test Strategy

**Manual testing**:
1. Start Astro dev server: `cd website && npm run dev`
2. Navigate to http://localhost:4321/quick-start
3. Verify page renders correctly with all 4 steps
4. Test all code blocks have copy buttons that work
5. Click all links in Step 4 - verify they navigate correctly
6. Test on mobile (375px width) - verify responsive layout
7. Test with keyboard navigation (Tab through elements)
8. Time yourself following the guide on a clean system - should complete in < 5 minutes

---

## Risks & Mitigations

**Risk: Automated install script doesn't exist**
- Mitigation: Use "cargo install caro" as primary method; note script as "Coming Soon" if needed

**Risk: Backend confusion (which LLM is being used)**
- Mitigation: Clearly state default backend in Step 3 or link to setup page for configuration

**Risk: Example command fails on some systems**
- Mitigation: Use universally safe command like "list files" - works on all Unix systems

---

## Definition of Done Checklist

- [ ] T006: quick-start.astro page structure created with Layout
- [ ] T007: Step 1 (Install) written with install command
- [ ] T008: Step 2 (Verify) written with version check
- [ ] T009: Step 3 (Generate First Command) written with example
- [ ] T010: Step 4 (Next Steps) written with links to other docs
- [ ] Page accessible at /quick-start URL
- [ ] All code blocks use CodeBlock component
- [ ] Page is mobile-responsive
- [ ] Page reading time is < 3 minutes
- [ ] `tasks.md` checkboxes for T006-T010 marked complete

---

## Review Guidance

**Acceptance checkpoints**:
1. Can a new user complete guide in < 5 minutes?
2. Are all commands copy-pasteable and correct?
3. Is page layout clean and sequential (numbered steps 1-4)?
4. Do links in Next Steps navigate correctly?
5. Is page responsive on mobile devices?

---

## Activity Log

- 2025-12-31T00:00:00Z – system – lane=planned – Prompt created.
- 2025-12-31T09:28:30Z – claude – shell_pid=8158 – lane=doing – Starting Quick Start page implementation
