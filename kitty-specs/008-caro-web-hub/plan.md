# Implementation Plan: Caro Web Hub - Privacy-First Social Platform

**Branch**: `claude/caro-web-hub-014Ku7d3Yf4a6FwX1TBskyHy` | **Date**: 2026-01-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `kitty-specs/008-caro-web-hub/spec.md`

**Planning Status**: All discovery and planning questions resolved. Technical approach confirmed.

## Summary

Implement Caro Web Hub as a privacy-first social platform where developers safely share terminal expertise, operational runbooks, and workflows with professional guilds. Built on Next.js 16 with Bluesky AT Protocol integration, the hub provides a transparent telemetry dashboard showing all locally collected CLI data before any network access, automatic PII/credential redaction, and explicit user consent for every share. Features include command artifact sharing, multi-step runbook creation (with example: "Rebase with main, then continue"), epic fail reporting with redacted logs, professional guild communities (15+ default guilds), win story posting, and Bluesky OAuth authentication. The implementation follows a phased MVP approach: Phase 1 focuses on privacy dashboard + Bluesky auth + basic command sharing; Phase 2 adds runbooks, guilds, and win stories; Phase 3 implements epic fail reporting and advanced features.

## Technical Context

**Framework/Version**: Next.js 16.1.1 (App Router, Turbopack), React 19.2.0, TypeScript 5.x
**Primary Dependencies**:
- **@atproto/api** (Bluesky AT Protocol SDK for auth and publishing)
- **Tailwind CSS 4** (styling with 8-bit pixel art design system)
- **next-auth** or **@atproto/oauth-client** (Bluesky OAuth flow)
- **zustand** or **jotai** (client-side state management)
- **zod** (schema validation for privacy redaction patterns)
- **marked** or **react-markdown** (markdown rendering for stories/runbooks)

**Storage**:
- **Client-side**: localStorage (encrypted) for OAuth tokens, IndexedDB for local CLI data cache
- **Server-side**: Bluesky PDS (Personal Data Server) for all published artifacts
- **Local CLI Integration**: File System Access API (Chrome), local HTTP server (localhost:3000), or Electron/Tauri wrapper

**Testing**: Vitest + React Testing Library (unit), Playwright (E2E), Pa11y (accessibility)
**Target Platform**: Modern browsers (Chrome 90+, Firefox 88+, Safari 14+, Edge 90+)
**Deployment**: Vercel (static SSG + edge functions), hub.caro.sh domain

**Project Type**: Next.js 16 App Router application (hybrid SSG/SSR)

**Performance Goals**:
- Privacy dashboard loads local data in <500ms (10,000 commands)
- Privacy scan processes 1,000-line artifact in <2 seconds
- Guild feed renders first 20 items in <1 second
- OAuth flow completes in <3 seconds

**Constraints**:
- **Privacy-first**: Zero network requests for privacy dashboard; all operations offline-first
- **Bluesky dependency**: Publishing requires Bluesky network availability (handle gracefully with offline queue)
- **Browser compatibility**: Modern evergreen browsers only (no IE11)
- **8-bit design language**: Must follow existing apps/devrel/ design system (Game Boy palette, pixel fonts, ANSI constraints)
- **Local CLI data access**: Browser security restrictions require careful handling (File API, local server, or native wrapper)

**Scale/Scope**:
- **38 functional requirements** (FR-001 to FR-038)
- **12 non-functional requirements** (NFR-001 to NFR-012)
- **10 success criteria** with measurable outcomes
- **7 user stories** (3 P1, 3 P2, 1 P3)
- **New file estimate**: ~60 files (components, pages, lib, types)
- **LOC estimate**: ~8,000-10,000 lines (excluding tests)

## Constitution Check

*GATE: Must pass before implementation. Re-check after each phase.*

**Constitution Version**: v1.0.0 (`.specify/memory/constitution.md`)

### Principle I: Simplicity
**Status**: ⚠️ CONDITIONAL PASS

**Analysis**:
- Large feature scope (38 FR, 7 user stories) suggests complexity
- **Mitigation**: Phased MVP approach breaks work into manageable increments
  - Phase 1: Privacy dashboard + Bluesky auth + basic command sharing (core value)
  - Phase 2: Runbooks + guilds + win stories (social layer)
  - Phase 3: Epic fails + advanced features (quality improvement)
