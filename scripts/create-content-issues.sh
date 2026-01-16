#!/bin/bash
# Script to create GitHub issues for Caro Content Strategy
# Run with: ./scripts/create-content-issues.sh
# Requires: gh CLI authenticated with appropriate permissions

set -e

REPO="wildcard/caro"
PROJECT_NAME="Caro Content Strategy"

echo "=== Caro Content Strategy Issue Creator ==="
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed."
    echo "Install it with: brew install gh (macOS) or see https://cli.github.com/"
    exit 1
fi

# Check authentication
if ! gh auth status &> /dev/null; then
    echo "Error: Not authenticated with GitHub CLI."
    echo "Run: gh auth login"
    exit 1
fi

echo "Creating labels..."

# Create labels (ignore errors if they already exist)
gh label create "content" --color "1D76DB" --description "Content creation tasks" -R "$REPO" 2>/dev/null || true
gh label create "seo" --color "5319E7" --description "SEO optimization tasks" -R "$REPO" 2>/dev/null || true
gh label create "priority:p0" --color "B60205" --description "Critical priority" -R "$REPO" 2>/dev/null || true
gh label create "priority:p1" --color "D93F0B" --description "High priority" -R "$REPO" 2>/dev/null || true
gh label create "priority:p2" --color "FBCA04" --description "Medium priority" -R "$REPO" 2>/dev/null || true
gh label create "priority:p3" --color "0E8A16" --description "Lower priority" -R "$REPO" 2>/dev/null || true
gh label create "tutorial" --color "C5DEF5" --description "Tutorial content" -R "$REPO" 2>/dev/null || true
gh label create "comparison" --color "BFD4F2" --description "Comparison content" -R "$REPO" 2>/dev/null || true
gh label create "guide" --color "D4C5F9" --description "Comprehensive guide" -R "$REPO" 2>/dev/null || true
gh label create "landing-page" --color "F9D0C4" --description "Marketing landing page" -R "$REPO" 2>/dev/null || true

echo "Labels created/verified."
echo ""
echo "Creating issues..."
echo ""

# ====================
# PRIORITY 0 ISSUES
# ====================

echo "Creating P0 Issues..."

gh issue create -R "$REPO" \
    --title "[Content] Landing page: Natural Language to Shell Commands" \
    --label "content,landing-page,priority:p0,seo" \
    --body "$(cat <<'EOF'
## Description
Create a dedicated landing page optimized for high-intent keywords around natural language to shell command conversion.

## Target Keywords
- natural language to shell command
- convert text to terminal command
- ai write shell commands
- english to bash command

## Content Requirements
- [ ] Hero section with clear value proposition
- [ ] Interactive demo/GIF showing Caro in action
- [ ] Key features section (safety, local AI, cross-platform)
- [ ] Comparison table with competitors
- [ ] Installation quick-start
- [ ] User testimonials/use cases
- [ ] FAQ section with schema markup

## SEO Checklist
- [ ] Meta title: "Natural Language to Shell Commands | Caro CLI"
- [ ] Meta description optimized for CTR
- [ ] H1 contains primary keyword
- [ ] FAQ schema markup implemented
- [ ] Mobile responsive
- [ ] Page speed optimized

## Success Metrics
- Rank top 10 for "natural language to shell command"
- Drive 500+ monthly organic visits

