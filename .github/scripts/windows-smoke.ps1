# Windows runtime smoke test — guards against the field-confirmed
# regressions in caro-1.3.0-windows-amd64.exe (PR #1043).
#
# Mode (passed via -Mode):
#   "fast"   — assertions that don't require model load (default).
#              Suitable for ci.yml on every PR push. Tests Bug #1
#              (stdin hang) via --show-config and Bug #3 (shell
#              mis-detection) via the show-config output.
#   "full"   — adds the Bug #2 assertion (no POSIX leak from the
#              static matcher). Requires the embedded model to be
#              loaded; appropriate for release.yml where slow runs
#              are acceptable, or for ci.yml once model caching is
#              wired up. Bug #2 is also covered by unit tests in
#              src/backends/static_matcher.rs (run by Unit Tests
#              (windows-latest)) — this is the runtime-pipeline
#              version of the same invariant.
#
# Bug #1 (stdin hang) is exercised by `--show-config`, which the fix
# explicitly handles BEFORE the resolve_prompt stdin read (see
# src/main.rs:3312 "Handle --show-config BEFORE any stdin read").
# If the binary blocks on --show-config with inherited stdin, the
# Bug #1 fix has regressed at the configuration code path.
#
# Single source of truth: called from both `ci.yml` (PR-gating,
# every push, Mode=fast) and `release.yml` (release-gating,
# post-build, Mode=full). Edits propagate to both.
#
# Usage:
#   pwsh ./.github/scripts/windows-smoke.ps1 -ExePath <path> [-Mode fast|full]
param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath,

    [ValidateSet("fast", "full")]
    [string]$Mode = "fast"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $ExePath)) {
    throw "binary missing: $ExePath"
}

# --no-telemetry on every invocation: skips the first-run consent
# prompt at src/main.rs:3388 (a separate interactive stdin reader,
# unrelated to Bug #1's resolve_prompt hang). Without this the
# prompt eats the timeout budget on a fresh runner.

Write-Host "::group::Smoke 1 — Bug #1: --show-config must not hang on inherited stdin"
# --show-config is one of the canonical hang scenarios the PR author
# cited. After the fix it must return promptly because show_config is
# handled BEFORE the stdin read (see src/main.rs:3308-3323). Cap at
# 15 s — show_config does ConfigManager::load() + a few println!s.
$p = Start-Process -FilePath $ExePath `
       -ArgumentList @('--no-telemetry', '--show-config') `
       -PassThru -NoNewWindow `
       -RedirectStandardOutput out1.txt -RedirectStandardError err1.txt
if (-not $p.WaitForExit(15000)) {
    try { $p.Kill() } catch {}
    Write-Host "----- captured stdout -----"
    Get-Content out1.txt -ErrorAction SilentlyContinue | Write-Host
    Write-Host "----- captured stderr -----"
    Get-Content err1.txt -ErrorAction SilentlyContinue | Write-Host
    throw "BUG #1 REGRESSION: caro --show-config hung for >15 s with inherited stdin (canonical Bug #1 case per PR description)"
}
if ($p.ExitCode -ne 0) {
    Write-Host "----- captured stderr -----"
    Get-Content err1.txt -ErrorAction SilentlyContinue | Write-Host
    throw "caro --show-config exited $($p.ExitCode); expected 0"
}
$out1 = Get-Content out1.txt -Raw
Write-Host $out1
Write-Host "  ok (exit=0, $((Get-Item out1.txt).Length) bytes stdout, completed in $([int]$p.TotalProcessorTime.TotalMilliseconds) ms cpu)"
Write-Host "::endgroup::"

Write-Host "::group::Smoke 3 — Bug #3: default shell label must not be Bash on Windows"
# The same --show-config output from Smoke 1 dumps the resolved
# default shell (see show_configuration() at src/main.rs:4135 —
# "Default shell: {ShellType:?}"). Bug #3 was that CliConfig::default
# hardcoded ShellType::Bash; after the fix it must auto-detect to
# PowerShell or Cmd on a Windows host.
if ($out1 -match 'Default shell:\s*Bash') {
    throw "BUG #3 REGRESSION: shell labelled 'Bash' on Windows host. --show-config output above."
}
if ($out1 -notmatch 'Default shell:\s*(PowerShell|Cmd)') {
    throw "Default shell line missing or unexpected in --show-config output. Expected PowerShell or Cmd on Windows; got: $out1"
}
Write-Host "  ok (Default shell line: $(($out1 -split "`n" | Select-String 'Default shell:').Line.Trim()))"
Write-Host "::endgroup::"

if ($Mode -eq "full") {
    Write-Host "::group::Smoke 2 (full mode) — Bug #2: no POSIX commands on --shell powershell"
    # Bug #2: static matcher emitted POSIX commands on Windows. After
    # the fix, matcher returns BackendUnavailable on ProfileType::Windows,
    # so the embedded LLM takes over. This requires the model to be
    # loaded — slow on a fresh runner without model caching. The unit
    # test in src/backends/static_matcher.rs covers the matcher
    # invariant deterministically; this assertion validates the
    # end-to-end produced command on the actual binary.
    $out2 = "" | & $ExePath --no-telemetry --shell powershell -p "list files in current directory" --dry-run
    $out2 | Write-Host
    if ($out2 -match '\bls\s+-la\b' `
        -or $out2 -match 'find\s+\.\s+-exec' `
        -or $out2 -match '\bgrep\s+-r\b' `
        -or $out2 -match "awk\s+'\{") {
        throw "BUG #2 REGRESSION: POSIX command emitted on Windows: $out2"
    }
    Write-Host "  ok (no POSIX leak in --shell powershell output)"
    Write-Host "::endgroup::"
} else {
    Write-Host "Smoke 2 (Bug #2, no POSIX leak) skipped in fast mode — covered by Unit Tests (windows-latest) in src/backends/static_matcher.rs. Use -Mode full in release.yml for end-to-end LLM validation."
}

Write-Host "Windows runtime smoke passed (mode: $Mode)."
