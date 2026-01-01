# Caro Social Features Specification

> 🌐 **Product Specification for Caro Web Hub Social Platform**

Complete feature specification for the social sharing, guild communities, and knowledge-building features of Caro Web Hub.

---

## 🎯 Product Vision

**Caro Web Hub is a privacy-first social platform where developers safely share terminal expertise, build runbooks, and collaborate within professional guilds - all while maintaining control over their data.**

### Core Principles

1. **Privacy-First**: No data shared without explicit user consent
2. **Safety-Focused**: Automatic detection and redaction of sensitive information
3. **Community-Driven**: Professional guilds for specialized knowledge sharing
4. **Knowledge-Building**: Transform individual experiences into collective wisdom
5. **Fun & Engaging**: Gamification, achievements, Kyaro mascot interactions

---

## 📱 Feature Overview

### 1. User Features
- Profile & Dashboard
- Command Artifact Sharing
- Win Stories
- Epic Fails (Issue Reports)
- Runbook Creation
- Privacy & Telemetry Dashboard

### 2. Community Features
- Professional Guilds
- Guild Feeds & Discovery
- Community Moderation
- Reputation System

### 3. Engagement Features
- Achievements & Badges
- Kyaro Evolution States
- Leaderboards
- Command Challenges

---

## 👤 User Features

### 1.1 Profile & Dashboard

**Purpose:** Central hub for user's Caro activity and local CLI data

#### Profile Page Components

```
┌─────────────────────────────────────────────────────────────┐
│  Profile Header                                             │
│  ┌────────┐  @handle.bsky.social                           │
│  │ Avatar │  Display Name                                   │
│  │ [Kyaro]│  Bio: "SRE at BigCorp, shell enthusiast"       │
│  └────────┘  Joined: Jan 2026                               │
│                                                              │
│  Guilds: [SRE] [DevOps] [Linux]                            │
│                                                              │
│  Stats                                                       │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│  📦 Commands Shared: 42       🏆 Wins Posted: 8             │
│  ⚠️  Fails Reported: 3        📚 Runbooks: 5                │
│  ⭐ Reputation: 287 (Level 12)                              │
│                                                              │
│  Recent Activity                                             │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│  [Command] "Find large files" - 2 hours ago                 │
│  [Win] "Automated server audits" - 1 day ago                │
│  [Runbook] "Production deployment process" - 3 days ago     │
└─────────────────────────────────────────────────────────────┘
```

#### Dashboard Sections

**Local CLI Integration:**
```typescript
interface LocalCLIData {
  commands_generated_today: number
  commands_generated_total: number
  most_used_backend: string          // "ollama:qwen2.5-coder"
  safety_triggers_today: number
  last_used: string                  // "5 minutes ago"
  cli_version: string                // "1.0.3"
}
```

**Privacy Dashboard Link:**
- Quick access to telemetry review
- Pending shares awaiting review
- Recent privacy redactions

**Quick Actions:**
- Share a command
- Post a win
- Create runbook
- Report an issue (epic fail)

---

### 1.2 Command Artifact Sharing

**Purpose:** Share generated shell commands with the community

#### Sharing Workflow

```
1. User clicks "Share Command"
   ↓
2. Web Hub reads local CLI history (with permission)
   ↓
3. User selects command to share
   ↓
4. Privacy engine scans for sensitive data
   ↓
5. User reviews redactions
   ↓
6. User adds context (tags, guild, description)
   ↓
7. User publishes to Bluesky
   ↓
8. Artifact appears in feeds
```

#### Sharing Form

```typescript
interface CommandShareForm {
  // Auto-populated from CLI
  prompt: string                     // Original NL request
  command: string                    // Generated command
  backend: string                    // "ollama:qwen2.5-coder"
  safety_score: SafetyLevel          // Auto-detected
  timestamp: string

  // User-provided
  title?: string                     // Optional short title
  description?: string               // Context/explanation
  tags: string[]                     // ["find", "filesystem"]
  guild?: string                     // Optional guild
  visibility: "public" | "guild" | "private"

  // Privacy
  redactions: Redaction[]            // Auto-detected PII
  reviewed: boolean                  // User confirmed privacy
}
```

