# Caro on Flox

This directory documents how caro distributes itself through the
[Flox](https://flox.dev/) ecosystem and how users can pin a release into
their own Flox environment today, before the upstream catalog submission
lands.

It is the package-author counterpart to the user-facing landing page at
[caro.sh/use-cases/flox](https://caro.sh/use-cases/flox) and the
cross-layer doctrine at
[`.claude/rules/coder-agent-isolation.md`](../.claude/rules/coder-agent-isolation.md).

## Status

| Surface | State | Tracking |
|---|---|---|
| `.flox/env/manifest.toml` for caro's own dev env | ✅ Shipped | This PR |
| Release-time Flox manifest stub (`caro-vX.Y.Z.toml` on every GitHub Release) | ✅ Shipped | `.github/workflows/packages.yml` flox job |
| `caro` binary in `flox-floxpkgs` catalog | ⏳ Deferred | Needs Flox-side review; tracked as a beads follow-up |
| `caro-skill` (Claude Code skill bundle) in catalog | ⏳ Deferred | Novel pattern; needs Flox team alignment on skill-bundle shape |

## Why two packages

Caro is consumed two ways:

1. **As a binary** — drop `caro` into a Flox env so the env's tools have
   a safe command-generation layer. This is the obvious split and the
   one most users will reach for.
2. **As a Claude Code skill bundle** — a Flox env that also installs
   `claude-code` benefits from caro's skill at
   `.claude/skills/caro-shell/SKILL.md` being installed alongside. That
   skill turns caro from a CLI you call into a safety layer Claude
   reaches for automatically.

Splitting the two lets a user opt into the binary without dragging the
skill (or vice versa) — same pattern as
[`ripgrep`](https://github.com/BurntSushi/ripgrep) vs
[`ripgrep-all`](https://github.com/phiresky/ripgrep-all): one tool, two
distribution surfaces.

## Pin a release in your own Flox env (today)

Until catalog submission lands, the most reliable way to pull a tagged
caro release into a Flox env is to pin the GitHub release directly:

```toml
# In your Flox env's manifest.toml
version = 1

[install]
caro.flake = "github:wildcard/caro/v1.4.0"

# Optional: ship claude-code in the same env for the agent-loop pattern
claude-code.pkg-path = "claude-code"
```

Every GitHub Release also carries a pre-built `caro-vX.Y.Z.toml` snippet
(uploaded by the `flox` job in
[`.github/workflows/packages.yml`](../.github/workflows/packages.yml))
that you can drop directly into your manifest, plus a
`caro-vX.Y.Z.sha256` with the per-system hashes a downstream catalog
submission needs.

## Activating caro's own dev env

If you want to contribute to caro itself, the project ships its own Flox
manifest:

```bash
git clone https://github.com/wildcard/caro
cd caro
flox activate
cargo check --no-default-features --features embedded-cpu
```

The manifest at [`.flox/env/manifest.toml`](../.flox/env/manifest.toml)
pins rustup, pkg-config, openssl, and the dev utilities our skills
expect (`git`, `gh`, `jq`, `ripgrep`). The same env works on every Flox
boundary in
[`.claude/rules/coder-agent-isolation.md`](../.claude/rules/coder-agent-isolation.md):
bare host, sandflox, container, full VM.

## The three-layer view

Caro doesn't compete with sandboxes. It sits beside them.

```
L3   caro                  (command safety)
L2   sandflox / container / VM   (boundary)
L1   Flox manifest         (environment)
```

This split is the whole reason caro adopts Flox: the manifest is Layer 1,
the boundary is Layer 2, caro is Layer 3, and each picks the right tool
for its concern. Read the full rule at
[`.claude/rules/coder-agent-isolation.md`](../.claude/rules/coder-agent-isolation.md).

## See also

- [flox.dev](https://flox.dev/) — the Flox project
- [github.com/flox/sandflox](https://github.com/flox/sandflox) — the
  macOS kernel sandbox referenced in the Layer-2 row
- [caro.sh/use-cases/flox](https://caro.sh/use-cases/flox) — the
  user-facing articulation of this page's content
