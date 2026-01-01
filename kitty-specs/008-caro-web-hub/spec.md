# Feature Specification: Caro Web Hub - Privacy-First Social Platform for Terminal Expertise

**Feature Branch**: `008-caro-web-hub`
**Created**: 2026-01-01
**Status**: Draft
**Input**: Implement Caro Hub as a privacy-first social platform where developers share terminal expertise, runbooks, and workflows with professional guilds, built on Bluesky AT Protocol

## User Scenarios & Testing *(mandatory)*

###User Story 1 - Privacy Dashboard & Local Telemetry Review (Priority: P1)

Users can view all locally collected telemetry from their Caro CLI agent in a clear, transparent dashboard before any data leaves their machine. The system automatically flags sensitive data (API keys, tokens, paths, emails) and offers redaction suggestions.

**Why this priority**: Privacy-first is the core value proposition. Users must have complete visibility and control over their data before any social sharing happens. Without this foundation, users won't trust the platform.

**Independent Test**: User opens Web Hub, sees their local CLI command history, telemetry data, and privacy review UI with flagged sensitive items - all working entirely offline with no network requests.

**Acceptance Scenarios**:

1. **Given** a user has generated 50 commands locally with Caro CLI, **When** they open the Web Hub privacy dashboard, **Then** they see all 50 commands with metadata (timestamp, backend, safety score) and zero network requests are made
2. **Given** collected telemetry contains 3 API keys and 5 home directory paths, **When** user reviews the privacy dashboard, **Then** all 8 sensitive items are highlighted with specific redaction suggestions (e.g., `[REDACTED_API_KEY]`, `/home/[USER]`)
3. **Given** a user wants to understand what data is collected, **When** they view the telemetry transparency section, **Then** they see categorized data (commands generated, prompts saved, safety validations, backend usage, error logs) with clear explanations and zero items shared by default

---

### User Story 2 - Share Command Artifact to Guild (Priority: P1)

Users can select a generated command from their local history, review it for sensitive data, add context (title, description, tags), choose a professional guild (e.g., "SRE Guild"), and publish it to Bluesky's decentralized network - only after explicit consent.

**Why this priority**: This is the core social feature that delivers the "share your expertise" value. Command sharing is the atomic unit of the social platform.

**Independent Test**: User selects a command (`find . -type f -mtime -1`), adds context, picks "SRE Guild", confirms privacy review, publishes to Bluesky, and sees it appear in the guild feed.

**Acceptance Scenarios**:

1. **Given** a user generated the command `git fetch origin && git rebase origin/main`, **When** they click "Share Command" and add title "Rebase with main", tags ["git", "workflow"], and select "DevOps Guild", **Then** a draft artifact is created with automatic privacy scanning
2. **Given** the command contains `/home/alice`, **When** privacy engine scans the draft, **Then** it flags the home path and suggests replacement with `/home/[USER]`, requiring user confirmation
3. **Given** user confirms all redactions and publishes, **When** the share completes, **Then** the artifact is posted to Bluesky with custom record type `app.caro.share.command`, appears in the DevOps Guild feed, and is added to user's "Shared" history in privacy dashboard

---

### User Story 3 - Create and Share Operational Runbook (Priority: P1)

Users can document multi-step terminal workflows (e.g., "Production Deployment Process") by combining commands from their history, adding natural language descriptions for each step, and sharing the complete runbook with their guild.

**Why this priority**: Runbooks are the highest-value social artifact - they capture operational knowledge and are inherently reusable. This differentiates Caro Hub from simple command snippet sharing.

**Independent Test**: User creates "Rebase with main, then continue" runbook with 5 steps, shares to "DevOps Guild", and another user can fork/adapt it.

**Acceptance Scenarios**:

