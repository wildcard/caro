# Work Packages: Caro Web Hub (Phase 1 MVP)

**Feature**: 008-caro-web-hub
**Branch**: claude/caro-web-hub-014Ku7d3Yf4a6FwX1TBskyHy
**Phase**: 1 - Privacy Dashboard + Bluesky Auth + Basic Sharing
**Generated**: 2026-01-01

## Phase 1 Scope

**User Stories**: 1 (Privacy Dashboard), 2 (Command Sharing), 6 (Bluesky Auth)
**Estimated Duration**: 3-4 weeks
**LOC Estimate**: ~4,000-5,000 lines

## Work Package Summary

| WP | Title | Priority | Est. Effort | Dependencies | Status |
|----|-------|----------|-------------|--------------|--------|
| WP01 | Project Setup & Foundation | P0 | 4h | None | 🟢 Ready |
| WP02 | Privacy Redaction Engine | P1 | 8h | WP01 | ⬜ Planned |
| WP03 | Local CLI Data Interface | P1 | 12h | WP01 | ⬜ Planned |
| WP04 | Privacy Dashboard UI | P1 | 16h | WP02, WP03 | ⬜ Planned |
| WP05 | Bluesky OAuth Authentication | P1 | 12h | WP01 | ⬜ Planned |
| WP06 | Bluesky Publishing Client | P1 | 12h | WP05 | ⬜ Planned |
| WP07 | Command Sharing Flow | P1 | 16h | WP02, WP05, WP06 | ⬜ Planned |
| WP08 | Guild System (Read-Only) | P2 | 12h | WP06 | ⬜ Planned |
| WP09 | Testing & Quality Assurance | P1 | 16h | WP01-WP08 | ⬜ Planned |
| WP10 | Documentation & Deployment | P2 | 8h | WP01-WP09 | ⬜ Planned |

**Total Estimated Effort**: 116 hours (~3 weeks full-time)

---

## WP01: Project Setup & Foundation

**Priority**: P0 (Blocker)
**Estimated Effort**: 4 hours
**Dependencies**: None
**Owner**: TBD

### Objectives

- Set up Next.js 16 project structure in apps/devrel/
- Configure TypeScript, ESLint, Tailwind CSS 4
- Set up testing infrastructure (Vitest, Playwright, Pa11y)
- Create base types and utilities
- Verify existing design system integration

### Tasks

1. **Verify Next.js 16 Setup** ✅ (Already done)
   - [x] package.json with Next.js 16.1.1 (CVE patched)
   - [x] Tailwind CSS 4 configured
   - [x] TypeScript 5.x
   - [x] ESLint configured

2. **Add Testing Dependencies**
   ```bash
   npm install -D vitest @vitest/ui @testing-library/react @testing-library/jest-dom
   npm install -D playwright @playwright/test
   npm install -D pa11y axe-core
   npm install -D msw  # Mock Service Worker for API mocking
   ```

3. **Create Base TypeScript Types**
   - [ ] Create `types/artifacts.ts` with all artifact interfaces (from data-model.md)
   - [ ] Create `types/user.ts` with UserProfile, PrivacySettings
   - [ ] Create `types/guild.ts` with Guild interface
   - [ ] Create `types/bluesky.ts` with AT Protocol types
   - [ ] Create `types/privacy.ts` with scan result types

4. **Set Up Testing Infrastructure**
   - [ ] Create `vitest.config.ts`
   - [ ] Create `playwright.config.ts`
   - [ ] Create `__tests__/setup.ts` (test utilities)
   - [ ] Add test scripts to package.json
   - [ ] Create `__tests__/helpers/mockData.ts` (shared fixtures)

5. **Create Base Utilities**
   - [ ] Create `lib/utils/storage.ts` (localStorage/IndexedDB helpers)
   - [ ] Create `lib/utils/crypto.ts` (encryption helpers for OAuth tokens)
   - [ ] Create `lib/utils/validation.ts` (Zod schemas)

### Acceptance Criteria

- ✅ All dependencies installed (no npm audit vulnerabilities)
- ✅ `npm run dev` starts development server
- ✅ `npm run build` produces successful production build
- ✅ `npm run test` runs Vitest tests
- ✅ `npm run test:e2e` runs Playwright tests
- ✅ All TypeScript types compile without errors
- ✅ Existing design system (globals.css) still works

### Files Created/Modified

**New Files** (~10 files):
- types/artifacts.ts
- types/user.ts
- types/guild.ts
- types/bluesky.ts
- types/privacy.ts
- lib/utils/storage.ts
- lib/utils/crypto.ts
- lib/utils/validation.ts
- vitest.config.ts
- playwright.config.ts
- __tests__/setup.ts
- __tests__/helpers/mockData.ts

**Modified Files**:
- package.json (add test scripts)
- tsconfig.json (if needed)

---

## WP02: Privacy Redaction Engine

**Priority**: P1 (Critical)
**Estimated Effort**: 8 hours
**Dependencies**: WP01
**Owner**: TBD

### Objectives

