# Inter-Agent Communication Protocol

> How Hermes communicates with Claude Code, Crush, and other agents
> in the Caro project ecosystem.

## Channels

### 1. Notification Log (real-time event stream)

**File**: `.claude/notifications.log`
**Tool**: `bin/notify`
**Pattern**: Fire-and-forget, append-only

```bash
bin/notify hermes "triage: PR #1065 flagged for safety review"
bin/notify hermes "digest: daily PR digest ready at .hermes/digests/2026-05-12.md"
bin/notify hermes "alert: 3 PRs stale >5 days — #1043 #1045 #1061"
```

All agents should `tail -f .claude/notifications.log` to stay aware of
cross-agent activity. Hermes uses the `hermes` role prefix.

**Reserved role prefixes**:
- `hermes` — Strategic intelligence & coordination
- `coder` — Claude Code coder-loop
- `reviewer` — Review gate agent
- `qa` — QA automation
- `pr-mgmt` — PR management loop
- `crush` — Crush agent (when integrated)
- `integrator` — Nightly integration agent

### 2. Structured Messages (async, persistent)

**Directory**: `.hermes/messages/`
**Pattern**: Drop a markdown file, target agent picks it up

Use cases:
- PR triage results that need action
- Coordination alerts (conflicting branches, duplicate work)
- Routing recommendations (which agent should handle what)

**Naming convention**: `<type>-<YYYY-MM-DD>.md`

Example file: `.hermes/messages/pr-triage-2026-05-12.md`

```markdown
# PR Triage — 2026-05-12 (Hermes)

> Drop file for Claude Code / Crush / pr-management-loop.
> Pick up during your next grooming cycle (Phase A6).

## IMMEDIATE ACTION ITEMS

### 1. External PR #1065 — Route to safety review
- **What:** Fix mkfs regex bypass
- **Who:** Quillenar-dev (first-time contributor)
- **Action:** Review via safety-pattern-developer agent. Merge if CI passes.

### 2. Rebase PR #1036 — Safety fix stuck in conflict
- **Action:** `gh pr checkout 1036 && git rebase origin/main && git push --force-with-lease`

## NO ACTION NEEDED
PRs #1078, #1074, #1071, #1069, #1067, #1042, #1028 are active and mergeable.
```

### 3. GitHub Issue/PR Comments (public, async)

**Tool**: `gh issue comment`, `gh pr comment`
**Pattern**: Structured comments with agent attribution

All Hermes comments on GitHub follow the canonical `[agent]` tag format
(from `~/.claude/rules/pr-comment-agent-tag.md` and
`~/.claude/rules/pr-comment-agent-identity.md`):

```
`[agent]`

**Agent:** Hermes (`strategic-intelligence`)
**Run:** <context — e.g. "daily PR triage">

---

<content>
```

### 4. Digest Files (daily/weekly summaries)

**Directory**: `.hermes/digests/`
**Pattern**: One file per digest, date-stamped

Daily PR digests include:
- PR count by status
- Action items with routing
- Staleness alerts
- External contributor status

## Message Flow

```
                    ┌──────────────┐
                    │   Kobi       │
                    │  (decides)   │
                    └──────┬───────┘
                           │ briefings
                    ┌──────▼───────┐
                    │   Hermes     │
                    │ (synthesizes)│
                    └──┬───┬───┬──┘
            triage ┌───┘   │   └───┐ coordination
                   ▼       ▼       ▼
            ┌──────┐ ┌─────────┐ ┌──────┐
            │ GH   │ │ Claude  │ │Crush │
            │Issues│ │ Code    │ │(CLI) │
            │PRs   │ │(app)    │ │      │
            └──────┘ └─────────┘ └──────┘
                   ▲       ▲       ▲
                   │       │       │
            bin/notify + .hermes/messages/ + gh comments
```

## Rules

1. **Hermes never edits code directly.** All repo changes go through PRs.
2. **Hermes never merges PRs.** That's pr-management-loop's job.
3. **Hermes comments are structured.** Always use the canonical `[agent]` tag.
4. **Hermes respects agent boundaries.** Don't tell Claude Code *how* to code.
   Tell it *what* needs attention and *why*.
5. **Message cleanup is Hermes' responsibility.** After Claude Code's
   grooming loop processes `.hermes/messages/` (via Phase A6 of
   `caro-backlog-groom.md`), the loop moves them to
   `.hermes/messages/processed/`. Hermes may prune `processed/` files
   older than 30 days via a housekeeping PR.
6. **bin/notify is the real-time channel.** Use it for anything
   time-sensitive. Use `.hermes/messages/` for structured, actionable data.

## Discovery

Agents discover Hermes' presence via:
- This file (`.hermes/PROTOCOL.md`)
- The `hermes` role in `bin/notify` output
- GitHub comments tagged `[agent]` (from Hermes)
- CLAUDE.md reference (added via PR)
