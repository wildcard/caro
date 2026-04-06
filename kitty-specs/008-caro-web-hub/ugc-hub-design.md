# CARO Hub UGC Design: Command Recipe Marketplace

**Feature Branch**: `claude/caro-hub-ugc-design-Ifv9i`
**Created**: 2026-04-06
**Status**: Draft
**Extends**: `kitty-specs/008-caro-web-hub/spec.md`
**Input**: Product owner vision for consumer-facing UGC marketplace of reproducible, safe terminal recipes

---

## Context

CARO (v1.1.0) is a Rust CLI that converts natural language to safe shell commands. An existing spec (`kitty-specs/008-caro-web-hub/`) designs a Bluesky AT Protocol social platform for **developers** (DevOps guilds, SRE sharing). This document extends that vision into a **consumer-facing marketplace of reproducible, safe terminal recipes** targeting everyone - not just developers.

**The core insight**: People pay for GUI apps (PDF converters, video processors, image editors) that are just wrappers around CLI tools like FFmpeg, ImageMagick, Ghostscript. CARO Hub makes these tools accessible to non-developers through safe, reproducible, community-vetted recipes - replacing paid software with transparent terminal commands.

**Positioning**: CARO = "Executable StackOverflow + Safe Terminal Runtime" - sitting between fragmented Google/StackOverflow results, raw ChatGPT output that requires expertise, and opaque full-blown agents that cost tokens and remove user control.

**Key principle**: "A user should be able to go from problem to working solution without paying. Tokens only improve confidence, speed, reliability, sophistication."

---

## Resolved Design Decisions

1. **Identity model**: Machine fingerprint/mnemonic (web3-style) as base identity. Users get a unique machine identity from the CLI automatically. Later, they can "claim" their account online by linking to email/password, Bluesky, GitHub, or other OAuth via BetterAuth. Bluesky AT Protocol is specifically for social content posting - a perk of being a CARO terminal user.

2. **External agent API**: Keep at Stage 5. Build the recipe/execution foundation first. The agent validation API comes naturally after the execution infrastructure is solid.

3. **Document scope**: Full design doc covering schema, architecture, CLI integration, token model, and bootstrapping strategy.

---

## What Changes vs. Existing 008 Spec

The existing Bluesky spec is **preserved, not replaced**. The UGC Hub adds a consumer-facing layer on top:

| Dimension | Existing 008 Spec (Kept) | New UGC Layer (Added) |
|-----------|--------------------------|----------------------|
| Identity | Bluesky OAuth + DID | **Machine fingerprint** (CLI-first, claim later via BetterAuth) |
| Content unit | CommandArtifact + Runbook | **CommandRecipe** (superset wrapping both) |
| Discovery | Bluesky guild feeds | **SEO-first web pages** (Google-indexed) |
| Execution | View/share only | **"Run in CARO"** (one-click CLI execution) |
| Audience | Developers (15 guilds) | Everyone (consumer categories) |
| Monetization | All free | **Freemium** (free baseline + token enhancements) |
| Trust | Upvotes/comments | **Run count, ratings, confidence levels** |
| Privacy | Redaction engine | Same (reused for recipe publishing) |
| Design | 8-bit pixel art | Same (extended to recipe pages) |

---

## 1. Recipe Schema (Core Data Model)

The `CommandRecipe` is the canonical content unit. It wraps existing types and supports the full evolution from static to semi-agent.

### Recipe Type Hierarchy

```
CommandRecipe (web-discoverable wrapper)
  |
  +-- StaticPayload        (Stage 1: single command)
  |     maps to existing CommandArtifact
  |
  +-- ParameterizedPayload (Stage 2: template with user inputs)
  |     e.g., "Generate {{count}} images of color {{color}}"
  |
  +-- ComposablePayload    (Stage 3: multi-step workflow)
  |     maps to existing Runbook
  |
  +-- ConditionalPayload   (Stage 4: branching + approval gates)
```

### Key Schema Fields