- Implement privacy scanning engine with regex patterns
- Detect 10 types of sensitive data (API keys, tokens, paths, emails, etc.)
- Provide redaction suggestions
- Achieve 95%+ recall (detect almost all sensitive data)
- Create comprehensive test suite

### Tasks

1. **Create Privacy Pattern Definitions**
   - [ ] Create `lib/privacy/patterns.ts` with all regex patterns (from research.md)
   - [ ] Export `REDACTION_PATTERNS` constant
   - [ ] Add pattern metadata (severity, replacement template)

2. **Implement Scanner Engine**
   - [ ] Create `lib/privacy/scanner.ts`
   - [ ] Implement `scanText(text: string): PrivacyScanResult`
   - [ ] Implement `scanArtifact(artifact: Artifact): PrivacyScanResult`
   - [ ] Support all 10 sensitive data types
   - [ ] Return DetectedItem[] with positions and suggestions

3. **Implement Redaction Logic**
   - [ ] Create `lib/privacy/redactor.ts`
   - [ ] Implement `applyRedactions(text: string, redactions: Redaction[]): string`
   - [ ] Implement `generateDiff(original: string, redacted: string): DiffResult`
   - [ ] Support preview mode (show before/after)

4. **Create Privacy Types**
   - [ ] Create `lib/privacy/types.ts`
   - [ ] Define PrivacyScanResult, DetectedItem, Redaction
   - [ ] Export all privacy-related types

5. **Write Comprehensive Tests**
   - [ ] Create `lib/privacy/__tests__/scanner.test.ts`
   - [ ] Test all 10 pattern types with positive/negative cases
   - [ ] Test edge cases (empty input, very long strings, unicode)
   - [ ] Test false positive rate (acceptable: <40%)
   - [ ] Test recall rate (target: 95%+)
   - [ ] Create test dataset with 100+ known sensitive strings

### Acceptance Criteria

- ✅ Scanner detects all 10 sensitive data types
- ✅ Test suite has 95%+ recall on test dataset
- ✅ False positive rate <40%
- ✅ Scanner processes 1,000-line text in <2 seconds
- ✅ All tests pass (`npm run test lib/privacy`)
- ✅ Zero TypeScript errors
- ✅ 80%+ code coverage for lib/privacy/

### Files Created

**New Files** (~5 files):
- lib/privacy/patterns.ts
- lib/privacy/scanner.ts
- lib/privacy/redactor.ts
- lib/privacy/types.ts
- lib/privacy/__tests__/scanner.test.ts
- lib/privacy/__tests__/redactor.test.ts
- lib/privacy/__tests__/fixtures/sensitive-data.json

---

## WP03: Local CLI Data Interface

**Priority**: P1 (Critical)
**Estimated Effort**: 12 hours
**Dependencies**: WP01
**Owner**: TBD

### Objectives

- Create abstraction for accessing local Caro CLI data
- Implement File System Access API strategy (Chrome/Edge)
- Implement fallback HTTP server client (universal)
- Provide mock data for development
- Handle errors gracefully with clear user feedback

### Tasks

1. **Create CLI Data Interface**
   - [ ] Create `lib/caro-cli/interface.ts`
   - [ ] Define `CLIDataProvider` interface
   - [ ] Implement `getCommandHistory(): Promise<CommandHistory[]>`
   - [ ] Implement `getPromptHistory(): Promise<PromptHistory[]>`
   - [ ] Implement `getSafetyEvents(): Promise<SafetyEvent[]>`
   - [ ] Implement `getBackendStats(): Promise<BackendStat[]>`
   - [ ] Implement `getErrorLogs(): Promise<ErrorLog[]>`

2. **Implement File System Access API Strategy**
   - [ ] Create `lib/caro-cli/file-api.ts`
   - [ ] Implement `FileSystemCLIProvider implements CLIDataProvider`
   - [ ] Request directory permission (showDirectoryPicker)
   - [ ] Read `.caro/history.json`
   - [ ] Parse and validate data (Zod schema)
   - [ ] Handle permission denial gracefully

3. **Implement HTTP Server Strategy**
   - [ ] Create `lib/caro-cli/http-server.ts`
   - [ ] Implement `HTTPServerCLIProvider implements CLIDataProvider`
   - [ ] Fetch from `http://localhost:3000/api/history`
   - [ ] Handle CORS and authentication
   - [ ] Retry logic with exponential backoff

4. **Implement Mock Data Provider**
   - [ ] Create `lib/caro-cli/mock-data.ts`
   - [ ] Implement `MockCLIProvider implements CLIDataProvider`
   - [ ] Generate realistic test data (100+ commands, varied backends)
   - [ ] Include sensitive data for privacy testing

5. **Create Provider Factory**
   - [ ] Create `lib/caro-cli/factory.ts`
   - [ ] Implement `getCLIProvider(): CLIDataProvider`
   - [ ] Try File API → HTTP → Mock (progressive fallback)
   - [ ] Store strategy preference in localStorage

6. **Create React Hook**
   - [ ] Create `hooks/useLocalCLI.ts`
   - [ ] Implement `useLocalCLI()` hook
   - [ ] Return `{ data, loading, error, refresh, strategy }`
   - [ ] Handle loading states
   - [ ] Cache data in state

