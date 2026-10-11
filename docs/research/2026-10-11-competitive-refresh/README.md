# Competitive Refresh: Raw Notes (2026-10-11)

These are the raw notes behind
[`../2026-10-11-competitive-landscape-refresh.md`](../2026-10-11-competitive-landscape-refresh.md).
Research agents wrote them on 2026-10-11 from vendor docs, repos, license
files and changelogs. Each claim carries a URL, a source date and a
confidence level. The parent session spot-checked the claims that change
our messaging. Check a claim against its source again before you quote it
on the website.

Lines such as "Needs you" or "website says X" describe the site **before**
PR #1583, which fixed them.

| File | Question it answers |
|---|---|
| [`warp.md`](./warp.md) | Is each claim on our Warp pages still true? Open source scope, cloud/offline, telemetry, platforms, command safety, terminal-agnostic CLI, 2026 launches. |
| [`copilot-kiro-opencode.md`](./copilot-kiro-opencode.md) | The same check for Copilot CLI, Kiro CLI and OpenCode, plus the hook points an external validator can use in each. |
| [`auto-modes.md`](./auto-modes.md) | How does each agent harness decide to auto-approve a shell command (rules, LLM judge, sandbox)? Is it open, model-locked, and hookable? |
| [`guardrail-tools.md`](./guardrail-tools.md) | Which tools gate agent commands across harnesses (guards, sandboxes, policy engines, rule databases)? Who is closest to `caro guard`? |
| [`discovery.md`](./discovery.md) | Which NL→shell tools, AI terminals and agentic CLIs exist, and which deserve a `/compare` page? |

## Re-running the refresh

Ask the same question per file, with today's date, and tell the
researcher:

- Use primary sources first.
- Give a URL and a source date for every claim.
- Mark anything unverified.
- Never invent a URL, a star count or a date.

Then diff the result against these notes.
