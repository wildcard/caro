---
work_package_id: "WP04"
subtasks:
  - "T020"
  - "T021"
  - "T022"
  - "T023"
  - "T024"
  - "T025"
  - "T026"
title: "Setup & Configuration Page - Post-Install"
phase: "Phase 2 - User Story 3 (P3)"
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

# Work Package Prompt: WP04 – Setup & Configuration Page - Post-Install

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

Create **Setup & Configuration Page** documenting post-install ergonomics to enhance developer experience.

**Success**: Users can successfully set up shell completions, configure aliases, select backend, and integrate with developer tools. Documentation is comprehensive for bash, zsh, and fish shells.

**Acceptance scenarios** (from spec.md, User Story 3):
1. User can set up shell completions for their preferred shell
2. User can configure useful aliases for common workflows
3. User understands how to select and configure backends
4. User knows which environment variables Caro uses

---

## Context & Constraints

**Reference documents**:
- `kitty-specs/008-installation-and-setup/spec.md` (User Story 3, lines 42-57)
- `kitty-specs/008-installation-and-setup/plan.md` (Page contract for setup.astro, lines 185-190)
- `kitty-specs/008-installation-and-setup/quickstart.md` (Phase 4 implementation guide, lines 134-152)

**Design constraints**:
- Use PlatformTabs component for shell-specific examples
- Mark unimplemented features with ComingSoonBadge
- Provide copy-pasteable configuration examples
- Include explanations of what each configuration does

**Content priority** (from plan.md):
1. Shell Completions (bash, zsh, fish) with PlatformTabs
2. Shell Aliases (example workflows and custom shortcuts)
3. Backend Configuration (MLX, vLLM, Ollama selection)
4. Environment Variables (if Caro uses any)
5. Tool Integrations (mise, direnv - likely "Coming Soon")

**Research requirement**: Check `src/main.rs` for shell completion capability before documenting.

---

## Subtasks & Detailed Guidance

### Subtask T020 – Create setup.astro page structure
- **Purpose**: Set up page foundation using existing Layout.astro pattern.
- **Steps**:
  1. Create `website/src/pages/setup.astro`
  2. Import components:
     ```astro
     import Layout from '../layouts/Layout.astro';
     import Navigation from '../components/Navigation.astro';
     import Footer from '../components/Footer.astro';
     import CodeBlock from '../components/docs/CodeBlock.astro';
     import PlatformTabs from '../components/docs/PlatformTabs.astro';
     import ComingSoonBadge from '../components/docs/ComingSoonBadge.astro';
     ```
  3. Add Layout wrapper with title and meta description:
     ```astro
     <Layout
       title="Setup & Configuration | Caro"
       description="Configure Caro for optimal developer experience with shell completions, aliases, backend selection, and tool integrations."
     >
       <Navigation />
       <!-- Content sections go here (T021-T026) -->
       <Footer />
     </Layout>
     ```
  4. Add hero section with title "Setup & Configuration" and subtitle
  5. Create container div for content (max-width: 1000px, centered)
- **Files**: `website/src/pages/setup.astro` (create new)
- **Parallel?**: Must complete before T021-T026.

### Subtask T021 – Research shell completion capability
- **Purpose**: Determine if Caro CLI supports shell completion generation before documenting.
- **Steps**:
  1. Read `src/main.rs` to check for clap completion features
  2. Look for `clap::CommandFactory` or `generate_completions` usage
  3. Test if `caro --generate-completion bash` or similar works
  4. Check if completion files exist in repository
  5. Document findings:
     - **Option A**: Completions exist → Document how to generate and install
     - **Option B**: Completions don't exist → Mark as "Coming Soon"
- **Files**: `src/main.rs` (read only)
- **Parallel?**: Must complete before T022 (determines what to write).
- **Notes**: This is research only - no writing yet. Findings inform T022.

