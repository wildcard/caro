# Broken Windows — Handoff Register

Things an agent found broken **outside its own task**. The finder records the
problem here and moves on. A fixer claims an entry and fixes it. The fixer is
either the **caro broken-window sweep** routine (`trig_01TAiuoMoZ6nofW3ceYL3Csy`,
daily 15:30 UTC) or any agent already working in that area.

Rules (see `.claude/rules/good-boy-scout.md` → "Stay in your lane"):

- **Found one?** Search open issues first. If none exists, file one, then add
  an entry here that links it. Never spend your own task on it.
- **Claiming:** set `Status: claimed-by <branch>` before starting, and fix it
  in its own PR (not bundled into unrelated work).
- **Needs human: yes** means a security-policy, release or product decision
  (audit suppressions, safety patterns, dependency majors). Don't apply the
  change yourself; the issue is the ask.
- **Fixed:** set `Status: fixed (<PR>)` and leave the entry for a week so
  finders can see it's done, then delete it.

---

### BW-001: `cargo audit` red on main — h2 0.3.27 (RUSTSEC-2026-0258)

**Found:** 2026-09-24, PR #1470 session
**Issue:** #1472 (reqwest 0.11→0.12 + wiremock 0.5→0.6); lockfile part is #1471
**Status:** open
**Needs human?:** yes. The only fixes are a manifest-major upgrade (#1472)
or an `.cargo/audit.toml` suppression, and the suppression is a security-policy
call that the auto-mode classifier refuses without explicit maintainer approval.
**Next step:** maintainer picks between #1472 and a suppression. Until then
every PR shows a red `Security Audit` / `cargo-audit`; that is not the PR's fault.

### BW-002: CLAUDE.md version/MSRV drift, filed ~15× as duplicates

**Found:** 2026-09-27, PR #1470 session
**Issue:** oldest open is #1397; duplicates include #1407, #1411, #1414, #1420,
#1425, #1426, #1427, #1431, #1434, #1442, #1444, #1450, #1456, #1469, #1474, #1476
**Status:** open
**Needs human?:** no
**Next step:** fix CLAUDE.md (version 1.5.0, MSRV from `Cargo.toml`
`rust-version`), then close the duplicates as duplicates of the fixing issue.
Root cause: the daily QA routine (`trig_01Tk7DxyXV7LeYcFjgmTG1mZ`, 14:00 UTC)
re-files instead of commenting on the existing issue, and nothing fixes.

### BW-003: stale harness references (caro-eval, current-tasks.md, AGENTS.md, .kittify, --skill)

**Found:** 2026-09-24, google/ax harness survey
**Issue:** #1479
**Status:** open
**Needs human?:** no
**Next step:** see the issue table. Each row is a one-line docs/config fix.