- Each phase delivers independent value and can be shipped separately
- No over-engineering: Uses standard Next.js patterns, existing design system, proven libraries (@atproto/api)
- Component design follows atomic principles (atoms → molecules → organisms)

**Decision**: PASS with phased approach. Each phase maintains simplicity independently.

### Principle II: Library-First Architecture
**Status**: ✅ PASS

**Structure**:
```
lib/
├── atproto/         # Bluesky client (pure functions, no UI)
├── privacy/         # PII redaction engine (pure, testable)
├── caro-cli/        # Local CLI data interface (abstracted)
└── utils/           # Shared utilities
```

- Privacy redaction engine (`lib/privacy/`) is pure TypeScript with regex patterns (testable independent of React)
- Bluesky client (`lib/atproto/`) wraps @atproto/api with custom Lexicon schemas (testable)
- Local CLI data interface (`lib/caro-cli/`) abstracts access method (File API vs. HTTP server vs. native wrapper)
- UI components consume library functions; no business logic in components
- All data transformations testable in isolation

**Decision**: PASS - Clear separation between business logic and presentation.

### Principle III: Test-First Development (TDD)
**Status**: ✅ PASS

**Test Strategy**:
1. **Unit Tests** (Vitest): Privacy redaction patterns, Bluesky client functions, data transformations
2. **Integration Tests** (Vitest + MSW): Bluesky OAuth flow, artifact publishing, guild feeds
3. **E2E Tests** (Playwright): Complete user flows from spec.md acceptance scenarios
4. **Accessibility Tests** (Pa11y, axe-core): WCAG AA compliance for all pages
5. **Visual Regression** (Percy/Chromatic): 8-bit design system consistency

**TDD Workflow** (per work package):
1. RED: Write failing test from acceptance scenario
2. GREEN: Implement minimum code to pass
3. REFACTOR: Extract to lib/, apply design patterns
4. REPEAT: Next acceptance scenario

**Test Coverage Target**: 80%+ for lib/, 60%+ for components

**Decision**: PASS - Comprehensive test strategy with TDD workflow.

### Principle IV: Safety-First Development
**Status**: ✅ PASS

**Security Measures**:
1. **Privacy Redaction**: Regex patterns for API keys, JWT, AWS keys, SSH keys, emails, IPs, paths, env vars
2. **OAuth Token Storage**: Encrypted localStorage with secure key derivation (Web Crypto API)
3. **XSS Prevention**: React's built-in escaping + DOMPurify for markdown content
4. **CSRF Protection**: Next.js CSRF tokens for form submissions
5. **Content Security Policy**: Strict CSP headers via Next.js config
6. **Rate Limiting**: Client-side queue for Bluesky API calls (prevent abuse)
7. **Input Validation**: Zod schemas for all user inputs
8. **Dependency Scanning**: npm audit + Dependabot alerts

**Privacy-First Safeguards**:
- All local data operations work offline (zero network dependency)
- Explicit user confirmation required for every share
- Diff preview before publish (show original vs. redacted)
- Secondary validation prevents manual redaction bypass
- Complete audit trail of all shares (what, when, where, redactions applied)

**Decision**: PASS - Comprehensive security and privacy measures.

### Principle V: Observability & Versioning
**Status**: ✅ PASS

**Observability**:
- **Client-side**: Browser console logs (development), Sentry error tracking (production)
- **Privacy dashboard**: Shows all collected data, sharing history, redaction stats
- **Bluesky publishing**: Retry logs, failure reasons, offline queue status
- **Performance**: Web Vitals tracking (LCP, FID, CLS)

**Versioning**:
- **Semantic versioning**: Starting at v1.0.0 (feature complete MVP)
- **Feature flags**: Environment variables for phased rollout
- **Breaking changes**: None expected (additive features only)
- **Bluesky Lexicon versions**: All custom record types versioned (app.caro.share.command.v1)

**Monitoring**:
- Privacy scan false negative rate (goal: <5%)
- Bluesky OAuth success rate (goal: >98%)
- Publishing success rate (goal: 100% when online)
- Privacy dashboard load time (goal: <500ms)

**Decision**: PASS - Strong observability and versioning strategy.

**Overall Assessment**: ✅ ALL GATES PASSED - Phased MVP approach maintains simplicity. Library-first architecture, TDD workflow, comprehensive security, and observability meet all constitutional principles.

## Project Structure

### Documentation (this feature)