```
CommandRecipe:
  # Identity
  id: ULID                    # Globally unique, sortable
  slug: String                # URL-friendly: "convert-video-to-mp4"
  version: u32                # Monotonic version counter

  # Discovery (SEO-critical)
  title: String               # "Convert any video to MP4"
  description: String         # Rich description, max 500 chars
  intent: String              # Canonical user intent (search anchor)
  category: RecipeCategory    # practical_utility | creative | dev_power | replacement_tool | system_admin | data_processing
  tags: Vec<String>           # max 15
  searchKeywords: Vec<String> # Additional SEO terms

  # Execution Payload (union type)
  payload: RecipePayload      # One of: Static | Parameterized | Composable | Conditional

  # Dependencies
  dependencies: Vec<ToolDependency>  # ffmpeg, imagemagick, etc. with install hints per platform

  # Safety & Validation
  confidenceLevel: ConfidenceLevel   # safe (green) | needs_review (yellow) | risky (red)
  safetyValidation: SafetyReport     # Pattern matches, sandbox result, validator version
  sandboxable: bool
  deterministic: bool

  # Social Proof
  stats: RecipeStats          # run_count, success_rate, ratings (up/down), fork_count

  # Authorship
  authorId: String            # Machine fingerprint (CARO identity)
  authorHandle: Option        # Display name (claimed via BetterAuth)
  originalRecipeId: Option    # Fork source

  # Moderation
  status: draft | pending_review | published | flagged | archived

  # Token tier
  tier: free | enhanced       # Which features require tokens
```

### Payload Types

```typescript
// Stage 1: Single command
interface StaticPayload {
  type: "static";
  prompt: string;              // What user asked
  command: string;             // The shell command
  shell: ShellType;
  explanation: string;
  expectedOutput?: string;
}

// Stage 2: Template with user inputs
interface ParameterizedPayload {
  type: "parameterized";
  prompt: string;
  commandTemplate: string;     // "ffmpeg -i {{input}} -vf scale={{width}}:{{height}} {{output}}"
  parameters: RecipeParameter[];
  shell: ShellType;
  explanation: string;
  expectedOutput?: string;
}

interface RecipeParameter {
  name: string;                // "width"
  label: string;               // "Output Width"
  type: "string" | "number" | "file" | "enum" | "boolean";
  default?: string;
  required: boolean;
  validation?: string;         // Regex or range "1-8192"
  enumValues?: string[];
  description?: string;
}

// Stage 3: Multi-step workflow
interface ComposablePayload {
  type: "composable";
  steps: RecipeStep[];
  prerequisites?: string[];
  estimatedTime?: string;
  difficulty?: "beginner" | "intermediate" | "advanced";
}

interface RecipeStep {
  order: number;
  title: string;
  prompt: string;
  commandTemplate: string;
  parameters?: RecipeParameter[];
  shell: ShellType;
  safetyLevel: ConfidenceLevel;
  notes?: string;
  expectedOutput?: string;
  continueOnError?: boolean;
}

// Stage 4: Branching + approval gates
interface ConditionalPayload {
  type: "conditional";
  steps: ConditionalStep[];
  approvalGates: ApprovalGate[];
}

interface ConditionalStep extends RecipeStep {
  condition?: string;          // "exit_code == 0"
  onFailure?: "abort" | "skip" | "retry" | "branch";
  branchTo?: number;
}

interface ApprovalGate {
  afterStep: number;
  message: string;
  autoApprove?: boolean;       // Token-gated auto-approval
}
```

### Confidence Level Mapping to Existing Rust Types

| Confidence | Existing `RiskLevel` | Display |
|------------|---------------------|---------|
| `safe` | `RiskLevel::Safe` | Green shield |
| `needs_review` | `RiskLevel::Moderate` | Yellow shield |
| `risky` | `RiskLevel::High` / `RiskLevel::Critical` | Red shield |

### Dependency Declaration

