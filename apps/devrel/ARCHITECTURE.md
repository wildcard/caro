# Caro System Architecture

> 🏗️ **Complete System Design for Caro CLI + Web Hub**

This document describes the full Caro product architecture, including the CLI tool, marketing website, and web hub social platform.

---

## 📊 System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Caro Product Ecosystem                   │
└─────────────────────────────────────────────────────────────────┘
           │
           ├──▶ Caro CLI (Rust)
           │    └─▶ Local command generation
           │        Safe shell companion
           │        Privacy-first telemetry
           │
           ├──▶ Marketing Website (Astro)
           │    └─▶ caro.sh
           │        Product landing page
           │        Documentation hub
           │        Download/installation
           │
           └──▶ Web Hub (Next.js 16)
                └─▶ hub.caro.sh (planned)
                    Social sharing platform
                    Profile & dashboard
                    Guild communities
                    Privacy controls
                    Bluesky integration
```

---

## 🎯 Product Components

### 1. **Caro CLI** (Root Directory: `/`)

**Location:** Rust project in root directory
**Purpose:** Local shell companion that generates safe POSIX commands
**License:** AGPL-3.0

**Key Features:**
- Natural language → shell command generation
- Multiple LLM backends (MLX, Ollama, vLLM, remote APIs)
- Comprehensive safety validation
- Local telemetry collection (opt-in)
- Privacy-first design (no data sent without consent)

**Tech Stack:**
- **Language:** Rust (2021 edition)
- **CLI Framework:** clap
- **Async Runtime:** tokio
- **Backends:** MLX (Apple Silicon), Ollama, vLLM
- **Safety:** Custom validation engine

**Repository Structure:**
```
/
├── src/                # Rust source code
│   ├── main.rs
│   ├── backends/       # LLM backend integrations
│   ├── safety/         # Command validation
│   ├── cli/            # CLI interface
│   └── ...
├── Cargo.toml          # Rust dependencies
├── README.md           # Project README
└── CLAUDE.md           # Claude AI development guide
```

**Key Responsibilities:**
- Generate shell commands from natural language
- Validate command safety before execution
- Collect local telemetry (with user consent)
- Provide data to Web Hub for sharing (privacy-protected)
- Work 100% offline when needed

---

### 2. **Marketing Website** (Directory: `/website/`)

**Location:** `/website/` directory
**Purpose:** Marketing landing page and documentation hub
**URL:** [caro.sh](https://caro.sh)
**License:** AGPL-3.0

**Key Features:**
- Product introduction and value proposition
- Download/installation instructions
- Documentation portal
- Kyaro/Caro story and branding
- Community links

**Tech Stack:**
- **Framework:** Astro 4
- **Styling:** Scoped CSS (component-based)
- **Build:** Static Site Generation (SSG)
- **Deployment:** Vercel
- **Integrations:** Sitemap, Storybook

**Repository Structure:**
```
website/
├── src/
│   ├── components/     # Astro components
│   │   ├── Hero.astro
│   │   ├── Terminal.astro
│   │   ├── Features.astro
│   │   └── ...
│   ├── layouts/
│   │   └── Layout.astro
│   └── pages/
│       └── index.astro
├── public/             # Static assets
├── astro.config.mjs
└── README.md
```

**Design:**
- Warm color palette (#ff8c42, #ff6b35)
- Professional, friendly tone
- Kyaro/Caroline/GLaDOS narrative
- Feature showcase
- Video demonstrations

**Key Responsibilities:**
- Attract new users
- Explain product value
- Provide installation instructions
- Document core features
- Build brand awareness

---

### 3. **Web Hub** (Directory: `/apps/devrel/`)

**Location:** `/apps/devrel/` directory
**Purpose:** Social platform for sharing terminal expertise
**URL:** hub.caro.sh (planned)
**License:** AGPL-3.0

**Key Features:**
- User authentication (Bluesky)
- Profile dashboard (local CLI data)
- Social sharing (commands, wins, fails, runbooks)
- Professional guilds (SRE, AppSec, DevOps, etc.)
- Privacy-first telemetry dashboard
- PII/credential redaction
- Bluesky/AT Protocol integration

**Tech Stack:**
- **Framework:** Next.js 16 (App Router, Turbopack)
- **Styling:** Tailwind CSS 4
- **Language:** TypeScript
- **Social Protocol:** AT Protocol (Bluesky)
- **Auth:** Bluesky OAuth
- **Database:** TBD (Local-first with optional sync)
- **Deployment:** Vercel (static + edge functions)

**Repository Structure:**
```
apps/devrel/
├── app/
│   ├── globals.css
│   ├── layout.tsx
│   ├── page.tsx           # Home/landing
│   ├── profile/           # User dashboard
│   ├── share/             # Sharing interface
│   ├── guilds/            # Community pages
│   ├── telemetry/         # Privacy dashboard
│   └── auth/              # Authentication
├── components/
│   ├── Profile/
│   ├── Share/
│   ├── Telemetry/
│   └── ...
├── lib/
│   ├── atproto/           # Bluesky SDK integration
│   ├── caro-cli/          # CLI data interface
│   ├── privacy/           # Redaction engine
│   └── utils/
└── public/
    └── mascot/            # Kyaro sprites