```
kitty-specs/008-caro-web-hub/
├── spec.md              # Feature specification ✅ COMPLETE
├── meta.json            # Feature metadata ✅ COMPLETE
├── plan.md              # This file (Phase 1 output) ✅ COMPLETE
├── research.md          # Phase 0 output ⏳ PENDING
├── data-model.md        # Phase 0 output ⏳ PENDING
├── quickstart.md        # Phase 1 output ⏳ PENDING
├── contracts/           # Phase 1 output ⏳ PENDING
│   ├── privacy-engine.ts       # Privacy redaction contract
│   ├── bluesky-client.ts       # Bluesky publishing contract
│   ├── local-cli-interface.ts  # Local data access contract
│   └── artifact-schemas.ts     # Data model contracts
└── tasks.md             # Phase 2 output (generated by /spec-kitty.tasks)
```

### Source Code (apps/devrel/)

**Structure**: Next.js 16 App Router application

```
apps/devrel/
├── app/
│   ├── layout.tsx               # Root layout ✅ EXISTS
│   ├── page.tsx                 # Landing page ✅ EXISTS
│   ├── globals.css              # 8-bit design system ✅ EXISTS
│   ├── auth/
│   │   ├── login/page.tsx       # Bluesky OAuth login 🆕
│   │   ├── callback/page.tsx    # OAuth callback 🆕
│   │   └── logout/page.tsx      # Logout handler 🆕
│   ├── dashboard/
│   │   ├── page.tsx             # Privacy dashboard 🆕
│   │   ├── profile/page.tsx     # User profile 🆕
│   │   ├── settings/page.tsx    # Privacy settings 🆕
│   │   └── telemetry/page.tsx   # Telemetry review 🆕
│   ├── share/
│   │   ├── command/page.tsx     # Share command artifact 🆕
│   │   ├── runbook/page.tsx     # Create runbook 🆕
│   │   ├── win/page.tsx         # Post win story 🆕
│   │   └── fail/page.tsx        # Report epic fail 🆕
│   ├── guilds/
│   │   ├── page.tsx             # Guild discovery 🆕
│   │   ├── [id]/page.tsx        # Guild detail + feed 🆕
│   │   └── join/page.tsx        # Join guild flow 🆕
│   ├── api/
│   │   ├── bluesky/
│   │   │   ├── auth/route.ts    # OAuth endpoints 🆕
│   │   │   └── publish/route.ts # Publishing proxy 🆕
│   │   └── cli/
│   │       └── data/route.ts    # Local CLI data access 🆕
│   └── artifact/
│       └── [id]/page.tsx        # View shared artifact 🆕
├── components/
│   ├── Hero.tsx                 # Hero section ✅ EXISTS
│   ├── Features.tsx             # Feature grid ✅ EXISTS
│   ├── Navigation.tsx           # Top nav ✅ EXISTS - UPDATE for auth
│   ├── Footer.tsx               # Footer ✅ EXISTS
│   ├── TerminalWindow.tsx       # Terminal display ✅ EXISTS
│   ├── Profile/
│   │   ├── ProfileHeader.tsx    # Profile header 🆕
│   │   ├── StatsCard.tsx        # Stats display 🆕
│   │   └── GuildList.tsx        # Joined guilds 🆕
│   ├── Privacy/
│   │   ├── TelemetryDashboard.tsx       # Main dashboard 🆕
│   │   ├── PrivacyScanResults.tsx       # Scan results 🆕
│   │   ├── RedactionReview.tsx          # Redaction UI 🆕
│   │   ├── SharingHistory.tsx           # Share history 🆕
│   │   └── PrivacySettings.tsx          # Settings panel 🆕
│   ├── Share/
│   │   ├── CommandForm.tsx      # Command share form 🆕
│   │   ├── RunbookEditor.tsx    # Runbook editor 🆕
│   │   ├── WinStoryForm.tsx     # Win story form 🆕
│   │   ├── FailReportForm.tsx   # Fail report form 🆕
│   │   └── ArtifactCard.tsx     # Artifact display 🆕
│   ├── Guild/
│   │   ├── GuildCard.tsx        # Guild preview 🆕
│   │   ├── GuildFeed.tsx        # Guild feed 🆕
│   │   ├── FeedFilters.tsx      # Feed filtering 🆕
│   │   └── JoinButton.tsx       # Join/leave guild 🆕
│   └── Auth/
│       ├── LoginButton.tsx      # Bluesky login 🆕
│       ├── LogoutButton.tsx     # Logout 🆕
│       └── AuthGuard.tsx        # Protected route wrapper 🆕
├── lib/
│   ├── atproto/
│   │   ├── client.ts            # Bluesky client 🆕
│   │   ├── auth.ts              # OAuth helpers 🆕
│   │   ├── publish.ts           # Publishing functions 🆕
│   │   ├── lexicons/
│   │   │   ├── command.ts       # Command Lexicon schema 🆕
│   │   │   ├── runbook.ts       # Runbook Lexicon 🆕
│   │   │   ├── win.ts           # Win Lexicon 🆕
│   │   │   └── fail.ts          # Fail Lexicon 🆕
│   │   └── feeds.ts             # Custom feed algorithms 🆕
│   ├── privacy/
│   │   ├── scanner.ts           # PII detection engine 🆕
│   │   ├── patterns.ts          # Regex patterns 🆕
│   │   ├── redactor.ts          # Redaction logic 🆕
│   │   └── types.ts             # Privacy types 🆕
│   ├── caro-cli/
│   │   ├── interface.ts         # Abstract data interface 🆕
│   │   ├── file-api.ts          # File System Access API 🆕
│   │   ├── http-server.ts       # Local HTTP client 🆕
│   │   └── mock-data.ts         # Development mock data 🆕
│   ├── guilds/
│   │   ├── defaults.ts          # 15+ default guilds 🆕
│   │   ├── moderationts        # Moderation logic 🆕
│   │   └── types.ts             # Guild types 🆕
│   └── utils/
│       ├── crypto.ts            # Encryption helpers 🆕
│       ├── validation.ts        # Zod schemas 🆕
│       └── storage.ts           # LocalStorage/IndexedDB 🆕
├── types/
│   ├── artifacts.ts             # All artifact types 🆕
│   ├── user.ts                  # User profile types 🆕
│   ├── guild.ts                 # Guild types 🆕
│   └── bluesky.ts               # Bluesky API types 🆕
├── hooks/
│   ├── useAuth.ts               # Auth state hook 🆕
│   ├── usePrivacyScan.ts        # Privacy scanning hook 🆕
│   ├── useLocalCLI.ts           # Local CLI data hook 🆕
│   └── useGuild.ts              # Guild operations hook 🆕
└── public/
    └── mascot/                  # Kyaro sprites ✅ EXISTS (placeholder)
```