7. **Write Tests**
   - [ ] Create `lib/caro-cli/__tests__/file-api.test.ts`
   - [ ] Create `lib/caro-cli/__tests__/http-server.test.ts`
   - [ ] Create `lib/caro-cli/__tests__/factory.test.ts`
   - [ ] Create `hooks/__tests__/useLocalCLI.test.tsx`
   - [ ] Mock File API with vitest
   - [ ] Mock HTTP with MSW

### Acceptance Criteria

- ✅ File API strategy works in Chrome/Edge (manual test)
- ✅ HTTP strategy works when CLI server is running
- ✅ Mock provider generates realistic data
- ✅ Factory correctly tries strategies in order
- ✅ useLocalCLI hook handles loading/error states
- ✅ All tests pass
- ✅ 70%+ code coverage for lib/caro-cli/

### Files Created

**New Files** (~10 files):
- lib/caro-cli/interface.ts
- lib/caro-cli/file-api.ts
- lib/caro-cli/http-server.ts
- lib/caro-cli/mock-data.ts
- lib/caro-cli/factory.ts
- hooks/useLocalCLI.ts
- lib/caro-cli/__tests__/file-api.test.ts
- lib/caro-cli/__tests__/http-server.test.ts
- lib/caro-cli/__tests__/factory.test.ts
- hooks/__tests__/useLocalCLI.test.tsx

---

## WP04: Privacy Dashboard UI

**Priority**: P1 (Critical)
**Estimated Effort**: 16 hours
**Dependencies**: WP02 (Privacy Engine), WP03 (CLI Interface)
**Owner**: TBD

### Objectives

- Create privacy dashboard page showing all local CLI data
- Display telemetry in categorized sections
- Show privacy scan results with color-coding
- Display sharing history
- Provide privacy settings panel
- Achieve <500ms load time for 10,000 commands
- Zero network requests (offline-first)

### Tasks

1. **Create Dashboard Page**
   - [ ] Create `app/dashboard/page.tsx`
   - [ ] Implement layout with sidebar navigation
   - [ ] Add loading states
   - [ ] Handle error states (no CLI data, permission denied)

2. **Create Telemetry Dashboard Component**
   - [ ] Create `components/Privacy/TelemetryDashboard.tsx`
   - [ ] Display command count, backend usage, safety triggers
   - [ ] Show charts (backend distribution, commands over time)
   - [ ] Use existing .terminal-window style

3. **Create Command History View**
   - [ ] Create `components/Privacy/CommandHistory.tsx`
   - [ ] Display command list with filters (by date, backend, safety score)
   - [ ] Implement virtualized list for performance (react-window)
   - [ ] Add search functionality

4. **Create Privacy Scan Results Component**
   - [ ] Create `components/Privacy/PrivacyScanResults.tsx`
   - [ ] Display detected sensitive data grouped by type
   - [ ] Color-code by severity (red=critical, yellow=moderate)
   - [ ] Show count per type

5. **Create Sharing History Component**
   - [ ] Create `components/Privacy/SharingHistory.tsx`
   - [ ] Display all shares (public, guild, private)
   - [ ] Show redactions applied per share
   - [ ] Allow filtering by visibility, guild, date

6. **Create Privacy Settings Panel**
   - [ ] Create `components/Privacy/PrivacySettings.tsx`
   - [ ] Toggle switches for auto-redaction rules
   - [ ] Default visibility selector
   - [ ] Telemetry collection toggles
   - [ ] Save to localStorage

7. **Implement State Management**
   - [ ] Create `stores/privacyStore.ts` (Zustand)
   - [ ] Define PrivacyStore interface
   - [ ] Implement actions (loadData, updateSettings, addShare)
   - [ ] Persist settings to localStorage

8. **Add Navigation**
   - [ ] Update `components/Navigation.tsx`
   - [ ] Add "Dashboard" link (when authenticated)
   - [ ] Add user avatar/menu dropdown

9. **Write Tests**
   - [ ] Create `__tests__/dashboard.test.tsx`
   - [ ] Create `components/Privacy/__tests__/TelemetryDashboard.test.tsx`
   - [ ] Create `components/Privacy/__tests__/PrivacySettings.test.tsx`
   - [ ] Create E2E test: `__tests__/e2e/privacy-dashboard.spec.ts`
   - [ ] Test loading states, error states, data display
   - [ ] Verify zero network requests (offline mode)

### Acceptance Criteria

- ✅ Dashboard loads in <500ms with 10,000 commands
- ✅ Zero network requests when viewing dashboard
- ✅ All telemetry data displayed correctly
- ✅ Privacy settings save and persist
- ✅ Virtualized list handles large datasets smoothly
- ✅ Color-coding clearly distinguishes severity levels
- ✅ E2E test passes (SC-001 from spec)
- ✅ Pa11y accessibility test passes (zero errors)
- ✅ 60%+ component test coverage

### Files Created