1. **Given** a user wants to document their git rebase workflow, **When** they create a new runbook titled "Rebase with main, then continue" with 5 steps: (1) fetch remote, (2) start rebase, (3) resolve conflicts, (4) continue rebase, (5) force push safely, **Then** each step has a natural language prompt, generated command, safety level, and optional notes
2. **Given** step 5 contains `git push --force-with-lease`, **When** privacy engine scans, **Then** it validates the command is safe (uses `--force-with-lease` not `--force`) and marks safety level as "moderate" with explanation
3. **Given** user publishes the runbook to "DevOps Guild", **When** another guild member views it, **Then** they can see all 5 steps, fork the runbook to customize for their workflow, and the original author gets credit via Bluesky DID

---

### User Story 4 - Report Epic Fail with Privacy-Protected Logs (Priority: P2)

When Caro CLI generates a dangerous or incorrect command, users can file an "epic fail" report with verbose logs pulled directly from the local agent, but only after the system redacts all sensitive data from the logs.

**Why this priority**: Critical for improving Caro's safety and accuracy, but secondary to core social sharing. This is a specialized workflow for quality assurance.

**Independent Test**: User encounters dangerous command (`rm -rf /var/lib/postgresql` when asking to "backup database"), reports it with logs, system redacts paths and outputs, maintainers see sanitized issue.

**Acceptance Scenarios**:

1. **Given** Caro generated `rm -rf /var/lib/postgresql` for prompt "backup my postgres database", **When** user clicks "Report Epic Fail" from the error screen, **Then** Web Hub pre-fills the report with prompt, generated command, expected behavior field, and local verbose logs
2. **Given** logs contain API endpoints, database connection strings, and file paths, **When** privacy engine processes the logs, **Then** it automatically redacts: (a) API URLs → `[REDACTED_URL]`, (b) connection strings → `[REDACTED_CREDENTIALS]`, (c) home paths → `/home/[USER]`, and highlights each redaction for user review
3. **Given** user confirms redactions and submits, **When** the fail report is published, **Then** it's posted with severity "critical", backend "mlx:qwen2.5-coder-1.5b", Caro version, and redacted logs - visible to maintainers for triaging and to community for confirmation if user opts in

---

### User Story 5 - Join Professional Guild and View Feed (Priority: P2)

Users can discover and join professional guilds (communities like "SRE Guild", "AppSec Guild", "Kubernetes Operators") to see a feed of shared commands, runbooks, and win stories from other practitioners in their domain.

**Why this priority**: Guilds are the social layer that makes shared knowledge discoverable and contextual. Secondary to creation features but essential for the platform's network effects.

**Independent Test**: User browses guild directory, joins "SRE Guild", sees feed of recent command artifacts and runbooks from other SRE practitioners, sorted by "helpful" votes.

**Acceptance Scenarios**:

1. **Given** a new user opens the Guild Discovery page, **When** they browse available guilds, **Then** they see 15+ default guilds (SRE, AppSec, DevOps, Frontend, Backend, Data, Cloud, Linux, MacOS, Kubernetes, Docker, etc.) with member counts, descriptions, and top contributors
2. **Given** user joins "SRE Guild", **When** they view the guild feed, **Then** they see recent artifacts sorted by: (a) Recent (last 7 days), (b) Helpful (most upvotes), (c) Trending (recent + high engagement), with filters for artifact type (commands, runbooks, wins, fails)
3. **Given** user wants to contribute, **When** they share a command artifact, **Then** the guild selection UI shows their joined guilds first, allows multi-guild sharing, and displays guild-specific tags (e.g., "monitoring", "incident-response" for SRE Guild)

---

### User Story 6 - Authenticate with Bluesky and Manage Profile (Priority: P2)

Users can authenticate using their existing Bluesky account (or create a new one), see their Caro profile with stats (commands shared, runbooks created, reputation points), and manage privacy settings.

**Why this priority**: Authentication is foundational for social features, but the OAuth flow is standard. Priority is ensuring it integrates smoothly with local-first architecture.

**Independent Test**: User clicks "Login with Bluesky", authorizes the Caro Hub app, returns to see their profile with Bluesky handle (@user.bsky.social) and local CLI stats merged.

**Acceptance Scenarios**:

1. **Given** a new user visits Caro Hub for the first time, **When** they click "Login with Bluesky", **Then** they are redirected to Bluesky OAuth, authorize the app with requested permissions (read profile, publish posts, read/write custom records), and return to Caro Hub with access token stored securely in encrypted localStorage
2. **Given** authenticated user's local CLI has generated 127 commands this month, **When** they view their profile dashboard, **Then** they see: (a) Bluesky identity (DID, handle, avatar), (b) Local stats (127 commands, 12 safety triggers, 85% Ollama usage), (c) Social stats (8 shares, 234 helpful votes received, Level 12 reputation), (d) Joined guilds (SRE, DevOps, Kubernetes)
3. **Given** user wants to control sharing defaults, **When** they open Privacy Settings, **Then** they can configure: (a) Default visibility (Public/Guild/Private), (b) Automatic redaction rules (API keys ✓, Home paths ✓, Environment variables ✓, Emails ✓, Generic paths ✗), (c) Telemetry collection toggles (Commands ✓, Safety validations ✓, Error logs ✓, Usage analytics ✗)

---

### User Story 7 - Post Win Story and Link Artifacts (Priority: P3)

Users can celebrate successful automations or "aha moments" by writing a win story (title + markdown narrative + impact statement) and linking related command artifacts they previously shared.

**Why this priority**: Win stories add narrative context and community engagement, but they're secondary to core command/runbook sharing. Nice-to-have for community building.

**Independent Test**: User writes "Automated 200 Server Audits in 5 Minutes" win story, links 2 command artifacts (parallel-ssh-audit, log-aggregator), posts to DevOps Guild, receives upvotes.

**Acceptance Scenarios**:

1. **Given** user successfully automated a tedious task, **When** they create a new win story with title "Automated 200 server audits in 5 minutes", story text explaining the use case, impact "Saved 8 hours of manual work", and linked artifacts [command-1, command-2], **Then** the draft win is created with privacy scan of the markdown story text
2. **Given** the story mentions specific server hostnames and internal URLs, **When** privacy engine scans, **Then** it flags hostnames and URLs, suggests redactions like `server-prod-01` → `[SERVER_NAME]` and `https://internal.example.com` → `[INTERNAL_URL]`
3. **Given** user publishes the win to "DevOps Guild", **When** it appears in the feed, **Then** other users can upvote it ("helpful"), comment with their own experiences, and click through to the linked command artifacts to see implementation details

---

### Edge Cases

- **Offline-first**: Web Hub works entirely offline for privacy dashboard and local data review; network only required for authentication and publishing
- **Empty local data**: User with brand new Caro CLI (zero commands generated) sees empty privacy dashboard with helpful onboarding message
- **Multiple Bluesky accounts**: User can log out and log in with different Bluesky account; local CLI data persists but social identity switches
- **Simultaneous edits**: User editing draft runbook in browser while CLI continues generating commands locally - changes sync on next dashboard refresh
- **Large logs in epic fails**: Fail reports with 10MB+ of logs are automatically truncated to first/last 1000 lines with summary statistics, preventing upload bloat
- **Guild membership limits**: User can join unlimited guilds, but UI suggests focusing on 3-5 for best experience
- **Bluesky network downtime**: If Bluesky PDS is unreachable, publishing fails gracefully with offline queue option
- **Malicious redaction bypass**: User attempting to bypass privacy checks by manually editing redacted text before publish triggers secondary validation
- **Unicode and special characters**: Runbooks with emoji, code blocks, and international characters render correctly in feeds

## Requirements *(mandatory)*

### Functional Requirements

**Authentication & Identity:**
- **FR-001**: System MUST support OAuth authentication via Bluesky using AT Protocol OAuth flow
- **FR-002**: System MUST store user's Bluesky DID (decentralized identifier) as primary identity
- **FR-003**: System MUST support logout and account switching without data loss
- **FR-004**: System MUST work offline for all local data review and privacy dashboard features