## Reference
See `docs/SEO_KEYWORD_RESEARCH.md` for detailed keyword analysis.
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Comparison: Best AI CLI Tools for Developers (2025/2026)" \
    --label "content,comparison,priority:p0,seo" \
    --body "$(cat <<'EOF'
## Description
Create a comprehensive comparison article covering the AI CLI tool landscape, positioning Caro among competitors.

## Target Keywords
- best ai cli tools 2025
- ai terminal assistant
- ai powered command line
- ai command generator

## Content Outline
1. Introduction - The rise of AI CLI tools
2. What to look for in an AI CLI tool
3. Tool-by-tool breakdown:
   - Caro (featured prominently)
   - Warp
   - ShellGPT
   - AI Shell (BuilderIO)
   - GitHub Copilot CLI
   - Amazon Q CLI
4. Comparison table (features, pricing, local support)
5. Use case recommendations
6. Conclusion and recommendations

## Content Requirements
- [ ] Feature comparison table
- [ ] Pros/cons for each tool
- [ ] Code examples showing usage
- [ ] Installation instructions for each
- [ ] Clear winner recommendations by use case
- [ ] Regular update schedule (quarterly)

## Success Metrics
- Rank top 5 for "best ai cli tools"
- Generate referral traffic from AI assistants

## Reference
See `docs/SEO_KEYWORD_RESEARCH.md` for competitor analysis.
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: Getting Started with Caro - Complete Guide" \
    --label "content,tutorial,priority:p0" \
    --body "$(cat <<'EOF'
## Description
Create the definitive getting started guide for new Caro users, optimized for both search and AI assistant recommendations.

## Target Keywords
- caro cli tutorial
- caro installation guide
- ai shell tool setup

## Content Outline
1. What is Caro?
2. Installation options:
   - Quick install script
   - Pre-built binaries
   - Cargo install
   - Building from source
3. First command walkthrough
4. Understanding safety features
5. Configuration basics
6. Common use cases
7. Troubleshooting
8. Next steps

## Content Requirements
- [ ] Step-by-step screenshots/GIFs
- [ ] Copy-paste code blocks
- [ ] Platform-specific instructions (macOS, Linux, Windows)
- [ ] Troubleshooting section
- [ ] HowTo schema markup

## Reference
See `docs/SEO_KEYWORD_RESEARCH.md` for keyword analysis.
EOF
)"

echo "P0 issues created."
echo ""

# ====================
# PRIORITY 1 ISSUES
# ====================

echo "Creating P1 Issues..."

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: How to Find Files in Linux (Without Memorizing Syntax)" \
    --label "content,tutorial,priority:p1,seo" \
    --body "$(cat <<'EOF'
## Description
Create an educational tutorial on the `find` command, showing how Caro makes it easier.

## Target Keywords
- find command linux
- search files terminal
- find files by name linux
- locate file command line

## Content Outline
1. The challenge of the find command
2. Traditional find syntax explained
3. Common find use cases:
   - Find by name
   - Find by type
   - Find by size
   - Find by date
   - Find by content
4. The Caro alternative - natural language examples
5. Side-by-side comparisons
6. When to use find vs Caro

## Content Requirements
- [ ] Real command examples
- [ ] Caro natural language equivalents
- [ ] Interactive examples or code playground
- [ ] Downloadable cheat sheet
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Comparison: Caro vs ShellGPT vs Warp - Which AI Terminal Tool?" \
    --label "content,comparison,priority:p1,seo" \
    --body "$(cat <<'EOF'
## Description
Direct comparison article targeting users searching for alternatives.

## Target Keywords
- shellgpt alternative
- warp alternative free
- caro vs shellgpt
- ai terminal comparison

## Content Outline
1. Quick comparison table
2. Caro deep dive (local AI, safety, Rust)
3. ShellGPT deep dive (flexibility, Ollama)
4. Warp deep dive (terminal replacement)
5. Feature-by-feature comparison
6. Pricing comparison
7. Privacy comparison
8. Recommendations by user type
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: Safe Shell Commands - Protecting Against rm -rf" \
    --label "content,tutorial,priority:p1,seo" \
    --body "$(cat <<'EOF'
## Description
Educational content about dangerous commands and how Caro's safety features protect users.

## Target Keywords
- dangerous linux commands
- prevent rm rf
- safe shell scripting
- terminal safety

## Content Outline
1. The most dangerous Linux commands
2. Real-world horror stories
3. Traditional protection methods
4. How Caro detects dangerous commands
5. Caro's 52+ safety patterns explained
6. Configuration and safety levels
7. Building safe habits
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Guide: Local LLMs for Terminal - A Privacy-First Approach" \
    --label "content,guide,priority:p1,seo" \
    --body "$(cat <<'EOF'
## Description
Position Caro as the privacy-first choice for developers concerned about cloud AI.

## Target Keywords
- local llm shell
- offline ai terminal
- private ai coding
- no cloud ai tool

## Content Outline
1. Why privacy matters for developers
2. Cloud AI risks (data training, leakage)
3. The local AI revolution
4. Caro's embedded model architecture
5. MLX for Apple Silicon explained
6. CPU fallback for universal support
7. Comparison with cloud alternatives
8. Enterprise use cases
EOF
)"

echo "P1 issues created."
echo ""

# ====================
# PRIORITY 2 ISSUES
# ====================

echo "Creating P2 Issues..."

gh issue create -R "$REPO" \
    --title "[Content] Guide: POSIX Shell Scripting for Cross-Platform Compatibility" \
    --label "content,guide,priority:p2,seo" \
    --body "$(cat <<'EOF'
## Description
Educational content about POSIX compliance, highlighting Caro's platform-aware generation.

## Target Keywords
- posix compliant shell
- cross platform bash
- portable shell scripts
- bsd vs gnu commands

## Content Outline
1. What is POSIX?
2. Common non-POSIX "bashisms"
3. BSD vs GNU differences
4. Writing portable scripts
5. How Caro ensures POSIX compliance
6. Platform detection in Caro
7. Testing scripts for portability
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Listicle: 50 Common Terminal Commands Explained in Plain English" \
    --label "content,tutorial,priority:p2,seo" \
    --body "$(cat <<'EOF'
## Description
High-volume educational content targeting beginners searching for command explanations.

## Target Keywords
- linux commands beginners
- terminal commands list
- basic shell commands
- command line cheat sheet

## Content Outline
Categories:
1. Navigation (cd, pwd, ls)
2. File operations (cp, mv, rm, touch)
3. Text processing (cat, grep, sed, awk)
4. Searching (find, locate)
5. Compression (tar, gzip, zip)
6. System info (df, du, top, ps)
7. Networking (curl, wget, ping)
8. Git (status, commit, push, pull)
9. Permissions (chmod, chown)
10. Process management (kill, bg, fg)

For each command include:
- Plain English description
- Basic syntax
- Common examples
- Caro natural language equivalent
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: How to Check Disk Space in Linux (df vs du)" \
    --label "content,tutorial,priority:p2,seo" \
    --body "$(cat <<'EOF'
## Description
Educational tutorial on disk commands, showing Caro alternatives.

## Target Keywords
- check disk space linux
- df command tutorial
- du command examples
- disk usage terminal
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: How to Compress and Extract Files with tar and gzip" \
    --label "content,tutorial,priority:p2,seo" \
    --body "$(cat <<'EOF'
## Description
Comprehensive guide to compression, with Caro natural language examples.

## Target Keywords
- tar gzip tutorial
- compress files linux
- extract tar.gz
- archive command line
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Guide: Apple Silicon MLX for Developers" \
    --label "content,guide,priority:p2,seo" \
    --body "$(cat <<'EOF'
## Description
Technical deep-dive into MLX and how Caro leverages it for optimal Apple Silicon performance.

## Target Keywords
- mlx apple silicon
- m1 m2 m3 m4 ai
- local llm mac
- apple neural engine development
EOF
)"

echo "P2 issues created."
echo ""

# ====================
# PRIORITY 3 ISSUES
# ====================

echo "Creating P3 Issues..."

gh issue create -R "$REPO" \
    --title "[Content] Use Case: Automating DevOps Workflows with Natural Language" \
    --label "content,priority:p3,seo" \
    --body "$(cat <<'EOF'
## Description
Show how DevOps engineers can use Caro to speed up terminal workflows.

## Target Keywords
- devops automation ai
- shell scripting automation
- cli automation tools
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Use Case: From Terminal Fear to Terminal Fluency with AI" \
    --label "content,priority:p3,seo" \
    --body "$(cat <<'EOF'
## Description
Address terminal intimidation and show how Caro helps beginners.

## Target Keywords
- command line intimidating
- terminal for beginners
- learn terminal with ai
- terminal phobia
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: Git Commands Made Easy with Natural Language" \
    --label "content,tutorial,priority:p3,seo" \
    --body "$(cat <<'EOF'
## Description
Common git operations and how to describe them in natural language.

## Target Keywords
- git commands beginners
- git tutorial terminal
- git cheat sheet
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Thought Leadership: Open Source vs Cloud AI Coding Tools" \
    --label "content,comparison,priority:p3" \
    --body "$(cat <<'EOF'
## Description
Position Caro within the broader open source AI movement.

## Target Keywords
- open source ai coding
- github copilot alternatives free
- self hosted coding ai
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] FAQ: Frequently Asked Questions About Caro" \
    --label "content,priority:p3,seo" \
    --body "$(cat <<'EOF'
## Description
Comprehensive FAQ targeting question-based searches.

## Target Questions
- What is Caro?
- Is Caro free?
- Does Caro work offline?
- Is my data private with Caro?
- What models does Caro support?
- How is Caro different from ChatGPT?
- Can Caro run dangerous commands?
- What platforms does Caro support?
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: How to Search File Contents with grep" \
    --label "content,tutorial,priority:p3,seo" \
    --body "$(cat <<'EOF'
## Description
Grep tutorial with natural language alternatives.

## Target Keywords
- grep command examples
- search file contents terminal
- find text in files linux
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Tutorial: Managing Processes in Linux Terminal" \
    --label "content,tutorial,priority:p3,seo" \
    --body "$(cat <<'EOF'
## Description
Process management tutorial.

## Target Keywords
- linux process management
- kill process terminal
- background jobs linux
EOF
)"

gh issue create -R "$REPO" \
    --title "[Content] Guide: Shell Customization for Productivity" \
    --label "content,guide,priority:p3" \
    --body "$(cat <<'EOF'
## Description
Shell customization guide.

## Target Keywords
- zsh customization
- bash aliases
- shell productivity
EOF
)"

echo "P3 issues created."
echo ""

# ====================
# SEO ISSUES
# ====================

echo "Creating SEO Issues..."

gh issue create -R "$REPO" \
    --title "[SEO] Implement Schema Markup Across Content Pages" \
    --label "seo,priority:p1" \
    --body "$(cat <<'EOF'
## Description
Add appropriate schema markup to all content pages.

## Tasks
- [ ] SoftwareApplication schema on main pages
- [ ] FAQ schema on FAQ page
- [ ] HowTo schema on tutorials
- [ ] Article schema on blog posts
- [ ] Organization schema on about page
EOF
)"

gh issue create -R "$REPO" \
    --title "[SEO] Optimize Site Structure and Internal Linking" \
    --label "seo,priority:p1" \
    --body "$(cat <<'EOF'
## Description
Create optimal site structure for SEO.

## Tasks
- [ ] Create content hub pages
- [ ] Implement breadcrumb navigation
- [ ] Add related content links
- [ ] Create topic clusters
- [ ] Optimize URL structure
EOF
)"

gh issue create -R "$REPO" \
    --title "[SEO] Optimize Content for AI Assistant Visibility" \
    --label "seo,priority:p2" \
    --body "$(cat <<'EOF'
## Description
Ensure content is structured for AI assistants (ChatGPT, Claude, Gemini).

## Tasks
- [ ] Clear, factual content structure
- [ ] Unique data and insights
- [ ] Regular content updates
- [ ] Authoritative sourcing
- [ ] Comprehensive coverage
EOF
)"

echo "SEO issues created."
echo ""

echo "=== Complete! ==="
echo ""
echo "Summary:"
echo "- P0 Issues: 3 (Critical)"
echo "- P1 Issues: 4 (High)"
echo "- P2 Issues: 5 (Medium)"
echo "- P3 Issues: 8 (Lower)"
echo "- SEO Issues: 3"
echo "- Total: 23 issues"
echo ""
echo "Next steps:"
echo "1. Create GitHub Project: gh project create '$PROJECT_NAME' --owner wildcard"
echo "2. Add issues to project manually or with: gh project item-add"
echo "3. Prioritize and assign issues"
echo "4. Start creating content!"
