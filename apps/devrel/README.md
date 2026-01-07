# Caro Web Hub

> 🐕 **Your Loyal Shell Companion's Web Home**

The web frontend for **Caro** - a privacy-first social platform where developers share their terminal expertise, command discoveries, and operational knowledge safely.

---

## 🌟 What is Caro Web Hub?

Caro Web Hub is the counterpart to the Caro CLI - a **safe space** for developers to:

- **Share their wins**: Generated commands, "aha moments," and success stories
- **Report issues**: Epic fails with verbose logs for improvement
- **Build runbooks**: Document workflows in natural language
- **Join guilds**: Connect with professional communities (SRE, AppSec, DevOps)
- **Control privacy**: Review and redact sensitive data before sharing
- **Create knowledge**: Share dotfiles, customizations, and expertise
- **Stay safe**: Caro double-checks all shares for sensitive data

**Built on [Bluesky's AT Protocol](https://atproto.com/)** - An open, federated social network that respects user autonomy and data ownership.

---

## 🎯 Core Features

### 1. **Profile & Local Agent Management**
- Interact with your Caro CLI profile and configuration
- Manage local agent settings safely
- View command history and statistics
- **No data leaves your machine without explicit consent**

### 2. **Social Sharing Platform**
- **Command Artifacts**: Share generated commands with context
- **Win Stories**: Celebrate successful automations and discoveries
- **Epic Fails**: Report issues with detailed logs (privacy-protected)
- **Runbooks**: Document operational procedures in natural language
- **Dotfiles & Configs**: Share your terminal setup safely

### 3. **Privacy-First Telemetry Dashboard**
- Clear visualization of locally collected data
- **PII/Sensitive Data Redaction** by Caro before sharing
- Granular control: global or per-group sharing
- Full transparency: see exactly what you shared and what stayed private
- **User always in control** - Caro asks permission, never assumes

### 4. **Professional Guilds** (Like Subreddits for Terminals)
- **SRE Guild**: Site Reliability Engineering workflows
- **AppSec Guild**: Application Security practices
- **DevOps Guild**: Automation and infrastructure
- **Custom Communities**: Create your own specialized groups

### 5. **Safety & Trust**
- Caro actively scans for credentials, API keys, and secrets before sharing
- Double-confirmation for potentially dangerous shares
- Community moderation and trust systems
- **Always watching, always protecting** - That's Caro's promise

### 6. **Modern, Fun, Safe**
- 8-bit pixel art aesthetic (Game Boy + Pokémon era)
- Terminal-first design constraints (ANSI, monospace, box drawing)
- Kyaro mascot with 11 emotional states
- Retro gaming nostalgia meets modern web technology

---

## 🏗️ Architecture

### Two Websites, One Product

| Site | Purpose | Tech | URL (Planned) |
|------|---------|------|---------------|
| **website/** | Marketing landing page | Astro | caro.sh |
| **apps/devrel/** | Web hub/frontend | Next.js 16 | hub.caro.sh |

**This project is `apps/devrel/`** - the web hub where users interact with their Caro CLI data and community.

### Tech Stack

- **Framework:** [Next.js 16](https://nextjs.org/) (App Router, Turbopack)
- **Styling:** [Tailwind CSS 4](https://tailwindcss.com/)
- **Language:** TypeScript
- **Social Protocol:** [AT Protocol](https://atproto.com/) (Bluesky)
- **Authentication:** (TBD - Likely Bluesky auth)
- **Database:** (TBD - Local-first with optional cloud sync)
- **Deployment:** Static export + Edge functions (Vercel)

### Design Philosophy

**Terminal Constraints:**
- ✅ Monospace fonts only
- ✅ ANSI 16-color palette
- ✅ Box drawing characters (┌ ┐ └ ┘ │ ─)
- ✅ ASCII fallbacks for limited terminals
- ❌ No gradients, photos, shadows, blur

**Privacy-First:**
- Data stays local by default
- User controls every share
- Caro redacts PII automatically
- Transparent telemetry dashboard
- Bluesky's decentralized model (no corporate lock-in)

**8-Bit Aesthetic:**
- Game Boy color palette (#0f380f to #9bbc0f)
- Neon accents (#39ff14, #00f0ff, #ff10f0)
- Kyaro mascot (Shiba-inspired Pokémon companion)
- Retro animations (sprite bounce, scanlines, CRT effects)

---

## 📁 Project Structure

```
apps/devrel/
├── app/
│   ├── globals.css          # 8-bit design system & theme
│   ├── layout.tsx           # Root layout with metadata
│   ├── page.tsx             # Landing/home page
│   ├── profile/             # User profile pages
│   ├── share/               # Sharing interface
│   ├── guilds/              # Community groups
│   ├── telemetry/           # Privacy dashboard
│   └── auth/                # Authentication flows
├── components/
│   ├── Hero.tsx             # Hero section with Kyaro
│   ├── Features.tsx         # Feature showcase
│   ├── Navigation.tsx       # Top navigation
│   ├── Footer.tsx           # Site footer
│   ├── TerminalWindow.tsx   # Terminal display
│   ├── Profile/             # Profile components
│   ├── Share/               # Sharing UI components
│   ├── Telemetry/           # Dashboard components
│   └── index.ts             # Component exports
├── lib/
│   ├── atproto/             # Bluesky/AT Protocol integration
│   ├── caro-cli/            # Caro CLI data interface
│   ├── privacy/             # PII redaction engine
│   └── utils/               # Shared utilities
├── public/
│   └── mascot/              # Kyaro sprite sheets
├── BRAND_IDENTITY.md        # Brand canvas & vision
├── KYARO_SPRITE_GUIDE.md    # Mascot design guide
├── TERMINAL_CONSTRAINTS.md  # UI technical requirements
├── ARCHITECTURE.md          # System architecture (NEW)
├── PRIVACY_MODEL.md         # Privacy & security design (NEW)
├── SOCIAL_FEATURES.md       # Social platform spec (NEW)
└── README.md                # This file
```

---

## 🛠️ Development

### Prerequisites

- **Node.js** 18+ (v22.21.1 recommended)
- **npm** 10+
- **Caro CLI** (optional, for local agent integration)

### Getting Started

```bash
# Navigate to the web hub directory
cd apps/devrel

# Install dependencies
npm install

# Start development server
npm run dev

# Open in browser
# Visit: http://localhost:3000
```

### Available Scripts

```bash
# Development server with hot reload
npm run dev

# Production build
npm run build

# Start production server
npm run start

# Lint code
npm run lint

# Type check
npm run type-check

# Run tests
npm run test
```

---

## 🎨 Design System

See comprehensive documentation:
- **[BRAND_IDENTITY.md](./BRAND_IDENTITY.md)** - Project canvas, vision, Kyaro mascot
- **[KYARO_SPRITE_GUIDE.md](./KYARO_SPRITE_GUIDE.md)** - 11 sprite states for Aci
- **[TERMINAL_CONSTRAINTS.md](./TERMINAL_CONSTRAINTS.md)** - Technical UI requirements
- **[DESIGN_GUIDELINES.md](./DESIGN_GUIDELINES.md)** - Complete design system

### Color Palette

**Backgrounds:**
- `#0f0f23` - Deep dark blue-black (primary)
- `#1a1a2e` - Dark navy (secondary)
- `#16213e` - Mid dark blue (tertiary)

**Neon Accents:**
- `#39ff14` - Electric green (primary)
- `#00f0ff` - Cyan
- `#ff10f0` - Magenta
- `#bf00ff` - Purple

**Terminal Colors:**
- `#00ff41` - Matrix green
- `#ffb000` - Warning amber
- `#ff3b3b` - Error red

**Game Boy Greens:**
- `#0f380f` to `#9bbc0f` - Classic Game Boy palette

---

## 🔐 Privacy & Security

### Data Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Caro CLI   │────▶│  Local Web   │────▶│  Bluesky    │
│  (Local)    │     │  Hub         │     │  Network    │
│             │     │  (Browser)   │     │  (Public)   │
└─────────────┘     └──────────────┘     └─────────────┘
      │                    │                     │
      │                    │                     │
      ▼                    ▼                     ▼
  No network        User reviews &        Only what user
  by default        redacts PII           explicitly shared
```

### Privacy Principles

1. **Local-First**: All data stays on your machine unless you share
2. **User Consent**: Explicit permission required for every share
3. **PII Redaction**: Caro automatically detects and redacts sensitive data
4. **Transparency**: Dashboard shows exactly what was collected and shared
5. **Granular Control**: Share globally, per-guild, or keep private
6. **Right to Delete**: Full control over your shared artifacts
7. **Open Protocol**: Bluesky's AT Protocol means no vendor lock-in

### Security Measures

- **Credential Detection**: API keys, passwords, tokens, SSH keys
- **Path Sanitization**: Remove usernames and home directory paths
- **Environment Variable Filtering**: Redact `$SECRET_*`, `$TOKEN_*`, etc.
- **Double-Confirmation**: High-risk shares require additional consent
- **Community Moderation**: Guild moderators and trust systems

**See [PRIVACY_MODEL.md](./PRIVACY_MODEL.md) for complete specification** (Coming Soon)

---

## 🌐 Bluesky Integration

### Why Bluesky/AT Protocol?

- **Open & Decentralized**: No single company controls your data
- **Portable Identity**: Bring your identity and data anywhere
- **Algorithmic Choice**: You control what you see
- **Developer-Friendly**: Well-documented APIs and SDKs
- **Community-Driven**: Similar ethos to open source

### Integration Plan

1. **Phase 1**: Use Bluesky auth for user accounts
2. **Phase 2**: Store artifacts as custom record types
3. **Phase 3**: Enable cross-platform discovery and sharing
4. **Phase 4**: Custom feeds for guilds and topics
5. **Phase 5**: Federation with other AT Protocol apps

**See [BLUESKY_INTEGRATION.md](./BLUESKY_INTEGRATION.md) for technical details** (Coming Soon)

---

## 🎮 Social Features

### Command Artifacts

Share your generated commands with the community:

```typescript
{
  type: "command_artifact",
  prompt: "find all files modified in the last 24 hours",
  command: "find . -type f -mtime -1",
  safety_score: "safe",
  backend: "ollama:qwen2.5-coder",
  timestamp: "2026-01-01T10:30:00Z",
  tags: ["find", "filesystem", "linux"],
  guild: "sre"
}
```

### Win Stories

Celebrate your successes:

```typescript
{
  type: "win_story",
  title: "Automated 200 server audits in 5 minutes",
  story: "Used Caro to generate a parallel SSH script...",
  artifacts: ["command_1", "command_2"],
  impact: "Saved 8 hours of manual work",
  guild: "devops"
}
```

### Epic Fails (Issue Reports)

Help improve Caro:

```typescript
{
  type: "epic_fail",
  prompt: "backup database",
  generated_command: "rm -rf /var/lib/postgresql",
  expected: "pg_dump ...",
  logs: "[REDACTED verbose logs]",
  severity: "critical",
  backend: "mlx:qwen2.5-coder-1.5b"
}
```

### Runbooks

Document your operational knowledge:

```typescript
{
  type: "runbook",
  title: "How I Deploy to Production",
  steps: [
    { prompt: "check if tests pass", command: "npm test" },
    { prompt: "build for production", command: "npm run build" },
    { prompt: "deploy to vercel", command: "vercel --prod" }
  ],
  guild: "frontend",
  tags: ["deployment", "ci-cd"]
}
```

**See [SOCIAL_FEATURES.md](./SOCIAL_FEATURES.md) for complete specification** (Coming Soon)

---

## 👥 For the Team

### Aci/Alrezky (Art Director)

**Priority 1: Kyaro Sprite Sheets**

See **[KYARO_SPRITE_GUIDE.md](./KYARO_SPRITE_GUIDE.md)** for complete design specs.

**11 Essential States:**
1. Idle - Resting, waiting
2. Waiting - Anticipating input
3. Listening - Actively processing user input
4. Thinking - Deep processing (LLM inference)
5. Success - Happy, celebrating command success
6. Warning - Cautious about dangerous command
7. Error - Sad, command failed
8. Bored - User taking too long
9. Long-inference - Patience during heavy computation
10. Greeting - Welcoming new users
11. Farewell - Goodbye animation

**Deliverables:**
- 16×16px (compact terminals)
- 32×32px (standard size)
- 64×64px (retro but detailed)
- 128×128px (large hero)
- 256×256px (maximum detail)

**File Locations:**
```
/public/mascot/kyaro/
├── idle/
│   ├── 16x16.png
│   ├── 32x32.png
│   └── ...
├── thinking/
├── success/
└── ...
```

### Sulo (Frontend Dev & UI/UX)

**Priority 1: Core User Flows**

1. **Profile Dashboard**
   - View local Caro CLI data
   - Command history and statistics
   - Privacy settings
   - Telemetry transparency

2. **Sharing Interface**
   - Create new artifact (command, win, fail, runbook)
   - Review and redact sensitive data
   - Select visibility (global, guild-specific, private)
   - Publish to Bluesky

3. **Guild Discovery**
   - Browse professional communities
   - Join/leave guilds
   - View guild feeds
   - Guild-specific sharing

4. **Telemetry Dashboard**
   - Visualize collected data
   - Review shared vs. private artifacts
   - Manage data deletion
   - Export data

**See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guide**

---

## 🚀 Deployment

**For complete deployment instructions, see [DEPLOYMENT.md](./DEPLOYMENT.md)**

### Recommended Platform: Vercel

```bash
# Install Vercel CLI
npm install -g vercel

# Login
vercel login

# Deploy
cd apps/devrel
vercel --prod
```

### Planned Domains

- **Marketing**: caro.sh (website/ directory)
- **Web Hub**: hub.caro.sh (this project)
- **Storybook**: storybook.caro.sh (design system)

### CI/CD

GitHub Actions automatically:
- ✅ Builds and tests on every PR
- ✅ Checks accessibility and bundle size
- ✅ Verifies monorepo separation
- ✅ Runs security scans

See `.github/workflows/devrel-website.yml` for details.

---

## 📝 Roadmap

### Phase 1: Foundation (Current)
- [x] 8-bit design system
- [x] Kyaro mascot design guide
- [x] Terminal constraints documentation
- [x] Brand identity canvas
- [ ] Architecture documentation
- [ ] Privacy model specification
- [ ] Social features specification

### Phase 2: Core Features
- [ ] User authentication (Bluesky)
- [ ] Profile dashboard
- [ ] Command artifact sharing
- [ ] Basic privacy redaction
- [ ] Guild discovery

### Phase 3: Social Platform
- [ ] Win stories
- [ ] Epic fails (issue reporting)
- [ ] Runbook creation
- [ ] Guild feeds and communities
- [ ] Advanced privacy controls

### Phase 4: Gamification
- [ ] Achievement system
- [ ] Reputation and trust scores
- [ ] Command challenges
- [ ] Community events
- [ ] Kyaro evolution states

### Phase 5: Advanced Features
- [ ] Custom feeds
- [ ] Federation with other AT Protocol apps
- [ ] Mobile app (React Native)
- [ ] Browser extension
- [ ] Desktop app (Tauri)

---

## 🤝 Team

- **Product Vision:** Kobi Kadosh (Maintainer)
- **Brand & Art Direction:** Aci/Alrezky (8-bit Illustrator & Animator)
- **UI/UX Development:** Sulo (Frontend Dev & UI/UX Designer)
- **Implementation:** Claude AI Assistant
- **Community:** Caro Open Source Contributors

---

## 📄 License

This web hub is part of the Caro project, licensed under **AGPL-3.0**.

---

## 🐕 About Kyaro & Caro

**Kyaro** is Kobi Kadosh's beloved Shiba Inu - a loyal, protective companion.

**Caro** is the digitalization of Kyaro - a shell companion that embodies the same loyalty, protection, and companionship. Inspired by Portal's Caroline → GLaDOS transformation, but with warmth and trust instead of dark humor.

Caro's promise: **"Always watching, always protecting, always got your back."**

---

**Questions or feedback?** Open an issue in the main [Caro repository](https://github.com/wildcard/caro).

**Let's build the safest, friendliest terminal community together! 🐕✨**