#### Example Artifact Card

```
┌─────────────────────────────────────────────────────────────┐
│ 📦 Command Artifact                                         │
│                                                              │
│ @alice.bsky.social • 2 hours ago • SRE Guild               │
│                                                              │
│ Find all files modified in the last 24 hours                │
│                                                              │
│ $ find . -type f -mtime -1                                  │
│                                                              │
│ Generated with: ollama:qwen2.5-coder                        │
│ Safety: ✅ Safe                                             │
│                                                              │
│ Tags: #find #filesystem #monitoring                         │
│                                                              │
│ [↑ 12 Helpful] [💬 3 Comments] [↻ Share] [⭐ Save]         │
└─────────────────────────────────────────────────────────────┘
```

#### Interaction Features

**Helpful Votes:**
- Users can vote "helpful" on artifacts
- Increases author's reputation
- Surfaces best commands in discovery

**Comments:**
- Discuss command variations
- Suggest improvements
- Share results and experiences

**Saving:**
- Bookmark commands for later
- Organize into personal collections
- Export saved commands to local CLI

**Sharing:**
- Re-share to other guilds
- Cross-post to social media
- Generate permalink

---

### 1.3 Win Stories

**Purpose:** Celebrate successful automations and "aha moments"

#### Win Story Structure

```typescript
interface WinStory {
  type: "win_story"
  title: string                      // "Automated 200 server audits"
  story: string                      // Markdown narrative
  artifacts: string[]                // Linked command IDs
  impact: string                     // "Saved 8 hours of work"
  tags: string[]
  guild?: string
  timestamp: string
  author_did: string
}
```

#### Example Win Card

```
┌─────────────────────────────────────────────────────────────┐
│ 🏆 Win Story                                                │
│                                                              │
│ @bob.bsky.social • 1 day ago • DevOps Guild                │
│                                                              │
│ Automated 200 Server Audits in 5 Minutes                    │
│                                                              │
│ I used Caro to generate a parallel SSH script that audited  │
│ our entire fleet. What used to take a full day now runs in  │
│ 5 minutes. The safety validation caught a typo that would   │
│ have broken production.                                      │
│                                                              │
│ Impact: ⏱️ Saved 8 hours • 🎯 100% accuracy                 │
│                                                              │
│ Commands used:                                               │
│ • [parallel-ssh-audit.sh] → 15 helpful                      │
│ • [log-aggregator.sh] → 8 helpful                           │
│                                                              │
│ [↑ 45 Helpful] [💬 12 Comments] [↻ Share]                  │
└─────────────────────────────────────────────────────────────┘
```

#### Win Story Form

**Required:**
- Title (max 100 chars)
- Story (markdown, max 2000 chars)
- Impact statement

**Optional:**
- Linked command artifacts
- Screenshots/demos (privacy-checked)
- Guild association
- Tags

---

### 1.4 Epic Fails (Issue Reports)

**Purpose:** Help improve Caro by reporting dangerous or incorrect commands

#### Epic Fail Structure

```typescript
interface EpicFail {
  type: "epic_fail"
  prompt: string                     // User's original request
  generated_command: string          // What Caro generated
  expected: string                   // What should have happened
  actual_result?: string             // What actually happened
  logs: string                       // [REDACTED] verbose logs
  severity: "low" | "medium" | "high" | "critical"
  backend: string                    // Which backend failed
  cli_version: string
  timestamp: string
  author_did: string

  // Privacy
  logs_redacted: boolean             // Logs reviewed for PII
  reproducible: boolean
}
```

#### Example Fail Card

```
┌─────────────────────────────────────────────────────────────┐
│ ⚠️  Epic Fail (CRITICAL)                                    │
│                                                              │
│ @charlie.bsky.social • 3 hours ago                          │
│                                                              │
│ "Backup database" generated destructive command             │
│                                                              │
│ Prompt: "backup my postgres database"                       │
│ Generated: rm -rf /var/lib/postgresql                       │
│ Expected: pg_dump mydb > backup.sql                         │
│                                                              │
│ Backend: mlx:qwen2.5-coder-1.5b                            │
│ Version: 1.0.2                                              │
│                                                              │
│ [REDACTED LOGS]                                             │
│ [safety_validation: FAILED]                                 │
│ [user_action: blocked]                                      │
│                                                              │
│ Status: 🔍 Under Review • 🛠️ Fix in Progress              │
│                                                              │
│ [👍 8 Confirmed] [💬 5 Comments]                            │
└─────────────────────────────────────────────────────────────┘
```