```typescript
interface ToolDependency {
  name: string;                // "ffmpeg"
  command: string;             // "ffmpeg -version" (how to check)
  installHint: {
    macos: string;             // "brew install ffmpeg"
    ubuntu: string;            // "apt install ffmpeg"
    windows: string;           // "winget install ffmpeg"
  };
  optional: boolean;
  minVersion?: string;         // "5.0"
}
```

### Social Proof

```typescript
interface RecipeStats {
  runCount: number;
  successRate: number;         // 0.0-1.0
  ratings: { up: number; down: number };
  forkCount: number;
  commentCount: number;
  lastRunAt?: string;
}
```

---

## 2. Architecture: Three-Layer (Machine Identity + Web Discovery + Bluesky Social)

### Identity Model (Machine Fingerprint)

The identity layer is **CLI-first**. When a user installs CARO and runs it for the first time:

1. **Machine fingerprint generated** - deterministic hash from hardware/OS identifiers
2. **Mnemonic phrase derived** - web3-style word sequence (e.g., "brave ocean tiger delta") that uniquely identifies this machine
3. **Local keypair stored** - in `~/.config/caro/identity.toml`
4. **No account creation needed** - user can publish recipes immediately using their machine identity

Later, users can **claim their account** on hub.caro.sh:
- Link machine identity to email/password, GitHub, Bluesky, or other OAuth via **BetterAuth**
- Multiple machines can be linked to one account
- Bluesky linking is a **perk** - it enables posting to CARO's AT Protocol social feeds

```
Identity Flow:
  Install CARO CLI
    -> Machine fingerprint generated (automatic)
    -> Mnemonic phrase shown ("brave ocean tiger delta")
    -> Can publish recipes immediately (machine identity)
    -> Optional: claim account on hub.caro.sh
       -> BetterAuth: email/password, GitHub, Bluesky, etc.
       -> Bluesky linking: enables social posts to AT Protocol
```

### Architecture Overview

```
                      hub.caro.sh (Next.js SSG/SSR)
                      +---------------------------------+
                      | SEO Pages                       |
                      |   /recipe/convert-video-to-mp4  |
                      |   /category/creative            |
                      |   /search?q=ffmpeg              |
                      +--------+----------+-------------+
                               |          |
                  +------------+    +-----+----------+
                  |                 |                 |
          Recipe API          Bluesky AT         CARO CLI
          (CARO-owned)        Protocol           (Rust)
          - CRUD              - Social content   - Generate
          - Search            - Guild feeds      - Validate
          - Stats             - Win stories      - Execute
          - Ratings           - Comments         - Publish
          - Identity          |                  - Machine ID
                  |           |
          PostgreSQL +     AT Protocol
          Typesense        (perk for
          BetterAuth        terminal users)
```

**Key decisions:**
1. **Machine fingerprint as base identity** - zero-friction publishing from CLI, no signup required
2. **BetterAuth for account claiming** - email, GitHub, Bluesky, or any OAuth provider
3. **Bluesky AT Protocol for social content** - a perk of being a CARO user, not a requirement
4. **Recipes stored in CARO-owned PostgreSQL** with Typesense for full-text search
5. **No login required to browse/run** - account only needed for publish/rate/comment/fork
6. **JSON-LD `HowTo` structured data** on each recipe page for Google rich results
7. **Syndication bridge** - claimed Bluesky users get recipes auto-posted to AT Protocol feeds

---

## 3. CLI Integration: "Run in CARO"

### Deep Link Protocol

User finds recipe on web -> clicks "Run in CARO" -> runs locally:

1. **Custom URI scheme**: `caro://run/convert-video-to-mp4?v=3&input=video.avi&width=1920`
   - Registered during `caro init`
   - CLI receives, fetches recipe from API, validates, presents for confirmation

2. **Fallback**: Copyable command shown on web page:
   ```
   caro recipe run convert-video-to-mp4 --input video.avi --width 1920
   ```

### New CLI Subcommands

