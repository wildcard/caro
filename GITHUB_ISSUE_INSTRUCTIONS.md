# GitHub Issue Creation Instructions

## How to Create the Issue

Since the `gh` CLI is not available, please create the issue manually:

1. **Go to**: https://github.com/wildcard/caro/issues/new

2. **Title**:
   ```
   Feature: Add offline speech-to-text voice input support
   ```

3. **Body**:
   Copy the entire contents of `speech-to-text-integration-plan.md` (located in this directory)

4. **Labels**:
   - `enhancement`
   - `feature`
   - `voice-input`
   - `accessibility`

## Quick Summary for Issue

This feature request proposes adding offline speech-to-text capabilities to cmdai using whisper.cpp with Rust bindings (whisper-rs).

**Key Points**:
- Allows users to speak commands instead of typing them
- 100% offline and privacy-focused (aligns with cmdai's philosophy)
- Cross-platform support (macOS, Linux, Windows)
- 5-week implementation plan with 5 phases
- Uses whisper.cpp + whisper-rs (most mature solution)
- Optional Voice Activity Detection for better UX
- Minimal binary size increase (<10MB target)

**Recommended Workflow**: Use Spec-Kitty workflow for this medium-sized feature (~2 weeks actual work).

## Alternative: Use gh CLI

If you prefer to use the command line, install gh CLI first:

```bash
# macOS
brew install gh

# Linux
# See: https://github.com/cli/cli/blob/trunk/docs/install_linux.md

# Then authenticate
gh auth login

# Create the issue
gh issue create \
  --repo wildcard/caro \
  --title "Feature: Add offline speech-to-text voice input support" \
  --body-file speech-to-text-integration-plan.md \
  --label "enhancement,feature,voice-input,accessibility"
```

## Files Created

1. **speech-to-text-integration-plan.md** - Comprehensive integration plan (use as issue body)
2. **GITHUB_ISSUE_INSTRUCTIONS.md** - This file (instructions for creating the issue)
