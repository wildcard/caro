# Threat & Design: Path-Aware Allowlist for Scoped Critical Deletions (caro-qknc)

**Status:** DESIGN — awaiting policy sign-off before TDD implementation.
**Risk class:** Safety-critical (relaxes a Critical-command block). Requires
`safety-pattern-auditor` review before merge.

## Problem

`SafetyValidator::validate_command` ([src/safety/mod.rs:490](../../src/safety/mod.rs))
blocks **all** allowlist bypass when any built-in/CVE Critical pattern matches:

```rust
let has_critical_builtin_or_cve_match = … any Critical pattern matches;
if !has_critical_builtin_or_cve_match { /* apply allowlist */ }
```

So a user who allowlists `rm -rf /tmp/myapp_\d+` gets **nothing** — the Critical
recursive-rm pattern fires and the allowlist is ignored. `test_allowlist_functionality`
([tests/safety_validator_contract.rs:314](../../tests/safety_validator_contract.rs))
encodes the user-desired behavior (the scoped temp deletion *should* be allowed)
and therefore fails.

User decision (2026-06-07): *allow scoped deletions under safe paths (e.g. `/tmp`),
never under system/root paths — user opt-in, with agent guidance.*

## Threat model — what must NEVER be allowed (even with a user allowlist)

| # | Attack | Must stay blocked |
|---|--------|-------------------|
| T1 | `rm -rf /` , `rm -rf /etc` , `rm -rf /usr` , `rm -rf /var` | system/root paths |
| T2 | `rm -rf $HOME` , `rm -rf ~` , `rm -rf /home/other` | home roots / other users |
| T3 | Traversal escape: `rm -rf /tmp/../etc` , `/tmp/a/../../usr` | path that *resolves* outside safe root |
| T4 | Symlink escape: `/tmp/link` → `/etc` | symlinked target outside safe root |
| T5 | Glob escape: `rm -rf /tmp/../*` , bare `/tmp/*` only if every expansion stays inside | globs that can reach outside |
| T6 | Multiple targets where ANY is unsafe: `rm -rf /tmp/ok /etc` | reject whole command |
| T7 | Env/indirection: `rm -rf $X` , `rm -rf "$(…)"` | unresolvable target → reject |

**Invariant:** the override is *additive narrowing* — it can only turn a Critical
block into "allowed" for a command that is **provably** a deletion confined to a
safe root. Anything it cannot prove safe stays blocked. Fail-closed.

## Proposed mechanism

1. New config (opt-in): `[safety] allow_delete_paths = ["/tmp", "/var/tmp"]`
   (empty by default → feature off → behavior identical to today).
2. New classifier `is_scoped_safe_deletion(command, &allow_roots) -> bool` in
   `src/safety/` that returns true **only if** the command is a recursive
   deletion whose every target path, after lexical normalization (resolve `.`/`..`,
   reject `~`/`$`/command-subst/globs that can escape), is strictly contained in
   one of the configured safe roots.
3. In `validate_command`, the Critical guard becomes: bypass is permitted when
   `has_critical_match` **and** `is_scoped_safe_deletion(...)` **and** a user
   allowlist entry matches → return Allowed with a warning ("allowed: deletion
   confined to safe path `/tmp/...`; system paths would be blocked") = the
   "agent guidance".
4. Hard floor preserved: non-deletion Criticals (mkfs, dd, fork bomb, privilege
   escalation, network backdoor) are **never** eligible — only recursive-delete
   Criticals confined to safe roots.

## Open policy decisions (need sign-off)

- **D1 — safe-root set:** temp-only (`/tmp`, `/var/tmp`, `$TMPDIR`) vs. also
  project-scoped (paths under cwd like `./build`, `./node_modules`) vs.
  fully user-configured list (default empty).
- **D2 — path resolution strictness:** lexical-only normalization (fast, no FS
  access, rejects `..` escapes but not symlinks) vs. full canonicalization
  (`std::fs::canonicalize`, catches symlinks T4 but touches the filesystem and
  only works for existing paths).

## TDD test matrix (written before implementation)

ALLOWED (with `allow_delete_paths=["/tmp"]` + matching allowlist entry):
`rm -rf /tmp/myapp_123`, `rm -rf /tmp/build/`, `rm -fr /tmp/x` (flag order)

BLOCKED regardless of allowlist (T1–T7):
`rm -rf /`, `rm -rf /etc`, `rm -rf $HOME`, `rm -rf /tmp/../etc`,
`rm -rf /tmp/ok /usr`, `rm -rf $X`, `rm -rf /tmp/../*`, `mkfs /tmp/x`,
`dd if=/dev/zero of=/tmp/x` (not a deletion)

No-regression: with `allow_delete_paths=[]` (default), every existing safety test
passes unchanged.

## Security review (aegis, caro-qknc) — findings & resolution

- **CONFIRMED bypass (fixed):** `rm -rf /tmp/*/x` with a planted `/tmp/link -> /etc`.
  A glob in a **non-final** component means the literal `/tmp/*/x` doesn't exist,
  so `canonical_still_safe` anchored at `/tmp` and never resolved the symlink —
  but the shell expands `*` into `/tmp/link` and `rm` follows it to `/etc/x`
  (composition of T4∘T5). Resolution: `glob_position_ok` rejects any glob outside
  the final component and any glob with a trailing slash; leaf globs
  (`/tmp/myapp_*`) stay allowed because `rm` unlinks a matched symlink *operand*
  rather than recursing through it. Regression:
  `blocks_non_final_glob` + `blocks_glob_over_symlinked_dir_on_real_fs`
  (real symlink to `/etc`).
- **TOCTOU (documented):** the guarantee is check-time, not use-time; a symlink
  raced into a world-writable safe root after validation can still redirect a
  not-yet-existing target. Inherent to string validation; noted in the module doc.
- **Hardening (accepted, non-blocking, fail-closed):** `--` handling over-rejects
  rather than under-rejects; `ends_with("/rm")` is loose but requires an
  allowlist + planted binary. Left as-is to keep the surface minimal.
