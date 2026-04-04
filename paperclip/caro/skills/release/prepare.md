# Skill: Release Preparation

Wraps existing Caro command: `.claude/commands/caro.release.prepare.md`

## Purpose

Create release branch and run all pre-flight checks for a caro release.

## When to Use

- Starting a new release cycle
- Must be on `main` branch with clean working directory

## Workflow

1. **Pre-flight**: Verify on main branch, clean working directory
2. **Branch**: Create `release/vX.Y.Z` branch
3. **Security**: Run `/caro.release.security` for vulnerability audit
4. **Version**: Run `/caro.release.version` for version bump and changelog
5. **Publish**: Run `/caro.release.publish` for PR, merge, tag, publish
6. **Verify**: Run `/caro.release.verify` for crates.io verification

## Invocation

```
/caro.release.prepare [version]
```

## Key Files

- `.claude/commands/caro.release.prepare.md`
- `.claude/commands/caro.release.security.md`
- `.claude/commands/caro.release.version.md`
- `.claude/commands/caro.release.publish.md`
- `.claude/commands/caro.release.verify.md`
- `docs/RELEASE_PROCESS.md`
