# Developer Quickstart: Installation and Setup Documentation

**For**: Developers implementing this feature
**Duration**: ~2-4 hours for MVP (Quick Start page only), ~8-12 hours for complete feature
**Prerequisites**: Familiarity with Astro, TypeScript/JavaScript, HTML/CSS

## What You're Building

Three documentation pages for Caro installation and setup:
1. **Quick Start** - 5-minute getting started guide (Priority 1)
2. **Installation** - Comprehensive installation methods (Priority 2)
3. **Setup** - Post-install configuration options (Priority 3)

## Quick Setup

1. **Navigate to feature worktree**:
   ```bash
   cd .worktrees/008-installation-and-setup
   ```

2. **Verify you're on the correct branch**:
   ```bash
   git branch --show-current
   # Should output: 008-installation-and-setup
   ```

3. **Start Astro dev server** (from worktree root):
   ```bash
   cd website
   npm install  # if dependencies not installed
   npm run dev
   ```

4. **Open browser to** http://localhost:4321

## File Locations

**New pages you'll create**:
- `website/src/pages/quick-start.astro`
- `website/src/pages/installation.astro`
- `website/src/pages/setup.astro`

**New components you'll create**:
- `website/src/components/docs/CodeBlock.astro`
- `website/src/components/docs/PlatformTabs.astro`
- `website/src/components/docs/InstallMethod.astro`
- `website/src/components/docs/ComingSoonBadge.astro`

**Files you'll modify**:
- `website/src/components/Navigation.astro` - Add Docs dropdown
- `website/src/pages/index.astro` - Add links to docs (optional)

