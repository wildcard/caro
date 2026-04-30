# VT Engine Decision — caro-terminal

**Tracking:** [#1010](https://github.com/wildcard/caro/issues/1010), epic [#1008](https://github.com/wildcard/caro/issues/1008).
**Spike:** [`caro-terminal/spikes/vt-engine-comparison/`](../../caro-terminal/spikes/vt-engine-comparison/)
**Decision date:** 2026-04-30
**Decision:** **Use the [`vte`](https://crates.io/crates/vte) crate.** Defer libghostty-vt indefinitely.

## Context

Epic [#1008](https://github.com/wildcard/caro/issues/1008) originally specified `libghostty-vt` (the embeddable artifact from ghostty-org/ghostty) as the VT engine for caro-terminal. Research pass 2 of [#1009](https://github.com/wildcard/caro/issues/1009) discovered that **warp itself uses the upstream `vte` crate** rather than a custom engine — the same crate that powers alacritty and wezterm. This made `vte` a serious alternative we hadn't considered when filing the epic.

This decision document closes the question.

## Comparison

| Criterion | `vte` (chosen) | `libghostty-vt` (deferred) |
|---|---|---|
| **Language** | Pure Rust | Zig (with C ABI) |
| **Build dep on contributor's machine** | None beyond Rust toolchain | Zig toolchain + ghostty source build (no `brew install libghostty`) |
| **Add to project** | `cargo add vte` (one line) | bindgen + build.rs + vendored static lib + Zig CI step |
| **Used in production by** | alacritty, wezterm, **warp** | ghostty, cmux |
| **Maintainer activity** | High (alacritty team) | Medium (ghostty solo dev) |
| **Throughput on a realistic 1 MiB mixed stream (M-series)** | **154 MiB/s** measured (this spike) | Not measured (requires Zig install) |
| **OSC 133 support** | Full — verified by spike | Likely — used by ghostty internally |
| **API style** | Push parser + `Perform` trait callbacks | Unknown until linked; Zig→C ABI |
| **License** | Apache-2.0 / MIT | MIT |
| **Cross-platform** | Yes (no platform-specific code) | Yes via Zig cross-compilation |

## Spike results — `vte` 0.13.1

The spike at [`caro-terminal/spikes/vt-engine-comparison/`](../../caro-terminal/spikes/vt-engine-comparison/) does three things:

1. **Parses a hand-crafted stream** of two complete blocks (prompt-start → prompt → prompt-end → command → command-start → output → command-end with exit code) and verifies that OSC 133 marker detection produces exactly the expected `BlockEvent` sequence.
2. **Detects all four OSC 133 sub-codes** (A/B/C/D) including the exit-code parameter on `D`.
3. **Benchmarks a 1 MiB realistic mixed stream** containing prompts, SGR color sequences, and command output.

### Numbers from the spike (M-series, release build)

```
Stream parsing
  Bytes printed to grid: 161
  OSC sequences seen:   8     (4 per block × 2 blocks)
  SGR changes:          0     (no colors in the smoke stream)

Block events detected:
   0: PromptStart
   1: PromptEnd
   2: CommandStart
   3: CommandEnd { exit_code: Some(0) }
   4: PromptStart
   5: PromptEnd
   6: CommandStart
   7: CommandEnd { exit_code: Some(1) }

Blocks detected: 2
Exit codes:      [Some(0), Some(1)]

Throughput benchmark (1 MiB):
  Duration: 6.5 ms
  Throughput: 154.4 MiB/s
```

### Why the throughput number matters

[#1014](https://github.com/wildcard/caro/issues/1014)'s acceptance criterion specified: *"feeding 1 MB of `cat /dev/random | head -c 1M` should not exceed 50 ms parse time on M-series."*

The spike beats this budget by **~7.7×** (6.5 ms vs 50 ms). On a realistic mixed stream that includes the full overhead of OSC 133 detection, SGR dispatch, and grid line tracking — not just a microbenchmark.

For perspective: `cat` on a fast SSD produces ~1 MiB/s of human-readable output. A 154 MiB/s parser has ~100× headroom over real-world PTY rates.

### Why libghostty-vt was not measured

Multiple toolchain barriers before we can even compile a libghostty-vt spike:

1. **Zig is not installed** on the spike machine. Anyone reproducing this spike must `brew install zig` first.
2. **`libghostty` is not packaged** by Homebrew, apt, or any package manager. The `brew install ghostty` cask installs `Ghostty.app` only — no headers, no shared library.
3. **Building libghostty from ghostty source** requires `git clone ghostty-org/ghostty`, then a Zig build with build flags to expose the C ABI artifact. This is undocumented for downstream consumers; cmux does it via Swift FFI which is mac-only and not portable.
4. **CI cost:** Adding Zig to GitHub Actions adds ~30 s to every CI run for downloading the toolchain. With `vte`, CI is unchanged.

These are not blockers we can't solve. They're costs we'd pay forever, on every contributor's machine, for the lifetime of the project. The benefit case for libghostty-vt would have to be substantial to justify them.

There is no such case visible from the inventory or the comparison above.

## Decision

**Use `vte` 0.13.x.** Add to caro-terminal's main Cargo workspace as a direct dep. Implement the grid model and block parser on top of `vte::Perform` callbacks (warp's pattern, validated by this spike).

## Consequences

### For [#1010](https://github.com/wildcard/caro/issues/1010) (this spike)

Closeable. The artifact (this doc + the spike crate) answers the question.

### For [#1014](https://github.com/wildcard/caro/issues/1014) (FFI bindings + screen diff)

**Rescope.** The original AC mentioned "`libghostty-vt-sys` raw FFI" which is no longer relevant. New AC:

- Caro-terminal links `vte = "0.13"` directly.
- Implement `caro-terminal/src-tauri/src/vt.rs` as a `vte::Perform` impl that maintains a grid model + emits `BlockEvent`s.
- Screen-state diff serialization to JSON for the Tauri channel.
- Throughput benchmark already met by the spike (154 MiB/s on M-series). Re-validate after grid integration.

### For [#1015](https://github.com/wildcard/caro/issues/1015) (block parser)

The spike already implements OSC 133 A/B/C/D detection in 25 lines inside `Perform::osc_dispatch`. Lift it into the production block parser; expand to handle the edge cases (missing markers, interleaved markers, malformed parameters).

### For epic [#1008](https://github.com/wildcard/caro/issues/1008)

Update architecture diagram: replace "libghostty-vt FFI" with "vte parser" in the Rust core box.

### Future re-evaluation

Reasons to revisit `libghostty-vt`:

- Ghostty stabilizes a packaged `libghostty` (Homebrew/apt/etc.).
- A specific Ghostty feature we want is missing from `vte` (kitty graphics protocol, Sixel rendering, …).
- A real performance regression on `vte` is identified.

None of these conditions hold today.

## Reproducing the spike

```bash
cd caro-terminal/spikes/vt-engine-comparison
cargo run --release
```

Expected output: 2 blocks detected with exit codes `[Some(0), Some(1)]`, throughput in the 100–200 MiB/s range on Apple Silicon. Failure of either is a regression.

## License attribution

`vte` is dual Apache-2.0 / MIT. AGPL-3.0 + Apache-2.0/MIT is compatible (more-permissive licenses can be vendored into AGPL projects). No NOTICE entry strictly required, but adding one is good practice.