```

**Design:**
- 8-bit pixel art aesthetic
- Game Boy + Pokémon era inspiration
- Terminal constraints (monospace, ANSI colors)
- Kyaro mascot with 11 emotional states
- Neon accent colors (#39ff14, #00f0ff, #ff10f0)

**Key Responsibilities:**
- Enable safe social sharing
- Build professional communities (guilds)
- Provide telemetry transparency
- Redact PII and credentials automatically
- Create knowledge runbooks
- Integrate with Bluesky network
- Gamification and engagement

---

## 🔗 Data Flow Architecture

### Local-First Privacy Model

```
┌──────────────────────────────────────────────────────────────────┐
│  User's Local Machine                                            │
│                                                                   │
│  ┌────────────┐                                                  │
│  │ Caro CLI   │  Generates commands, collects telemetry         │
│  │ (Rust)     │  Saves to local storage (~/.caro/)              │
│  └─────┬──────┘                                                  │
│        │                                                          │
│        │ (1) User opens web hub in browser                       │
│        ▼                                                          │
│  ┌────────────┐                                                  │
│  │ Web Hub    │  Reads local CLI data via file API              │
│  │ (Browser)  │  or local HTTP server                            │
│  └─────┬──────┘                                                  │
│        │                                                          │
│        │ (2) User creates share (command, win, fail)             │
│        ▼                                                          │
│  ┌────────────┐                                                  │
│  │ Privacy    │  Scans for PII, credentials, secrets            │
│  │ Redaction  │  User reviews and confirms redactions           │
│  │ Engine     │                                                  │
│  └─────┬──────┘                                                  │
│        │                                                          │
│        │ (3) User explicitly publishes                           │
└────────┼──────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────────┐
│  Bluesky Network (AT Protocol)                                 │
│                                                                 │
│  ┌────────────┐                                                │
│  │ AT Proto   │  Stores as custom record types                 │
│  │ Repository │  (app.caro.share.command, etc.)                │
│  └─────┬──────┘                                                │
│        │                                                        │
│        ▼                                                        │
│  ┌────────────┐                                                │
│  │ Feeds &    │  Guild feeds, discovery, search                │
│  │ Discovery  │  Community moderation                          │
│  └────────────┘                                                │
└────────────────────────────────────────────────────────────────┘
```

### Key Privacy Principles

1. **No network by default**: CLI works 100% offline
2. **Explicit consent**: User must explicitly publish each share
3. **Privacy review**: Dashboard shows all collected data before sharing
4. **Automatic redaction**: Scans for credentials, API keys, PII
5. **Granular control**: Share globally, per-guild, or keep private
6. **Right to delete**: Full control over published artifacts
7. **Open protocol**: Bluesky's AT Protocol (no vendor lock-in)

---

## 🗂️ Data Models

### 1. Command Artifact

Shared command with metadata:

```typescript
interface CommandArtifact {
  type: "command_artifact"
  prompt: string                    // Original natural language
  command: string                   // Generated shell command
  safety_score: "safe" | "moderate" | "high" | "critical"
  backend: string                   // "ollama:qwen2.5-coder"
  timestamp: string                 // ISO 8601
  tags: string[]                    // ["find", "filesystem"]
  guild?: string                    // Optional guild association
  author_did: string                // Bluesky DID
}
```

### 2. Win Story

Success narrative:

```typescript
interface WinStory {
  type: "win_story"
  title: string                     // Short headline
  story: string                     // Markdown narrative
  artifacts: string[]               // Related command artifact IDs
  impact: string                    // "Saved 8 hours"
  guild?: string
  timestamp: string
  author_did: string
}
```

### 3. Epic Fail

Issue report with logs:

```typescript
interface EpicFail {
  type: "epic_fail"
  prompt: string                    // What user asked for
  generated_command: string         // What Caro generated
  expected: string                  // What should have happened
  logs: string                      // [REDACTED] verbose logs
  severity: "low" | "medium" | "high" | "critical"
  backend: string
  timestamp: string
  author_did: string
}
```

### 4. Runbook

Operational knowledge:

```typescript
interface Runbook {
  type: "runbook"
  title: string                     // "How I Deploy to Prod"
  description: string
  steps: RunbookStep[]
  guild?: string
  tags: string[]
  timestamp: string
  author_did: string
}

