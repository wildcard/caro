# Caro Integrator — Nightly Playbook

> Operational checklist for the **caro-integrator** persistent sub-agent.
> Re-read in full at the start of every nightly pass (cron `0 23 * * *`,
> task id `caro-integrator-nightly`).
>
> Persona doc: `~/.claude/projects/-Users-kobik-private-workspace-caro/memory/agent_caro_integrator.md`
> Status matrix: `./integrations-status.md` (this dir)
> Log: `./integrator-log.md` (this dir)

---

## Loop (one pass = one PR maximum)

### 1. Bootstrap

```bash
cd ~/workspace/caro
git fetch
bin/sk-new-feature integrator-$(date +%Y%m%d)
cd .worktrees/<NNN>-integrator-*/
```

If the worktree helper fails, fall back to:

```bash
git worktree add .worktrees/integrator-$(date +%Y%m%d) -b integrator/$(date +%Y%m%d)
```

### 2. Validate against the published binary

```bash
cargo install caro --force --locked    # latest crates.io
caro --version                          # confirm
```

When validating a not-yet-released change in tonight's PR, use `cargo install --path . --force` from the worktree branch instead — and note that explicitly in the validation evidence.

### 3. Read state

```bash
cat .claude/memory/integrations-status.md
cat .claude/memory/integrator-log.md | tail -20
gh issue list --label integration --state open --limit 30
gh pr list --search "integration OR mcp OR openai OR skill" --state open --limit 20
```

### 4. Research (≤ 6 web queries, ~10 min budget)

Topics to scan:
- New coder agents / agentic IDEs launched since last pass
- MCP server registry / Claude Code skill marketplace updates
- OpenRouter routing / model-selection news
- Release notes for already-tracked tools (Claude Code, Codex, opencode, crush, droid, …)

For anything new, add a row to `integrations-status.md` with status `not-yet`.

### 5. Validate top 3 stale rows

Pick the 3 rows in `integrations-status.md` with the oldest `last-validated` dates. For each:

| Surface | How to validate |
|---|---|
| Claude Code skill | Fresh CC session, install/invoke the `caro-shell` skill, ask for a command, confirm `caro` is shelled out and a validated suggestion returns |
| MCP server | `caro mcp serve` + minimal handshake (`initialize`, `tools/list`) via `mcp-inspect` or curl |
| OpenAI-compat shim | `curl -X POST http://localhost:PORT/v1/chat/completions ...` with a tool/function call, expect an OpenAI-shaped response |
| Native backend (Ollama/vLLM/Claude/Exo/Gemini/OpenRouter) | `caro --backend <name> --dry-run "list pdfs"` and inspect output |
| Long-tail tool integration | Follow whatever copy-paste snippet the website integrations page advertises; if it doesn't work end-to-end, the snippet is wrong — file a P1 |

Mark each row PASS / FAIL in the matrix with the date and a one-line evidence string (command + summary of output).

### 6. Triage failures

| Severity | Criteria | Action |
|---|---|---|
| **P0** | Advertised integration broken; user-visible regression | Fix PR same session, no exceptions |
| **P1** | Works but not as documented; copy-paste snippet wrong | Fix PR same session if scoped, else GH issue with `P1` |
| **P2** | Polish / minor friction | GH issue, no deadline |
| **P3** | Long-tail / out-of-scope | GH issue, defer |

Always dedup: `gh issue list --label integration --state all --search "<keywords>"` + check closed.

### 7. Ship the night's PR

Pick the topmost unblocked row from the **Priority queue** (in `integrations-status.md`). One PR, scoped tight.

- Branch: feature branch per `dev-process.md`
- Commits: conventional, `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`
- PR title: `feat(integrations): <one-line>`
- PR body: rationale / what changed / how to test / screenshot or asciinema if UX-facing / dedup statement (which existing issues this addresses)
- PR comment shape per `~/.claude/rules/pr-comment-structure.md`

### 8. Market for agents

When something new ships:

1. Update `website/src/data/integrations.ts` — add or transition the row to `working`, with copy-paste snippet for end users.
2. Update `README.md` "Use caro from your agent" matrix.
3. Optional: a 1-paragraph blog snippet at `website/src/content/blog/<date>-<tool>-integration.md` — only when it's a real announcement, not a polish PR.

### 9. Close the loop

```bash
# Update artifacts
$EDITOR .claude/memory/integrations-status.md   # mark validations + new rows
$EDITOR .claude/memory/integrator-log.md        # append one-line entry

git add -A
git commit -m "feat(integrations): <one-line>

<body>

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push -u origin <branch>
gh pr create --title "..." --body "..."

# beads sync if .beads/ is present (it is in this repo)
bd sync || true
```

### 10. Idea capture

Anything noticed during research/validation but out of tonight's scope: file as deduped GH issue with labels `integration` + `nightly-discovery` + appropriate priority.

---

## Hard constraints

- **Validate against the published binary first.** Never claim "X works" without `✓ VERIFIED` evidence. (Project-wide claim-verification rule.)
- **No finance details** in any caro file, issue, PR, or website page.
- **Feature branches only.** A PreToolUse hook blocks commits to main.
- **One PR per night.** Don't bundle. Don't refactor adjacent code.
- **Dedup every issue** before filing — open + closed.
- **`--dry-run`** for any `caro` invocation that would execute a shell command during validation.

---

## Strategic priority queue (initial seed; updated in `integrations-status.md`)

1. Claude Code skill (`caro-shell`) ✅ shipped first night
2. Website integrations landing page ✅ shipped first night
3. Claude Code MCP server (`caro mcp serve`)
4. OpenAI-compat HTTP shim (`caro serve --openai`)
5. Claude Code session-token reuse backend
6. OpenRouter backend (incl. `auto`)
7. Gemini / Jules backend (coordinate w/ in-flight PR #782)
8. Long tail: opencode, crush, droid, Sourcegraph Amp, Letta, Tabby, Pi, Aug, CodePal, qwen — most satisfied by #4 (OpenAI-compat)

Once the topmost unblocked row is shipped, demote and re-rank. Add new rows discovered during research.

---

## Sister roles

- **caro-qa-agent** — daily QA pass. We run on adjacent schedules; their findings about user-visible regressions sometimes surface as integration regressions on my plate.
- **caro-coder-loop** — the v1.2.0 feature delivery loop. They claim ready beads tasks; I file issues that may become beads tasks.

---

*Last updated: 2026-04-26 (initial seed, first night)*