**Privacy & Telemetry Dashboard:**
- **FR-005**: System MUST display all locally collected CLI telemetry in categorized dashboard (commands, prompts, safety validations, backend usage, errors)
- **FR-006**: System MUST scan artifacts for sensitive data using regex patterns: API keys, JWT tokens, AWS keys, SSH keys, emails, IP addresses, home paths, environment variables
- **FR-007**: System MUST highlight flagged sensitive data with color-coding (red=critical like API keys, yellow=moderate like paths)
- **FR-008**: System MUST require explicit user confirmation for each redaction before publishing
- **FR-009**: System MUST show diff preview of original vs. redacted content before publish
- **FR-010**: System MUST track sharing history: what was shared, when, to which guild, with which redactions applied

**Command Artifact Sharing:**
- **FR-011**: System MUST allow user to select command from local CLI history for sharing
- **FR-012**: System MUST support adding metadata: title, description, tags, guild association, visibility (public/guild/private)
- **FR-013**: System MUST publish to Bluesky using custom record type `app.caro.share.command` with Lexicon schema
- **FR-014**: System MUST support upvoting ("helpful") and commenting on shared commands
- **FR-015**: System MUST allow user to save (bookmark) others' commands for later reference

**Runbook Creation:**
- **FR-016**: System MUST support creating multi-step runbooks with: title, description, steps array, prerequisites, estimated time, difficulty level
- **FR-017**: Each runbook step MUST include: order number, title, natural language prompt, generated command, optional notes, safety level, expected output
- **FR-018**: System MUST allow user to fork (copy) others' runbooks for customization
- **FR-019**: System MUST publish runbooks to Bluesky using custom record type `app.caro.share.runbook`
- **FR-020**: System MUST track runbook forks and credit original author

**Epic Fail Reporting:**
- **FR-021**: System MUST pre-fill fail reports with: prompt, generated command, expected behavior, backend, CLI version, timestamp
- **FR-022**: System MUST pull verbose logs from local CLI agent (if available)
- **FR-023**: System MUST apply aggressive redaction to logs: URLs, credentials, paths, IPs, hostnames
- **FR-024**: System MUST allow user to mark fails as public (community-visible) or private (maintainers-only)
- **FR-025**: System MUST publish fails to Bluesky using custom record type `app.caro.share.fail` with severity classification

**Guild Communities:**
- **FR-026**: System MUST provide 15+ default guilds at launch: SRE, AppSec, DevOps, Frontend, Backend, Data, Cloud, Linux, MacOS, Windows, Kubernetes, Docker, AWS, Git, Bash
- **FR-027**: System MUST support guild discovery with: member count, description, icon, top contributors
- **FR-028**: System MUST allow users to join/leave guilds without limit
- **FR-029**: System MUST provide guild-specific feeds with sort options: Recent, Helpful, Trending
- **FR-030**: System MUST filter guild feeds by artifact type: Commands, Runbooks, Wins, Fails

**Win Stories:**
- **FR-031**: System MUST support win story creation with: title, markdown story, impact statement, linked artifact IDs
- **FR-032**: System MUST scan markdown story content for sensitive data (same patterns as commands)
- **FR-033**: System MUST publish wins to Bluesky using custom record type `app.caro.share.win`
- **FR-034**: System MUST support upvoting and commenting on win stories

**Bluesky AT Protocol Integration:**
- **FR-035**: System MUST define custom Lexicon schemas for all Caro record types (command, runbook, win, fail)
- **FR-036**: System MUST use Bluesky PDS (Personal Data Server) for publishing all artifacts
- **FR-037**: System MUST implement custom feed algorithms for guild feeds (filter by record type + guild tag)
- **FR-038**: System MUST support Bluesky's portable identity (users can move their DID to different PDS)

### Non-Functional Requirements

**Performance:**
- **NFR-001**: Privacy dashboard MUST load local data in <500ms for datasets up to 10,000 commands
- **NFR-002**: Privacy scan MUST process 1,000-line artifact in <2 seconds
- **NFR-003**: Guild feed MUST render first 20 items in <1 second

**Security:**
- **NFR-004**: OAuth tokens MUST be stored in encrypted localStorage with secure key derivation
- **NFR-005**: PII redaction patterns MUST be regularly updated to catch new formats (quarterly review)
- **NFR-006**: System MUST prevent XSS attacks in user-generated markdown content

