# v1.2.0 Product Specification: Delivering on the Promise

**Version**: 1.2.0
**Date**: March 26, 2026
**Author**: Product Management (generated)
**Status**: Draft
**Milestone**: April 30, 2026

---

## 1. Product Vision

Caro is a CLI tool that converts natural language into safe POSIX shell commands using local LLMs. The v1.2.0 release is about **credibility**: making the product match its marketing promise so that every feature claim on caro.sh is verifiably true.

## 2. Problem Statement

Two comprehensive audits (#790, #791) reveal that **40+ features advertised on caro.sh, docs.caro.sh, and in the Claude Skill documentation do not work as described**. Users who follow the documentation encounter:

- **Immediate hard errors**: `--quiet`, `--no-telemetry`, `-e`, `--model` flags don't exist
- **Silent failures**: `caro telemetry show` falls through to command generation
- **Wrong commands**: "delete all log files" returns `echo 'Unable to generate command'`
- **Empty results**: `alternatives` field is always empty even when safety blocks
- **Fabricated config**: 12 of 14 documented TOML config keys don't work

This erodes trust. A developer who tries caro based on the website and encounters these failures will not come back.

## 3. Target Users

| Persona | Current Experience | v1.2.0 Experience |
|---------|-------------------|-------------------|
| **Website visitor** | Reads FAQ, tries `--quiet`, gets error, uninstalls | Reads FAQ, tries `--quiet`, it works, stays |
| **Claude Code user** | Installs skill, tries config TOML from docs, fails | Installs skill, config keys work, trusts the tool |
| **DevOps engineer** | Wants `--output json` for CI scripting, no `alternatives` field | Gets structured JSON with safety metadata |
| **New developer** | Asks "delete all log files", gets `echo 'Unable to generate'` | Gets working `find` command with safety warning |
| **Privacy-conscious user** | Wants `--no-telemetry`, gets error | Flag works, telemetry disabled for session |

## 4. User Stories

### P0 — Immediate Errors

**US-001**: As a user, I can run `caro --quiet "list files"` so that non-essential output is suppressed.
- **Acceptance**: `--quiet` is a valid flag, timing output is hidden, command is generated correctly.
- **Test**: `caro --quiet "list files" | grep -v "Generated in"` returns only the command.

**US-002**: As a user, I can run `caro -e "list files"` so that I have a short form for `--execute`.
- **Acceptance**: `-e` is equivalent to `--execute`, command runs immediately.
- **Test**: `caro -e "echo hello"` prints "hello".

**US-003**: As a user, I can run `caro --no-telemetry "list files"` so that telemetry is disabled for this session.
- **Acceptance**: Flag is recognized, no telemetry events are recorded for this invocation.
- **Test**: `caro --no-telemetry "list files"` works without error; telemetry `show` reflects no new events.

**US-004**: As a user, I can run `caro telemetry show` to see what telemetry data has been collected.
- **Acceptance**: Shows session count, command count, last session time. No network access required.
- **Test**: `caro telemetry show` outputs readable summary of local SQLite telemetry data.

**US-005**: As a user, I can run `caro telemetry export -o data.json` to export my telemetry for review.
- **Acceptance**: Creates valid JSON file with all local telemetry data.
- **Test**: `caro telemetry export -o /tmp/test.json` produces parseable JSON.

**US-006**: As a user, I can run `caro config set telemetry.air_gapped true` so that no telemetry is sent.
- **Acceptance**: Config key is accepted, persisted, and read at startup.
- **Test**: `caro config set telemetry.air_gapped true && caro config get telemetry.air_gapped` → `true`

**US-007**: As a user, I can set `CARO_TELEMETRY_ENABLED=false` as an environment variable to disable telemetry.
- **Acceptance**: Env var overrides config file setting.
- **Test**: `CARO_TELEMETRY_ENABLED=false caro "list files"` works without telemetry.

**US-008**: As a user, I can set `safety.level` in my config TOML file.
- **Acceptance**: `caro config set safety.level strict` works. Available levels: strict, moderate, permissive.
- **Test**: `caro config set safety.level strict && caro config get safety.level` → `strict`

**US-009**: As a user, I can set `backend.primary` in my config to choose my default backend.
- **Acceptance**: `caro config set backend.primary ollama` works.
- **Test**: `caro config set backend.primary ollama` accepted, `caro config get backend` shows it.

### P1 — Misleading Behavior

**US-010**: As a user, when my command is blocked by safety, I see a safer alternative suggestion.
- **Acceptance**: Blocking a command shows "Try this instead: [safer_command]". The `alternatives` field in JSON output is populated.
- **Test**: `caro "delete everything in root directory"` shows CRITICAL warning with suggestion.

**US-011**: As a user, I see colored safety levels (🟢 Safe, 🟡 Moderate, 🟠 High, 🔴 Critical) in my terminal.
- **Acceptance**: ANSI colored output in TTY, plain text when piped.
- **Test**: `caro "list files" | cat` shows plain text; direct terminal shows colors.

**US-012**: As a user, I can use `caro --shell powershell "list files"` and get Windows-appropriate commands.
- **Acceptance**: PowerShell syntax (`Get-ChildItem`, `dir`, etc.) instead of POSIX.
- **Test**: `caro --shell powershell "list files"` returns `Get-ChildItem` or `dir`.

**US-013**: As a user, I see a `confidence_score` in JSON output so I can programmatically evaluate command quality.
- **Acceptance**: `caro --output json "list files"` includes `confidence_score` field.
- **Test**: `caro --output json "list files" | jq .confidence_score` returns a number between 0 and 1.

**US-014**: As a user, "delete all log files" generates a working command, not `echo 'Unable to generate command'`.
- **Acceptance**: Returns something like `find . -name "*.log" -type f -mtime +30 -delete` or a safe variant.
- **Test**: `caro "delete all log files"` generates a `find`-based command.

**US-015**: As a user, "check disk space" generates `df -h`, not `ls -la`.
- **Acceptance**: Returns `df -h` or equivalent.
- **Test**: `caro "check disk space"` returns `df -h`.

### P2 — Website Accuracy

**US-016**: As a website visitor, I can follow any example on caro.sh/faq and it works.
- **Acceptance**: Zero errors when following documented examples. All flags/subcommands/config keys work as described.

**US-017**: As a website visitor, I can follow any example on caro.sh/telemetry and it works.
- **Acceptance**: `caro telemetry show`, `caro telemetry export`, `--no-telemetry`, config keys all work.

**US-018**: As a Claude Code user, I can follow the SKILL.md config TOML examples and they work.
- **Acceptance**: All config keys referenced in skill docs are accepted by `caro config set`.

---

## 5. Key Performance Indicators (KPIs)

| Metric | Current (v1.1.3) | v1.2.0 Target |
|--------|-------------------|---------------|
| Documented features that actually work | ~60% | **100%** |
| Eval test pass rate (static matcher) | 31% | **60%+** |
| Embedded model pass rate | ~30% | **45%+** |
| "Common query" pass rate (top 20 queries) | ~40% | **80%+** |
| Config keys that work | 4 of 14 documented | **14 of 14** |
| CLI flags that work | Missing 5 of 22 documented | **All 22** |
| Safety alternatives populated | 0% (always empty) | **100% of blocked commands** |
| Website claim accuracy | ~60% | **100%** |

---

## 6. Competitive Positioning

v1.2.0 positions caro against:

| Competitor | caro Advantage | v1.2.0 Status |
|------------|---------------|---------------|
| **GitHub Copilot CLI** | Offline, privacy-first, free | **Verified** — works without API |
| **Warp AI** | Works in any terminal, not proprietary | **Verified** — POSIX CLI tool |
| **Kiro CLI** | Open source, no cloud dependency | **Verified** — AGPL-3.0 |
| **Fig/Copilot in terminal** | Local inference, zero telemetry by default | **Verified** — embedded backend |

**The competitive advantage claim is legitimate for offline/privacy. The gap was in feature parity of the UX itself.**

---

## 7. Go-to-Market Alignment

### 7.1 Website Updates Needed

| Page | Action |
|------|--------|
| `/faq` | Remove or update 6 flags that don't exist (or implement them in v1.2) |
| `/telemetry` | Update subcommand/config examples to match implementation |
| `/explore` | Fix MCP server status to "Coming Soon", fix performance claims |
| `/compare` | Mark "Customizable safety rules" as "Planned" |
| `/roadmap` | Reflect actual v1.1.x completion |
| `/compare/warp` | Verify offline claim against actual behavior |
| `/compare/github-copilot-cli` | Verify "blocks dangerous commands" claim |
| `/blog/announcing-caro` | Add note about v1.2 improvements |

### 7.2 Docs.caro.sh Alignment

- Remove `caro assess`, `caro knowledge`, `caro profile` from active docs (mark experimental)
- Fix quick-start y/n prompt to match actual interactive behavior
- Update performance claims to real numbers

### 7.3 Claude Skill Alignment

- Rewrite SKILL.md to reference only working features
- Fix config TOML examples
- Remove keyboard shortcuts that don't exist
- Update installation checker script

### 7.4 i18n

- Target: 15 locales, Tier 1 ≥95% coverage before website launch
- Complete WP07 (automation) and WP08 (switcher)
- Merge i18n bot PRs (#747-#760)

---

## 8. Success Criteria for v1.2.0 Release

### Must Have (ship blockers)

- [ ] All 9 Tier 0 gaps implemented or removed from docs
- [ ] All 8 Tier 1 gaps implemented (safer alternatives, color output, etc.)
- [ ] Eval test pass rate ≥60% (up from 31%)
- [ ] All website examples work without errors
- [ ] All skill doc config examples work
- [ ] `alternatives` field populated for blocked commands
- [ ] `--shell powershell` generates Windows syntax
- [ ] Telemetry subcommands operational (show/export)
- [ ] Env var overrides work (CARO_TELEMETRY_ENABLED, CARO_BACKEND, CARO_SAFETY)
- [ ] Zero cargo clippy warnings with `-D warnings`
- [ ] All tests pass

### Should Have

- [ ] i18n coverage Tier 1 ≥95%
- [ ] Website launched at caro.sh with accurate content
- [ ] Docs site live at docs.caro.sh
- [ ] SEO meta tags on all pages
- [ ] Changelog updated for v1.2.0

### Nice to Have

- [ ] Embedded model pass rate ≥45%
- [ ] Homebrew formula with SHA256 (#595)
- [ ] Nix support complete (#620)
- [ ] NixOS explore page (#164)

---

## 9. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Prompt engineering doesn't improve model quality enough | Medium | High | Static matcher expansion as safety net |
| Telemetry subcommands need more than uncommenting | Medium | Medium | Spike first; allocate 3 days |
| Config expansion breaks backward compat | Low | High | Add config migration logic |
| Website PRs (#130, #639) block launch | Medium | High | Deprioritize visual polish; content accuracy first |
| 23 blocked PRs (#681) create cascading conflicts | High | High | Resolve #681 in Week 1 as P0 |
| i18n completion delays launch | Low | Medium | Tier 1 languages only for v1.2; others can follow |

---

## 10. Release Checklist

- [ ] All P0 features implemented and tested
- [ ] All P1 features implemented and tested
- [ ] gap-analysis tests pass (new test suite validating #790/#791 items)
- [ ] `cargo test` — all pass
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --all --check` — zero formatting issues
- [ ] `cargo audit` — no known vulnerabilities
- [ ] CHANGELOG.md updated
- [ ] website/src/config/site.ts version bumped
- [ ] Release branch created per release process
- [ ] PR reviewed and merged
- [ ] Tagged and published to crates.io
- [ ] Website content verified against new behavior
- [ ] Skill docs verified against new behavior
- [ ] Social media / blog post announcing v1.2.0