### Subtask T022 – Write Shell Completions section
- **Purpose**: Document shell completion setup for bash, zsh, and fish.
- **Steps**:
  1. Add section heading: "Shell Completions"
  2. **Option A** (if completions exist, based on T021 findings):
     ```astro
     <h2>Shell Completions</h2>
     <p>Enable tab completion for Caro commands in your shell:</p>
     <PlatformTabs platforms={[
       {
         name: "bash",
         icon: "🐚",
         content: `
           # Generate completion file
           caro --generate-completion bash > ~/.caro-completion.bash

           # Add to ~/.bashrc
           echo 'source ~/.caro-completion.bash' >> ~/.bashrc

           # Reload shell
           source ~/.bashrc
         `
       },
       {
         name: "zsh",
         icon: "⚡",
         content: `
           # Generate completion file
           mkdir -p ~/.zsh/completions
           caro --generate-completion zsh > ~/.zsh/completions/_caro

           # Add to ~/.zshrc if not present
           fpath=(~/.zsh/completions $fpath)
           autoload -U compinit && compinit

           # Reload shell
           source ~/.zshrc
         `
       },
       {
         name: "fish",
         icon: "🐠",
         content: `
           # Generate completion file
           caro --generate-completion fish > ~/.config/fish/completions/caro.fish

           # Reload shell
           source ~/.config/fish/config.fish
         `
       }
     ]} />
     ```
  3. **Option B** (if completions don't exist):
     ```astro
     <h2>Shell Completions <ComingSoonBadge feature="Shell Completions" /></h2>
     <p>Tab completion for bash, zsh, and fish is planned.</p>
     ```
- **Files**: `website/src/pages/setup.astro` (modify)
- **Parallel?**: Depends on T021 research. Can draft in parallel with T023-T026.

### Subtask T023 – Write Shell Aliases section
- **Purpose**: Provide example aliases for common workflows.
- **Steps**:
  1. Add section heading: "Shell Aliases"
  2. Provide useful alias examples:
     ```astro
     <h2>Shell Aliases</h2>
     <p>Speed up your workflow with these aliases:</p>

     <h3>Quick command generation</h3>
     <CodeBlock
       code='alias c="caro"'
       language="bash"
     />
     <p>Usage: <code>c "list all files"</code></p>

     <h3>Auto-execute safe commands</h3>
     <CodeBlock
       code='alias cx="caro --execute"'
       language="bash"
     />
     <p>Usage: <code>cx "show current directory"</code></p>

     <h3>Common workflows</h3>
     <CodeBlock
       code={`# Find and preview files
alias cf='caro "find files matching" --preview'

# Git shortcuts
alias cgit='caro "git command for"'`}
       language="bash"
     />
     ```
  3. Add note about adding aliases to shell config (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
  4. Provide PlatformTabs if alias syntax differs significantly between shells
- **Files**: `website/src/pages/setup.astro` (modify)
- **Parallel?**: Can draft in parallel with T022, T024-T026.
- **Notes**: Verify that --execute and other flags exist before documenting

### Subtask T024 – Write Backend Configuration section
- **Purpose**: Explain how to select and configure inference backends.
- **Steps**:
  1. Add section heading: "Backend Configuration"
  2. Explain backend selection:
     ```astro
     <h2>Backend Configuration</h2>
     <p>Caro supports multiple inference backends. Choose based on your platform and preferences:</p>

     <h3>Available Backends</h3>
     <ul>
       <li><strong>MLX</strong> - Apple Silicon optimized (M1/M2/M3 Macs)</li>
       <li><strong>vLLM</strong> - High-performance inference server</li>
       <li><strong>Ollama</strong> - Local model management</li>
     </ul>

     <h3>Selecting a Backend</h3>
     <CodeBlock
       code="caro --backend mlx \"your prompt here\""
       language="bash"
     />

     <h3>Default Backend Configuration</h3>
     <p>Set your preferred backend in config file:</p>
     <CodeBlock
       code={`# ~/.config/caro/config.toml
[backend]
default = "mlx"  # or "vllm", "ollama"

[backend.mlx]
model = "mlx-community/Llama-3.2-1B-Instruct-4bit"

[backend.vllm]
endpoint = "http://localhost:8000"

[backend.ollama]
endpoint = "http://localhost:11434"`}
       language="toml"
     />
     ```
  3. Verify config file format by checking codebase for actual config loading
- **Files**: `website/src/pages/setup.astro` (modify)
- **Parallel?**: Can draft in parallel with T022-T023, T025-T026.
- **Notes**: Check codebase for actual config format (TOML, YAML, JSON?)

### Subtask T025 – Write Environment Variables section
- **Purpose**: Document environment variables Caro uses.
- **Steps**:
  1. Research codebase for environment variable usage (grep for `env::var`, `std::env`)
  2. Add section heading: "Environment Variables"
  3. **Option A** (if env vars exist):
     ```astro
     <h2>Environment Variables</h2>
     <p>Caro respects these environment variables:</p>

     <CodeBlock
       code={`# Backend selection
export CARO_BACKEND="mlx"

# Model cache directory
export CARO_CACHE_DIR="~/.cache/caro"

# Log level (debug, info, warn, error)
export RUST_LOG="caro=info"`}
       language="bash"
     />

     <p>Add to your shell config (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)</p>
     ```
  4. **Option B** (if no env vars):
     ```astro
     <h2>Environment Variables</h2>
     <p>Caro currently uses CLI flags for configuration. Environment variable support is planned.</p>
     ```
- **Files**: `website/src/pages/setup.astro` (modify)
- **Parallel?**: Can draft in parallel with T022-T024, T026.
- **Notes**: This requires codebase research - check src/ for actual env var usage

### Subtask T026 – Write Tool Integrations section
- **Purpose**: Document integrations with developer tools (mise, direnv).
- **Steps**:
  1. Add section heading: "Tool Integrations"
  2. Document planned integrations:
     ```astro
     <h2>Tool Integrations</h2>

     <h3>mise <ComingSoonBadge feature="mise integration" /></h3>
     <p>Integration with mise for project-specific Caro configuration is planned.</p>

     <h3>direnv <ComingSoonBadge feature="direnv integration" /></h3>
     <p>Integration with direnv for automatic environment setup is planned.</p>

     <h3>Shell Integration</h3>
     <p>Current integrations:</p>
     <ul>
       <li>Works with any POSIX-compliant shell</li>
       <li>Compatible with shell history (commands are saved to history)</li>
       <li>Respects shell aliases and environment</li>
     </ul>
     ```
  3. Verify these integrations don't already exist before marking "Coming Soon"
- **Files**: `website/src/pages/setup.astro` (modify)
- **Parallel?**: Can draft in parallel with T022-T025.
- **Notes**: These are likely not implemented yet, but verify first

---

## Test Strategy

**Manual testing**:
1. Start Astro dev server: `cd website && npm run dev`
2. Navigate to http://localhost:4321/setup
3. Verify page renders correctly with all sections
4. Test all code blocks have copy buttons that work
5. Test PlatformTabs component switches between shells
6. Verify PlatformTabs persists selection to localStorage
7. Click all links - verify they navigate correctly
8. Test ComingSoonBadge components show correctly
9. Test on mobile (375px width) - verify responsive layout
10. Test with keyboard navigation (Tab through elements)
11. Verify configuration examples are accurate (try them if possible)

---

## Risks & Mitigations

**Risk: Shell completion not implemented**
- Mitigation: Research T021 determines this; mark as "Coming Soon" if not available

**Risk: Config file format uncertain**
- Mitigation: Check codebase for actual config loading code; use most common format (TOML) if unclear

**Risk: Environment variables not documented**
- Mitigation: Research codebase for env var usage; clearly state if none exist

**Risk: Tool integrations don't exist**
- Mitigation: Verify before marking "Coming Soon"; document actual integrations if any exist

---

## Definition of Done Checklist

- [ ] T020: setup.astro page structure created with Layout
- [ ] T021: Shell completion capability researched in src/main.rs
- [ ] T022: Shell Completions section written (or marked Coming Soon)
- [ ] T023: Shell Aliases section written with useful examples
- [ ] T024: Backend Configuration section written with config examples
- [ ] T025: Environment Variables section written (or marked as not used)
- [ ] T026: Tool Integrations section written with ComingSoonBadge
- [ ] Page accessible at /setup URL
- [ ] All code blocks use CodeBlock component
- [ ] Shell-specific examples use PlatformTabs component
- [ ] Unimplemented features show ComingSoonBadge
- [ ] Page is mobile-responsive
- [ ] Configuration examples are accurate
- [ ] `tasks.md` checkboxes for T020-T026 marked complete

---

## Review Guidance

**Acceptance checkpoints**:
1. Can users successfully set up shell completions (if available)?
2. Are alias examples useful and copy-pasteable?
3. Is backend configuration clearly explained?
4. Are environment variables documented accurately?
5. Is page responsive on mobile devices?
6. Do PlatformTabs switch correctly and persist selection?

---

## Activity Log

- 2025-12-31T00:00:00Z – system – lane=planned – Prompt created.
- 2025-12-31T09:36:28Z – claude – shell_pid=40231 – lane=doing – Starting Setup & Configuration page implementation
