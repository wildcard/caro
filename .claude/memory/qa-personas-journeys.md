# QA Personas and Journeys

Every test case in `qa-test-design.md` names a persona (P*) and a journey (J*). Write findings from the user's view first, then the mechanism.

## Personas

| ID | Persona | Goal | Typical environment | Risk if Caro fails them |
|----|---------|------|---------------------|-------------------------|
| P1 | New user | Get one useful command in 5 minutes | Fresh install, no model, tty | Abandons at first error or prompt |
| P2 | Power CLI user | Fast, terse queries; scripts around caro | Configured, pipes, `--output json` | Loses trust after one wrong command |
| P3 | CI / agent integrator | Run caro non-interactively in a subprocess | No tty, open stdin, fresh HOME | Pipeline hangs |
| P4 | Cautious sysadmin | Never run a destructive command by accident | Servers, `--safety strict` | Data loss; the worst outcome |
| P5 | Non-English user | Ask in their own language | UTF-8 locales, emoji | Gets an unrelated answer |
| P6 | Hostile prompt author | Make caro emit or hide a dangerous command | Prompt injection, obfuscation | Safety bypass |

## Journeys

| ID | Journey | Steps |
|----|---------|-------|
| J1 | First run | install -> `--version` -> `doctor` -> first prompt -> consent prompt -> command |
| J2 | Daily query | `caro -p "..." --dry-run` -> read command -> accept |
| J3 | Scripted use | pipe or subprocess -> `--output json` -> parse -> exit code |
| J4 | Configure | `config show/set/get/reset` -> verify effect |
| J5 | Dangerous request | prompt with risky intent -> expect block, warning or clarification |
| J6 | Wrong backend | bad `--backend`, missing key, no network -> expect clear error |