**Usability:**
- **NFR-007**: Privacy review UI MUST use color-coding (red/yellow/green) for visual accessibility
- **NFR-008**: All forms MUST provide inline validation with clear error messages
- **NFR-009**: System MUST work on latest versions of Chrome, Firefox, Safari, Edge

**Reliability:**
- **NFR-010**: Local data operations MUST work offline with zero dependency on network
- **NFR-011**: Publishing to Bluesky MUST retry failed requests 3 times with exponential backoff
- **NFR-012**: System MUST gracefully handle Bluesky network downtime with offline queue

### Key Entities

**User Profile:**
- Bluesky DID (decentralized identifier)
- Handle (@user.bsky.social)
- Display name, avatar, bio
- Local CLI stats (commands generated, safety triggers)
- Social stats (shares, helpful votes, reputation points, level)
- Joined guilds
- Privacy settings

**Command Artifact:**
- Prompt (natural language request)
- Command (generated shell command)
- Safety score (safe/moderate/high/critical)
- Backend (ollama:qwen2.5-coder, mlx, etc.)
- Timestamp
- Tags
- Guild association
- Author DID

**Runbook:**
- Title, description
- Steps array (order, title, prompt, command, notes, safety, expected output)
- Prerequisites, estimated time, difficulty
- Guild, tags
- Author DID
- Fork count, original runbook reference

**Win Story:**
- Title, markdown story
- Impact statement
- Linked artifact IDs
- Guild, tags
- Author DID

**Epic Fail:**
- Prompt, generated command, expected behavior
- Actual result (what happened)
- Redacted logs
- Severity (low/medium/high/critical)
- Backend, CLI version
- Visibility (public/private)
- Author DID

**Guild:**
- ID, name, slug
- Description, icon
- Member count, moderators
- Tags, featured artifacts
- Feed algorithm

**Privacy Scan Result:**
- Detected items (type, value, start index, end index)
- Redaction suggestions
- User confirmations
- Applied redactions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Privacy dashboard loads local CLI data for 100% of authenticated users with zero network requests for offline review
- **SC-002**: Privacy scan detects 95%+ of sensitive data patterns (API keys, tokens, paths, emails) in test dataset of 1,000 artifacts
- **SC-003**: Users can publish command artifacts to Bluesky successfully 100% of the time when network is available
- **SC-004**: Users can create and publish multi-step runbooks with 100% success rate for runbooks with 1-20 steps
- **SC-005**: Epic fail reports are successfully created and logs are redacted for 100% of test cases
- **SC-006**: Users can join guilds and view guild feeds with 100% success rate
- **SC-007**: Bluesky OAuth authentication completes successfully 98%+ of the time (accounting for network failures)
- **SC-008**: Win stories render correctly with markdown formatting in guild feeds 100% of the time
- **SC-009**: System operates entirely offline for privacy dashboard with zero errors for 100% of users
- **SC-010**: Custom Bluesky record types (app.caro.share.*) validate against Lexicon schemas 100% of the time

## Assumptions

1. **Bluesky Availability**: Bluesky network and OAuth provider have 99%+ uptime; temporary outages are handled via offline queue
2. **Local CLI Data Access**: Web Hub can access local Caro CLI data via: (a) File API for browser-based access, OR (b) Local HTTP server (localhost:3000) running alongside CLI, OR (c) Electron/Tauri wrapper for native file access
3. **Regex Coverage**: Privacy redaction regex patterns cover 95%+ of common sensitive data formats; edge cases may require manual review
4. **Guild Moderation**: Initial launch relies on community reporting + volunteer moderators; automated moderation (spam detection, profanity filter) added in Phase 2
5. **Bluesky Lexicon Schemas**: Caro custom record types will be submitted to Bluesky Lexicon registry for official recognition
6. **Browser Compatibility**: Focus on modern evergreen browsers (Chrome 90+, Firefox 88+, Safari 14+, Edge 90+); IE11 not supported

## Out of Scope

