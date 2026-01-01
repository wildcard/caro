# Phase 0 Data Model: Caro Web Hub

**Feature**: 008-caro-web-hub
**Date**: 2026-01-01
**Status**: Data Model Complete

## Core Entities

### 1. User Profile

**Purpose**: Represents a user's identity, stats, and settings across Caro CLI and Bluesky

```typescript
interface UserProfile {
  // Bluesky Identity
  did: string;                    // Decentralized identifier (e.g., "did:plc:abc123...")
  handle: string;                 // Bluesky handle (e.g., "@user.bsky.social")
  displayName: string;            // User's display name
  avatar?: string;                // Avatar URL (optional)
  bio?: string;                   // User bio (optional)

  // Local CLI Stats
  localStats: {
    commandsGeneratedToday: number;
    commandsGeneratedTotal: number;
    mostUsedBackend: string;      // "ollama:qwen2.5-coder"
    safetyTriggersToday: number;
    lastUsed: string;             // ISO 8601 timestamp
    cliVersion: string;           // "1.0.3"
  };

  // Social Stats
  socialStats: {
    commandsShared: number;
    winsPosted: number;
    failsReported: number;
    runbooksCreated: number;
    helpfulVotesReceived: number;
    reputationPoints: number;
    level: number;                // 1-100
  };

  // Guild Membership
  joinedGuilds: string[];         // Array of guild IDs ["sre", "devops"]

  // Privacy Settings
  privacySettings: PrivacySettings;

  // Timestamps
  createdAt: string;              // ISO 8601
  updatedAt: string;
}
```

---

### 2. Command Artifact

**Purpose**: A shared command with context

```typescript
interface CommandArtifact {
  // Metadata
  id: string;                     // UUID or AT URI
  type: 'command_artifact';

  // Content
  title?: string;                 // Optional short title
  description?: string;           // Context/explanation
  prompt: string;                 // Original natural language request
  command: string;                // Generated shell command

  // Technical Details
  safetyScore: SafetyLevel;       // "safe" | "moderate" | "high" | "critical"
  backend: string;                // "ollama:qwen2.5-coder"
  timestamp: string;              // ISO 8601

  // Social
  tags: string[];                 // ["find", "filesystem"]
  guild?: string;                 // Optional guild ID
  visibility: Visibility;         // "public" | "guild" | "private"

  // Author
  authorDid: string;              // Bluesky DID

  // Engagement
  helpfulVotes: number;
  comments: Comment[];
  saves: number;                  // Bookmark count

  // Privacy
  redactionsApplied: Redaction[];
}

type SafetyLevel = 'safe' | 'moderate' | 'high' | 'critical';
type Visibility = 'public' | 'guild' | 'private';
```

---

### 3. Runbook

**Purpose**: Multi-step terminal workflow

```typescript
interface Runbook {
  // Metadata
  id: string;
  type: 'runbook';

  // Content
  title: string;                  // "Production Deployment Process"
  description: string;            // Purpose and context
  steps: RunbookStep[];

  // Workflow Metadata
  prerequisites?: string[];       // ["Git repository", "Feature branch"]
  estimatedTime?: string;         // "~15 minutes"
  difficulty?: Difficulty;        // "beginner" | "intermediate" | "advanced"

  // Social
  tags: string[];
  guild?: string;
  visibility: Visibility;

  // Author & Forking
  authorDid: string;
  forkCount: number;
  originalRunbook?: string;       // ID of source runbook (if forked)

  // Engagement
  helpfulVotes: number;
  comments: Comment[];
  saves: number;

  // Timestamps
  createdAt: string;
  updatedAt: string;
}

interface RunbookStep {
  order: number;                  // 1, 2, 3...
  title: string;                  // "Run tests"
  prompt: string;                 // "check if tests pass"
  command: string;                // "npm test"
  notes?: string;                 // Additional context
  safetyLevel: SafetyLevel;
  expectedOutput?: string;        // What to expect
}

type Difficulty = 'beginner' | 'intermediate' | 'advanced';
```

