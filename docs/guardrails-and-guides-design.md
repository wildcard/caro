# Guardrails & Guides Design Document

**Version:** 1.0
**Status:** Draft
**Created:** 2025-11-28

## Executive Summary

This document proposes a Warp Terminus-inspired knowledge system for cmdai that:
1. **Visualizes Guardrails** - Makes cmdai's 52 safety patterns discoverable, searchable, and community-editable
2. **Curates Community Guides** - Provides a library of common tasks with natural language → command examples

**Goals:**
- Increase user trust through transparency of safety rules
- Reduce friction in learning what cmdai can/can't do
- Build community engagement via contributions
- Create SEO-friendly content that drives acquisition

## 1. TL;DR

cmdai will create a dual-purpose knowledge hub:
- **Guardrails Browser** - Interactive catalog of all 52+ safety patterns with examples, risk explanations, and community override proposals
- **Guides Library** - Terminus-like collection of "how do I..." articles with prompt examples, generated commands, and "try in cmdai" functionality

This supports cmdai's product goals by building trust (transparency), driving adoption (SEO), and creating a feedback loop (community contributions improve the product).

## 2. What We're Building

### 2.1 Guardrails Visualization

**Purpose:** Make cmdai's safety system transparent and participatory

**Features:**
- Browse all 52+ safety patterns by category (filesystem, network, privilege, etc.)
- Search patterns by command, risk level, or description
- View pattern details: regex, risk level, shell compatibility, examples
- See statistics: most-triggered patterns, community-proposed additions
- Submit community pattern proposals via GitHub

**Example User Flow:**
```
User runs: cmdai guardrails search "rm"

Output:
┌─────────────────────────────────────────────────────────────┐
│ Found 8 guardrails matching "rm"                            │
├─────────────────────────────────────────────────────────────┤
│ [CRITICAL] Recursive deletion of root or home directory     │
│   Pattern: rm\s+(-[rfRF]*\s+)*(/|~|\$HOME|/\*|~/\*)         │
│   Blocks: rm -rf /, rm -rf ~, rm -rf $HOME                  │
│   Learn more: cmdai guardrails show 1                        │
│                                                              │
│ [CRITICAL] Force recursive deletion from root                │
│   Pattern: rm\s+-rf\s+/                                      │
│   Blocks: rm -rf /                                           │
│   Learn more: cmdai guardrails show 2                        │
│                                                              │
│ [HIGH] Delete files with elevated privileges                 │
│   Pattern: sudo\s+rm\s                                       │
│   Blocks: sudo rm -rf /var/log/*, sudo rm important.db       │
│   Learn more: cmdai guardrails show 3                        │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Community Guides

**Purpose:** Provide a curated library of common developer tasks

**Features:**
- Browse guides by topic (Git, Docker, Files, Networking, etc.)
- Search guides by task description or command type
- View guide details: prompt, generated command, explanation, safety notes
- Rate/vote on guide quality
- Submit new guides via GitHub
- "Try in cmdai" to execute the example locally

**Example User Flow:**
```
User visits: https://cmdai.dev/guides or runs: cmdai guides search "git"

