# Windows runtime smoke test — guards against three field-confirmed
# regressions in caro-1.3.0-windows-amd64.exe (PR #1043):
#   1. stdin hang on launch when stdin is inherited (not a console handle)
#   2. POSIX commands emitted on native Windows (--shell powershell)
#   3. Shell mis-detection ("shell: Bash" reported on a Windows host)
#
# Each assertion would have blocked v1.3.0. Compile-time `cargo test` cannot
# catch these — they only manifest when the produced `caro.exe` is invoked
# as a subprocess.
#
# Single source of truth: called from both `ci.yml` (PR-gating, every push)
# and `release.yml` (release-gating, post-build). Edits here propagate to
# both workflows automatically.
#
# Usage:
#   pwsh ./.github/scripts/windows-smoke.ps1 -ExePath <path-to-caro.exe>
param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $ExePath)) {
    throw "binary missing: $ExePath"
}

# All smoke invocations pass --no-telemetry to skip the first-run
# telemetry consent prompt (a separate interactive stdin reader at
# src/main.rs:3388 that would otherwise wait for y/n input and is NOT
# what this smoke is here to test). Bug #1 is specifically about the
# resolve_prompt stdin read, which --no-telemetry leaves untouched.

Write-Host "::group::Smoke 1 — must not hang when stdin is inherited"
# Launch caro with the parent (PowerShell) stdin handle inherited —
# same shape as a real user typing at the prompt. Bug #1 caused
# resolve_prompt to call read_to_string on inherited stdin and block
# forever waiting for EOF. The new should_consult_stdin predicate must
# now skip the read because -p sets the prompt flag. Cap at 30 s.
$p = Start-Process -FilePath $ExePath `
       -ArgumentList @('--no-telemetry', '-p', 'list files', '--dry-run') `
       -PassThru -NoNewWindow `
       -RedirectStandardOutput out1.txt -RedirectStandardError err1.txt
if (-not $p.WaitForExit(30000)) {
    try { $p.Kill() } catch {}
    Get-Content out1.txt, err1.txt -ErrorAction SilentlyContinue | Write-Host
    throw "BUG #1 REGRESSION: caro hung for >30 s with inherited stdin"
}
$outSize = (Get-Item out1.txt).Length
Write-Host "  ok (exit=$($p.ExitCode), $outSize bytes stdout)"
Write-Host "::endgroup::"

Write-Host "::group::Smoke 2 — must not emit POSIX commands on Windows"
# PowerShell 7 (runner default) rejects `< $null` ("The '<' operator is
# reserved for future use"). Pipe an empty string instead so the child
# inherits a closed-on-EOF stdin without invoking the parser-reserved
# `<` operator. Equivalent semantics for caro: stdin reads return EOF
# immediately, the should_consult_stdin predicate decides whether to
# consume it based on flag/trailing-args presence.
$out2 = "" | & $ExePath --no-telemetry --shell powershell -p "list files in current directory" --dry-run
$out2 | Write-Host
if ($out2 -match '\bls\s+-la\b' `
    -or $out2 -match 'find\s+\.\s+-exec' `
    -or $out2 -match '\bgrep\s+-r\b' `
    -or $out2 -match "awk\s+'\{") {
    throw "BUG #2 REGRESSION: POSIX command emitted on Windows: $out2"
}
Write-Host "  ok (no POSIX leak)"
Write-Host "::endgroup::"

Write-Host "::group::Smoke 3 — must not label shell as Bash on Windows"
$out3 = "" | & $ExePath --no-telemetry -p "list files" --dry-run
$out3 | Write-Host
if ($out3 -match 'shell:\s*Bash') {
    throw "BUG #3 REGRESSION: shell labelled 'Bash' on Windows host"
}
Write-Host "  ok"
Write-Host "::endgroup::"

Write-Host "All three Windows runtime smoke assertions passed."