---

### 4. Win Story

**Purpose**: Celebrate successful automations

```typescript
interface WinStory {
  // Metadata
  id: string;
  type: 'win_story';

  // Content
  title: string;                  // "Automated 200 Server Audits"
  story: string;                  // Markdown narrative (max 2000 chars)
  impact: string;                 // "Saved 8 hours of work"

  // Linked Artifacts
  linkedArtifacts: string[];      // IDs of related commands/runbooks

  // Social
  tags: string[];
  guild?: string;
  visibility: Visibility;

  // Author
  authorDid: string;

  // Engagement
  helpfulVotes: number;
  comments: Comment[];

  // Timestamps
  createdAt: string;
  updatedAt: string;
}
```

---

### 5. Epic Fail

**Purpose**: Report dangerous/incorrect commands

```typescript
interface EpicFail {
  // Metadata
  id: string;
  type: 'epic_fail';

  // Content
  prompt: string;                 // User's original request
  generatedCommand: string;       // What Caro generated
  expectedBehavior: string;       // What should have happened
  actualResult?: string;          // What actually happened

  // Logs (redacted)
  logs: string;                   // Verbose logs (PII redacted)
  logsRedacted: boolean;          // Confirmation of redaction

  // Technical Details
  severity: Severity;             // "low" | "medium" | "high" | "critical"
  backend: string;                // Which backend failed
  cliVersion: string;             // Caro CLI version
  timestamp: string;

  // Visibility
  visibility: 'public' | 'private'; // Public = community, Private = maintainers

  // Author
  authorDid: string;

  // Engagement
  confirmedBy: string[];          // DIDs of users who confirmed same issue
  comments: Comment[];

  // Status
  status: FailStatus;             // "under_review" | "confirmed" | "fix_in_progress" | "resolved"
  resolution?: string;            // Fix description (when resolved)

  // Timestamps
  createdAt: string;
  resolvedAt?: string;
}

type Severity = 'low' | 'medium' | 'high' | 'critical';
type FailStatus = 'under_review' | 'confirmed' | 'fix_in_progress' | 'resolved';
```

---

### 6. Guild

**Purpose**: Professional community

```typescript
interface Guild {
  // Identity
  id: string;                     // "sre"
  name: string;                   // "SRE Guild"
  slug: string;                   // "sre" (URL-friendly)

  // Content
  description: string;
  icon: string;                   // Emoji or icon URL
  color: string;                  // Brand color (#39ff14)

  // Membership
  memberCount: number;
  moderators: string[];           // DIDs

  // Content
  tags: string[];                 // Related topics
  featuredArtifacts: string[];    // Pinned posts (IDs)

  // Moderation
  rules: string[];
  guidelines: string;

  // Feed
  feedAlgorithm?: string;         // Custom feed logic
  sortDefault: FeedSort;          // "recent" | "helpful" | "trending"

  // Timestamps
  createdAt: string;
}

type FeedSort = 'recent' | 'helpful' | 'trending';
```

---

### 7. Comment

**Purpose**: Discussion on artifacts

```typescript
interface Comment {
  id: string;
  artifactId: string;             // Parent artifact
  authorDid: string;
  content: string;                // Markdown (max 500 chars)
  createdAt: string;
  updatedAt?: string;
  helpfulVotes: number;
}
```

---

### 8. Privacy Scan Result

**Purpose**: Detected sensitive data for user review