**Legend**:
- ✅ EXISTS: Already implemented
- ✅ EXISTS - UPDATE: Needs modification
- 🆕: New file to create

**File Count**: ~60 new files + ~5 updates = 65 files total

## Complexity Tracking

**Potential Constitutional Violations**: None identified. Phased MVP approach mitigates scope complexity.

**Complexity Mitigations**:

1. **Large Feature Scope** (38 FR, 7 user stories)
   - **Mitigation**: Phased MVP (P1→P2→P3), each phase independently shippable
   - **Risk**: Medium
   - **Status**: Mitigated

2. **Bluesky AT Protocol Integration** (new dependency)
   - **Mitigation**: Use official @atproto/api SDK (battle-tested), extensive docs available
   - **Risk**: Medium
   - **Status**: Mitigated

3. **Privacy Redaction Engine** (custom implementation)
   - **Mitigation**: Start with conservative regex patterns (flag more, not less), iterate based on false positive/negative rates
   - **Risk**: High (false negatives = leaked credentials)
   - **Status**: Requires careful testing, quarterly pattern updates

4. **Local CLI Data Access** (browser security restrictions)
   - **Mitigation**: Multi-strategy approach (File API, local HTTP server, Electron/Tauri wrapper), graceful degradation
   - **Risk**: High
   - **Status**: Prototype all strategies in Phase 1

5. **Cross-Browser Compatibility** (File System Access API limited to Chrome)
   - **Mitigation**: Fallback to local HTTP server (universal), document hybrid approach in README
   - **Risk**: Medium
   - **Status**: Acceptable (local server works everywhere)

## Implementation Approach

### Phase 1: Privacy Dashboard + Bluesky Auth + Basic Sharing (MVP Core)

**Goal**: Deliver core privacy-first value - users can review local CLI data and publish simple command artifacts to Bluesky.

**Scope** (User Stories 1, 2, 6):
- Privacy dashboard showing local telemetry (FR-005 to FR-010)
- Bluesky OAuth authentication (FR-001 to FR-004)
- Basic command artifact sharing (FR-011 to FR-015)
- Privacy redaction engine (FR-006, FR-007, FR-008)
- Simple guild feed (read-only, FR-026 to FR-030)

