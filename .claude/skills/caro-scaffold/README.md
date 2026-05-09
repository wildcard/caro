# caro-scaffold

A Claude Code / Cursor skill that helps users scaffold CaroML task files
(`.caro`) from natural-language descriptions.

## Install

In any skill-aware coder agent:

```sh
caro skill install   # copies into ~/.claude/skills/caro-scaffold/
```

Or copy the `SKILL.md` file manually into your local skills directory.

## Use

Trigger by asking the agent:

- "Make a CaroML task that ..."
- "Scaffold a runbook for ..."
- "Create a .caro for ..."
- `/caro new <name>`

The skill will ask 1–3 clarifying questions, then write the file (or call
`caro new <name> "<description>"` if Caro is on PATH).

## See also

- CaroML language: `docs/caroml/grammar.md` (in the caro repo)
- Caro CLI: `caro --help`
- Companion skill: `caro-shell` (shell-command inference)