Output:
┌─────────────────────────────────────────────────────────────┐
│ Git Guides (24 found)                                       │
├─────────────────────────────────────────────────────────────┤
│ 1. Undo last commit but keep changes                        │
│    Prompt: "undo my last git commit but keep the changes"   │
│    Command: git reset --soft HEAD~1                          │
│    Safety: ✓ Safe                                           │
│    Try: cmdai guides run 1                                   │
│                                                              │
│ 2. Create new branch from current changes                   │
│    Prompt: "create a new git branch with my uncommitted..."  │
│    Command: git checkout -b feature-name                     │
│    Safety: ✓ Safe                                           │
│    Try: cmdai guides run 2                                   │
│                                                              │
│ 3. Squash last 3 commits                                    │
│    Prompt: "squash my last 3 commits into one"              │
│    Command: git rebase -i HEAD~3                             │
│    Safety: ⚠ Moderate (modifies git history)                │
│    Try: cmdai guides run 3                                   │
└─────────────────────────────────────────────────────────────┘
```

## 3. Core Ideology

### 3.1 Beliefs About Users

1. **Users distrust black boxes**
   Developers won't adopt AI command generation if they don't understand what's being blocked and why. Transparency builds trust.

2. **Users learn by example**
   Abstract documentation is less effective than "here's what I typed → here's what it generated" examples.

3. **Users want control**
   Power users should be able to propose new safety patterns or override existing ones with clear understanding of tradeoffs.

4. **Users are the best QA**
   Community contributions reveal edge cases and use cases the core team might miss.

### 3.2 Product Beliefs

1. **Safety is a feature, not a constraint**
   By visualizing guardrails, we turn a potential objection ("why can't I do X?") into a value proposition ("cmdai protects you from X").

2. **In-context help reduces friction**
   Users shouldn't have to leave their terminal or search Google to understand cmdai's capabilities.

3. **Community creates moat**
   A library of community-validated guides becomes a defensible asset that's hard for competitors to replicate.

4. **Documentation is marketing**
   SEO-optimized guides drive top-of-funnel awareness ("how to undo git commit" → discover cmdai).

### 3.3 Workflow Beliefs

1. **Prompt-first is the future**
   cmdai guides train users to think in natural language first, reinforcing the core UX.

2. **Safety validation should be collaborative**
   The line between "safe" and "dangerous" is contextual. Community input improves that line.

3. **Executable docs are better than static docs**
   "Try in cmdai" buttons turn passive reading into active learning.

## 4. Funnel Mechanics

### 4.1 Acquisition (How people find it)

**SEO Intent Capture:**
- Target long-tail queries: "how to recursively delete files git", "undo last commit", "find large files linux"
- Each guide is a landing page optimized for specific developer tasks
- Guides answer the question AND introduce cmdai as a better way to work

**Developer Platforms:**
- Share guides on Hacker News, Reddit (r/commandline, r/rust), Dev.to
- "Here's how cmdai safely handles recursive deletes" posts
- Guardrails transparency as differentiation in security-conscious communities

**Referral from cmdai CLI:**
- When a command is blocked, show: "Learn why: cmdai guardrails show <id> or visit cmdai.dev/guardrails"
- When a prompt fails to generate good output, suggest: "Try a guide: cmdai guides search <topic>"

### 4.2 Activation (First value)

**Immediate usefulness:**
- User lands on guide page, sees exactly what they need, copies command
- User runs `cmdai guides run <id>` and gets immediate result
- User runs `cmdai guardrails list` and understands what's protected

**Aha moments:**
1. "I can just ask cmdai to do this instead of searching Stack Overflow"
2. "cmdai blocks dangerous stuff but tells me why and how to work around it"
3. "This community has already figured out the best way to do X"

**Frictionless trial:**
- Guides show both the manual command AND the cmdai prompt
- Users can copy-paste the prompt to try cmdai immediately
- No account needed, no installation friction (for web viewers)

### 4.3 Retention (How it becomes a habit)

**In-terminal reference:**
- `cmdai guides` becomes muscle memory for "how do I..."
- `cmdai guardrails` builds understanding of safety boundaries
- Users start contributing their own patterns/guides

**Feedback loops:**
- Users vote on guide quality → best guides surface
- Users propose new guardrails → feel ownership of safety system
- Users see their contributions merged → become ambassadors

**Progressive disclosure:**
- New users: browse common guides, understand basic safety
- Power users: contribute custom patterns, build complex guides
- Experts: maintain the knowledge base, review contributions

### 4.4 Expansion (Path to paid features)

**Community → Enterprise:**
- Free: Public guides, standard guardrails
- Teams: Custom guardrails, private guide libraries
- Enterprise: Audit logging of safety overrides, compliance reporting

**Network effects:**
- More users → more guides → better SEO → more users
- More guides → more pattern coverage → safer product → higher trust
- Higher trust → easier enterprise sales

**Upsell opportunities:**
- "This guide requires elevated permissions. Upgrade to Teams for role-based safety controls."
- "Export your team's custom guardrails. Available in Enterprise plan."

## 5. What We're Optimizing For

### Top 3 Metrics

1. **Contribution Rate**
   - **What:** % of active users who submit a guide or guardrail proposal per month
   - **Why:** Measures community engagement and product stickiness
   - **Target:** 5% contribution rate within 6 months

2. **Guide Execution Rate**
   - **What:** % of guide views that result in "try in cmdai" executions
   - **Why:** Measures conversion from passive browsing to active usage
   - **Target:** 30% execution rate on web, 60% in CLI

3. **Safety Override Requests**
   - **What:** Number of times users request to bypass a blocked command with understanding
   - **Why:** Measures effectiveness of transparency (informed overrides vs. frustrated churn)
   - **Target:** <5% of blocked commands result in frustrated churn

### Secondary Metrics

- **SEO Traffic:** Organic visits to guide pages
- **Time to First Guide Contribution:** Days from first cmdai use to first guide submission
- **Guide Quality Score:** Community votes + execution success rate
- **Pattern Coverage:** % of real-world dangerous commands caught by guardrails

## 6. Technical Architecture

### 6.1 Data Model

**Guardrail Metadata** (extends existing `DangerPattern`):
```rust
pub struct GuardrailMeta {
    pub id: String,                    // Unique identifier
    pub pattern: DangerPattern,        // Existing pattern struct
    pub category: GuardrailCategory,   // Filesystem, Network, Privilege, etc.
    pub examples_blocked: Vec<String>, // Commands this blocks
    pub examples_safe: Vec<String>,    // Similar commands that are safe
    pub learn_more_url: Option<String>,// Link to detailed explanation
    pub community_notes: Vec<Note>,    // User-contributed insights
    pub stats: GuardrailStats,         // Usage statistics
}