**Phase 1 (MVP):**
- **Gamification**: Achievements, badges, reputation levels, leaderboards (planned for Phase 2)
- **Advanced Moderation**: Automated spam detection, profanity filters, trust scores (Phase 2)
- **Custom Feeds**: User-defined feed algorithms beyond default guild feeds (Phase 2)
- **Mobile App**: React Native mobile app (Phase 3)
- **Real-time Features**: Live notifications, real-time feed updates (Phase 2 with WebSocket)
- **Command Challenges**: Weekly/monthly community challenges (Phase 2)
- **Kyaro Evolution**: Mascot evolution states based on user activity (Phase 2 gamification)
- **Private Guilds**: Enterprise/team-specific private communities (Phase 3)
- **Multi-language Support**: i18n/l10n for non-English users (Phase 3)
- **Advanced Analytics**: Personal insights dashboard with trends and predictions (Phase 3)
- **Federation**: Full AT Protocol federation with other apps (Phase 4)

**Explicitly Not Planned:**
- **Cryptocurrency/Tokens**: No blockchain, crypto, NFTs, or token economy
- **Paid Features**: All social features remain free and open-source (monetization via optional enterprise support only)
- **Advertising**: No ads, trackers, or third-party analytics
- **Content Algorithm Manipulation**: No algorithmic recommendation manipulation; users control feed sorting
- **Data Mining**: No user data mining, profiling, or sale to third parties

## Dependencies

**External:**
- **Bluesky AT Protocol SDK** (@atproto/api npm package) for authentication and publishing
- **Bluesky OAuth Provider** for user authentication flow
- **Next.js 16** for web framework
- **Tailwind CSS 4** for styling
- **TypeScript** for type safety