```typescript
interface PrivacyScanResult {
  // Artifact Being Scanned
  artifactId: string;
  artifactType: ArtifactType;

  // Detected Items
  detectedItems: DetectedItem[];

  // Status
  reviewed: boolean;              // User reviewed all items
  allConfirmed: boolean;          // User confirmed all redactions

  // Applied Redactions
  appliedRedactions: Redaction[];

  // Timestamps
  scannedAt: string;
  reviewedAt?: string;
}

interface DetectedItem {
  type: SensitiveDataType;
  value: string;                  // Original text
  startIndex: number;             // Position in original text
  endIndex: number;
  severity: 'critical' | 'moderate'; // Red vs yellow flag
  suggestion: string;             // e.g., "[REDACTED_API_KEY]"
  userAction?: 'redact' | 'keep'; // User decision
}

interface Redaction {
  type: SensitiveDataType;
  original: string;               // Original text
  redacted: string;               // Replacement text
  appliedAt: string;
}

type SensitiveDataType =
  | 'api_key'
  | 'jwt_token'
  | 'aws_key'
  | 'ssh_key'
  | 'github_token'
  | 'email'
  | 'ipv4'
  | 'home_path_unix'
  | 'home_path_windows'
  | 'env_var_secret'
  | 'connection_string';

type ArtifactType = 'command' | 'runbook' | 'win' | 'fail';
```

---

### 9. Privacy Settings

**Purpose**: User's privacy preferences

```typescript
interface PrivacySettings {
  // Default Visibility
  defaultVisibility: Visibility;  // "public" | "guild" | "private"

  // Automatic Redaction Rules
  autoRedaction: {
    apiKeys: boolean;             // Default: true
    tokens: boolean;              // Default: true
    paths: boolean;               // Default: true
    emails: boolean;              // Default: true
    ips: boolean;                 // Default: true
    genericPaths: boolean;        // Default: false (/usr/bin, /etc)
    envVars: boolean;             // Default: true
  };

  // Telemetry Collection (Local Only)
  telemetryCollection: {
    commandEvents: boolean;       // Default: true
    safetyTriggers: boolean;      // Default: true
    errorLogs: boolean;           // Default: true (local only)
    usageAnalytics: boolean;      // Default: false
  };
}
```

---

### 10. Local CLI Data

**Purpose**: Data collected locally by Caro CLI

```typescript
interface LocalCLIData {
  // Commands
  commands: CommandHistory[];

  // Prompts
  prompts: PromptHistory[];

  // Safety Events
  safetyValidations: SafetyEvent[];

  // Backend Usage
  backendUsage: BackendStat[];

  // Error Logs
  errorLogs: ErrorLog[];

  // Metadata
  lastSync: string;               // When data was last loaded
}

interface CommandHistory {
  id: string;
  prompt: string;
  command: string;
  backend: string;
  safetyScore: SafetyLevel;
  timestamp: string;
  executed: boolean;              // Did user run the command?
}

interface PromptHistory {
  id: string;
  prompt: string;
  timestamp: string;
}

interface SafetyEvent {
  id: string;
  commandId: string;
  severity: SafetyLevel;
  reason: string;                 // Why it was flagged
  userAction: 'blocked' | 'overridden' | 'modified';
  timestamp: string;
}

interface BackendStat {
  backend: string;                // "ollama:qwen2.5-coder"
  usageCount: number;
  lastUsed: string;
}

interface ErrorLog {
  id: string;
  error: string;
  stackTrace?: string;
  timestamp: string;
}
```

---

### 11. Sharing History

**Purpose**: Track what user has shared

```typescript
interface SharingHistory {
  // Shares
  shares: Share[];

  // Summary Stats
  totalShares: number;
  publicShares: number;
  guildShares: number;
  privateShares: number;

  // Redaction Stats
  totalRedactions: number;
  redactionsByType: Record<SensitiveDataType, number>;
}

interface Share {
  id: string;
  artifactId: string;
  artifactType: ArtifactType;
  visibility: Visibility;
  guild?: string;
  redactionsApplied: number;
  sharedAt: string;
  blueskyUri?: string;            // AT URI on Bluesky PDS
  status: 'draft' | 'published' | 'failed';
  failureReason?: string;
}
```

---

## Bluesky AT Protocol Lexicons

### Command Artifact Lexicon

**Collection**: `app.caro.share.command`

