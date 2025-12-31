---
work_package_id: "WP03"
subtasks:
  - "T011"
  - "T012"
  - "T013"
  - "T014"
  - "T015"
  - "T016"
  - "T017"
  - "T018"
  - "T019"
title: "Installation Page - Comprehensive Methods"
phase: "Phase 2 - User Story 2 (P2)"
lane: "for_review"
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

# Work Package Prompt: WP03 – Installation Page - Comprehensive Methods

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

Create **Installation Page** documenting all installation methods for power users who want to choose their preferred approach.

**Success**: Power users can find their preferred installation method, understand requirements, and successfully install Caro. Automated script is prominently featured for simplicity-seekers.

**Acceptance scenarios** (from spec.md, User Story 2):
1. User finds preferred installation method quickly
2. User understands prerequisites and system requirements
3. User successfully installs using chosen method
4. User can troubleshoot common issues

---

## Context & Constraints

**Reference documents**:
- `kitty-specs/008-installation-and-setup/spec.md` (User Story 2, lines 25-40)
- `kitty-specs/008-installation-and-setup/plan.md` (Page contract for installation.astro, lines 178-184)
- `kitty-specs/008-installation-and-setup/quickstart.md` (Phase 3 implementation guide, lines 109-133)
- `README.md` (extract installation content from here)

**Design constraints**:
- Automated script must be prominently featured at top
- Each method must use InstallMethod component for consistency
- Unsupported methods must show ComingSoonBadge
- All commands must be copy-pasteable via CodeBlock
- Include verification commands for each method

**Content priority** (from plan.md):
1. Hero section: "Install Caro" with brief intro
2. Automated Script (featured, large code block)
3. Manual methods: Cargo, Binaries, Source, Package Managers
4. Troubleshooting and Uninstall sections

---

## Subtasks & Detailed Guidance

### Subtask T011 – Create installation.astro page structure
- **Purpose**: Set up page foundation using existing Layout.astro pattern.
- **Steps**:
  1. Create `website/src/pages/installation.astro`
  2. Import components:
     ```astro
     import Layout from '../layouts/Layout.astro';
     import Navigation from '../components/Navigation.astro';
     import Footer from '../components/Footer.astro';
     import CodeBlock from '../components/docs/CodeBlock.astro';
     import InstallMethod from '../components/docs/InstallMethod.astro';
     import ComingSoonBadge from '../components/docs/ComingSoonBadge.astro';
     ```
  3. Add Layout wrapper with title and meta description:
     ```astro
     <Layout
       title="Installation | Caro"
       description="Install Caro using automated script, Cargo, pre-built binaries, or build from source. Multiple installation methods for all platforms."
     >
       <Navigation />
       <!-- Content sections go here (T012-T019) -->
       <Footer />
     </Layout>
     ```
  4. Add hero section with title "Install Caro" and subtitle
  5. Create container div for content (max-width: 1200px, centered)
- **Files**: `website/src/pages/installation.astro` (create new)
- **Parallel?**: Must complete before T012-T019.