**Deliverables**:
1. Privacy dashboard page (`app/dashboard/page.tsx`)
2. Bluesky OAuth flow (`app/auth/*`, `lib/atproto/auth.ts`)
3. Privacy scanner (`lib/privacy/*`)
4. Command share form (`app/share/command/page.tsx`)
5. Guild discovery and feed (`app/guilds/*`)
6. Bluesky publishing (`lib/atproto/publish.ts`)

**Success Criteria** (from spec):
- SC-001: Privacy dashboard loads offline with zero network requests
- SC-002: Privacy scan detects 95%+ sensitive data
- SC-003: Command artifacts publish to Bluesky 100% success (when online)
- SC-007: Bluesky OAuth completes 98%+ success rate

**Estimated Effort**: 3-4 weeks (baseline for subsequent phases)

---

### Phase 2: Runbooks + Guilds + Win Stories (Social Layer)

**Goal**: Enable richer knowledge sharing with multi-step runbooks, guild communities, and celebration stories.

**Scope** (User Stories 3, 5, 7):
- Runbook creation and sharing (FR-016 to FR-020)
- Guild join/leave, moderation basics (FR-026 to FR-030)
- Win story posting (FR-031 to FR-034)
- Feed sorting (Recent, Helpful, Trending)
- Forking/bookmarking features

**Deliverables**:
1. Runbook editor (`app/share/runbook/page.tsx`, `components/Share/RunbookEditor.tsx`)
2. Guild membership system (`lib/guilds/*`)
3. Win story form (`app/share/win/page.tsx`)
4. Feed algorithms (`lib/atproto/feeds.ts`)
5. Example runbook: "Rebase with main, then continue" (pre-populated)

**Success Criteria**:
- SC-004: Runbooks with 1-20 steps publish 100% success
- SC-006: Guild join/view operations 100% success
- SC-008: Win stories render markdown 100% accuracy

**Estimated Effort**: 2-3 weeks

---

### Phase 3: Epic Fails + Advanced Features (Quality Improvement)

**Goal**: Complete the feedback loop - users can report dangerous commands to improve Caro's safety.

**Scope** (User Story 4):
- Epic fail reporting (FR-021 to FR-025)
- Advanced privacy settings (auto-redaction rules)
- Sharing history management (edit, delete, revoke)
- Performance optimizations (lazy loading, virtualization)

**Deliverables**:
1. Fail report form (`app/share/fail/page.tsx`)
2. Log redaction (aggressive patterns)
3. Privacy settings panel (`app/dashboard/settings/page.tsx`)
4. Sharing history UI (`components/Privacy/SharingHistory.tsx`)
5. Performance audit and optimization

**Success Criteria**:
- SC-005: Epic fail reports create/redact logs 100% success
- SC-009: System operates entirely offline for dashboard

**Estimated Effort**: 1-2 weeks

---

### Key Technical Decisions

#### 1. Local CLI Data Access Strategy

**Selected**: Multi-strategy hybrid approach

**Options Considered**:
- **Option A**: File System Access API (Chrome only)
- **Option B**: Local HTTP server (universal, requires CLI to run server)
- **Option C**: Electron/Tauri native wrapper (desktop app)

**Decision**: Implement all three with graceful fallback
1. Try File System Access API (best UX, Chrome/Edge only)
2. Fallback to local HTTP server at localhost:3000 (works everywhere, requires CLI running)
3. Provide Electron/Tauri wrapper for desktop users (Phase 3+)
4. Development mode: Use mock data (`lib/caro-cli/mock-data.ts`)

**Rationale**:
- File API provides best UX but limited browser support
- Local HTTP server is universal and simple (CLI already runs as daemon)
- Hybrid approach maximizes compatibility without compromising UX

---

#### 2. Privacy Redaction Pattern Strategy

**Selected**: Conservative regex patterns with quarterly updates

**Patterns** (from spec FR-006):
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

**Strategy**:
- Start conservative (flag more, not less)
- Track false positive/negative rates in telemetry
- Quarterly review and pattern updates
- Community reporting for new formats
- Secondary validation before publish (prevent manual bypass)

**Rationale**: Privacy leaks are catastrophic; better to over-redact and let users manually approve than under-redact and leak credentials.

---

#### 3. Bluesky Lexicon Schema Design