Added to existing `Commands` enum in `src/main.rs`:

```
caro recipe run <slug>        # Run a recipe from the hub
caro recipe search <query>    # Browse and search recipes
caro recipe publish [file]    # Publish a recipe
caro recipe deps <slug>       # Check dependencies for a recipe
```

### Execution Flow

```
User clicks "Run in CARO"
  -> CLI fetches recipe from API
  -> Check dependencies (e.g., is ffmpeg installed?)
     -> If missing: show install hint per platform
  -> Safety validation (reuse existing SafetyValidator, 52+ patterns)
  -> Show command + explanation + confidence level
  -> User confirms
  -> Execute in sandbox (if sandboxable) or directly
  -> Report run result back to API (opt-in telemetry)
```

### Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `src/recipe/mod.rs` | Create | Recipe types (Rust mirror of TypeScript schema) |
| `src/recipe/deps.rs` | Create | Dependency checker (runs `ffmpeg -version` etc.) |
| `src/recipe/client.rs` | Create | API client for fetching recipes from hub |
| `src/recipe/template.rs` | Create | Template interpolation for parameterized recipes |
| `src/recipe/workflow.rs` | Create | Multi-step workflow executor (Stage 3) |
| `src/recipe/token.rs` | Create | Token balance checking for gated features |
| `src/identity/mod.rs` | Create | Machine fingerprint generation + mnemonic derivation |
| `src/identity/keypair.rs` | Create | Local keypair storage (`~/.config/caro/identity.toml`) |
| `src/main.rs` | Modify | Add `Recipe` subcommand to `Commands` enum |
| `src/models/mod.rs` | Extend | Reuse `RiskLevel`, `ShellType`, `GeneratedCommand` |
| `src/safety/mod.rs` | Reuse | `SafetyValidator::validate_command()` for all recipes |
| `src/execution/executor.rs` | Reuse | `CommandExecutor` for recipe execution |

---

## 4. Token Architecture: Free vs. Paid

### The Rule
Free = knowledge layer + basic execution. Paid = intelligence + safety amplification.

| Feature | Free | Token-Gated |
|---------|------|-------------|
| Generate commands via CLI | Yes | - |
| Browse/search recipes | Yes | - |
| Run static recipes | Yes | - |
| Run parameterized recipes (manual params) | Yes | - |
| Publish recipes | Yes | - |
| Basic safety validation (52 patterns) | Yes | - |
| Fork/remix recipes | Yes | - |
| **Advanced validation** (sandbox test, determinism check) | - | Yes |
| **Guided execution** (step-by-step coaching, error recovery) | - | Yes |
| **AI debugging** (when command fails, AI diagnoses) | - | Yes |
| **Parameter intelligence** (AI suggests optimal values) | - | Yes |
| **Complex workflows** (>5 steps, branching) | - | Yes |
| **Recipe analytics** (who ran, success rate) | - | Yes |

### UX Framing

Never "Upgrade to use AI." Instead:
- "Run with guidance" (token-enhanced)
- "Analyze before execution" (token-enhanced)
- "Auto-fix if something breaks" (token-enhanced)

Confidence levels naturally drive token usage without forcing it:
- Green (Safe) - run freely
- Yellow (Needs review) - "token optional" for deeper analysis
- Red (Risky) - "token strongly recommended" for sandbox + validation

### Implementation
- CLI checks token balance via API (cached 5 minutes) in `src/recipe/token.rs`
- API key stored in `~/.config/caro/config.toml`
- Server-side enforcement - token balance tracked on CARO Hub API
- Never blocks free path - shows upgrade prompt but always allows basic execution

---

## 5. Bootstrapping: Seeding 300 Recipes

### Phase 1: Seed Content (CARO team creates)

**Strategy**: Use CARO's own CLI + eval framework to generate recipes, then human-review.