**Internal:**
- **Caro CLI** (Rust) for generating local telemetry data
- **apps/devrel/** existing codebase for 8-bit design system and components
- **Privacy redaction engine** (to be implemented)
- **Bluesky Lexicon schemas** (to be defined)

## Risks & Mitigation

**Risk 1: Bluesky AT Protocol Adoption**
- **Risk**: Bluesky may not reach critical mass of users; betting on unproven social protocol
- **Impact**: Low guild membership, low engagement, network effects don't materialize
- **Mitigation**: (a) AT Protocol is open and federatable - not locked to Bluesky platform, (b) If Bluesky fails, pivot to Mastodon/ActivityPub or standalone platform, (c) Local-first architecture means value exists even without social network

**Risk 2: Privacy Redaction False Negatives**
- **Risk**: Regex patterns miss new sensitive data formats; users accidentally leak credentials
- **Impact**: Critical security incident, user trust destroyed, legal liability
- **Mitigation**: (a) Conservative redaction (flag more, not less), (b) Regular pattern updates (quarterly), (c) Secondary validation before publish, (d) Community reporting for leaked data, (e) Incident response plan for takedowns

**Risk 3: Local CLI Data Access**
- **Risk**: Browsers restrict file access; can't read local CLI data without Electron/Tauri wrapper
- **Impact**: Privacy dashboard requires native app; web-only version is limited
- **Mitigation**: (a) Implement local HTTP server (localhost:3000) that CLI runs to serve data, (b) Explore File System Access API (Chrome), (c) Provide Electron/Tauri wrapper for desktop, (d) Document hybrid approach in README

**Risk 4: Bluesky Rate Limits**
- **Risk**: Bluesky PDS rate limits block power users from publishing many artifacts quickly
- **Impact**: User frustration, failed publishes, bad UX
- **Mitigation**: (a) Implement client-side rate limiting with queue, (b) Show clear feedback ("3 publishes remaining this hour"), (c) Batch publishing for multiple artifacts, (d) Contact Bluesky for higher limits for verified apps

**Risk 5: Guild Spam & Moderation**
- **Risk**: Guilds get flooded with spam, low-quality content, or malicious artifacts
- **Impact**: Users leave guilds, platform reputation damaged, moderators overwhelmed
- **Mitigation**: (a) Phase 1 relies on community reporting, (b) Phase 2 adds automated spam detection, (c) Appoint volunteer moderators per guild, (d) Rate limiting on publishing, (e) Trust scores and reputation system (Phase 2)

## Example: "Rebase with main, then continue" Social Runbook

**Title**: Rebase with main, then continue

**Description**: Keep your feature branch current with main, resolve conflicts cleanly, and finish the rebase safely without breaking your remote branch.

**Prerequisites**:
- Git repository with main branch
- Feature branch checked out
- No uncommitted changes (or stash them first)

**Estimated Time**: ~5-10 minutes (depending on conflicts)

**Difficulty**: Intermediate

**Steps**:

1. **Fetch the latest remote changes**
   - **Prompt**: "get latest changes from remote main"
   - **Command**: `git fetch origin`
   - **Safety**: ✅ Safe
   - **Notes**: This doesn't modify your local branches, just updates remote tracking
   - **Expected Output**: `Fetching origin...`

2. **Start the rebase onto origin/main**
   - **Prompt**: "rebase my branch onto main"
   - **Command**: `git rebase origin/main`
   - **Safety**: ✅ Safe (can be aborted with `git rebase --abort`)
   - **Notes**: If conflicts happen, Git will pause and show conflict markers
   - **Expected Output**: Either success message or `CONFLICT (content): Merge conflict in file.txt`

3. **Check status if conflicts appear**
   - **Prompt**: "show me which files have conflicts"
   - **Command**: `git status`
   - **Safety**: ✅ Safe
   - **Notes**: Conflicting files are marked as "both modified"
   - **Expected Output**: List of files with conflict markers

4. **Resolve conflicts and stage changes**
   - **Prompt**: "mark all conflicts as resolved"
   - **Command**: `git add -A`
   - **Safety**: ⚠️  Moderate (ensure you actually resolved conflicts first)
   - **Notes**: Only run this AFTER manually editing files to remove conflict markers
   - **Expected Output**: Silent success (no output)

5. **Continue the rebase**
   - **Prompt**: "continue the rebase after fixing conflicts"
   - **Command**: `git rebase --continue`
   - **Safety**: ✅ Safe
   - **Notes**: Repeat steps 3-5 if more conflicts appear; rebase completes when all commits are applied
   - **Expected Output**: `Successfully rebased and updated refs/heads/feature-branch`

6. **Force push safely to update remote branch**
   - **Prompt**: "safely force push my rebased branch"
   - **Command**: `git push --force-with-lease`
   - **Safety**: ✅ Safe (uses `--force-with-lease` not `--force`)
   - **Notes**: `--force-with-lease` protects against overwriting others' work
   - **Expected Output**: `+ abcd123...ef45678 feature-branch -> feature-branch (forced update)`

**Abort Option**: If you want to stop the rebase at any point:
- **Prompt**: "abort the rebase and go back to before"
- **Command**: `git rebase --abort`
- **Safety**: ✅ Safe
- **Notes**: Returns your branch to the state before you started the rebase

**Guild**: DevOps Guild
**Tags**: #git, #rebase, #workflow, #conflict-resolution
**Visibility**: Public

**Privacy Scans Applied**:
- No sensitive data detected ✓
- All commands use safe Git operations ✓
- No hardcoded branch names (uses `origin/main`) ✓

**Example Share Metadata**:
```json
{
  "type": "app.caro.share.runbook",
  "title": "Rebase with main, then continue",
  "description": "Keep your feature branch current with main...",
  "steps": [ /* 6 steps as above */ ],
  "prerequisites": ["Git repository", "Feature branch checked out"],
  "estimated_time": "5-10 minutes",
  "difficulty": "intermediate",
  "guild": "devops",
  "tags": ["git", "rebase", "workflow", "conflict-resolution"],
  "author_did": "did:plc:abc123...",
  "timestamp": "2026-01-01T12:00:00Z",
  "visibility": "public"
}
```

---

This runbook can be:
- **Forked** by other users to customize for their workflow
- **Upvoted** ("helpful") by users who found it useful
- **Commented** on with variations or tips
- **Saved** to personal bookmarks for quick reference
- **Linked** from win stories about successful rebases