**New Files** (~15 files):
- app/dashboard/page.tsx
- app/dashboard/telemetry/page.tsx
- app/dashboard/settings/page.tsx
- components/Privacy/TelemetryDashboard.tsx
- components/Privacy/CommandHistory.tsx
- components/Privacy/PrivacyScanResults.tsx
- components/Privacy/SharingHistory.tsx
- components/Privacy/PrivacySettings.tsx
- stores/privacyStore.ts
- __tests__/dashboard.test.tsx
- components/Privacy/__tests__/TelemetryDashboard.test.tsx
- components/Privacy/__tests__/PrivacySettings.test.tsx
- __tests__/e2e/privacy-dashboard.spec.ts

**Modified Files**:
- components/Navigation.tsx (add dashboard link)

---

## WP05: Bluesky OAuth Authentication

**Priority**: P1 (Critical)
**Estimated Effort**: 12 hours
**Dependencies**: WP01
**Owner**: TBD

### Objectives

- Implement Bluesky OAuth 2.0 PKCE flow
- Create login/logout pages
- Store encrypted OAuth tokens in localStorage
- Implement automatic token refresh
- Achieve 98%+ OAuth success rate
- Handle network errors gracefully

### Tasks

1. **Install Bluesky SDK**
   ```bash
   npm install @atproto/api @atproto/oauth-client
   ```

2. **Create Bluesky Auth Client**
   - [ ] Create `lib/atproto/auth.ts`
   - [ ] Implement `initiateOAuth(): Promise<{ url: string, codeVerifier: string }>`
   - [ ] Implement `handleOAuthCallback(code: string, verifier: string): Promise<Session>`
   - [ ] Implement `refreshToken(refreshToken: string): Promise<Session>`
   - [ ] Store tokens in encrypted localStorage

3. **Create Auth Pages**
   - [ ] Create `app/auth/login/page.tsx`
   - [ ] Create `app/auth/callback/page.tsx` (OAuth redirect handler)
   - [ ] Create `app/auth/logout/page.tsx`
   - [ ] Add loading states and error handling

4. **Create Auth Components**
   - [ ] Create `components/Auth/LoginButton.tsx`
   - [ ] Create `components/Auth/LogoutButton.tsx`
   - [ ] Create `components/Auth/AuthGuard.tsx` (protected route wrapper)
   - [ ] Add Kyaro mascot to login page (use .sprite-animate)

5. **Implement Auth State Management**
   - [ ] Create `stores/authStore.ts` (Zustand)
   - [ ] Define AuthStore interface (user, token, isAuthenticated, login, logout)
   - [ ] Persist auth state to encrypted localStorage
   - [ ] Implement token refresh logic

6. **Create Auth Hook**
   - [ ] Create `hooks/useAuth.ts`
   - [ ] Implement `useAuth()` hook
   - [ ] Return `{ user, isAuthenticated, login, logout, loading }`
   - [ ] Auto-refresh token before expiry

7. **Add Auth to Navigation**
   - [ ] Update `components/Navigation.tsx`
   - [ ] Show "Login" button when not authenticated
   - [ ] Show user avatar + menu when authenticated
   - [ ] Add logout option to menu

8. **Write Tests**
   - [ ] Create `lib/atproto/__tests__/auth.test.ts`
   - [ ] Create `stores/__tests__/authStore.test.ts`
   - [ ] Create `hooks/__tests__/useAuth.test.tsx`
   - [ ] Create E2E test: `__tests__/e2e/bluesky-oauth.spec.ts`
   - [ ] Mock Bluesky OAuth endpoints with MSW
   - [ ] Test successful login, logout, token refresh
   - [ ] Test error handling (network failure, invalid credentials)

### Acceptance Criteria

- ✅ OAuth flow completes successfully (manual test with real Bluesky account)
- ✅ Tokens stored encrypted in localStorage
- ✅ Auto-refresh works before token expiry
- ✅ Login/logout flows work smoothly
- ✅ AuthGuard correctly protects routes
- ✅ E2E test passes (SC-007 from spec: 98%+ success rate)
- ✅ All tests pass
- ✅ 70%+ code coverage for lib/atproto/auth.ts

### Files Created

**New Files** (~12 files):
- lib/atproto/auth.ts
- app/auth/login/page.tsx
- app/auth/callback/page.tsx
- app/auth/logout/page.tsx
- components/Auth/LoginButton.tsx
- components/Auth/LogoutButton.tsx
- components/Auth/AuthGuard.tsx
- stores/authStore.ts
- hooks/useAuth.ts
- lib/atproto/__tests__/auth.test.ts
- hooks/__tests__/useAuth.test.tsx
- __tests__/e2e/bluesky-oauth.spec.ts

**Modified Files**:
- components/Navigation.tsx (add login/logout UI)

---

## WP06: Bluesky Publishing Client

**Priority**: P1 (Critical)
**Estimated Effort**: 12 hours
**Dependencies**: WP05 (Auth)
**Owner**: TBD

### Objectives

- Create Bluesky publishing client for custom record types
- Define Lexicon schemas for command, runbook, win, fail
- Implement retry logic with exponential backoff
- Handle rate limits gracefully
- Achieve 100% publish success when network available

### Tasks