interface RunbookStep {
  prompt: string                    // Natural language step
  command: string                   // Generated command
  notes?: string                    // Additional context
  order: number
}
```

### 5. User Profile

```typescript
interface UserProfile {
  did: string                       // Bluesky DID
  handle: string                    // @user.bsky.social
  display_name: string
  avatar?: string
  bio?: string
  guilds: string[]                  // Joined guilds
  stats: {
    commands_shared: number
    wins_posted: number
    fails_reported: number
    runbooks_created: number
  }
  settings: {
    default_visibility: "public" | "guild" | "private"
    auto_redact_paths: boolean
    auto_redact_env_vars: boolean
  }
}
```

### 6. Guild (Community)

```typescript
interface Guild {
  id: string                        // "sre", "appsec", "devops"
  name: string                      // "SRE Guild"
  description: string
  icon: string                      // Emoji or URL
  moderators: string[]              // DIDs
  member_count: number
  tags: string[]                    // Related topics
  feed_algorithm?: string           // Custom feed logic
}
```

---

## 🔐 Security Architecture

### PII Redaction Engine

**Regex Patterns:**
```typescript
const REDACTION_PATTERNS = {
  API_KEYS: /[a-zA-Z0-9]{32,}/,
  JWT_TOKENS: /eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+/,
  AWS_ACCESS_KEY: /AKIA[0-9A-Z]{16}/,
  SSH_PRIVATE_KEY: /-----BEGIN (RSA|OPENSSH) PRIVATE KEY-----/,
  EMAIL: /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/,
  IP_ADDRESS: /\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b/,
  HOME_PATH: /\/home\/[a-zA-Z0-9_-]+/,
  ENV_VARS: /\$[A-Z_]+_?(SECRET|TOKEN|KEY|PASSWORD)/,
}
```

**Redaction Workflow:**
1. User creates share (command, win, fail, runbook)
2. Privacy engine scans content with regex patterns
3. Detected items highlighted in UI (red/yellow)
4. User reviews and confirms redactions
5. System applies replacements: `[REDACTED_API_KEY]`, `[REDACTED_PATH]`
6. User previews final content
7. User explicitly publishes (or cancels)

**Redaction UI:**
```typescript
<PrivacyReview artifact={draft}>
  <DetectedItem
    type="api_key"
    value="sk_live_abcd1234..."
    replacement="[REDACTED_API_KEY]"
    action="redact" // or "keep" (dangerous)
  />
  <DetectedItem
    type="home_path"
    value="/home/kobi"
    replacement="/home/[USER]"
    action="redact"
  />