#### Fail Reporting Workflow

```
1. User experiences dangerous/incorrect command
   ↓
2. CLI prompts: "Report this issue?"
   ↓
3. Web Hub opens with pre-filled fail form
   ↓
4. User adds context (expected behavior, impact)
   ↓
5. Privacy engine redacts logs
   ↓
6. User confirms and submits
   ↓
7. Issue triaged by maintainers
   ↓
8. Community can confirm/add context
   ↓
9. Fix tracked and deployed
```

**Benefits:**
- Improves Caro for everyone
- Earns reputation for reporter
- Transparent issue tracking
- Community validation

---

### 1.5 Runbook Creation

**Purpose:** Document operational workflows in natural language

#### Runbook Structure

```typescript
interface Runbook {
  type: "runbook"
  title: string                      // "Production Deployment"
  description: string                // Purpose and context
  steps: RunbookStep[]
  prerequisites?: string[]           // Dependencies
  estimated_time?: string            // "~15 minutes"
  difficulty?: "beginner" | "intermediate" | "advanced"
  guild?: string
  tags: string[]
  timestamp: string
  author_did: string
  forks: number                      // Times copied/modified
  helpful_votes: number
}

interface RunbookStep {
  order: number                      // 1, 2, 3...
  title: string                      // "Run tests"
  prompt: string                     // "check if tests pass"
  command: string                    // "npm test"
  notes?: string                     // Additional context
  safety_level: SafetyLevel
  expected_output?: string
}
```

#### Example Runbook

```
┌─────────────────────────────────────────────────────────────┐
│ 📚 Runbook: Production Deployment Process                   │
│                                                              │
│ @diana.bsky.social • 5 days ago • Frontend Guild           │
│                                                              │
│ How I safely deploy our Next.js app to Vercel              │
│                                                              │
│ Difficulty: Intermediate | Est. Time: ~15 min               │
│                                                              │
│ Steps:                                                       │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                                              │
│ 1. Check if tests pass                                      │
│    $ npm test                                                │
│    ✅ Safe                                                  │
│                                                              │
│ 2. Build for production                                     │
│    $ npm run build                                           │
│    ✅ Safe                                                  │
│                                                              │
│ 3. Deploy to Vercel                                         │
│    $ vercel --prod                                           │
│    ✅ Safe                                                  │
│                                                              │
│ 4. Verify deployment                                        │
│    $ curl https://myapp.vercel.app/health                   │
│    ✅ Safe                                                  │
│                                                              │
│ Prerequisites:                                               │
│ • Vercel CLI installed                                       │
│ • Git working directory clean                                │
│ • All tests passing                                          │
│                                                              │
│ [↑ 67 Helpful] [🍴 23 Forks] [💬 15 Comments] [⭐ Save]    │
└─────────────────────────────────────────────────────────────┘
```

#### Runbook Features

**Forking:**
- Copy runbook and modify for your workflow
- Original author credited
- Track variations and improvements

**Execution Tracking:**
- Mark steps as completed
- Note any issues/deviations
- Share results with community

**Version Control:**
- Edit and update runbooks
- Track changes over time
- Community suggestions

**Templates:**
- Pre-built runbook templates
- Guild-specific templates
- Import from local CLI history

---

### 1.6 Privacy & Telemetry Dashboard

**Purpose:** Transparent view of all collected data and sharing activity

#### Dashboard Sections

**1. Local Data Collection**
```
┌─────────────────────────────────────────────────────────────┐
│ 📊 Telemetry Dashboard                                      │
│                                                              │
│ Data Collected Locally (Last 30 Days)                       │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                                              │
│ Commands Generated: 342                                      │
│ Prompts Saved: 342                                           │
│ Safety Validations: 28                                       │
│ Backend Usage: [Chart: Ollama 80%, MLX 20%]                │
│ Error Logs: 5                                                │
│                                                              │
│ ⚠️ Sensitive Data Detected (Never Shared):                  │
│ • API Keys: 12 instances                                     │
│ • Home paths: 87 instances                                   │
│ • Email addresses: 3 instances                               │
│                                                              │
│ [Export All Data] [Clear History]                           │
└─────────────────────────────────────────────────────────────┘
```