1. **Define Lexicon Schemas**
   - [ ] Create `lib/atproto/lexicons/command.ts`
   - [ ] Create `lib/atproto/lexicons/runbook.ts`
   - [ ] Create `lib/atproto/lexicons/win.ts`
   - [ ] Create `lib/atproto/lexicons/fail.ts`
   - [ ] Export all Lexicon definitions (from data-model.md)

2. **Create Publishing Client**
   - [ ] Create `lib/atproto/client.ts`
   - [ ] Implement `publishCommand(artifact: CommandArtifact): Promise<string>` (returns AT URI)
   - [ ] Implement `publishRunbook(runbook: Runbook): Promise<string>`
   - [ ] Implement `publishWin(win: WinStory): Promise<string>`
   - [ ] Implement `publishFail(fail: EpicFail): Promise<string>`
   - [ ] Use BskyAgent from @atproto/api

3. **Implement Retry Logic**
   - [ ] Create `lib/atproto/retry.ts`
   - [ ] Implement exponential backoff (2s, 4s, 8s, 16s)
   - [ ] Retry on network errors (max 3 retries)
   - [ ] Don't retry on auth errors (401, 403)
   - [ ] Return clear error messages

4. **Implement Rate Limiting**
   - [ ] Create `lib/atproto/rate-limiter.ts`
   - [ ] Track publish count per hour
   - [ ] Show warning when approaching limit
   - [ ] Queue publishes when rate limited
   - [ ] Clear feedback to user

5. **Create Publishing Hook**
   - [ ] Create `hooks/usePublish.ts`
   - [ ] Implement `usePublish()` hook
   - [ ] Return `{ publish, publishing, error, queue }`
   - [ ] Handle loading states
   - [ ] Show toast notifications (success/error)

6. **Write Tests**
   - [ ] Create `lib/atproto/__tests__/client.test.ts`
   - [ ] Create `lib/atproto/__tests__/retry.test.ts`
   - [ ] Create `lib/atproto/__tests__/rate-limiter.test.ts`
   - [ ] Create `hooks/__tests__/usePublish.test.tsx`
   - [ ] Mock Bluesky API with MSW
   - [ ] Test successful publish, retry on network error
   - [ ] Test rate limiting behavior

### Acceptance Criteria