**Pipeline:**
1. Create `seeds/intents.yaml` with 300 curated intents across categories
2. For each intent, run `caro --prompt "<intent>" --output json`
3. Capture `GeneratedCommand` output (command, explanation, safety_level, confidence)
4. Run through `SafetyValidator` (52+ patterns)
5. Human review: add titles, descriptions, dependency declarations, expected outputs
6. Publish via `caro recipe publish`

**Existing eval dataset** (`src/eval/dataset.yaml`) provides ~100 seed recipes directly:
- `TestCase.input_request` -> `Recipe.intent` + `Recipe.prompt`
- `TestCase.expected_command` -> `Recipe.command`
- `TestCase.tags` -> `Recipe.tags`

**Target distribution:**

| Category | Count | Examples |
|----------|-------|---------|
| Practical Utility | 60 | Find large files, compress dir, clean temp |
| Creative | 50 | Resize images batch, convert video, generate thumbnails |
| Dev/Power | 80 | Git workflows, Docker cleanup, log analysis |
| Replacement Tools | 60 | PDF merge, image compress, CSV to JSON |
| System Admin | 30 | Service status, DNS lookup, SSL cert check |
| Data Processing | 20 | JSON extract, CSV filter, text dedup |

### Phase 2: Curated Contributors
- Invite indie hackers, dev influencers, technical creators
- Incentives: visibility, reputation, early monetization
- Recipe review queue with auto-validation

### Phase 3: UGC Opens
- Submit recipe web flow
- Auto safety validation + sandbox test
- Moderation queue (community reporting + trust scores)

---

## 6. Evolution Stages -> Technical Milestones

### Stage 1: Static Recipes (Months 1-2)
- `StaticPayload` only
- Recipe API (PostgreSQL + Typesense)
- SSG recipe pages with SEO (JSON-LD HowTo)
- `caro recipe run/search` CLI commands
- Dependency checking
- 300 seed recipes published
- Free tier only

### Stage 2: Parameterized Recipes (Months 3-4)
- Template interpolation engine (`src/recipe/template.rs`)
- Interactive parameter prompting in CLI
- UGC submission flow (web form + auto-validation)
- Token infrastructure (API keys, balance checking)
- Token-gated: Parameter Intelligence

### Stage 3: Composable Workflows (Months 5-7)
- Multi-step workflow executor (`src/recipe/workflow.rs`)
- Step-by-step execution with output capture between steps
- Recipe composition (reference other recipes as steps)
- Fork/remix with attribution chain
- Run reporting + ratings
- Token-gated: Guided Execution

### Stage 4: Semi-Agent Layer (Months 8-10)
- Conditional execution based on step output
- Approval gates (human-in-the-loop)
- AI debugging when steps fail
- Token-gated: AI Debugging

### Stage 5: Trusted Execution for External Agents (Months 11+)
- `POST /api/v1/validate` - external agents send commands for CARO validation
- Returns ValidationResult with confidence level
- CARO becomes the "execution safety layer" for Anthropic, OpenAI agents
- Rate-limited, token-gated, full audit trail

---

## 7. How Existing 008 Spec Work Is Preserved

| Existing Feature | What Happens |
|------------------|-------------|
| Bluesky OAuth (FR-001 to FR-004) | **Evolved** - Bluesky becomes one BetterAuth provider (perk), not primary auth. Machine fingerprint is base identity. |
| Privacy dashboard (FR-005 to FR-010) | **Kept as-is** - used for reviewing data before publishing recipes |
| CommandArtifact Lexicon | **Extended** - can be "promoted" to a Recipe via syndication bridge |
| Runbook Lexicon | **Extended** - multi-step runbooks become ComposablePayload recipes |
| Guilds (FR-026 to FR-030) | **Enhanced** - guilds coexist with consumer categories; guild feeds show recipes |
| Epic Fails (FR-021 to FR-025) | **Kept** - now link to recipe IDs for "this recipe failed for me" |
| Win Stories (FR-031 to FR-034) | **Kept** - link to recipes as testimonials |
| Privacy redaction engine | **Reused** - recipes go through same scan before publishing |
| 8-bit design system | **Kept** - recipe pages use same Game Boy palette + pixel fonts |

