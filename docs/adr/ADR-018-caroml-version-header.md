# ADR-018: CaroML Version Header

**Status**: Accepted

**Date**: 2026-10-10

**Authors**: Caro maintainers

**Target**: Community

## Context

People save `.caro` task files and Carofiles and run them for months. The
files carry no version, so a grammar change would silently change the
behavior of every saved file. Only `caroml.lock` has a version
(`SCHEMA_VERSION = 2`).

google/ax puts `apiVersion: ax.io/v1alpha1` on every resource. AX never
checks it: any string is accepted (`pkg/apis/v1alpha1/types.go`). A version
that nobody checks protects nothing. See
`docs/research/2026-09-24-google-ax-lessons.md`, section 2.

## Decision

1. A CaroML file (`.caro` or Carofile) may start with `CAROML <major>`.
2. The header must be the first non-`REM` line, and it may appear once.
   Otherwise the parser returns `MisplacedVersion`.
3. A file without the header is read as major version 1. All existing
   files stay valid.
4. The parser rejects any value other than a known major version with
   `UnsupportedVersion`. Today the only known value is `1`
   (`CAROML_MAJOR_VERSION`). `1.1`, `v1` and an empty value are rejected.
5. A breaking grammar change raises the major version. The parser then
   accepts the old and the new major, or refuses the old one with an error
   that names it. It never reads an old file with new rules.

## Rationale

- **Enforced, not declared.** The parser rejects unknown versions. That is
  the part AX lacks.
- **Optional.** No existing file or tutorial has to change.
- **Major only.** A minor version could not change behavior without
  breaking the contract. Additive grammar needs no version bump.
- **Same rule in both parsers.** `check_version_header` in
  `src/caroml/ast.rs` serves `.caro` and Carofile alike.

## Consequences

### Benefits

- A future grammar change can fail loudly on old files instead of
  changing their behavior.
- `caro check` reports a version problem with a line number, like any
  other parse error.

### Trade-offs

- One more keyword (`CAROML`) in the grammar.
- Files written for a newer caro fail on an older caro. This is intended.

### Risks

- If maintainers make a breaking change without raising the major
  version, the header gives no protection. Review must catch that.

## Alternatives Considered

### Alternative 1: `REM caroml:v1`

A magic comment keeps the keyword set unchanged. But `REM` means "ignored
everywhere", and a comment that changes parsing breaks that rule.

### Alternative 2: Version only in `caroml.lock`

The lock already has a schema version. But the lock describes generated
output, not the source grammar, and many task files have no lock.

### Alternative 3: Required header

A required header would make every existing file invalid on upgrade.

## Implementation Notes

- `src/caroml/ast.rs`: `CAROML_MAJOR_VERSION`, `check_version_header`,
  and the `UnsupportedVersion` and `MisplacedVersion` error kinds.
- `src/caroml/parser.rs` and `src/caroml/carofile.rs` call the check.
- Tests: `caroml::parser::tests::version_header_v1_parses_like_no_header`,
  `rejects_unknown_version`, `rejects_misplaced_or_repeated_version`, and
  `caroml::carofile::tests::carofile_reads_version_header`.
- Docs: `docs/caroml/grammar.md`, `docs/caroml/carofile.md`.

## Success Metrics

- The first breaking grammar change ships with `CAROML 2`. A `CAROML 1`
  file still parses, or fails with an error that names version 1.

## References

- `docs/research/2026-09-24-google-ax-lessons.md`, section 2
- `docs/caroml/lock-schema.md` (the lock's own `SCHEMA_VERSION`)

## Revision History

| Date | Change |
|---|---|
| 2026-10-10 | Initial version |
