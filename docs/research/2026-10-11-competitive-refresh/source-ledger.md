# Source Ledger for Published Competitor Claims

**Scope**: the competitor claims that the `/compare` pages, the landing
matrix, `/use-cases` and the GTM table publish after PR #1583.
**Rule**: one row per claim. A claim without a primary source is marked
`unknown` on the site ("Not documented"), never "no".

Status values:

- **primary**: read on the vendor's own page on the date shown.
- **scoped**: true only in the stated flow or plan.
- **conflict**: sources disagree; the site makes no general claim.
- **unknown**: no primary source found.

| Product | Claim on the site | Source | Source date | Checked | Status |
|---|---|---|---|---|---|
| Warp | Client is AGPL-3.0 (UI crates MIT) since 2026-04-28 | https://www.warp.dev/blog/warp-is-now-open-source ; https://github.com/warpdotdev/warp | 2026-04-28 | 2026-10-11 | primary |
| Warp | Server, Drive backend, orchestration and agent harness are not in the repo | warpdotdev/warp `FAQ.md` | undated | 2026-10-11 | primary |
| Warp | AI features need a network; no offline mode | https://docs.warp.dev/support-and-community/troubleshooting-and-support/using-warp-offline/ | 2026-10-09 | 2026-10-11 | primary |
| Warp | App works without an account; AI tied to a Warp account | https://docs.warp.dev/getting-started/quickstart/installation-and-setup/ ; https://docs.warp.dev/changelog/2026/ (2026.07.03) | 2026-10-09 | 2026-10-11 | primary |
| Warp | Agent CLI runs in any terminal; needs login or `WARP_API_KEY` | https://www.warp.dev/blog/introducing-the-warp-agent-cli-coding-agent ; https://docs.warp.dev/agents/cli/ | 2026-08-04 | 2026-10-11 | primary |
| Warp | Personal custom endpoint: prompt, context, endpoint and key pass through Warp; localhost/private URLs rejected | https://docs.warp.dev/agents/inference/custom-inference-endpoint/ | 2026-10-11 | 2026-10-13 (owner packet) | scoped |
| Warp | General AI routing (BYOK, pricing FAQ, Enterprise BYOLLM) | https://docs.warp.dev/agents/inference/bring-your-own-api-key/ ; https://www.warp.dev/pricing | varies by snapshot | 2026-10-13 | conflict |
| Warp | Regex allow/deny lists; denylist asks in normal modes; "Run until completion" bypasses the user denylist by default; admin rules not bypassed | https://docs.warp.dev/agents/capabilities/agent-profiles-permissions/ | 2026-10-09 | 2026-10-11 | primary |
| Warp | Risk level per command | none found | — | 2026-10-11 | unknown |
| Warp | macOS, Linux, Windows; no BSD build listed | https://docs.warp.dev/getting-started/quickstart/installation-and-setup/ | 2026-10-09 | 2026-10-11 | primary (BSD: absence only) |
| Warp | GNU/BSD awareness, platform detection | none found | — | 2026-10-13 | unknown |
| Copilot CLI | Proprietary "GitHub Copilot CLI License" | https://github.com/github/copilot-cli/blob/main/LICENSE.md | undated | 2026-10-11 | primary |
| Copilot CLI | BYOK incl. local models; offline needs a local model + `COPILOT_OFFLINE=true`; no GitHub login with own provider | https://github.blog/changelog/2026-04-07-copilot-cli-now-supports-byok-and-local-models/ ; https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-byok-models | 2026-04-07 | 2026-10-11 | primary |
| Copilot CLI | Local-model discovery installs nothing and does not enable offline mode | https://github.blog/changelog/2026-10-07-discover-local-models-in-github-copilot-cli/ | 2026-10-07 | 2026-10-13 (owner packet) | primary |
| Copilot CLI | OS sandbox GA | https://github.blog/changelog/2026-10-07-local-sandboxing-for-gitHub-copilot-now-generally-available | 2026-10-07 | 2026-10-13 | primary |
| Copilot CLI | Sandbox on or off by default | the GA changelog does not say | — | 2026-10-13 | unknown |
| Copilot CLI | Built-in pattern library; risk levels | none found | — | 2026-10-11 | unknown |
| Copilot CLI | Deny rules (`--deny-tool`), hooks | https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/allowing-tools | undated | 2026-10-11 | primary |
| Copilot CLI | Plans (Free includes the CLI; paid $10–$100, $19–$39/seat; AI Credits since 2026-06) | https://docs.github.com/en/copilot/get-started/plans | undated | 2026-10-11 | primary |
| Kiro CLI | Closed source (AWS IP License) | https://kiro.dev/license/ | undated | 2026-10-11 | primary |
| Kiro CLI | Models on Bedrock; no local or custom endpoint listed; sign-in required | https://kiro.dev/docs/models/ ; https://kiro.dev/faq/ | 2026-10-08 | 2026-10-11 | primary |
| Kiro CLI | `permissions.yaml` deny/ask/allow; read-only commands auto-run | https://kiro.dev/docs/permissions/ | 2026-10-10 | 2026-10-11 | primary |
| Kiro CLI | Paid plans from $20/month | https://kiro.dev/pricing/ | undated | 2026-10-11 | primary |
| Kiro CLI | Free tier covers the CLI | not confirmed | — | 2026-10-11 | unknown (not published) |
| OpenCode | MIT | https://github.com/anomalyco/opencode | 2026-10-11 | 2026-10-11 | primary |
| OpenCode | Most permissions allow; `doom_loop`, `external_directory` ask; `.env` reads deny (`.env.example` allowed) | https://opencode.ai/docs/permissions/ | 2026-10-13 | 2026-10-13 (owner packet) | primary |
| OpenCode | Offline after local provider + disabling model/update/LSP downloads | https://opencode.ai/docs/providers/ ; https://opencode.ai/docs/config/ | undated | 2026-10-11 | primary |
| shell-gpt | MIT; OpenAI by default, Ollama via LiteLLM | https://github.com/TheR1D/shell_gpt | 2026-05-06 (1.5.1) | 2026-10-11 | primary |

**Caro's own claims** (from code, at `origin/main` `c3169411`):

| Claim | Source in repo |
|---|---|
| Risk levels Safe, Moderate, High, Critical | `src/models/mod.rs` (`RiskLevel`) |
| Default safety level `moderate`: Critical blocked, High asks | `src/models/mod.rs` (`is_blocked`, `requires_confirmation`, `default_safety_level`) |
| User controls: `[safety]` custom patterns and allowlist | `src/safety/mod.rs` (`SafetyConfig`) |
| Default model Qwen2.5-Coder 1.5B, Apache-2.0, downloaded on first use | `src/model_catalog.rs`; `models/qwen2.5-coder-1.5b/README.md`; `src/model_loader.rs` |
| Telemetry opt-in on GA builds | `src/telemetry/config.rs` |