### Subtask T012 – Extract installation content from README.md
- **Purpose**: Gather accurate installation commands and requirements from existing documentation.
- **Steps**:
  1. Read `README.md` installation section
  2. Note all installation methods mentioned
  3. Copy exact commands (don't modify - ensure accuracy)
  4. Extract system requirements and prerequisites
  5. Check `website/src/components/Download.astro` for automated script reference
  6. Document which methods are actually supported vs. planned
- **Files**: `README.md` (read only), `website/src/components/Download.astro` (read only)
- **Parallel?**: Can do in parallel with T013-T019 drafting.
- **Notes**: This is research only - no writing yet. Compile findings for use in T013-T019.

### Subtask T013 – Write Automated Script section
- **Purpose**: Feature the simplest installation method prominently for new users.
- **Steps**:
  1. Add large hero-style section at top of page
  2. Heading: "Automated Installation (Recommended)"
  3. Show installation script in large CodeBlock:
     ```astro
     <h2>Automated Installation (Recommended)</h2>
     <p>The fastest way to get started:</p>
     <CodeBlock
       code="curl -fsSL https://install.caro.sh | sh"
       language="bash"
     />
     ```
  4. Add "What this does" explanation:
     - Downloads latest binary for your platform
     - Installs to appropriate location
     - Sets up PATH automatically
  5. Add platform requirements section (macOS/Linux)
  6. If automated script doesn't exist, mark as "Coming Soon" and use cargo install as primary method
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T014-T019 after T011.
- **Notes**: Check Download.astro for actual script URL

### Subtask T014 – Write Cargo Install section
- **Purpose**: Document installation via Rust package manager.
- **Steps**:
  1. Add section heading: "Install via Cargo"
  2. Use InstallMethod component:
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
  3. Add note about Rust installation if needed: "Don't have Rust? Install from https://rustup.rs"
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T013, T015-T019.

### Subtask T015 – Write Pre-built Binaries section
- **Purpose**: Provide downloadable binaries for users who don't want to compile.
- **Steps**:
  1. Add section heading: "Pre-built Binaries"
  2. Check if binaries exist in GitHub releases
  3. **Option A** (if binaries exist):
     ```astro
     <h2>Pre-built Binaries</h2>
     <p>Download pre-compiled binaries for your platform:</p>
     <ul>
       <li><a href="https://github.com/wildcard/caro/releases/latest/download/caro-macos-aarch64">macOS (Apple Silicon)</a></li>
       <li><a href="https://github.com/wildcard/caro/releases/latest/download/caro-macos-x86_64">macOS (Intel)</a></li>
       <li><a href="https://github.com/wildcard/caro/releases/latest/download/caro-linux-x86_64">Linux (x86_64)</a></li>
     </ul>
     <CodeBlock
       code="chmod +x caro-*\nsudo mv caro-* /usr/local/bin/caro"
       language="bash"
     />
     ```
  4. **Option B** (if binaries don't exist):
     ```astro
     <h2>Pre-built Binaries <ComingSoonBadge feature="Binaries" /></h2>
     <p>Pre-compiled binaries for macOS and Linux are planned.</p>
     ```
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T013-T014, T016-T019.
- **Notes**: Check GitHub releases to verify binary availability

### Subtask T016 – Write Build from Source section
- **Purpose**: Document manual compilation for advanced users.
- **Steps**:
  1. Add section heading: "Build from Source"
  2. Use InstallMethod component:
     ```astro
     <InstallMethod
       name="Build from Source"
       description="Compile Caro manually with full control over build options"
       difficulty="advanced"
       platforms={["macOS", "Linux"]}
       status="available"
       installCommand="git clone https://github.com/wildcard/caro.git\ncd caro\ncargo build --release\nsudo cp target/release/caro /usr/local/bin/"
       verificationCommand="caro --version"
       prerequisites={["Rust 1.75+", "Git", "Build tools (gcc/clang)"]}
     />
     ```
  3. Add optional build flags section (if applicable):
     - `--features mlx` for MLX backend on Apple Silicon
     - Other feature flags
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T013-T015, T017-T019.

### Subtask T017 – Write Package Managers section
- **Purpose**: Document future package manager integrations.
- **Steps**:
  1. Add section heading: "Package Managers"
  2. For each package manager (Homebrew, apt, AUR), add:
     ```astro
     <h3>Homebrew <ComingSoonBadge feature="Homebrew" issueUrl="https://github.com/wildcard/caro/issues/XXX" /></h3>
     <p>Installation via Homebrew is planned.</p>
     <CodeBlock
       code="# Coming soon\nbrew install caro"
       language="bash"
     />

     <h3>apt (Debian/Ubuntu) <ComingSoonBadge feature="apt" /></h3>
     <p>Installation via apt repository is planned.</p>

     <h3>AUR (Arch Linux) <ComingSoonBadge feature="AUR" /></h3>
     <p>Installation via AUR is planned.</p>
     ```
  3. Verify package managers are not yet available (search GitHub, crates.io, Homebrew)
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T013-T016, T018-T019.
- **Notes**: Check for actual package availability before marking "Coming Soon"

### Subtask T018 – Write Troubleshooting section
- **Purpose**: Help users resolve common installation issues.
- **Steps**:
  1. Add section heading: "Troubleshooting"
  2. Add common issues as collapsible sections:
     ```astro
     <h2>Troubleshooting</h2>
     <details>
       <summary>"command not found: caro"</summary>
       <p>Make sure /usr/local/bin is in your PATH:</p>
       <CodeBlock
         code="echo $PATH | grep /usr/local/bin"
         language="bash"
       />
       <p>If missing, add to your shell config (~/.bashrc, ~/.zshrc):</p>
       <CodeBlock
         code='export PATH="/usr/local/bin:$PATH"'
         language="bash"
       />
     </details>

     <details>
       <summary>"permission denied" errors</summary>
       <p>Use sudo for installation commands or install to user directory:</p>
       <CodeBlock
         code="cargo install --path . --root ~/.local"
         language="bash"
       />
     </details>

     <details>
       <summary>Cargo build fails</summary>
       <p>Ensure Rust toolchain is up to date:</p>
       <CodeBlock
         code="rustup update"
         language="bash"
       />
     </details>
     ```
  3. Add link to GitHub issues for other problems
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T013-T017, T019.

### Subtask T019 – Write Uninstall instructions section
- **Purpose**: Document how to remove Caro from system.
- **Steps**:
  1. Add section heading: "Uninstall"
  2. Provide uninstall instructions for each method:
     ```astro
     <h2>Uninstall</h2>

     <h3>Cargo Install</h3>
     <CodeBlock
       code="cargo uninstall caro"
       language="bash"
     />

     <h3>Manual Binary</h3>
     <CodeBlock
       code="sudo rm /usr/local/bin/caro"
       language="bash"
     />

     <h3>Build from Source</h3>
     <CodeBlock
       code="sudo rm /usr/local/bin/caro\nrm -rf ~/path/to/caro/repo"
       language="bash"
     />

     <p>To remove configuration files (optional):</p>
     <CodeBlock
       code="rm -rf ~/.config/caro"
       language="bash"
     />
     ```
- **Files**: `website/src/pages/installation.astro` (modify)
- **Parallel?**: Can draft in parallel with T013-T018.

---

## Test Strategy

**Manual testing**:
1. Start Astro dev server: `cd website && npm run dev`
2. Navigate to http://localhost:4321/installation
3. Verify page renders correctly with all sections
4. Test all code blocks have copy buttons that work
5. Click all links - verify they navigate correctly
6. Test InstallMethod components display metadata correctly
7. Test ComingSoonBadge components show and link to issues
8. Test collapsible troubleshooting sections expand/collapse
9. Test on mobile (375px width) - verify responsive layout
10. Test with keyboard navigation (Tab through elements)
11. Verify all commands are accurate (try them if possible)

---

## Risks & Mitigations

**Risk: Automated install script doesn't exist**
- Mitigation: Mark as "Coming Soon" and promote cargo install as primary method

**Risk: Binary releases don't exist**
- Mitigation: Mark as "Coming Soon" with GitHub issue link

**Risk: Installation commands are outdated**
- Mitigation: Extract from README.md and verify with actual project state

**Risk: Package manager availability uncertain**
- Mitigation: Verify on Homebrew, apt, AUR before claiming "Coming Soon"

---

## Definition of Done Checklist

- [ ] T011: installation.astro page structure created with Layout
- [ ] T012: Installation content extracted from README.md
- [ ] T013: Automated Script section written (or marked Coming Soon)
- [ ] T014: Cargo Install section written with InstallMethod component
- [ ] T015: Pre-built Binaries section written (or marked Coming Soon)
- [ ] T016: Build from Source section written with InstallMethod component
- [ ] T017: Package Managers section written with ComingSoonBadge
- [ ] T018: Troubleshooting section written with collapsible details
- [ ] T019: Uninstall section written with method-specific instructions
- [ ] Page accessible at /installation URL
- [ ] All code blocks use CodeBlock component
- [ ] All manual methods use InstallMethod component
- [ ] All unsupported methods show ComingSoonBadge
- [ ] Page is mobile-responsive
- [ ] All commands are accurate and tested
- [ ] `tasks.md` checkboxes for T011-T019 marked complete

---

## Review Guidance

**Acceptance checkpoints**:
1. Can power users find their preferred installation method quickly?
2. Are all commands copy-pasteable and accurate?
3. Are system requirements clearly stated for each method?
4. Do ComingSoonBadge components show for unsupported methods?
5. Is page responsive on mobile devices?
6. Are troubleshooting steps helpful and actionable?

---

## Activity Log

- 2025-12-31T00:00:00Z – system – lane=planned – Prompt created.
- 2025-12-31T09:32:54Z – claude – shell_pid=40231 – lane=doing – Starting Installation page implementation
- 2025-12-31T09:35:31Z – claude – shell_pid=40231 – lane=for_review – Completed Installation page (T011-T019)
