# Coder Agent Isolation — Three Layers, Three Concerns

**APPLIES TO**: Any time the project spawns, recommends, or documents an
isolated coder agent (Claude Code, Crush, kraken, sandflox loops, the
caro coder-loop skill, devcontainer-based agent recipes, etc.).

Codified from the 2026-05-25 async exchange with the Flox team (Ben
Futoriansky, Head of Bus Ops) about running isolated coder agents on
top of Caro. The exchange made it clear that the project had been
silently conflating *boundary* (the OS-level enforcement layer) and
*environment* (the reproducible dependency set). Splitting them is the
whole point of the rule.

## The principle

**Boundary and environment are independent choices.** Pick each on its
own merits. Don't substitute one for the other.

```
┌────────────────────────────────────────────────────────────────────┐
│  Layer 3 — Command safety                                          │
│  Caro's pattern validator. Always on. Owned by src/safety/.        │
├────────────────────────────────────────────────────────────────────┤
│  Layer 2 — Boundary (OS-level isolation)                           │
│  macOS sandbox (sandflox)  →  container  →  full VM                │
│  faster, lighter  ←──────────────────────────→  stronger, heavier  │
├────────────────────────────────────────────────────────────────────┤
│  Layer 1 — Environment (reproducible dep set)                      │
│  Flox manifest. Inspectable before activation. Relocates unchanged │
│  across every boundary in Layer 2.                                 │
└────────────────────────────────────────────────────────────────────┘
```

Read top to bottom: every command an agent emits passes through Caro
(L3); the process runs inside whichever boundary you chose (L2); the
binaries it can invoke are exactly the ones the Flox env (L1) installed.

## Layer 1 — Environment (Flox)

- A Flox manifest (`.flox/env/manifest.toml`) pins the exact dep set
  the agent can see. The set is Nix-derived and inspectable *before*
  activation, so the user can audit the supply chain in the same posture
  Caro takes at the command layer.
- Same manifest, same packages, every boundary in Layer 2. That's the
  reason to use Flox at all — write the env once, harden the boundary
  later without rebuilding.
- Use Flox **any time** the agent needs reproducible tooling, even when
  the boundary is `none` (bare host). The reproducibility win exists
  independently of the isolation win.

## Layer 2 — Boundary (sandflox / container / VM)

Pick by threat model and platform. Defaults:

| Boundary | When to default to it | Trade-off |
|---|---|---|
| **sandflox** (macOS kernel sandbox, [github.com/flox/sandflox](https://github.com/flox/sandflox)) | Mac ARM, single-developer machine, agent loops within a trusted repo | Native speed, ARM Mac only today |
| **Container** (Docker/Podman/Colima, ideally via `flox containerize` or `containerd-shim-flox` so there's no Dockerfile to drift) | Crossing OS lines, needing Linux-only deps, sharing the same env across CI + dev | Slower bind-mount I/O on macOS |
| **Full VM** | Boundary integrity > performance — untrusted code, regulated workloads, multi-tenant hosts | Slowest, RAM + disk overhead, painful shared folders on macOS |

Don't reach for the strongest boundary by default. The Mac sandbox is
kernel-enforced and built for AI agents; the perf cost of jumping
straight to a VM is real.

## Layer 3 — Command safety (Caro)

Always on, independent of L1 and L2. Caro's pattern validator catches
the things the agent's training set rationalized as safe but actually
aren't. The other two layers contain *blast radius*; Caro prevents
*ignition*.

## Anti-patterns

| Anti-pattern | Why it's wrong |
|---|---|
| "We already have a Dockerfile, skip Flox." | You lose env inspectability and same-env-across-boundaries portability. Dockerfile pins a snapshot; Flox pins a derivation graph. |
| "Flox alone is isolation enough." | Flox is L1. With no L2, the agent still runs as the user. An `npm install` of a malicious package can do whatever the user can do. |
| "Sandflox is just the Mac equivalent of Docker." | Different layers. Sandflox is L2 (kernel boundary); Docker is L2 too but heavier and not Mac-native. The Flox env you put *inside* either is L1. |
| "VM is the safe default for coder agents." | Premature pessimization. Use the cheapest boundary that matches the threat model. Hardening later is a manifest edit, not a rebuild. |
| "Let the agent run on the host so it's fast." | Without L2, a hallucinated `rm -rf $HOME` reaches the host. L3 (Caro) catches *most* such cases but L2 is the backstop. |

## When this rule fires

- **Spawning an agent loop** — pick L1 + L2 explicitly. The Flox
  manifest at `.flox/env/manifest.toml` is the project's reference L1
  for caro's own build env; reuse it or extend it, don't reinvent.
- **Writing a use-case or runbook** that mentions agent isolation —
  cite this rule and the three-layer diagram. Don't describe Flox as a
  Docker alternative.
- **Reviewing a PR** that introduces an isolation mechanism — confirm
  the author named *which layer* they're touching. A PR that "adds
  sandbox support" should say whether it's L2 enforcement, L1 env, or
  L3 command-validation.

## See also

- [github.com/flox/sandflox](https://github.com/flox/sandflox) — L2,
  macOS kernel sandbox built for AI coding agents
- [flox.dev](https://flox.dev/) — L1, Nix-based reproducible envs
- `docs/adr/ADR-010-bubblewrap-sandbox-execution.md` — caro's L2 spec
  for Linux (bubblewrap) and the macOS/Windows alternatives; this rule
  is the cross-layer framing that ADR-010 implements one slice of
- `.flox/env/manifest.toml` — caro's own dev env (L1 reference)
- `website/src/pages/use-cases/flox.astro` — the public-facing
  articulation of this three-layer model for the project's users