</PrivacyReview>
```

---

## 🌐 Bluesky/AT Protocol Integration

### Authentication Flow

```
1. User clicks "Login with Bluesky"
2. Web Hub redirects to Bluesky OAuth
3. User authorizes app permissions
4. Bluesky returns access token + DID
5. Web Hub stores token (encrypted localStorage)
6. User is logged in, profile loaded
```

### Custom Record Types

Caro uses custom Lexicons for artifact types:

```
app.caro.share.command         # Command artifacts
app.caro.share.win             # Win stories
app.caro.share.fail            # Epic fails
app.caro.share.runbook         # Runbooks
app.caro.guild.membership      # Guild associations
app.caro.profile.preferences   # User settings
```

### Feed Algorithms

**Guild Feeds:**
- Filter records by guild tag
- Sort by recency or engagement
- Custom algorithms per guild

**Discovery Feeds:**
- "Trending Commands" (most liked/shared)
- "Recent Wins" (success stories)
- "Top Contributors" (leaderboard)
- "Your Guilds" (personalized feed)

---

## 🎮 Gamification System

### Achievement Examples

```typescript
const ACHIEVEMENTS = {
  FIRST_SHARE: {
    name: "First Share",
    description: "Shared your first command",
    icon: "🎉",
    kyaro_state: "success"
  },
  SAFETY_GUARDIAN: {
    name: "Safety Guardian",
    description: "Reported 10 dangerous commands",
    icon: "🛡️",
    kyaro_state: "warning"
  },
  RUNBOOK_AUTHOR: {
    name: "Runbook Author",
    description: "Created 5 comprehensive runbooks",
    icon: "📚",
    kyaro_state: "thinking"
  },
  GUILD_LEADER: {
    name: "Guild Leader",
    description: "Top contributor in a guild",
    icon: "👑",
    kyaro_state: "greeting"
  }
}
```

### Reputation System

```typescript
interface Reputation {
  total_points: number
  breakdown: {
    commands_shared: number        // +1 per share
    wins_posted: number            // +5 per win
    fails_reported: number         // +3 per fail
    runbooks_created: number       // +10 per runbook
    helpful_votes: number          // +2 per upvote
    guild_contributions: number    // +5 per guild post
  }
  level: number                    // 1-100
  badges: string[]                 // Achievement IDs
}
```

---

## 📱 Future Integrations

### Planned Extensions

1. **Mobile App (React Native)**
   - View guild feeds on mobile
   - Quick command sharing from terminal (via SSH)
   - Push notifications for guild activity

2. **Browser Extension**
   - One-click sharing from terminal emulators
   - In-browser command preview
   - Quick access to runbooks

3. **Desktop App (Tauri)**
   - Native integration with local CLI
   - System tray for quick access
   - Offline-first functionality

4. **VS Code Extension**
   - Share commands from integrated terminal
   - Browse guild runbooks
   - Command snippet library

5. **Slack/Discord Integration**
   - Post artifacts to team channels
   - Guild announcements
   - Weekly digest of top commands

---

## 🚀 Deployment Architecture

### Production Infrastructure

```
┌─────────────────────────────────────────────────────────────┐
│  Vercel Edge Network                                        │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │ caro.sh      │         │ hub.caro.sh  │                 │
│  │ (Astro SSG)  │         │ (Next.js)    │                 │
│  │              │         │              │                 │
│  │ Marketing    │         │ • Static SSG │                 │
│  │ Landing Page │         │ • Edge Funcs │                 │
│  └──────────────┘         │ • ISR        │                 │
│                           └──────────────┘                 │
└─────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                       ┌────────────────────────┐
                       │ Bluesky PDS (Personal  │
                       │ Data Server)           │
                       │                        │
                       │ User data & artifacts  │
                       └────────────────────────┘
```

### Build Process

**Marketing Site (website/):**
```bash
cd website
npm run build          # Astro builds to dist/
vercel deploy --prod   # Deploy static site
```

**Web Hub (apps/devrel/):**
```bash
cd apps/devrel
npm run build          # Next.js builds to .next/
vercel deploy --prod   # Deploy with edge functions
```

### Environment Variables

**Web Hub (.env.production):**
```env
NEXT_PUBLIC_BSKY_API=https://bsky.social/xrpc
NEXT_PUBLIC_HUB_URL=https://hub.caro.sh
BSKY_CLIENT_ID=caro-web-hub
BSKY_CLIENT_SECRET=[encrypted]
```

---

## 📊 Monitoring & Analytics

### Metrics to Track

**CLI (Local):**
- Commands generated per day
- Safety validation triggers
- Backend usage (MLX vs Ollama vs vLLM)
- Error rates

**Web Hub (Hub):**
- Active users (DAU/MAU)
- Shares per day (commands, wins, fails, runbooks)
- Guild membership growth
- Privacy redactions triggered
- Authentication success rate

**Marketing Site:**
- Page views
- Download conversions
- Time on site
- Bounce rate

### Privacy-Compliant Analytics

- **No tracking cookies** (use server-side analytics only)
- **No third-party analytics** (self-hosted Plausible or similar)
- **Aggregate data only** (no individual user tracking)
- **GDPR/CCPA compliant**

---

## 🔄 Development Workflow

### Monorepo Structure

```
/                              # Caro CLI (Rust)
├── src/
├── Cargo.toml
├── website/                   # Marketing (Astro)
│   ├── src/
│   └── package.json
└── apps/
    └── devrel/                # Web Hub (Next.js)
        ├── app/
        └── package.json
