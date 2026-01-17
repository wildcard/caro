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

    try {
        $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")

        # Check if already in PATH
        $pathArray = $currentUserPath -split ';' | Where-Object { $_ -ne '' }
        if ($pathArray -contains $PathToAdd) {
            return @{ Success = $true; AlreadyExists = $true }
        }

        # Add to PATH
        $newPath = if ($currentUserPath) { "$currentUserPath;$PathToAdd" } else { $PathToAdd }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")

        # Verify it was actually added
        $verifyPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($verifyPath -split ';' | Where-Object { $_ -eq $PathToAdd }) {
            # Also update current session
            $env:Path = "$env:Path;$PathToAdd"
            return @{ Success = $true; AlreadyExists = $false }
        } else {
            return @{ Success = $false; AlreadyExists = $false }
        }
    } catch {
        return @{ Success = $false; AlreadyExists = $false; Error = $_.Exception.Message }
    }
}

function Show-ManualPathInstructions {
    param([string]$PathToAdd)

    Write-Status "PATH setup required:" "warning"
    Write-Host ""
    Write-Host "  " -NoNewline
    Write-Host $PathToAdd -ForegroundColor Yellow -NoNewline
    Write-Host " needs to be added to your PATH."
    Write-Host ""
    Write-Host "  " -NoNewline
    Write-Host "Option 1: " -ForegroundColor Cyan -NoNewline
    Write-Host "Run this command (then restart terminal):"
    Write-Host ""
    Write-Host "    [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$PathToAdd', 'User')" -ForegroundColor Green
    Write-Host ""
    Write-Host "  " -NoNewline
    Write-Host "Option 2: " -ForegroundColor Cyan -NoNewline
    Write-Host "Manual setup via Settings:"
    Write-Host "    1. Press " -NoNewline
    Write-Host "Win + I" -ForegroundColor Yellow -NoNewline
    Write-Host " to open Settings"
    Write-Host "    2. Go to: " -NoNewline
    Write-Host "System > About > Advanced system settings" -ForegroundColor Yellow
    Write-Host "    3. Click: " -NoNewline
    Write-Host "Environment Variables" -ForegroundColor Yellow
    Write-Host "    4. Under 'User variables', select " -NoNewline
    Write-Host "Path" -ForegroundColor Yellow -NoNewline
    Write-Host " and click " -NoNewline
    Write-Host "Edit" -ForegroundColor Yellow
    Write-Host "    5. Click " -NoNewline
    Write-Host "New" -ForegroundColor Yellow -NoNewline
    Write-Host " and add: " -NoNewline
    Write-Host $PathToAdd -ForegroundColor Green
    Write-Host "    6. Click OK, then restart your terminal"
    Write-Host ""
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
            Show-ManualPathInstructions -PathToAdd $installDir
        } else {
            # Automatically add to PATH
            Write-Host "  Adding to PATH..." -ForegroundColor Gray
            $result = Add-ToUserPath -PathToAdd $installDir

            if ($result.Success -and -not $result.AlreadyExists) {
                Write-Status "Added $installDir to your PATH" "success"
                Write-Host ""
                Write-Host "  " -NoNewline
                Write-Host "Note:" -ForegroundColor Yellow -NoNewline
                Write-Host " Restart your terminal for PATH changes to take effect,"
                Write-Host "        or run: " -NoNewline
                Write-Host "`$env:Path += ';$installDir'" -ForegroundColor Cyan
                Write-Host ""
            } elseif ($result.AlreadyExists) {
                Write-Host "  (Already in PATH)" -ForegroundColor Gray
                Write-Host ""
            } else {
                # Auto-add failed, show manual instructions
                Write-Host ""
                Show-ManualPathInstructions -PathToAdd $installDir
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