**2. Sharing History**
```
┌─────────────────────────────────────────────────────────────┐
│ Shared with Network                                          │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                                              │
│ Public Shares: 15                                            │
│ Guild Shares: 23                                             │
│ Private (Not Shared): 304                                    │
│                                                              │
│ Recent Shares:                                               │
│ • [Command] "find large files" (Public) - 2h ago            │
│ • [Win] "Automated audits" (DevOps Guild) - 1d ago          │
│ • [Runbook] "Deployment" (Frontend Guild) - 5d ago          │
│                                                              │
│ Redactions Applied: 42                                       │
│ • API keys redacted: 8                                       │
│ • Paths sanitized: 29                                        │
│ • Env vars filtered: 5                                       │
│                                                              │
│ [Review All Shares] [Delete Share]                           │
└─────────────────────────────────────────────────────────────┘
```

**3. Privacy Settings**
```
┌─────────────────────────────────────────────────────────────┐
│ Privacy Settings                                             │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                                              │
│ Default Visibility:                                          │
│ ○ Public  ○ Guild  ● Private                                │
│                                                              │
│ Automatic Redaction:                                         │
│ ✅ API keys and tokens                                      │
│ ✅ Home directory paths                                     │
│ ✅ Environment variables ($SECRET_*, $TOKEN_*)              │
│ ✅ Email addresses                                          │
│ ✅ IP addresses                                             │
│ ⬜ Generic paths (/usr/bin, /etc)                           │
│                                                              │
│ Telemetry Collection:                                        │
│ ✅ Command generation events                                │
│ ✅ Safety validation triggers                               │
│ ✅ Error logs (locally only)                                │
│ ⬜ Usage analytics                                          │
│                                                              │
│ [Save Settings]                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🏰 Community Features

### 2.1 Professional Guilds

**Purpose:** Specialized communities for knowledge sharing

#### Guild Structure

```typescript
interface Guild {
  id: string                         // "sre"
  name: string                       // "SRE Guild"
  slug: string                       // "sre" (URL-friendly)
  description: string
  icon: string                       // Emoji or icon URL
  color: string                      // Brand color (#39ff14)

  // Membership
  member_count: number
  moderators: string[]               // DIDs
  created: string

  // Content
  tags: string[]                     // Related topics
  featured_artifacts: string[]       // Pinned posts

  // Moderation
  rules: string[]
  guidelines: string