New Bluesky Lexicon added: `app.caro.share.recipe` (reference record linking to CARO Recipe API).

---

## 8. Categories (Replacing Pure Guild Model)

Consumer-facing categories (anyone can browse):
- **Practical Utility** - file cleanup, disk usage, backups
- **Creative** - image generation, batch editing, audio/video (FFmpeg, ImageMagick)
- **Dev/Power** - scripts, automation, git workflows
- **Replacement Tools** - PDF tools, converters, compressors
- **System Admin** - networking, services, monitoring
- **Data Processing** - CSV, JSON, text transforms

Developer guilds (existing 15+) still exist as a social overlay - a recipe can be in category "Creative" AND shared to "SRE Guild" if the author wants.

---

## 9. Trust Signals

### Run Count + Success Rate
- CLI reports runs back to API (opt-in, respects telemetry consent)
- Displayed on recipe page: "12,400 runs | 98% success rate"

### Ratings
- Thumbs up/down (requires account - machine identity sufficient)
- Net score displayed alongside run count

### Confidence Badge
Computed from:
1. Static safety analysis (SafetyValidator, always free)
2. Sandbox test result (token-gated)
3. Community success rate (aggregated from run reports)
4. Dependency commonality (common tools = higher confidence)

Display:
- Green shield: Safe commands, sandbox tested, high success rate
- Yellow shield: Some moderate risk, or untested in sandbox
- Red shield: High/Critical risk commands, low success rate

---

## 10. Verification Plan

### How to test the design (before full implementation)

1. **Recipe schema validation**: Create 5 sample recipes in each payload type (static, parameterized, composable) as YAML files, validate they capture all needed information
2. **CLI prototype**: Implement `caro recipe run` for static recipes only, test with 10 seed recipes
3. **Dependency checker**: Test against common tools (ffmpeg, imagemagick, ghostscript, pandoc) on macOS and Linux
4. **Safety validation**: Run all 300 seed recipe commands through existing `SafetyValidator`, verify zero false positives
5. **SEO validation**: Generate a sample recipe SSG page, test with Google's Rich Results tool for HowTo schema
6. **Deep link**: Test `caro://` protocol handler registration on macOS and Linux

### End-to-end flow test
```
1. User visits hub.caro.sh/recipe/convert-video-to-mp4
2. Page shows: title, description, command, confidence badge, run count
3. User clicks "Run in CARO" -> caro:// deep link fires
4. CLI: fetches recipe -> checks ffmpeg installed -> shows command + explanation
5. User confirms -> command executes -> success reported back to API
6. Recipe page run count increments
```

---

## Remaining Open Questions

1. **Sandbox technology**: Docker containers for sandboxed execution, or lighter-weight (bubblewrap/firejail on Linux, sandbox-exec on macOS)? Docker is heavier but more universal. Can defer to Stage 2.

2. **Recipe versioning**: When someone updates a recipe, do we keep history (like wiki edits)? Or just overwrite? Suggest: keep version history (immutable versions, latest shown by default).

3. **Machine fingerprint specifics**: Which hardware/OS signals to use for the fingerprint? Need to balance uniqueness vs. stability (shouldn't change on OS update). Suggest: MAC address + hostname + OS type + disk serial, hashed.

---

## Implementation: What to Build First

1. **Recipe schema in Rust** (`src/recipe/mod.rs`) - define `CommandRecipe`, `RecipePayload`, `ToolDependency` types
2. **Machine identity** (`src/identity/mod.rs`) - fingerprint generation + mnemonic derivation
3. **`caro recipe run`** - fetch and execute a static recipe from the API
4. **Seed pipeline** - script to generate 300 recipes from curated intents
5. **Recipe API** - basic CRUD + search (PostgreSQL + Typesense)
6. **SEO pages** - SSG recipe pages on hub.caro.sh
