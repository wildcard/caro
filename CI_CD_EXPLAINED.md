# GitHub Actions Visual Testing - Explained Simply

This guide explains how the automated visual testing pipeline works, what it does, and how to use it. No prior CI/CD experience needed!

---

## Table of Contents

1. [What is GitHub Actions?](#what-is-github-actions)
2. [What Does Our Workflow Do?](#what-does-our-workflow-do)
3. [How It Works (Step-by-Step)](#how-it-works-step-by-step)
4. [Reading the Build Logs](#reading-the-build-logs)
5. [Understanding the PR Comments](#understanding-the-pr-comments)
6. [Troubleshooting](#troubleshooting)
7. [For Advanced Users](#for-advanced-users)

---

## What is GitHub Actions?

Think of GitHub Actions as a **robot that runs commands for you** whenever something happens in your repository.

### Real-World Analogy

Imagine you have a robot assistant in your workshop:
- 📦 **You drop off some wood** (push code to GitHub)
- 🤖 **Robot springs into action** (GitHub Actions triggers)
- 🔨 **Robot builds your furniture** (compiles your code)
- ✅ **Robot checks quality** (runs tests)
- 📸 **Robot takes photos** (creates visual snapshots)
- 📝 **Robot writes you a report** (posts PR comment)

That's exactly what GitHub Actions does, but with code instead of furniture!

### When Does It Run?

Our workflow runs automatically when:
1. You push changes to a branch named `claude/terminal-ui-storybook-*`
2. You create/update a pull request that modifies TUI component files

You don't need to do anything - it just happens!

---

## What Does Our Workflow Do?

Our workflow has **3 jobs** that run in parallel:

```
┌─────────────────────────────────────────────────────┐
│           GitHub Actions Workflow                   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Job 1: Component Showcase                         │
│  ├─ Build the tui-showcase binary                  │
│  ├─ Run linting (clippy + rustfmt)                 │
│  ├─ Display all components in build log            │
│  └─ Upload snapshots as artifacts                  │
│                                                     │
│  Job 2: Animation Recording                        │
│  ├─ Install VHS (terminal recorder)                │
│  ├─ Record animated demos of components            │
│  └─ Upload GIFs as artifacts                       │
│                                                     │
│  Job 3: PR Comment                                 │
│  ├─ Wait for other jobs to finish                  │
│  ├─ Generate component summary                     │
│  └─ Post comment on the pull request               │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Job 1: Component Showcase

**What it does**: Builds your components and displays them in the build logs.

**Output**: You can see exactly what each component looks like, right in the GitHub Actions logs!

**Example log output**:
```
┌─────────────────────────────────────────────┐
│ Component: Command Preview                  │
│ Category: Display                           │
│ Stories: 3                                  │
│ Description: Displays shell commands        │
│              with syntax highlighting       │
└─────────────────────────────────────────────┘
```

### Job 2: Animation Recording

**What it does**: Uses a tool called VHS to record animated GIFs of your components in action.

**Output**: Downloadable GIF files showing your components with animations!

**Why it's cool**: You can share these GIFs in PRs, documentation, or social media!

### Job 3: PR Comment

**What it does**: Posts a helpful comment on your pull request with:
- List of all 14 components
- How many stories each component has
- Testing instructions
- Links to documentation

**Output**: An automatic comment that helps reviewers understand your changes.

---

## How It Works (Step-by-Step)

Let's follow what happens when you push code:

### Step 1: You Push Code

```bash
git add src/tui/components/my_new_component.rs
git commit -m "Add awesome new component"
git push
```

### Step 2: GitHub Notices

GitHub sees your push and checks if it matches the workflow triggers:
- ✅ Branch name starts with `claude/terminal-ui-storybook-`? YES
- ✅ Modified files in `src/tui/**`? YES

**Trigger activated!** 🚀

### Step 3: Workflow Starts

GitHub Actions creates a fresh virtual machine (Ubuntu Linux) and starts running the workflow.

**What's happening behind the scenes**:
```yaml
runs-on: ubuntu-latest  # Fresh Ubuntu machine
```

### Step 4: Check Out Code

The workflow downloads your code:
```bash
# Equivalent to:
git clone <your-repo>
git checkout <your-branch>
```

### Step 5: Set Up Rust

Installs Rust toolchain:
```bash
# Equivalent to:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Step 6: Build the Showcase

Compiles your code:
```bash
cargo build --release --bin tui-showcase
```

This creates the `tui-showcase` binary that can run all your components.

### Step 7: Run Quality Checks

Checks code quality:
```bash
cargo clippy -- -D warnings    # Linting
cargo fmt -- --check           # Formatting
```

If these fail, the workflow fails and tells you what needs fixing!

### Step 8: Display Components

Runs the built binary and captures metadata:
```bash
./target/release/tui-showcase --list-components
```

This generates the formatted output you see in the logs.

### Step 9: Record Animations (Job 2)

Parallel job installs VHS:
```bash
# Install VHS from Charm
# VHS is like a movie director for your terminal
```

Then creates tape scripts and records GIFs:
```bash
vhs showcase_demo.tape  # Creates showcase_demo.gif
```

### Step 10: Post PR Comment (Job 3)

After jobs 1 and 2 finish, this job posts a comment with all the info.

### Step 11: Upload Artifacts

Saves build outputs for 30 days:
- Component snapshots
- Animated GIFs
- Build logs

You can download these from the Actions tab!

---

## Reading the Build Logs

### How to Find Them

1. Go to your repository on GitHub
2. Click the **"Actions"** tab at the top
3. Find your workflow run (named "TUI Component Visual Testing")
4. Click on it

You'll see something like this:

```
┌──────────────────────────────────────────┐
│ TUI Component Visual Testing             │
│ ✓ component-showcase (2m 34s)           │
│ ✓ animation-recording (3m 12s)          │
│ ✓ pr-comment (15s)                      │
└──────────────────────────────────────────┘
```

### Understanding Status Icons

- ✓ **Green checkmark** = Job succeeded
- ✗ **Red X** = Job failed
- ⏳ **Yellow dot** = Job is running
- ○ **Gray circle** = Job hasn't started yet

### Viewing Detailed Logs

Click on a job name (e.g., "component-showcase") to see detailed logs.

**What you'll see**:

```
┌─ Set up job
│  ✓ Virtual Environment: ubuntu-latest
│  ✓ Runner: ubuntu-22.04
│
┌─ Checkout code
│  ✓ Fetching repository
│  ✓ Checking out claude/terminal-ui-storybook-xxx
│
┌─ Setup Rust
│  ✓ Installing rustc 1.75.0
│  ✓ Installing cargo
│
┌─ Build tui-showcase
│  Compiling cmdai v0.1.0
│  ...
│  ✓ Finished release [optimized] target(s) in 2m 15s
│
┌─ Display Component Metadata
│  ┌─────────────────────────────────────┐
│  │ Component: Simple Text              │
│  │ Category: Display                   │
│  │ Stories: 3                          │
│  └─────────────────────────────────────┘
│  ...
```

### Finding Errors

If a job fails, scroll down to find the error:

```
error[E0425]: cannot find value `component` in this scope
  --> src/tui/components/my_component.rs:42:9
   |
42 |         component.render(frame);
   |         ^^^^^^^^^ not found in this scope
```

**Pro tip**: Errors are highlighted in red and show:
- The file path
- The line number
- What went wrong

---

## Understanding the PR Comments

When you create a pull request, the workflow posts a comment like this:

```markdown
## 🎨 TUI Component Visual Testing Results

### Component Summary
Total Components: **14**
Total Stories: **73+**
Categories: **5** (Display, Input, Feedback, Workflow, Help)

### Components in This Showcase

**Display Components (6)**
- ✓ Simple Text (3 stories)
- ✓ Command Preview (3 stories)
- ✓ Safety Indicator (4 stories)
...

### How to Test Locally
```bash
cargo run --bin tui-showcase
```

### Visual Testing Artifacts
- 📸 Component Snapshots: Download from Actions → Artifacts
- 🎬 Animated Demos: Download from Actions → Artifacts

### Documentation
- [Getting Started Guide](GETTING_STARTED.md)
- [Architecture Guide](ARCHITECTURE_GUIDE.md)
...
```

### What Each Section Means

**Component Summary**
- Quick stats about the showcase

**Components in This Showcase**
- Complete list of all components organized by category
- Each component shows how many stories it has

**How to Test Locally**
- Instructions for reviewers to run the showcase on their machine

**Visual Testing Artifacts**
- Links to download the generated snapshots and GIFs

**Documentation**
- Helpful links to learn more

---

## Troubleshooting

### Problem: Workflow Doesn't Run

**Symptom**: You pushed code but don't see the workflow in Actions tab.

**Possible Causes**:
1. Your branch name doesn't match `claude/terminal-ui-storybook-*`
2. You didn't modify any files in `src/tui/**`
3. The workflow file has a syntax error

**How to Check**:
```bash
# Check your branch name
git branch --show-current

# Should output something like:
# claude/terminal-ui-storybook-abc123
```

**Fix**:
- Rename your branch if needed
- Make sure you modified TUI component files
- Validate YAML syntax at https://www.yamllint.com/

### Problem: Build Fails with "error: could not compile"

**Symptom**: Red X on the workflow, logs show compilation errors.

**Possible Causes**:
- Syntax error in your Rust code
- Missing imports
- Type mismatch

**How to Fix**:
1. Look at the error message in the logs
2. Find the file and line number
3. Fix the issue locally
4. Test with `cargo build --release --bin tui-showcase`
5. Push again

**Example**:
```
error[E0425]: cannot find value `frame` in this scope
  --> src/tui/components/my_component.rs:25:9
```

This means you used `frame` but it's not defined. Check your function signature!

### Problem: Clippy Warnings Fail the Build

**Symptom**: Build succeeds but clippy step fails.

**Example**:
```
warning: unused variable: `metadata`
  --> src/tui/components/my_component.rs:15:9
```

**How to Fix**:
1. Run clippy locally: `cargo clippy -- -D warnings`
2. Fix the warnings (remove unused variables, etc.)
3. Push again

**Quick Fix for Unused Variables**:
```rust
// Instead of:
let metadata = component.metadata();

// Use:
let _metadata = component.metadata();  // Underscore prefix
```

### Problem: Format Check Fails

**Symptom**: rustfmt check fails.

**How to Fix**:
```bash
# Auto-format your code
cargo fmt

# Commit and push
git add .
git commit -m "Fix formatting"
git push
```

### Problem: Can't Find Artifacts

**Symptom**: Want to download snapshots/GIFs but can't find them.

**How to Find**:
1. Go to Actions tab
2. Click on your workflow run
3. Scroll to bottom of page
4. Look for "Artifacts" section
5. Click download icon

**Note**: Artifacts expire after 30 days!

### Problem: VHS Recording Fails

**Symptom**: animation-recording job fails.

**Possible Causes**:
- VHS installation failed
- Tape script has syntax errors
- Component doesn't render properly in headless mode

**How to Check**:
Look at the job logs for VHS-specific errors.

**Common Fix**:
This is usually temporary. Re-run the job:
1. Go to the failed workflow run
2. Click "Re-run failed jobs" button

---

## For Advanced Users

### Workflow File Location

The workflow is defined in:
```
.github/workflows/tui-visual-testing.yml
```

### Customizing the Workflow

Want to modify what the workflow does? Edit `tui-visual-testing.yml`.

**Common Customizations**:

**1. Change when it runs**
```yaml
on:
  push:
    branches:
      - 'main'  # Run on main branch
  pull_request:  # Run on all PRs
```

**2. Add more quality checks**
```yaml
- name: Run tests
  run: cargo test --all-features
```

**3. Change Rust version**
```yaml
- uses: dtolnay/rust-toolchain@stable
  with:
    toolchain: 1.75.0  # Specific version
```

**4. Upload more artifacts**
```yaml
- name: Upload test results
  uses: actions/upload-artifact@v4
  with:
    name: test-results
    path: target/test-results/
```

### Caching Strategy

The workflow uses Cargo caching to speed up builds:

```yaml
- uses: Swatinem/rust-cache@v2
```

**What it caches**:
- `~/.cargo/registry` - Downloaded dependencies
- `~/.cargo/git` - Git dependencies
- `target/` - Compiled artifacts

**Result**: Second and subsequent builds are much faster!

### VHS Tape Scripts

VHS uses "tape" files to script terminal recordings:

```tape
# showcase_demo.tape
Output showcase_demo.gif

Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "Dracula"

Type "cargo run --bin tui-showcase"
Enter
Sleep 2s

Type "j"
Sleep 500ms
Type "j"
Sleep 500ms

Enter
Sleep 2s

Type "q"
Sleep 500ms
```

**Customize these** to show different interactions!

### Matrix Builds

Want to test on multiple platforms? Use a matrix:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]

runs-on: ${{ matrix.os }}
```

This creates 9 jobs (3 OS × 3 Rust versions)!

### Secrets and Environment Variables

Need API keys or tokens?

```yaml
env:
  MY_SECRET: ${{ secrets.MY_SECRET }}
```

Add secrets in: Repository Settings → Secrets and Variables → Actions

### Manual Triggers

Want to run the workflow on-demand?

```yaml
on:
  workflow_dispatch:  # Adds "Run workflow" button
    inputs:
      component:
        description: 'Which component to test'
        required: false
```

### Debugging Workflows

Add debug logging:

```yaml
- name: Debug info
  run: |
    echo "Branch: ${{ github.ref }}"
    echo "Event: ${{ github.event_name }}"
    echo "Actor: ${{ github.actor }}"
    ls -la
    env | sort
```

Enable step debugging:
1. Repository Settings → Secrets and Variables → Actions
2. Add variable: `ACTIONS_STEP_DEBUG` = `true`

---

## Best Practices

### 1. Keep Jobs Fast

- Use caching aggressively
- Run tests in parallel
- Don't install unnecessary dependencies

### 2. Fail Fast

- Put quick checks first (linting before building)
- Use `--fail-fast` for cargo commands
- Set reasonable timeouts

```yaml
timeout-minutes: 10  # Kill job after 10 minutes
```

### 3. Informative Output

- Use clear step names
- Print summaries
- Format output nicely

```yaml
- name: Build showcase (this may take 2-3 minutes)
  run: cargo build --release --bin tui-showcase
```

### 4. Handle Failures Gracefully

- Use `continue-on-error` for optional steps
- Add retry logic for flaky operations
- Provide helpful error messages

```yaml
- name: Optional step
  run: some-command
  continue-on-error: true
```

### 5. Security

- Never log secrets
- Use `secrets.*` syntax
- Pin action versions

```yaml
- uses: actions/checkout@v4  # Pinned version
```

---

## Monitoring and Notifications

### GitHub UI

- **Actions tab**: See all workflow runs
- **Checks tab** (on PR): See status of all checks
- **Commit status**: Green checkmark or red X on commits

### Email Notifications

You'll get emails when:
- A workflow you triggered fails
- A workflow in your repository fails on the default branch

**Customize**: Settings → Notifications → Actions

### Status Badges

Add a badge to your README:

```markdown
![CI Status](https://github.com/USER/REPO/actions/workflows/tui-visual-testing.yml/badge.svg)
```

Shows: ![passing](https://img.shields.io/badge/build-passing-brightgreen) or ![failing](https://img.shields.io/badge/build-failing-red)

---

## Cost and Limits

### Free Tier

GitHub Actions is free for public repositories!

**Limits**:
- 2,000 minutes/month for private repos
- 20 concurrent jobs
- 6 hours maximum job runtime

### Our Workflow Usage

Typical run times:
- Component Showcase: ~2-3 minutes
- Animation Recording: ~3-5 minutes
- PR Comment: ~10-20 seconds

**Total**: ~5-8 minutes per run

### Optimization Tips

If you need to reduce usage:
1. Run only on PRs, not every push
2. Use matrix builds sparingly
3. Cache aggressively
4. Combine jobs when possible

---

## FAQ

**Q: Do I need to do anything to trigger the workflow?**
A: Nope! Just push code or create a PR. It runs automatically.

**Q: Can I see the workflow results before merging?**
A: Yes! The workflow runs on every push and PR. Check the status before merging.

**Q: What if I don't want the workflow to run?**
A: Add `[skip ci]` to your commit message:
```bash
git commit -m "Update docs [skip ci]"
```

**Q: Can I run the workflow locally?**
A: Sort of! Use [act](https://github.com/nektos/act) to run GitHub Actions locally:
```bash
brew install act
act push
```

**Q: The workflow is failing but I don't know why.**
A: Look at the logs! Click on the failed job and read the error messages. They usually tell you exactly what's wrong.

**Q: Can I customize the PR comment format?**
A: Yes! Edit the `pr-comment` job in the workflow file.

---

## Conclusion

The GitHub Actions workflow is like having a dedicated QA team that:
- ✅ Builds your code automatically
- ✅ Runs quality checks
- ✅ Creates visual documentation
- ✅ Posts helpful PR comments
- ✅ Never sleeps, never complains!

**You don't need to understand every detail.** The workflow "just works" for most contributors. But if you're curious or want to customize it, this guide has you covered!

---

**Questions?** Check out [FAQ.md](FAQ.md) or ask in GitHub issues!

**Want to extend the workflow?** See the "For Advanced Users" section above!
