# Skill: Release Publishing

Wraps existing Caro command: `.claude/commands/caro.release.publish.md`

## Purpose

Create PR, merge, tag, and publish caro release to crates.io and GitHub.

## When to Use

- After release preparation and version bumping is complete
- After security audit passes
- Requires board approval (governance gate)

## Workflow

1. **Create PR**: From release branch to main
2. **Verify CI**: Ensure all checks pass
3. **Merge**: Merge release PR
4. **Tag**: Create git tag for version
5. **Publish**: Push to crates.io
6. **Verify**: Confirm package is available

## Invocation

```
/caro.release.publish
```

## Key Files

- `.claude/commands/caro.release.publish.md`
- `Cargo.toml` — Version number
- `CHANGELOG.md` — Release notes