```

### Independent Deployments

Each component deploys independently:

1. **CLI (Rust)**:
   - CI: GitHub Actions
   - Build: `cargo build --release`
   - Publish: GitHub Releases + crates.io
   - Platforms: macOS, Linux, Windows binaries

2. **Marketing (website/)**:
   - CI: Vercel (auto-deploy on push to main)
   - Build: `npm run build` (Astro SSG)
   - Deploy: Vercel (caro.sh)

3. **Web Hub (apps/devrel/)**:
   - CI: Vercel (auto-deploy on push to main)
   - Build: `npm run build` (Next.js)
   - Deploy: Vercel (hub.caro.sh)

### Branch Strategy

```
main           # Production (all components)
├── develop    # Integration branch
├── feature/*  # Feature branches
└── hotfix/*   # Emergency fixes
```

### Pull Request Workflow

1. Create feature branch from `develop`
2. Make changes (CLI, website, or web hub)
3. Run local tests: `cargo test` or `npm test`
4. CI runs on PR:
   - Rust: clippy, tests, build
   - Website: build check
   - Web Hub: lint, type-check, build, accessibility
5. Review and merge to `develop`
6. Periodically merge `develop` → `main` for releases

---

## 📚 Documentation Structure

### For Developers

| Document | Audience | Location |
|----------|----------|----------|
| **README.md** | All users | Root `/` |
| **CLAUDE.md** | Claude AI | Root `/` |
| **website/README.md** | Marketing contributors | `/website/` |
| **apps/devrel/README.md** | Web hub developers | `/apps/devrel/` |
| **ARCHITECTURE.md** (this) | System architects | `/apps/devrel/` |
| **BRAND_IDENTITY.md** | Design team | `/apps/devrel/` |
| **KYARO_SPRITE_GUIDE.md** | Aci (artist) | `/apps/devrel/` |
| **PRIVACY_MODEL.md** | Security/privacy | `/apps/devrel/` |
| **SOCIAL_FEATURES.md** | Product managers | `/apps/devrel/` |

### For Users

| Resource | Purpose | URL |
|----------|---------|-----|
| Product landing | Introduction | caro.sh |
| Installation guide | Setup | caro.sh/download |
| CLI documentation | Usage | caro.sh/docs |
| Web hub guide | Social features | hub.caro.sh/help |
| Privacy policy | Data practices | caro.sh/privacy |
| Community guidelines | Conduct | hub.caro.sh/guidelines |

---

## 🎯 Success Metrics

### CLI Success

- ✅ 10,000+ installs in first 6 months
- ✅ < 1% error rate in command generation
- ✅ 90%+ user satisfaction (safety validation)
- ✅ Active contributors on GitHub

### Web Hub Success

- ✅ 1,000+ registered users in first 3 months
- ✅ 100+ active guilds
- ✅ 10,000+ shared artifacts (commands, wins, fails, runbooks)
- ✅ < 0.1% privacy incidents (leaked credentials)
- ✅ 80%+ user retention (30-day)

### Marketing Success

- ✅ 50,000+ monthly visitors to caro.sh
- ✅ 20%+ conversion to CLI install
- ✅ Top 3 in search results for "AI shell assistant"
- ✅ Featured in tech media (HackerNews, DevTo, etc.)

---

## 🔮 Long-Term Vision

### Caro as an AI Companion Ecosystem

1. **CLI** - Your local terminal companion
2. **Web Hub** - Your social knowledge sharing platform
3. **Mobile** - Your on-the-go command reference
4. **Desktop** - Your always-available assistant
5. **Extensions** - Integrated into your workflow

### Federated Network of Guilds

- SRE teams share runbooks across companies
- AppSec professionals collaborate on security patterns
- DevOps engineers build collective knowledge
- Open source maintainers document workflows
- **All privacy-protected, all user-controlled**

### Kyaro Evolution

As users engage more with Caro:
- Kyaro evolves through different stages
- Unlocks new sprite states and animations
- Gains new personality traits
- Becomes more personalized to user's workflow

**The ultimate goal**: Make terminal expertise accessible, shareable, and safe for everyone.

---

**Questions about the architecture?** Open an issue in the [Caro repository](https://github.com/wildcard/caro).

**Let's build the future of terminal collaboration! 🐕🚀**