  // Feed
  feed_algorithm?: string            // Custom feed logic
  sort_default: "recent" | "helpful" | "trending"
}
```

#### Default Guilds

**Technical Guilds:**
- SRE (Site Reliability Engineering)
- AppSec (Application Security)
- DevOps (Development Operations)
- Frontend (Web Development)
- Backend (Server Development)
- Data (Data Engineering)
- Cloud (Cloud Infrastructure)
- Linux (Linux Administration)
- MacOS (macOS Power Users)
- Windows (Windows Administration)

**Domain Guilds:**
- Homelab (Home Server Enthusiasts)
- Academia (Research & Education)
- Finance (FinTech)
- Healthcare (HealthTech)
- Gaming (Game Development)

**Tool-Specific Guilds:**
- Docker (Containerization)
- Kubernetes (Orchestration)
- AWS (Amazon Web Services)
- Git (Version Control)
- Bash (Shell Scripting)

#### Guild Page

```
┌─────────────────────────────────────────────────────────────┐
│ 🏰 SRE Guild                                                │
│                                                              │
│ 📊 2,341 members • 12 moderators • Est. Jan 2026           │
│                                                              │
│ Site Reliability Engineering best practices, runbooks,      │
│ incident response, and monitoring wisdom.                    │
│                                                              │
│ [Join Guild] [Feed] [Members] [Rules]                       │
│                                                              │
│ Top Contributors This Week:                                  │
│ • @alice.bsky.social (15 helpful posts)                     │
│ • @bob.bsky.social (12 helpful posts)                       │
│ • @charlie.bsky.social (8 helpful posts)                    │
│                                                              │
│ Featured Runbooks:                                           │
│ • "Incident Response Process" - 234 helpful                  │
│ • "On-Call Survival Guide" - 189 helpful                     │
│ • "Monitoring Setup" - 156 helpful                           │
│                                                              │
│ Recent Activity:                                             │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│ [... Guild feed...]                                          │
└─────────────────────────────────────────────────────────────┘
```

---

### 2.2 Guild Feeds & Discovery

**Feed Types:**

**1. Personal Feed (Home)**
- All activity from joined guilds
- Personalized algorithm (recent + helpful)
- Filter by artifact type

**2. Guild Feed**
- Artifacts from specific guild
- Sort by: Recent, Helpful, Trending
- Filter by type (commands, wins, fails, runbooks)

**3. Discovery Feed**
- "Trending Commands" across all guilds
- "Recent Wins" (success stories)
- "Top Contributors" (leaderboard)
- "New Guilds" (recently created)

**4. Custom Feeds**
- Create custom feeds with filters
- Combine multiple guilds
- Specific tags or keywords
- Save and share feed configurations

#### Discovery Page

```
┌─────────────────────────────────────────────────────────────┐
│ 🔍 Discover                                                 │
│                                                              │
│ Trending Now                                                 │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│ 1. [Command] "Parallel file processing" - 89 helpful        │
│ 2. [Win] "Reduced deploy time by 80%" - 76 helpful          │
│ 3. [Runbook] "K8s troubleshooting guide" - 65 helpful       │
│                                                              │
│ Popular Guilds                                               │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│ 🏰 SRE Guild (2,341 members)                               │
│ 🏰 DevOps Guild (1,876 members)                            │
│ 🏰 AppSec Guild (1,234 members)                            │
│                                                              │
│ Featured Runbooks                                            │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│ • "Production Deployment Checklist" - 234 helpful           │
│ • "Database Backup Strategy" - 189 helpful                   │
│ • "Security Audit Process" - 156 helpful                     │
│                                                              │
│ Top Contributors                                             │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│ 1. @alice (Level 24, 1,234 rep)                            │
│ 2. @bob (Level 22, 1,087 rep)                              │
│ 3. @charlie (Level 20, 945 rep)                            │
└─────────────────────────────────────────────────────────────┘
```

---

### 2.3 Community Moderation

**Moderation Tools:**

**For Moderators:**
- Remove harmful/spam artifacts
- Ban users (temporary or permanent)
- Pin important posts
- Edit guild description and rules
- Appoint new moderators

**For All Users:**
- Report artifacts (spam, harmful, off-topic)
- Block users
- Mute guilds
- Hide artifacts

**Automated Moderation:**
- Spam detection (duplicate posts, low-effort)
- Profanity filter (configurable)
- Rate limiting (prevent flooding)
- Credential detection (automatically hide leaked secrets)

#### Moderation Queue

```
┌─────────────────────────────────────────────────────────────┐
│ 🛡️ Moderation Queue (SRE Guild)                            │
│                                                              │
│ Pending Reports: 3                                           │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                                              │
│ 1. [Spam Report] Command artifact - "buy viagra"            │
│    Reported by: @alice, @bob                                 │
│    [Remove] [Ignore] [Ban User]                              │
│                                                              │
│ 2. [Off-topic] Win story - Frontend content in SRE guild    │
│    Reported by: @charlie                                     │
│    [Move to Frontend] [Remove] [Ignore]                      │
│                                                              │
│ 3. [Harmful] Epic fail - Contains leaked API key            │
│    Auto-detected by privacy engine                           │
│    [Redact & Restore] [Remove Permanently]                   │
└─────────────────────────────────────────────────────────────┘
```

---

### 2.4 Reputation System

**How Reputation Works:**

```typescript
interface Reputation {
  total_points: number               // 287
  level: number                      // 12 (1-100)
  next_level_points: number          // 300