```typescript
{
  "lexicon": 1,
  "id": "app.caro.share.command",
  "defs": {
    "main": {
      "type": "record",
      "description": "A shared shell command with context",
      "key": "tid",
      "record": {
        "type": "object",
        "required": ["prompt", "command", "safetyScore", "backend", "timestamp"],
        "properties": {
          "$type": {
            "type": "string",
            "const": "app.caro.share.command"
          },
          "title": {
            "type": "string",
            "maxLength": 100
          },
          "description": {
            "type": "string",
            "maxLength": 500
          },
          "prompt": {
            "type": "string",
            "maxLength": 1000
          },
          "command": {
            "type": "string",
            "maxLength": 5000
          },
          "safetyScore": {
            "type": "string",
            "enum": ["safe", "moderate", "high", "critical"]
          },
          "backend": {
            "type": "string"
          },
          "timestamp": {
            "type": "string",
            "format": "datetime"
          },
          "tags": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "maxLength": 10
          },
          "guild": {
            "type": "string"
          },
          "redactionsApplied": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["type", "count"],
              "properties": {
                "type": {
                  "type": "string"
                },
                "count": {
                  "type": "integer"
                }
              }
            }
          }
        }
      }
    }
  }
}
```

---

### Runbook Lexicon

**Collection**: `app.caro.share.runbook`

```typescript
{
  "lexicon": 1,
  "id": "app.caro.share.runbook",
  "defs": {
    "main": {
      "type": "record",
      "description": "A multi-step terminal workflow",
      "key": "tid",
      "record": {
        "type": "object",
        "required": ["title", "description", "steps"],
        "properties": {
          "$type": {
            "type": "string",
            "const": "app.caro.share.runbook"
          },
          "title": {
            "type": "string",
            "maxLength": 200
          },
          "description": {
            "type": "string",
            "maxLength": 1000
          },
          "steps": {
            "type": "array",
            "items": {
              "$ref": "#/defs/step"
            },
            "maxLength": 50
          },
          "prerequisites": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "estimatedTime": {
            "type": "string"
          },
          "difficulty": {
            "type": "string",
            "enum": ["beginner", "intermediate", "advanced"]
          },
          "tags": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "guild": {
            "type": "string"
          },
          "originalRunbook": {
            "type": "string"
          }
        }
      }
    },
    "step": {
      "type": "object",
      "required": ["order", "title", "prompt", "command", "safetyLevel"],
      "properties": {
        "order": {
          "type": "integer"
        },
        "title": {
          "type": "string",
          "maxLength": 100
        },
        "prompt": {
          "type": "string",
          "maxLength": 500
        },
        "command": {
          "type": "string",
          "maxLength": 1000
        },
        "notes": {
          "type": "string",
          "maxLength": 500
        },
        "safetyLevel": {
          "type": "string",
          "enum": ["safe", "moderate", "high", "critical"]
        },
        "expectedOutput": {
          "type": "string",
          "maxLength": 500
        }
      }
    }
  }
}
```

---

### Win Story Lexicon

**Collection**: `app.caro.share.win`

```typescript
{
  "lexicon": 1,
  "id": "app.caro.share.win",
  "defs": {
    "main": {
      "type": "record",
      "description": "A success story celebrating automation wins",
      "key": "tid",
      "record": {
        "type": "object",
        "required": ["title", "story", "impact"],
        "properties": {
          "$type": {
            "type": "string",
            "const": "app.caro.share.win"
          },
          "title": {
            "type": "string",
            "maxLength": 200
          },
          "story": {
            "type": "string",
            "maxLength": 2000
          },
          "impact": {
            "type": "string",
            "maxLength": 200
          },
          "linkedArtifacts": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "tags": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "guild": {
            "type": "string"
          }
        }
      }
    }
  }
}
```

---

### Epic Fail Lexicon

**Collection**: `app.caro.share.fail`