**Reference files** (read, don't modify):
- `README.md` - Extract installation content from here
- `website/src/components/Download.astro` - Reference for install script

## Implementation Order

### Phase 1: Foundation (1-2 hours)

1. **Create component directory**:
   ```bash
   mkdir -p website/src/components/docs
   ```

2. **Build CodeBlock component** with copy button:
   - Use existing website styling patterns
   - Add clipboard.js or navigator.clipboard API
   - Test with various code snippets

3. **Build remaining components**:
   - PlatformTabs.astro (tabbed interface)
   - InstallMethod.astro (installation method card)
   - ComingSoonBadge.astro (simple badge)

### Phase 2: Quick Start Page (2-3 hours)

**Why start here**: Smallest scope, highest value, can be shipped independently.

1. **Create `website/src/pages/quick-start.astro`**:
   ```astro
   ---
   import Layout from '../layouts/Layout.astro';
   import Navigation from '../components/Navigation.astro';
   import Footer from '../components/Footer.astro';
   import CodeBlock from '../components/docs/CodeBlock.astro';
   ---
   <Layout title="Quick Start | Caro">
     <Navigation />
     <!-- Your content here -->
     <Footer />
   </Layout>
   ```

2. **Structure** (4 sections):
   - **Step 1**: Install (automated script or cargo install)
   - **Step 2**: Verify (run `caro --version`)
   - **Step 3**: Generate First Command (example: `caro "list all files"`)
   - **Step 4**: Next Steps (links to installation.astro and setup.astro)

3. **Content guidelines**:
   - Keep it simple (no explanations, just commands)
   - Use CodeBlock component for all commands
   - Include expected output for verification
   - Total reading time: < 3 minutes

4. **Test locally**: Visit http://localhost:4321/quick-start

### Phase 3: Installation Page (3-4 hours)

1. **Create `website/src/pages/installation.astro`**

2. **Structure**:
   - **Hero section**: "Install Caro" with brief intro
   - **Automated Script section** (featured at top):
     - Big code block with script
     - "What this does" explanation
     - Platform requirements
   - **Manual Methods sections**:
     - Cargo Install (with InstallMethod component)
     - Pre-built Binaries (with platform-specific downloads)
     - Build from Source (detailed steps)
     - Package Managers (Homebrew, apt, AUR - all "Coming Soon")
   - **Troubleshooting section**: Common issues
   - **Uninstall section**: How to remove Caro

3. **Content sources**:
   - Extract from `README.md` installation section
   - Check `website/src/components/Download.astro` for script reference
   - Verify commands by running them locally

4. **Test**: Try each installation method if possible

### Phase 4: Setup Page (2-3 hours)

1. **Create `website/src/pages/setup.astro`**

2. **Structure**:
   - **Hero section**: "Configure Caro"
   - **Shell Completions** (use PlatformTabs for bash/zsh/fish)
   - **Shell Aliases** (examples: `c` → `caro`, custom workflows)
   - **Backend Configuration** (MLX, vLLM, Ollama selection)
   - **Environment Variables** (if Caro uses any)
   - **Tool Integrations** (mise, direnv - "Coming Soon")

3. **Shell completion research**:
   - Check if `src/main.rs` has clap completions feature
   - If yes: Document how to generate completions
   - If no: Mark as "Coming Soon" with GitHub issue link

4. **Test**: Verify completion examples work in your shell

### Phase 5: Navigation Integration (1 hour)

1. **Update `website/src/components/Navigation.astro`**:
   - Add "Docs" dropdown between "Resources" and "Support"
   - Follow pattern from lines 60-97 (Resources dropdown)
   - Add three items: Quick Start (🚀), Installation (📦), Setup (⚙️)

2. **Mobile navigation**:
   - Add "Documentation" section to drawer (lines 226-247)
   - Include same three links

3. **Test**:
   - Desktop: Hover over "Docs" → dropdown appears
   - Mobile: Open drawer → see Documentation section

### Phase 6: Polish & Integration (1-2 hours)

1. **Homepage links** (optional):
   - Update Hero CTA to link to `/quick-start`
   - Update Download section to mention installation page

2. **Verify all links work**:
   ```bash
   # Use link checker or manually test
   curl http://localhost:4321/quick-start
   curl http://localhost:4321/installation
   curl http://localhost:4321/setup
   ```

3. **Responsive testing**:
   - Resize browser window
   - Test on mobile device
   - Verify code blocks don't overflow

4. **Accessibility check**:
   - Use browser dev tools accessibility panel
   - Verify keyboard navigation works
   - Check heading hierarchy (h1 → h2 → h3)

## Testing Checklist

- [ ] Quick Start page loads and displays correctly
- [ ] Installation page loads and displays correctly
- [ ] Setup page loads and displays correctly
- [ ] Docs dropdown appears in navigation (desktop)
- [ ] Docs section appears in mobile drawer
- [ ] All code blocks have copy buttons that work
- [ ] Copy buttons actually copy to clipboard
- [ ] Platform tabs work (if implemented)
- [ ] "Coming Soon" badges display correctly
- [ ] All internal links work
- [ ] Page is responsive (mobile, tablet, desktop)
- [ ] Accessibility score is 100 (Lighthouse)
- [ ] No JavaScript errors in console
- [ ] Dark mode works (if site has dark mode)

## Common Issues & Solutions

**Issue**: Astro dev server won't start
**Solution**: Run `npm install` in `website/` directory

**Issue**: Navigation dropdown doesn't work
**Solution**: Check JavaScript in Navigation.astro is loading; verify event listeners

**Issue**: Code blocks don't copy
**Solution**: Check clipboard API permissions; test in different browsers

**Issue**: Links return 404
**Solution**: Ensure file names match exactly (quick-start.astro → /quick-start)

**Issue**: Styles look different from rest of site
**Solution**: Use existing Layout.astro; check CSS class names match existing patterns

## Success Criteria

You're done when:
1. All three pages render correctly locally
2. Navigation dropdown works (desktop + mobile)
3. All code blocks are copyable
4. Responsive design works on all screen sizes
5. All links navigate correctly
6. Tests pass (see Testing Checklist above)

## Next Steps After Implementation

1. Run `/spec-kitty.review` for code review
2. Run `/spec-kitty.accept` for acceptance validation
3. Run `/spec-kitty.merge` to integrate into main branch
4. Deploy to staging environment
5. Verify in production-like environment
6. Get user feedback

## Need Help?

- **Astro docs**: https://docs.astro.build
- **Existing page examples**: Check `website/src/pages/*.astro`
- **Component patterns**: Check `website/src/components/*.astro`
- **Spec details**: See `spec.md` in this directory
- **Design decisions**: See `research.md` in this directory