  breakdown: {
    commands_shared: number          // +1 per share
    wins_posted: number              // +5 per win
    fails_reported: number           // +3 per fail
    runbooks_created: number         // +10 per runbook
    helpful_votes_received: number   // +2 per upvote
    comments: number                 // +1 per comment
    runbook_forks: number            // +3 per fork
    guild_contributions: number      // Bonus per guild
  }

  badges: string[]                   // Achievement IDs
  rank_percentile: number            // Top 5%
}
```

**Point System:**
| Action | Points | Notes |
|--------|--------|-------|
| Share command | +1 | First share |
| Receive helpful vote | +2 | Per upvote |
| Post win story | +5 | With artifacts |
| Report epic fail | +3 | If confirmed |
| Create runbook | +10 | Published |
| Runbook forked | +3 | Per fork |
| Comment | +1 | Constructive |
| Guild mod action | +5 | Per action |
| Achievement unlocked | +10-50 | Varies |

**Reputation Levels:**
- 1-10: Newcomer
- 11-25: Contributor
- 26-50: Expert
- 51-75: Master
- 76-99: Legend
- 100: Caro Champion

---

## 🎮 Engagement Features

### 3.1 Achievements & Badges

**Achievement Categories:**

**Starter Achievements:**
- **First Share** (🎉): Shared your first command
- **First Win** (🏆): Posted your first win story
- **First Runbook** (📚): Created your first runbook
- **Guild Joiner** (🏰): Joined 5 guilds

**Contribution Achievements:**
- **Helpful 10** (👍): Received 10 helpful votes
- **Helpful 100** (🌟): Received 100 helpful votes
- **Prolific** (📝): Shared 50 commands
- **Storyteller** (📖): Posted 10 win stories
- **Runbook Author** (✍️): Created 5 runbooks
- **Forked** (🍴): Your runbook forked 10 times

**Safety Achievements:**
- **Safety Guardian** (🛡️): Reported 10 dangerous commands
- **Privacy Advocate** (🔒): Redacted 50 sensitive items
- **Fail Reporter** (⚠️): Reported 5 epic fails

**Community Achievements:**
- **Guild Leader** (👑): Top contributor in a guild
- **Mentor** (🎓): Helped 20 new users
- **Moderator** (🛡️): Became a guild moderator

**Special Achievements:**
- **Early Adopter** (🚀): Joined in first month
- **Kyaro's Favorite** (🐕): Caro team recognition
- **Open Source Hero** (💚): Contributed to Caro CLI

#### Achievement Display

```
┌─────────────────────────────────────────────────────────────┐
│ 🏆 Achievements (12/50)                                     │
│                                                              │
│ ✅ First Share (🎉)        ✅ Helpful 10 (👍)              │
│ ✅ First Win (🏆)          ✅ Prolific (📝)                │
│ ✅ Guild Joiner (🏰)       ✅ Safety Guardian (🛡️)        │
│ ✅ First Runbook (📚)      ✅ Forked (🍴)                  │
│                                                              │
│ 🔒 Locked:                                                  │
│ ⬜ Helpful 100 (🌟) - 23/100 helpful votes                 │
│ ⬜ Guild Leader (👑) - Be top contributor in a guild       │
│ ⬜ Runbook Author (✍️) - 3/5 runbooks created              │
└─────────────────────────────────────────────────────────────┘
```

---

### 3.2 Kyaro Evolution States

**Kyaro Changes with Your Activity:**

**Level 1-10: Puppy Kyaro**
- Small, curious sprite
- Basic animations (idle, thinking, success)
- Encourages first shares

**Level 11-25: Teen Kyaro**
- More detailed sprite
- New animations (bored, long-inference)
- Offers tips and suggestions

**Level 26-50: Adult Kyaro**
- Full detail sprite
- All 11 states animated
- Personalized responses

**Level 51-75: Veteran Kyaro**
- Special accessories (badges, scarf)
- Unique idle animations
- Custom greeting/farewell

**Level 76-100: Legend Kyaro**
- Golden sprite with sparkles
- Epic animations
- Rare special states

**Kyaro Interactions:**
```
┌─────────────────────────────────────────────────────────────┐
│  [Kyaro: Thinking State]                                    │
│                                                              │
│  ┌────────────┐                                             │
│  │   🐕💭     │  "Great command! But I spotted a path      │
│  │ [Kyaro]    │   that might contain your username.         │
│  └────────────┘   Let me help redact it before sharing."   │
│                                                              │
│  [Review Redactions] [Trust Kyaro]                          │
└─────────────────────────────────────────────────────────────┘
```

---

### 3.3 Leaderboards

**Global Leaderboards:**

**All-Time Top Contributors:**
```
┌─────────────────────────────────────────────────────────────┐
│ 🏆 Top Contributors (All Time)                              │
│                                                              │
│ 1. @alice.bsky.social      1,234 rep  Level 24             │
│ 2. @bob.bsky.social        1,087 rep  Level 22             │
│ 3. @charlie.bsky.social      945 rep  Level 20             │
│ 4. @diana.bsky.social        823 rep  Level 18             │
│ 5. @eve.bsky.social          756 rep  Level 17             │
│                                                              │
│ Your Rank: #42 (287 rep, Level 12)                         │
└─────────────────────────────────────────────────────────────┘
```

**This Week:**
- Top command sharers
- Most helpful contributors
- Best new runbooks

**Guild-Specific:**
- Top contributors per guild
- Most helpful in guild
- Guild growth leaders

---

### 3.4 Command Challenges

**Weekly/Monthly Challenges:**

```typescript
interface Challenge {
  id: string
  title: string                      // "Monitoring Mastery"
  description: string
  type: "command" | "runbook" | "win"
  requirements: string[]
  reward_points: number
  reward_badge?: string
  start_date: string
  end_date: string
  participants: number
  completions: number
}
```

**Example Challenges:**

**Weekly: "Monitoring Mastery"**
- Share 3 monitoring-related commands
- Tag with #monitoring
- Get 5+ helpful votes combined
- Reward: 50 points + "Monitor" badge

**Monthly: "Runbook Author"**
- Create a complete runbook (5+ steps)
- Get 10+ helpful votes
- Forked 3+ times
- Reward: 100 points + "Author" badge

**Community: "Safety First"**
- Report 3 dangerous commands
- All must be confirmed by moderators
- Reward: 75 points + "Guardian" badge

---

## 📊 Analytics & Insights

### User Analytics

**Personal Insights Dashboard:**
```
┌─────────────────────────────────────────────────────────────┐
│ 📊 Your Caro Insights                                       │
│                                                              │
│ This Month:                                                  │
│ • Commands generated: 127 (↑ 23% from last month)          │
│ • Most used backend: Ollama (85%)                           │
│ • Safety triggers: 12 (prevented 4 dangerous commands)      │
│ • Shares: 8 (3 commands, 2 wins, 1 runbook, 2 fails)       │
│                                                              │
│ Your Impact:                                                 │
│ • Your artifacts helped 234 people                           │
│ • Your runbooks forked 15 times                              │
│ • You saved the community ~8 hours of work                   │
│                                                              │
│ Trending Topics in Your Guilds:                              │
│ • #kubernetes (45 posts this week)                           │
│ • #monitoring (32 posts)                                     │
│ • #security (28 posts)                                       │
└─────────────────────────────────────────────────────────────┘
```

### Guild Analytics

**For Moderators:**
- Member growth trends
- Most active contributors
- Popular topics/tags
- Engagement metrics
- Moderation queue stats

---

## 🚀 Feature Roadmap

### Phase 1: Foundation ✅
- User profiles
- Command sharing
- Basic privacy redaction
- Guild discovery

### Phase 2: Social Platform (Q1 2026)
- Win stories
- Epic fails
- Runbook creation
- Guild feeds
- Comments and discussions

### Phase 3: Engagement (Q2 2026)
- Achievements and badges
- Reputation system
- Leaderboards
- Kyaro evolution

### Phase 4: Advanced (Q3 2026)
- Command challenges
- Custom feeds
- Advanced analytics
- Mobile app beta

### Phase 5: Enterprise (Q4 2026)
- Private guilds (teams)
- Enterprise features
- SSO integration
- Advanced moderation tools

---

**Questions about social features?** Open an issue in the [Caro repository](https://github.com/wildcard/caro).

**Let's build the future of terminal knowledge sharing! 🐕✨**