```typescript
{
  "lexicon": 1,
  "id": "app.caro.share.fail",
  "defs": {
    "main": {
      "type": "record",
      "description": "A report of dangerous/incorrect command generation",
      "key": "tid",
      "record": {
        "type": "object",
        "required": ["prompt", "generatedCommand", "expectedBehavior", "severity", "backend", "cliVersion"],
        "properties": {
          "$type": {
            "type": "string",
            "const": "app.caro.share.fail"
          },
          "prompt": {
            "type": "string",
            "maxLength": 1000
          },
          "generatedCommand": {
            "type": "string",
            "maxLength": 5000
          },
          "expectedBehavior": {
            "type": "string",
            "maxLength": 1000
          },
          "actualResult": {
            "type": "string",
            "maxLength": 1000
          },
          "logs": {
            "type": "string",
            "maxLength": 10000,
            "description": "Redacted verbose logs"
          },
          "logsRedacted": {
            "type": "boolean"
          },
          "severity": {
            "type": "string",
            "enum": ["low", "medium", "high", "critical"]
          },
          "backend": {
            "type": "string"
          },
          "cliVersion": {
            "type": "string"
          },
          "reproducible": {
            "type": "boolean"
          }
        }
      }
    }
  }
}
```

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  Caro CLI (Local)                                           │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  • Generates commands                                        │
│  • Collects telemetry (LocalCLIData)                        │
│  • Stores in ~/.caro/history.json                           │
│  • No network by default                                     │
└─────────────────────────────────────────────────────────────┘
                        │
                        │ (1) Local access via File API / HTTP server
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  Privacy Dashboard (Browser)                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  • Loads LocalCLIData (offline)                             │
│  • Scans with PrivacyEngine → PrivacyScanResult            │
│  • User reviews DetectedItems                               │
│  • User confirms Redactions                                 │
└─────────────────────────────────────────────────────────────┘
                        │
                        │ (2) User decides to share
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  Share Flow (Browser)                                        │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  • User creates CommandArtifact / Runbook / Win / Fail      │
│  • PrivacyEngine scans → PrivacyScanResult                  │
│  • User reviews & confirms redactions                        │
│  • User adds metadata (title, tags, guild, visibility)      │
│  • User clicks "Publish"                                     │
└─────────────────────────────────────────────────────────────┘
                        │
                        │ (3) Publish to Bluesky
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  Bluesky PDS (Personal Data Server)                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  • Stores artifact with custom Lexicon schema               │
│  • Returns AT URI (at://did:plc:abc.../app.caro.share.*)   │
│  • Makes artifact discoverable in feeds                     │
└─────────────────────────────────────────────────────────────┘
                        │
                        │ (4) Appears in feeds
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  Guild Feeds (Browser)                                       │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  • Fetch artifacts from Bluesky feed algorithm              │
│  • Filter by guild tag, artifact type, author               │
│  • Sort by Recent / Helpful / Trending                       │
│  • Display ArtifactCard components                          │
└─────────────────────────────────────────────────────────────┘
```

---

## Storage Strategy

| Data Type | Storage Location | Sync Strategy |
|-----------|------------------|---------------|
| **LocalCLIData** | Caro CLI (`~/.caro/history.json`) | Read-only from browser |
| **UserProfile** | Bluesky PDS + localStorage | Fetch on login, cache locally |
| **CommandArtifact** | Bluesky PDS | Publish on share, fetch for feeds |
| **Runbook** | Bluesky PDS | Publish on share, fetch for feeds |
| **WinStory** | Bluesky PDS | Publish on share, fetch for feeds |
| **EpicFail** | Bluesky PDS | Publish on report, fetch for maintainers |
| **PrivacyScanResult** | IndexedDB (browser) | Local only, never uploaded |
| **PrivacySettings** | localStorage (encrypted) | Local only, Bluesky for backup |
| **SharingHistory** | IndexedDB (browser) | Local only, audit trail |
| **OAuth Token** | localStorage (encrypted) | Session only, auto-refresh |

---

**Data Model Complete**: All entities, Lexicons, and storage strategies defined. Ready for contract definitions and implementation.