**Selected**: Versioned custom record types per artifact

**Schema Namespace**: `app.caro.share.*`

**Record Types**:
```
app.caro.share.command.v1     # Command artifacts
app.caro.share.runbook.v1     # Runbooks
app.caro.share.win.v1         # Win stories
app.caro.share.fail.v1        # Epic fails
app.caro.guild.membership.v1  # Guild associations
app.caro.profile.prefs.v1     # User preferences
```

**Versioning Strategy**:
- All schemas start at v1
- Breaking changes increment version (v1 → v2)
- Non-breaking additions keep version (add optional fields)
- Older versions remain supported (read-only)

**Rationale**: Versioned schemas allow future changes without breaking existing artifacts; follows Bluesky best practices.

---

#### 4. State Management Approach

**Selected**: Zustand for global state, React hooks for local state

**State Slices**:
```typescript
// stores/authStore.ts
interface AuthStore {
  user: UserProfile | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (did: string, token: string) => void;
  logout: () => void;
}

// stores/privacyStore.ts
interface PrivacyStore {
  localCLIData: CLIData[];
  sharingHistory: Share[];
  privacySettings: PrivacySettings;
  scanResults: ScanResult[];
  updateSettings: (settings: PrivacySettings) => void;
}

// stores/guildStore.ts
interface GuildStore {
  joinedGuilds: Guild[];
  feedCache: Map<string, Artifact[]>;
  joinGuild: (guildId: string) => void;
  leaveGuild: (guildId: string) => void;
}
```

**Rationale**:
- Zustand is lightweight (~1KB), simple API, no boilerplate
- Avoids Context hell for deep component trees
- Easy to test (stores are plain JS objects)
- SSR-friendly (hydration support)

Alternative considered: Jotai (atom-based) - Rejected as overkill for this use case.

---

#### 5. Testing Strategy Details

**Unit Tests** (Vitest):
```
lib/privacy/__tests__/scanner.test.ts
lib/atproto/__tests__/publish.test.ts
lib/caro-cli/__tests__/interface.test.ts
hooks/__tests__/usePrivacyScan.test.ts
```

**Integration Tests** (Vitest + MSW):
```
__tests__/integration/bluesky-auth.test.tsx
__tests__/integration/command-publish.test.tsx
__tests__/integration/guild-feed.test.tsx
```

**E2E Tests** (Playwright):
```
__tests__/e2e/privacy-dashboard.spec.ts     # SC-001
__tests__/e2e/command-share-flow.spec.ts    # SC-003
__tests__/e2e/runbook-creation.spec.ts      # SC-004
__tests__/e2e/bluesky-oauth.spec.ts         # SC-007
```

**Accessibility Tests** (Pa11y):
```
__tests__/a11y/dashboard.test.ts
__tests__/a11y/share-forms.test.ts
__tests__/a11y/guild-pages.test.ts
```

