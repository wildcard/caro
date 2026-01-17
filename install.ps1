#Requires -Version 5.1
<#
.SYNOPSIS
    Caro installer for Windows
.DESCRIPTION
    Downloads and installs the latest Caro binary for Windows.
    Usage: irm https://raw.githubusercontent.com/wildcard/caro/main/install.ps1 | iex

    Options (set before running):
    $env:CARO_NO_MODIFY_PATH = "1"  # Skip automatic PATH modification
#>

$ErrorActionPreference = "Stop"

# Configuration
$repo = "wildcard/caro"
$binaryName = "caro.exe"
$installDir = Join-Path $env:USERPROFILE ".local\bin"

function Write-Status {
    param([string]$Message, [string]$Type = "info")
    switch ($Type) {
        "success" { Write-Host "`u{2714} $Message" -ForegroundColor Green }
        "warning" { Write-Host "`u{26A0} $Message" -ForegroundColor Yellow }
        "error"   { Write-Host "`u{2718} $Message" -ForegroundColor Red }
        default   { Write-Host "  $Message" }
    }
}

function Get-LatestVersion {
    $releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing
        return $release.tag_name -replace '^v', ''
    } catch {
        throw "Failed to fetch latest release info from GitHub: $_"
    }
}

function Add-ToUserPath {
    param([string]$PathToAdd)

    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")

    # Check if already in PATH
    $pathArray = $currentUserPath -split ';' | Where-Object { $_ -ne '' }
    if ($pathArray -contains $PathToAdd) {
        return $false  # Already in PATH
    }

    # Add to PATH
    $newPath = if ($currentUserPath) { "$currentUserPath;$PathToAdd" } else { $PathToAdd }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")

    # Also update current session
    $env:Path = "$env:Path;$PathToAdd"

    return $true  # Successfully added
}

function Install-Caro {
    Write-Host ""
    Write-Host "Setting up Caro..." -ForegroundColor Cyan
    Write-Host ""

    # Get latest version
    $version = Get-LatestVersion

    # Construct download URL (try versioned name first, fall back to legacy)
    $versionedAsset = "caro-$version-windows-amd64.exe"
    $legacyAsset = "caro-windows-amd64.exe"
    $downloadUrl = "https://github.com/$repo/releases/download/v$version/$versionedAsset"

    # Create install directory if it doesn't exist
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    $installPath = Join-Path $installDir $binaryName

    # Download binary
    Write-Host "  Downloading Caro v$version..." -ForegroundColor Gray
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $installPath -UseBasicParsing
    } catch {
        # Try legacy asset name
        Write-Host "  Trying alternate download URL..." -ForegroundColor Gray
        $downloadUrl = "https://github.com/$repo/releases/download/v$version/$legacyAsset"
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $installPath -UseBasicParsing
        } catch {
            throw "Failed to download Caro: $_"
        }
    }

    Write-Host ""
    Write-Status "Caro successfully installed!" "success"
    Write-Host ""

    # Get installed version
    try {
        $installedVersion = & $installPath --version 2>$null
        if ($installedVersion) {
            Write-Host "  Version: $installedVersion" -ForegroundColor White
        }
    } catch {
        Write-Host "  Version: $version" -ForegroundColor White
    }

    Write-Host ""
    Write-Host "  Location: $installPath" -ForegroundColor White
    Write-Host ""
    Write-Host ""
    Write-Host "  Next: Run " -NoNewline
    Write-Host "caro --help" -ForegroundColor Cyan -NoNewline
    Write-Host " to get started"
    Write-Host ""

    # Check if install directory is in PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")

    $inPath = $false
    if ($userPath -split ';' | Where-Object { $_ -eq $installDir }) { $inPath = $true }
    if ($machinePath -split ';' | Where-Object { $_ -eq $installDir }) { $inPath = $true }

    if (-not $inPath) {
        # Check if user wants to skip PATH modification
        if ($env:CARO_NO_MODIFY_PATH -eq "1") {
            Write-Status "Setup notes:" "warning"
            Write-Host "  * " -NoNewline
            Write-Host $installDir -ForegroundColor Yellow -NoNewline
            Write-Host " is not in your PATH (skipped by CARO_NO_MODIFY_PATH)."
            Write-Host ""
            Write-Host "  To add manually, run:" -ForegroundColor Gray
            Write-Host "  " -NoNewline
            Write-Host "[Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$installDir', 'User')" -ForegroundColor Cyan
            Write-Host ""
        } else {
            # Automatically add to PATH
            Write-Host "  Adding to PATH..." -ForegroundColor Gray
            $added = Add-ToUserPath -PathToAdd $installDir
            if ($added) {
                Write-Status "Added $installDir to your PATH" "success"
                Write-Host ""
                Write-Host "  " -NoNewline
                Write-Host "Note:" -ForegroundColor Yellow -NoNewline
                Write-Host " Restart your terminal for PATH changes to take effect,"
                Write-Host "        or run: " -NoNewline
                Write-Host "`$env:Path += ';$installDir'" -ForegroundColor Cyan
                Write-Host ""
            } else {
                Write-Host "  (Already in PATH)" -ForegroundColor Gray
                Write-Host ""
            }
        }
    }

    Write-Status "Installation complete!" "success"
    Write-Host ""
}

# Run installer
try {
    Install-Caro
} catch {
    Write-Host ""
    Write-Status "Installation failed: $_" "error"
    Write-Host ""
    Write-Host "  Manual installation:" -ForegroundColor Yellow
    Write-Host "  1. Download from: https://github.com/$repo/releases/latest"
    Write-Host "  2. Place the .exe in a directory in your PATH"
    Write-Host ""
    exit 1
}
