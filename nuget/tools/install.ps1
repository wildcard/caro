# Download and install caro binary for Windows
param(
    [string]$Version = "1.0.2",
    [string]$InstallPath = $PSScriptRoot
)

$ErrorActionPreference = "Stop"

$repo = "wildcard/caro"
$assetName = "caro-windows-amd64.exe"
$downloadUrl = "https://github.com/$repo/releases/download/v$Version/$assetName"
$destinationPath = Join-Path $InstallPath "caro.exe"

Write-Host "Downloading caro v$Version for Windows..."
Write-Host "URL: $downloadUrl"

try {
    # Use TLS 1.2
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    # Download the binary
    $webClient = New-Object System.Net.WebClient
    $webClient.DownloadFile($downloadUrl, $destinationPath)

    Write-Host "Successfully installed caro to: $destinationPath"

    # Verify the binary
    $version = & $destinationPath --version 2>&1
    Write-Host "Verified: $version"
}
catch {
    Write-Error "Failed to install caro: $_"
    Write-Host ""
    Write-Host "You can download manually from:"
    Write-Host "  https://github.com/$repo/releases/latest"
    exit 1
}