**Coverage Targets**:
- lib/* (business logic): 80%+
- components/* (UI): 60%+
- hooks/* (React hooks): 70%+
- Overall: 70%+

---

## Phase 1 Design Artifacts

### Still Required:

1. **research.md** - Phase 0 research findings
   - Bluesky AT Protocol documentation review
   - Privacy redaction pattern research (OWASP, CWE)
   - Local file access strategies comparison
   - 8-bit design system audit (existing apps/devrel/)

2. **data-model.md** - Phase 0 data model
   - TypeScript interfaces for all artifacts (Command, Runbook, Win, Fail)
   - User profile schema
   - Guild schema
   - Privacy scan result schema
   - Bluesky Lexicon schemas

3. **quickstart.md** - Phase 1 developer guide
   - Development setup (local server, mock data)
   - Running tests (unit, integration, E2E)
   - Bluesky sandbox setup
   - Common workflows (create artifact, test privacy scan)

4. **contracts/** - Phase 1 API contracts
   - `privacy-engine.ts`: Privacy scanner contract (input: artifact, output: ScanResult)
   - `bluesky-client.ts`: Publishing contract (input: artifact, output: AT URI)
   - `local-cli-interface.ts`: Data access contract (input: void, output: CLIData[])
   - `artifact-schemas.ts`: Zod schemas for all artifact types

**Next Step**: Execute `/spec-kitty.research` to create research.md and data-model.md, then proceed to `/spec-kitty.tasks` for work package generation.

---

## Risk Register

| Risk ID | Risk | Impact | Probability | Mitigation | Status |
|---------|------|--------|-------------|------------|--------|
| R-001 | Bluesky AT Protocol changes break compatibility | High | Low | Pin @atproto/api version, monitor Bluesky releases, maintain version adapter layer | Active |
| R-002 | Privacy redaction false negatives leak credentials | Critical | Medium | Conservative patterns, secondary validation, quarterly updates, incident response plan | Active |
| R-003 | Local CLI data access blocked by browser security | High | High | Multi-strategy hybrid (File API + HTTP server + native wrapper), graceful degradation | Mitigated |
| R-004 | Bluesky rate limits block power users | Medium | Medium | Client-side queue, clear feedback, request higher limits for verified app | Active |
| R-005 | Guild spam overwhelms moderators | Medium | High | Phase 1: community reporting, Phase 2: automated detection, appoint volunteers | Planned |
| R-006 | Performance degrades with large datasets (10,000+ commands) | Medium | Low | Lazy loading, virtualization, IndexedDB pagination, Web Workers for scanning | Planned |
| R-007 | OAuth token theft via XSS | Critical | Low | React escaping, DOMPurify, strict CSP, encrypted storage, short-lived tokens | Mitigated |
| R-008 | Bluesky network downtime blocks publishing | Low | Medium | Offline queue, retry with exponential backoff, clear user feedback | Planned |

---

## Success Metrics (Phase 1)

**Privacy Dashboard**:
- [ ] Loads local CLI data in <500ms for 10,000 commands
- [ ] Zero network requests for offline review
- [ ] Privacy scan detects 95%+ sensitive data in test dataset

**Bluesky Authentication**:
- [ ] OAuth flow completes in <3 seconds
- [ ] 98%+ success rate (excluding network failures)
- [ ] Token refresh works seamlessly (no re-login required)

**Command Sharing**:
- [ ] Publish to Bluesky succeeds 100% when online
- [ ] Redaction UI shows diff preview clearly
- [ ] Published artifacts appear in guild feed within 5 seconds

**Guild System**:
- [ ] 15+ default guilds available at launch
- [ ] Guild feed renders first 20 items in <1 second
- [ ] Sort/filter operations complete in <500ms

**8-Bit Design Consistency**:
- [ ] All pages follow existing design system (Game Boy palette, pixel fonts)
- [ ] Visual regression tests pass (no unintended style changes)
- [ ] WCAG AA accessibility compliance (Pa11y zero errors)

---

## Definition of Done (Phase 1 MVP)

**Feature Complete**:
- [ ] All Phase 1 user stories implemented (1, 2, 6)
- [ ] All Phase 1 functional requirements satisfied (FR-001 to FR-015, FR-026 to FR-030)
- [ ] Example "Rebase with main" runbook data available (for Phase 2 demo)

**Quality Gates**:
- [ ] 80%+ test coverage for lib/*
- [ ] All E2E tests pass (privacy dashboard, command share, OAuth)
- [ ] All accessibility tests pass (Pa11y zero errors)
- [ ] No critical security vulnerabilities (npm audit, Snyk)

**Performance**:
- [ ] Privacy dashboard <500ms load (10,000 commands)
- [ ] Privacy scan <2s processing (1,000-line artifact)
- [ ] Guild feed <1s render (first 20 items)
- [ ] Lighthouse score: Performance 90+, Accessibility 100

**Documentation**:
- [ ] README updated with Phase 1 features
- [ ] quickstart.md created for developers
- [ ] contracts/ defined for all APIs
- [ ] Inline code documentation (JSDoc for lib/*)

**Deployment**:
- [ ] Vercel preview deployment successful
- [ ] Build passes on CI (lint, type-check, test, build)
- [ ] Production build <5MB (optimized bundle)

**User Acceptance**:
- [ ] Privacy dashboard clearly shows all local data
- [ ] Privacy scan flags sensitive data accurately
- [ ] Command sharing flow is intuitive (≤5 clicks)
- [ ] Bluesky OAuth works smoothly (no errors)
- [ ] Guild feed displays artifacts correctly

**Phase 1 Shipped**: All above criteria met → Ready for Phase 2 (Runbooks + Win Stories).

---

**Next Actions**:
1. Run `/spec-kitty.research` to create research.md and data-model.md
2. Run `/spec-kitty.tasks` to generate work packages from this plan
3. Begin implementation with Phase 1, Work Package 1 (Privacy Dashboard)