- ✅ Publishes command artifacts to Bluesky successfully
- ✅ Returns valid AT URI (at://did:plc:.../app.caro.share.command/...)
- ✅ Retry logic works (manual test: simulate network failure)
- ✅ Rate limiter prevents exceeding limits
- ✅ All tests pass
- ✅ 80%+ code coverage for lib/atproto/

### Files Created

**New Files** (~12 files):
- lib/atproto/client.ts
- lib/atproto/publish.ts
- lib/atproto/retry.ts
- lib/atproto/rate-limiter.ts
- lib/atproto/lexicons/command.ts
- lib/atproto/lexicons/runbook.ts
- lib/atproto/lexicons/win.ts
- lib/atproto/lexicons/fail.ts
- hooks/usePublish.ts
- lib/atproto/__tests__/client.test.ts
- lib/atproto/__tests__/publish.test.ts
- lib/atproto/__tests__/retry.test.ts
- hooks/__tests__/usePublish.test.tsx

---

## WP07: Command Sharing Flow

**Priority**: P1 (Critical)
**Estimated Effort**: 16 hours
**Dependencies**: WP02 (Privacy), WP05 (Auth), WP06 (Publishing)
**Owner**: TBD

### Objectives

- Create command sharing page and form
- Integrate privacy scanning and redaction review
- Allow user to add metadata (title, description, tags, guild)
- Implement diff preview (original vs. redacted)
- Publish to Bluesky with explicit user confirmation
- Track sharing history

### Tasks

1. **Create Share Command Page**
   - [ ] Create `app/share/command/page.tsx`
   - [ ] Implement multi-step flow: Select → Review → Publish
   - [ ] Add loading states and progress indicator
   - [ ] Use AuthGuard to protect route

2. **Create Command Selection Component**
   - [ ] Create `components/Share/CommandSelector.tsx`
   - [ ] Display command history from local CLI
   - [ ] Allow user to select command
   - [ ] Show command details (prompt, command, backend, safety score)

3. **Create Command Share Form**
   - [ ] Create `components/Share/CommandForm.tsx`
   - [ ] Fields: title (optional), description, tags, guild, visibility
   - [ ] Validate inputs (Zod schema)
   - [ ] Use pixel-style inputs (.pixel-border)

4. **Create Redaction Review Component**
   - [ ] Create `components/Privacy/RedactionReview.tsx`
   - [ ] Display detected sensitive items
   - [ ] Color-code by severity (red/yellow)
   - [ ] Allow user to confirm/override each item
   - [ ] Show replacement suggestions

5. **Create Diff Preview Component**
   - [ ] Create `components/Share/DiffPreview.tsx`
   - [ ] Show side-by-side: original vs. redacted
   - [ ] Highlight differences
   - [ ] Final confirmation before publish

6. **Create Artifact Card Component**
   - [ ] Create `components/Share/ArtifactCard.tsx`
   - [ ] Display command artifact (for feeds)
   - [ ] Show metadata, engagement stats
   - [ ] Use .pixel-card style

7. **Implement Sharing Logic**
   - [ ] Create `lib/sharing/create-artifact.ts`
   - [ ] Implement `createCommandArtifact(data: FormData, scanResult: PrivacyScanResult): CommandArtifact`
   - [ ] Apply redactions
   - [ ] Validate artifact (Zod schema)

8. **Update Privacy Store**
   - [ ] Add `addShare(share: Share)` action
   - [ ] Update sharing history
   - [ ] Track redaction stats

9. **Write Tests**
   - [ ] Create `components/Share/__tests__/CommandForm.test.tsx`
   - [ ] Create `components/Privacy/__tests__/RedactionReview.test.tsx`
   - [ ] Create E2E test: `__tests__/e2e/command-share-flow.spec.ts`
   - [ ] Test complete flow: select → review → publish
   - [ ] Verify redactions applied correctly
   - [ ] Verify artifact appears in sharing history

### Acceptance Criteria

- ✅ User can select command from history
- ✅ Privacy scan runs automatically
- ✅ Redaction review shows all detected items
- ✅ Diff preview clearly shows changes
- ✅ Publish succeeds 100% when network available (SC-003)
- ✅ Sharing history updates correctly
- ✅ E2E test passes
- ✅ Pa11y accessibility test passes

### Files Created

**New Files** (~12 files):
- app/share/command/page.tsx
- components/Share/CommandSelector.tsx
- components/Share/CommandForm.tsx
- components/Privacy/RedactionReview.tsx
- components/Share/DiffPreview.tsx
- components/Share/ArtifactCard.tsx
- lib/sharing/create-artifact.ts
- components/Share/__tests__/CommandForm.test.tsx
- components/Privacy/__tests__/RedactionReview.test.tsx
- __tests__/e2e/command-share-flow.spec.ts

**Modified Files**:
- stores/privacyStore.ts (add sharing history actions)
- components/Navigation.tsx (add "Share" link)

---

## WP08: Guild System (Read-Only)

**Priority**: P2 (Important)
**Estimated Effort**: 12 hours
**Dependencies**: WP06 (Publishing)
**Owner**: TBD

### Objectives

- Create guild discovery page
- Display 15+ default guilds
- Implement guild detail page with feed
- Fetch artifacts from Bluesky feed algorithm
- Implement feed sorting (Recent, Helpful, Trending)
- Allow users to join/leave guilds

### Tasks

1. **Define Default Guilds**
   - [ ] Create `lib/guilds/defaults.ts`
   - [ ] Define 15+ guilds: SRE, AppSec, DevOps, Frontend, Backend, Data, Cloud, Linux, MacOS, Windows, Kubernetes, Docker, AWS, Git, Bash
   - [ ] Include metadata (name, description, icon, color)

2. **Create Guild Discovery Page**
   - [ ] Create `app/guilds/page.tsx`
   - [ ] Display grid of guild cards
   - [ ] Show member count, description, icon
   - [ ] Add search/filter functionality

3. **Create Guild Card Component**
   - [ ] Create `components/Guild/GuildCard.tsx`
   - [ ] Display guild preview
   - [ ] "Join" button with loading state
   - [ ] Use .pixel-card style with neon borders

4. **Create Guild Detail Page**
   - [ ] Create `app/guilds/[id]/page.tsx`
   - [ ] Display guild header (name, description, member count)
   - [ ] Show guild feed below

5. **Create Guild Feed Component**
   - [ ] Create `components/Guild/GuildFeed.tsx`
   - [ ] Fetch artifacts from Bluesky feed
   - [ ] Filter by guild tag
   - [ ] Display using ArtifactCard components
   - [ ] Implement infinite scroll

6. **Create Feed Filters Component**
   - [ ] Create `components/Guild/FeedFilters.tsx`
   - [ ] Sort options: Recent, Helpful, Trending
   - [ ] Filter by artifact type: All, Commands, Runbooks, Wins, Fails
   - [ ] Use .pixel-button style

7. **Implement Guild State Management**
   - [ ] Create `stores/guildStore.ts` (Zustand)
   - [ ] Define GuildStore interface
   - [ ] Implement `joinGuild(guildId)`, `leaveGuild(guildId)`
   - [ ] Track joined guilds in localStorage

8. **Create Guild Hook**
   - [ ] Create `hooks/useGuild.ts`
   - [ ] Implement `useGuild(guildId)` hook
   - [ ] Return `{ guild, feed, loading, error, join, leave }`
   - [ ] Cache feed data

9. **Implement Feed Algorithm**
   - [ ] Create `lib/atproto/feeds.ts`
   - [ ] Implement `fetchGuildFeed(guildId, sort): Promise<Artifact[]>`
   - [ ] Filter by guild tag using Bluesky query
   - [ ] Sort by: timestamp (recent), likes (helpful), score (trending)

10. **Write Tests**
    - [ ] Create `components/Guild/__tests__/GuildCard.test.tsx`
    - [ ] Create `components/Guild/__tests__/GuildFeed.test.tsx`
    - [ ] Create E2E test: `__tests__/e2e/guild-discovery.spec.ts`
    - [ ] Test guild join/leave flow
    - [ ] Verify feed filtering and sorting

### Acceptance Criteria

- ✅ 15+ guilds displayed on discovery page
- ✅ Guild feed renders in <1 second for first 20 items (SC from spec)
- ✅ Feed sorting works correctly
- ✅ Join/leave guild updates state
- ✅ Feed shows only artifacts tagged with guild
- ✅ E2E test passes
- ✅ 60%+ component test coverage

### Files Created

**New Files** (~12 files):
- app/guilds/page.tsx
- app/guilds/[id]/page.tsx
- components/Guild/GuildCard.tsx
- components/Guild/GuildFeed.tsx
- components/Guild/FeedFilters.tsx
- components/Guild/JoinButton.tsx
- lib/guilds/defaults.ts
- lib/atproto/feeds.ts
- stores/guildStore.ts
- hooks/useGuild.ts
- components/Guild/__tests__/GuildCard.test.tsx
- components/Guild/__tests__/GuildFeed.test.tsx
- __tests__/e2e/guild-discovery.spec.ts

---

## WP09: Testing & Quality Assurance

**Priority**: P1 (Critical)
**Estimated Effort**: 16 hours
**Dependencies**: WP01-WP08 (All previous packages)
**Owner**: TBD

### Objectives

- Achieve 80%+ test coverage for lib/
- Achieve 60%+ test coverage for components/
- All E2E tests pass (privacy dashboard, command share, OAuth, guild discovery)
- All accessibility tests pass (Pa11y zero errors)
- Performance tests meet targets (<500ms dashboard, <2s privacy scan, <1s feed)
- Zero critical security vulnerabilities (npm audit)

### Tasks

1. **Unit Test Coverage**
   - [ ] Review coverage reports (`npm run test:coverage`)
   - [ ] Write missing tests for lib/ (target: 80%+)
   - [ ] Write missing tests for hooks/ (target: 70%+)
   - [ ] Ensure all edge cases covered

2. **Component Test Coverage**
   - [ ] Review component coverage
   - [ ] Write missing component tests (target: 60%+)
   - [ ] Test loading states, error states, data display
   - [ ] Test user interactions (clicks, form submissions)

3. **E2E Test Suite**
   - [ ] Run all E2E tests (`npm run test:e2e`)
   - [ ] Fix failing tests
   - [ ] Add missing E2E scenarios from spec acceptance criteria
   - [ ] Test on multiple browsers (Chrome, Firefox, Safari)

4. **Accessibility Testing**
   - [ ] Run Pa11y on all pages (`npm run test:a11y`)
   - [ ] Fix all accessibility errors
   - [ ] Verify WCAG AA compliance
   - [ ] Test with screen reader (VoiceOver/NVDA)
   - [ ] Ensure keyboard navigation works

5. **Performance Testing**
   - [ ] Measure privacy dashboard load time (target: <500ms)
   - [ ] Measure privacy scan time (target: <2s for 1,000 lines)
   - [ ] Measure guild feed render time (target: <1s for 20 items)
   - [ ] Run Lighthouse audits (target: Performance 90+, Accessibility 100)
   - [ ] Optimize if needed (lazy loading, code splitting, caching)

6. **Security Audit**
   - [ ] Run `npm audit` (fix all critical/high vulnerabilities)
   - [ ] Review encrypted storage implementation
   - [ ] Test XSS prevention (sanitize markdown input)
   - [ ] Verify CSP headers configured
   - [ ] Test OAuth token handling (no leaks in console/network)

7. **Cross-Browser Testing**
   - [ ] Test on Chrome 90+
   - [ ] Test on Firefox 88+
   - [ ] Test on Safari 14+
   - [ ] Test on Edge 90+
   - [ ] Fix browser-specific issues

8. **Visual Regression Testing**
   - [ ] Set up Percy or Chromatic
   - [ ] Capture baselines for all pages
   - [ ] Verify 8-bit design consistency
   - [ ] Review any visual changes

### Acceptance Criteria

- ✅ 80%+ test coverage for lib/
- ✅ 60%+ test coverage for components/
- ✅ All E2E tests pass
- ✅ Pa11y reports zero accessibility errors
- ✅ Lighthouse score: Performance 90+, Accessibility 100
- ✅ Privacy dashboard loads in <500ms (10,000 commands)
- ✅ Privacy scan processes 1,000 lines in <2s
- ✅ Guild feed renders 20 items in <1s
- ✅ Zero critical npm audit vulnerabilities
- ✅ All browsers (Chrome, Firefox, Safari, Edge) work correctly

### Deliverables

- Comprehensive test reports (coverage, E2E results, accessibility)
- Performance audit results (Lighthouse, custom metrics)
- Security audit report (npm audit, CSP verification)
- Cross-browser compatibility matrix

---

## WP10: Documentation & Deployment

**Priority**: P2 (Important)
**Estimated Effort**: 8 hours
**Dependencies**: WP01-WP09 (All previous packages)
**Owner**: TBD

### Objectives

- Create developer quick-start guide
- Document deployment process
- Update README with Phase 1 features
- Deploy to Vercel for preview
- Prepare for Phase 2 handoff

### Tasks

1. **Create Quick-Start Guide**
   - [ ] Create `kitty-specs/008-caro-web-hub/quickstart.md`
   - [ ] Document development setup
   - [ ] Document local CLI data strategies
   - [ ] Document testing workflows
   - [ ] Document Bluesky sandbox setup

2. **Create API Contracts**
   - [ ] Create `kitty-specs/008-caro-web-hub/contracts/privacy-engine.ts`
   - [ ] Create `kitty-specs/008-caro-web-hub/contracts/bluesky-client.ts`
   - [ ] Create `kitty-specs/008-caro-web-hub/contracts/local-cli-interface.ts`
   - [ ] Create `kitty-specs/008-caro-web-hub/contracts/artifact-schemas.ts`

3. **Update README**
   - [ ] Update `apps/devrel/README.md` with Phase 1 features
   - [ ] Add "Getting Started" section
   - [ ] Document privacy-first architecture
   - [ ] Add screenshots/demos
   - [ ] Link to quickstart.md

4. **Update Main README**
   - [ ] Update root `README.md` mentioning web hub
   - [ ] Add link to hub.caro.sh (planned)
   - [ ] Update architecture diagram

5. **Deploy to Vercel**
   - [ ] Verify build succeeds (`npm run build`)
   - [ ] Configure Vercel project (Root Directory: `apps/devrel`)
   - [ ] Set environment variables (if needed)
   - [ ] Deploy preview
   - [ ] Test preview deployment
   - [ ] Configure custom domain (hub.caro.sh) if ready

6. **Create Phase 2 Handoff Doc**
   - [ ] Create `kitty-specs/008-caro-web-hub/phase2-plan.md`
   - [ ] Document remaining work (runbooks, win stories, epic fails)
   - [ ] List open questions and risks
   - [ ] Provide estimates for Phase 2 (2-3 weeks)

7. **Code Documentation**
   - [ ] Add JSDoc comments to all lib/ functions
   - [ ] Document component props (TypeScript interfaces)
   - [ ] Add inline code comments for complex logic
   - [ ] Generate API documentation (if using tool like TypeDoc)

### Acceptance Criteria

- ✅ quickstart.md created and comprehensive
- ✅ API contracts defined
- ✅ README updated with Phase 1 features
- ✅ Vercel preview deployment successful
- ✅ Build passes on CI (lint, type-check, test, build)
- ✅ All lib/ functions have JSDoc comments
- ✅ Phase 2 handoff document created

### Deliverables

- quickstart.md (developer guide)
- contracts/ (API contracts)
- Updated README files
- Vercel deployment preview URL
- Phase 2 planning document

---

## Phase 1 Definition of Done

**All work packages (WP01-WP10) must be completed:**

### Feature Complete ✅
- [ ] WP01: Project Setup complete
- [ ] WP02: Privacy Redaction Engine complete
- [ ] WP03: Local CLI Data Interface complete
- [ ] WP04: Privacy Dashboard UI complete
- [ ] WP05: Bluesky OAuth complete
- [ ] WP06: Bluesky Publishing complete
- [ ] WP07: Command Sharing Flow complete
- [ ] WP08: Guild System (read-only) complete
- [ ] WP09: Testing & QA complete
- [ ] WP10: Documentation & Deployment complete

### Quality Gates ✅
- [ ] 80%+ test coverage for lib/
- [ ] 60%+ test coverage for components/
- [ ] All E2E tests pass
- [ ] Pa11y zero accessibility errors
- [ ] Lighthouse: Performance 90+, Accessibility 100
- [ ] Zero critical security vulnerabilities

### Success Criteria from Spec ✅
- [ ] SC-001: Privacy dashboard loads offline <500ms (10,000 commands)
- [ ] SC-002: Privacy scan detects 95%+ sensitive data
- [ ] SC-003: Command publish 100% success (when online)
- [ ] SC-007: Bluesky OAuth 98%+ success rate

### Deployment ✅
- [ ] Vercel preview deployment successful
- [ ] Build passes on CI
- [ ] Production build optimized (<5MB)
- [ ] Custom domain configured (hub.caro.sh)

### Documentation ✅
- [ ] README updated with Phase 1 features
- [ ] quickstart.md created
- [ ] contracts/ defined
- [ ] Code documented (JSDoc)

**When all above are ✅, Phase 1 is COMPLETE** → Ready for Phase 2 (Runbooks + Win Stories).

---

## Estimation Summary

| Work Package | Effort | Priority |
|--------------|--------|----------|
| WP01 | 4h | P0 |
| WP02 | 8h | P1 |
| WP03 | 12h | P1 |
| WP04 | 16h | P1 |
| WP05 | 12h | P1 |
| WP06 | 12h | P1 |
| WP07 | 16h | P1 |
| WP08 | 12h | P2 |
| WP09 | 16h | P1 |
| WP10 | 8h | P2 |
| **Total** | **116h** | **~3 weeks** |

**Phase 1 Duration**: 3-4 weeks (full-time equivalent)
**Next Phase**: Phase 2 - Runbooks + Win Stories (2-3 weeks)
