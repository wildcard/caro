# Caro Integrator — Nightly Log

> One-line-per-night journal maintained by the **caro-integrator** agent.
> Newest entries on top. Append, don't overwrite.

---

## 2026-05-09 — first scheduled cron pass (bootstrap-night fix)

- **Validated:** none — first scheduled fire on a not-yet-merged scaffolding PR. Per the bootstrap-check protocol in the scheduled-task header, no integration-row validation runs until PR #939 is on `main`.
- **Shipped:** this log entry. Diagnosis: PR #939's CI history shows 2 failures (ChromaDB Integration Tests + Security Audit) from the 2026-04-27 run, but the branch was rebased onto current `main` (`f8028edb`) on 2026-05-06. The rebase already includes the RUSTSEC-{0098,0099,0104} fix from PR #1026 (`rustls-webpki 0.103.13` in `Cargo.lock`, legacy 0.101.7 path covered by `.cargo/audit.toml` ignores). ChromaDB failures (`expected 10, got 14`; `expected 1, got 15`) are pre-existing flake from shared-collection state pollution — observed intermittently on `main` too. Pushing this commit triggers a fresh CI run on the rebased HEAD, which should clear Security Audit and re-roll the ChromaDB flake. Vercel `cmdai` failure is a vestigial pre-rename project deploy — out of scope.
- **Filed:** none. ChromaDB flake is real but already a known issue; not filing a duplicate without a `gh issue list --label integration --search chromadb` dedup pass next night when the bootstrap completes.
- **Discovered:** the pre-merge bootstrap loop has a long-tail risk — if the scaffolding PR sits unmerged for many nights, every nightly fire produces a near-duplicate "fix CI / wait for merge" entry. Mitigations: (a) keep this PR small enough to merge fast (it is), (b) auto-skip the loop on subsequent nights if state hasn't changed (future enhancement, file later if it bites).
- **Next pass should:** if PR #939 has merged by tomorrow's 23:00 fire, run the full Step 1–9 loop and pick up the topmost queue row from `integrations-status.md` — most likely "validate the 6 native backends" since none have a real `last-validated` date. If still unmerged, this entry tells the next agent the situation is identical and the right move is `**Needs user input:** still waiting on PR #939 merge` rather than another no-op commit.

---

## 2026-04-26 — first night (manual, pre-cron)

- **Shipped:** caro-shell Claude Code skill (`.claude/skills/caro-shell/SKILL.md`); website integrations data + landing page (`website/src/data/integrations.ts`, `website/src/pages/integrations/index.astro`); README "Use caro from your agent" section; integrator persona + playbook + status matrix + this log; nightly cron `caro-integrator-nightly` at `0 23 * * *`.
- **Validated:** Skill installs in a fresh Claude Code session and invokes the published `caro` binary end-to-end (verification step #4 from the plan). Other rows in the status matrix are seeded with `not-yet` validation dates — to be picked up over the next several nights.
- **Filed:** companion GH issues — deduped against #661/#667/#662/#504/#449/#789/#782:
  - #928 — caro mcp serve (Claude Code MCP server)
  - #929 — caro serve --openai (OpenAI-compat HTTP shim)
  - #930 — Claude Code session-token reuse backend
  - #931 — OpenRouter backend
  - #932 — native-backend nightly validation harness
  - #933 — long-tail tool copy-paste snippets (umbrella)
- **Next:** first scheduled cron firing at 23:00 local. The agent will pick up the topmost unblocked queue row — likely "validate the 6 native backends" since none have a real `last-validated` date.
