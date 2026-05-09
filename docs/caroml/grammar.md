# CaroML grammar reference

Plain UTF-8, line-oriented, no required indentation, blank lines and trailing
whitespace are insignificant. The first non-`REM` token on each line determines
the line kind. Eight keywords — that's the entire surface area.

## Keywords

| Keyword | Multiplicity | Form |
|---|---|---|
| `TASK` | exactly 1 | `TASK <title>` |
| `WHY` | 0 or 1 | `WHY <reason>` |
| `NEED` | 0+ | `NEED <thing>` (e.g. `NEED sudo`, `NEED jq`) |
| `ON` | 0+ | `ON <platform> [PREFER <a, b>] [AVOID <c, d>]` |
| `LET` | 0+ | `LET <name> = <value>` |
| `DO` | 1+ | `DO <intent>` (the unit of generation) |
| `NOTE` | 0+ | `NOTE <hint>` (attaches to next `DO`) |
| `REM` | 0+ | `REM <comment>` (ignored everywhere) |

Recognized platforms: `macos`, `linux`, `windows`, `posix`.

## `LET` substitution

Reference a parameter inside any `DO` line as `{name}`:

```text
LET path = /var/log
LET days = 30

DO find regular log files in {path}
DO filter to those older than {days} days
```

Substitution happens at parse time. The lock stores the post-substitution
intent.

### Escaping literal braces

`{{` is a literal `{`. `}}` is a literal `}`. Lets shell snippets like
`awk '{{print $1}}'` survive parsing.

```text
DO run awk '{{print $1}}' on the file
```

This produces the intent text `run awk '{print $1}' on the file`.

## Errors

The parser is fail-fast in v0.1: it returns the first error it encounters
with a 1-based line number suitable for editor jump-to. The error kinds:

| Kind | Trigger |
|---|---|
| `MissingTaskHeader` | First non-comment line wasn't `TASK <title>` |
| `DuplicateTask` / `DuplicateWhy` | Multiple `TASK` or `WHY` lines |
| `UnknownKeyword` | Line started with a keyword that isn't one of the eight |
| `MalformedLet` | `LET` line couldn't be parsed as `LET name = value` |
| `MalformedOn` | `ON` line couldn't be parsed (unknown platform, malformed PREFER/AVOID) |
| `UnclosedInterpolation` | `{name` with no closing `}` |
| `EmptyInterpolation` | Empty `{}` (probably typo or wanted `{{`) |
| `UndefinedParam` | `{name}` with no prior `LET name = ...` |
| `EmptyTaskTitle` | `TASK` line with no title |
| `NoSteps` | File parsed cleanly but had zero `DO` lines |

## Future grammar (parsed-but-not-interpreted in v0.1)

`ASSERT`, `WHEN`, `UNLESS` are reserved for v0.2. The parser will accept
them but the interpreter ignores them today.