pub enum GuardrailCategory {
    FilesystemDestruction,
    DiskOperations,
    PrivilegeEscalation,
    NetworkBackdoors,
    ProcessManipulation,
    SystemModification,
    EnvironmentManipulation,
}

pub struct GuardrailStats {
    pub times_triggered: u64,
    pub times_overridden: u64,
    pub false_positive_reports: u64,
}
```

**Community Guide:**
```rust
pub struct CommunityGuide {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: GuideCategory,
    pub tags: Vec<String>,

    // The core content
    pub natural_language_prompt: String,
    pub generated_command: String,
    pub shell_type: ShellType,

    // Context and safety
    pub explanation: String,
    pub safety_notes: String,
    pub risk_level: RiskLevel,
    pub prerequisites: Vec<String>,

    // Community engagement
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub upvotes: u32,
    pub downvotes: u32,
    pub execution_count: u64,
    pub success_rate: f32,

    // Metadata
    pub related_guides: Vec<String>,
    pub related_guardrails: Vec<String>,
}

pub enum GuideCategory {
    Git,
    Docker,
    FileManagement,
    Networking,
    SystemAdministration,
    Development,
    Database,
}
```

### 6.2 Storage

**Option A: Git-based (recommended for v1)**
- Guardrails: `docs/guardrails/*.yml` (one file per guardrail)
- Guides: `docs/guides/{category}/*.md` (frontmatter + markdown)
- Advantages: Easy PR workflow, version control, no database dependency
- Disadvantages: No real-time analytics, limited query capabilities

**Option B: Hybrid (v2)**
- Static files for content (Git)
- SQLite database for stats/votes
- Sync on build/deploy

**File Format Examples:**

`docs/guardrails/001-rm-rf-root.yml`:
```yaml
id: "grd-001"
pattern:
  regex: "rm\\s+(-[rfRF]*\\s+)*(/|~|\\$HOME|/\\*|~/\\*)"
  risk_level: Critical
  description: "Recursive deletion of root or home directory"
  shell_specific: null

category: FilesystemDestruction

examples_blocked:
  - "rm -rf /"
  - "rm -rf ~"
  - "rm -rf $HOME"
  - "rm -rf /*"

examples_safe:
  - "rm -rf ./temp"
  - "rm -rf /tmp/my-folder"
  - 'echo "rm -rf /" > dangerous-script.sh'

explanation: |
  This pattern blocks attempts to recursively delete your entire filesystem
  from the root (/) or home (~) directory. Such commands can destroy your
  operating system in seconds and are almost never intentional.

learn_more_url: "https://cmdai.dev/guardrails/rm-rf-root"

community_notes:
  - author: "user123"
    date: "2024-01-15"
    note: "This saved me when I accidentally had a space in `rm -rf / tmp`"

stats:
  times_triggered: 1247
  times_overridden: 3
  false_positive_reports: 0
```

`docs/guides/git/undo-last-commit.md`:
```markdown
---
id: "guide-git-001"
title: "Undo last commit but keep changes"
description: "Reset your last Git commit while preserving your work"
category: Git
tags: [git, undo, reset, commit]
natural_language_prompt: "undo my last git commit but keep the changes"
generated_command: "git reset --soft HEAD~1"
shell_type: Bash
risk_level: Safe
author: "cmdai-community"
created_at: "2024-01-10T10:00:00Z"
updated_at: "2024-01-15T14:30:00Z"
upvotes: 142
downvotes: 3
execution_count: 1834
success_rate: 0.98
related_guides:
  - "guide-git-002"
  - "guide-git-015"
related_guardrails: []
---

# Undo Last Commit But Keep Changes

## What it does

Undoes your most recent Git commit, moving the changes back to your staging area. Your work is preserved, but the commit is removed from history.

## When to use this

- You committed too early and want to add more changes
- You made a typo in the commit message
- You want to split one commit into multiple smaller commits

## The cmdai way

Instead of remembering Git flags, just ask:

```bash
cmdai "undo my last git commit but keep the changes"
```

cmdai generates:
```bash
git reset --soft HEAD~1
```

## Manual command

```bash
git reset --soft HEAD~1
```

## Safety notes

✓ **Safe operation** - Your changes are preserved in the staging area
✗ **History modification** - Don't use this on commits you've already pushed to shared branches

## What happens next

After running this command:
1. Your last commit is removed from Git history
2. All changes from that commit are in your staging area (ready to commit again)
3. You can modify files, add more changes, or split into multiple commits

## Related guides

- [Undo last commit and discard changes](guide-git-002)
- [Amend last commit message](guide-git-015)

## Try it yourself

```bash
# Try this guide in cmdai
cmdai guides run guide-git-001

# Or execute the command directly
git reset --soft HEAD~1
```
```

### 6.3 CLI Commands

**Guardrails commands:**
```bash
cmdai guardrails list                    # List all guardrails
cmdai guardrails list --category filesystem  # Filter by category
cmdai guardrails list --risk critical    # Filter by risk level
cmdai guardrails search "rm"            # Search patterns
cmdai guardrails show grd-001           # Show detailed info
cmdai guardrails stats                  # Show usage statistics
cmdai guardrails test "rm -rf /"        # Test what would be blocked
```

**Guides commands:**
```bash
cmdai guides list                       # List all guides
cmdai guides list --category git        # Filter by category
cmdai guides search "undo commit"       # Search guides
cmdai guides show guide-git-001         # Show guide details
cmdai guides run guide-git-001          # Execute guide's command
cmdai guides contribute                 # Open contribution workflow
cmdai guides stats                      # Popular guides, trending topics
```

### 6.4 Web Interface

**Tech Stack:**
- Static site generator: Rust-based (e.g., Zola) or Next.js
- Hosting: GitHub Pages or Vercel (free)
- Search: Client-side with Fuse.js or server-side with Meilisearch
- Syntax highlighting: Prism.js or Shiki
- Interactive terminal demos: Asciinema or custom React component

**Key Pages:**
1. `/guardrails` - Browse all safety patterns
2. `/guardrails/{id}` - Detailed pattern page
3. `/guides` - Browse all guides
4. `/guides/{category}/{id}` - Individual guide page
5. `/contribute` - How to contribute
6. `/stats` - Community analytics

**SEO Optimization:**
- Server-side rendering for all content
- Semantic HTML with proper heading hierarchy
- Schema.org markup for HowTo guides
- Open Graph tags for social sharing
- Sitemap generation

### 6.5 Community Contribution Workflow

**For Guardrails:**
1. User clicks "Propose new guardrail" on web or runs `cmdai guardrails propose`
2. Fill template: pattern, risk level, description, examples
3. CLI validates regex, checks for duplicates
4. Creates GitHub PR with prefilled template
5. Maintainers review, test against corpus of safe/dangerous commands
6. Merge → auto-deploy to docs site and next CLI release

**For Guides:**
1. User clicks "Add guide" or runs `cmdai guides contribute`
2. Fill template: prompt, expected command, category, explanation
3. CLI validates guide format, runs cmdai to verify command generation
4. Creates GitHub PR with guide markdown
5. Community votes/comments on PR
6. Maintainers review for quality/safety
7. Merge → auto-deploy to docs site

**Quality Gates:**
- Automated tests: regex compilation, command safety validation
- Linting: markdown formatting, frontmatter completeness
- Duplicate detection: similarity search against existing content
- Manual review: maintainer approval required

## 7. Risks & Downsides

### 7.1 Content Trust

**Risk:** Community-submitted guides contain dangerous or incorrect commands

**Mitigation:**
- All guides run through safety validation before merge
- Maintainer approval required (not fully automated)
- Version control allows reverting bad contributions
- Community voting surfaces quality issues
- "Report issue" button on every guide

### 7.2 Maintenance Burden

**Risk:** Guides become outdated as tools evolve (Git 2.30 vs 2.40 syntax differences)

**Mitigation:**
- Automated testing: CI runs each guide's command in isolated container
- Deprecation warnings when tools change
- Community flags outdated content
- Periodic maintenance sprints to review old guides
- Version tagging for guides (works with Git >= 2.30)

### 7.3 Pattern Inflation

**Risk:** Too many guardrails create false positive fatigue

**Mitigation:**
- High bar for new patterns (must have real-world danger evidence)
- Regular pruning of low-trigger patterns
- Severity-based filtering (users can adjust safety level)
- A/B testing new patterns before promoting to default set

### 7.4 SEO Cannibalization

**Risk:** Guides compete with main cmdai marketing pages for ranking

**Mitigation:**
- Clear IA: marketing pages focus on product benefits, guides focus on specific tasks
- Internal linking from guides back to main marketing CTAs
- Different keywords: guides target long-tail task queries, marketing targets "AI command line tool"

### 7.5 Misalignment with Product Direction

**Risk:** Community guides teach patterns that conflict with future cmdai features

**Mitigation:**
- Public roadmap visible on contribution page
- Maintainer notes on deprecated approaches
- Migration guides when product direction shifts
- Active community management (respond to questions/proposals)

## 8. "If I Were Them" Next Improvements

### 8.1 Interactive Safety Sandbox
**What:** Web-based terminal emulator that shows how cmdai blocks dangerous commands in real-time
**Why:** Demo the safety system without requiring installation
**Implementation:** Xterm.js + WASM build of cmdai, runs entirely in browser

### 8.2 AI-Generated Guide Suggestions
**What:** cmdai analyzes your command history, suggests relevant guides
**Why:** Proactive discovery instead of reactive search
**Implementation:** Local history analysis (privacy-preserving), similarity search against guide database

### 8.3 Contextual Safety Overrides
**What:** Instead of binary block/allow, show safe alternatives
**Why:** Educational - teaches users safer ways to achieve their goal
**Example:** User tries `rm -rf /tmp/*` → cmdai suggests `rm -rf /tmp/my-specific-folder`

### 8.4 Guide Chains / Workflows
**What:** Multi-step guides that combine commands (like GitHub Actions for terminal)
**Why:** Handle complex workflows (deploy pipeline, database migration)
**Implementation:** YAML workflow files with conditional logic, loops, error handling

### 8.5 Safety Pattern Analytics Dashboard
**What:** Public analytics on which patterns trigger most, false positive rates
**Why:** Transparency builds trust, data informs pattern tuning
**Implementation:** Anonymized telemetry (opt-in), weekly digest of blocked command categories

## 9. Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Extend `DangerPattern` with metadata fields
- [ ] Create `GuardrailMeta` and `CommunityGuide` data structures
- [ ] Build YAML/Markdown parsers for content loading
- [ ] Implement basic CLI commands (`guardrails list`, `guides list`)
- [ ] Create contribution templates (GitHub issue/PR templates)

### Phase 2: Core Features (Week 3-4)
- [ ] Implement search functionality (fuzzy search for guides/guardrails)
- [ ] Add `guardrails show <id>` with detailed view
- [ ] Add `guides run <id>` to execute guide commands
- [ ] Create validation pipeline for contributions
- [ ] Build initial content: document all 52 existing guardrails, create 20 seed guides

### Phase 3: Web Presence (Week 5-6)
- [ ] Set up static site generator
- [ ] Design and implement web UI for browsing guardrails
- [ ] Design and implement web UI for browsing guides
- [ ] Add search functionality to web interface
- [ ] Implement "Try in cmdai" copy-to-clipboard functionality
- [ ] Deploy to cmdai.dev/guardrails and cmdai.dev/guides

### Phase 4: Community Features (Week 7-8)
- [ ] Implement voting system for guides
- [ ] Add analytics tracking (execution counts, success rates)
- [ ] Create contribution onboarding flow
- [ ] Set up automated testing for contributed content
- [ ] Launch beta with 10 community contributors

### Phase 5: Polish & Launch (Week 9-10)
- [ ] SEO optimization (meta tags, sitemaps, schema markup)
- [ ] Performance tuning (web and CLI)
- [ ] Documentation for contributors
- [ ] Marketing materials (blog post, demo video)
- [ ] Public launch announcement

## 10. Success Criteria

### Immediate (Month 1)
- ✓ All 52 existing guardrails documented with metadata
- ✓ 50+ high-quality community guides across 5+ categories
- ✓ CLI commands functional and tested
- ✓ Web interface live and indexed by Google
- ✓ Contribution workflow tested by 10 beta users

### Short-term (Months 2-3)
- 20+ community contributions merged
- 10,000+ guide page views from organic search
- 500+ "try in cmdai" executions from web
- 5% of cmdai users discover a guide through CLI search
- 3+ guardrail proposals from community (showing engagement)

### Medium-term (Months 4-6)
- 100+ guides covering 80% of common developer tasks
- Top 10 Google ranking for 5+ long-tail command queries
- 5% monthly active user contribution rate
- Guides referenced in external articles/Stack Overflow answers
- Feature requests for advanced guide features (workflows, parameters)

### Long-term (Months 7-12)
- 500+ guides, self-sustaining community curation
- Guides drive 30% of new cmdai installations
- Enterprise customers request custom guide libraries
- Other CLI tools adopt similar transparency approach (ecosystem impact)
- Published case study on community-driven safety engineering

## 11. Open Questions

1. **Localization:** Should guides support multiple languages? (Probably not v1, but worth planning for)
2. **Versioning:** How do we handle guides that work differently across OS/tool versions?
3. **Personalization:** Should cmdai learn which guides are most relevant to each user?
4. **Monetization:** Are there premium guide features worth exploring? (Custom team libraries, advanced analytics)
5. **Integration:** Should guides integrate with other tools (VS Code snippets, Alfred workflows)?
6. **Gamification:** Would badges/leaderboards for contributors help or hurt community culture?

## 12. Appendix: Competitive Analysis

### Warp Terminus
- **Strengths:** Clean UX, good SEO, actionable content
- **Weaknesses:** Generic (not Warp-specific), no community contribution, no safety context
- **Differentiation:** cmdai guides are executable, safety-aware, and community-driven

### tldr-pages
- **Strengths:** Massive command coverage, strong community, CLI-first
- **Weaknesses:** Manual commands only (no AI generation), no safety context
- **Differentiation:** cmdai guides show the natural language → command mapping

### ShellCheck
- **Strengths:** Real-time linting, educational error messages
- **Weaknesses:** Reactive (finds issues after writing), no generative AI
- **Differentiation:** cmdai is proactive (safe generation) + reactive (validation)

### Anthropic Claude safety cards
- **Strengths:** Transparent about AI limitations and safety measures
- **Weaknesses:** Focused on LLM safety (toxicity, bias), not command safety
- **Differentiation:** cmdai applies similar transparency to shell command domain

---

**Next Steps:**
1. Review this design with team and community
2. Gather feedback on priorities and technical approach
3. Create Phase 1 implementation tickets
4. Recruit beta testers for contribution workflow
5. Begin content creation (documenting existing guardrails, writing seed guides)
