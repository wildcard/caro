# Caro Installer for Windows
# Usage: irm https://raw.githubusercontent.com/wildcard/caro/main/install.ps1 | iex
#
# Or download and run:
#   Invoke-WebRequest -Uri https://raw.githubusercontent.com/wildcard/caro/main/install.ps1 -OutFile install.ps1
#   .\install.ps1

$ErrorActionPreference = "Stop"

# Configuration
$repo = "wildcard/caro"
$binaryName = "caro"
$installDir = "$env:LOCALAPPDATA\caro\bin"

# Colors
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

function Write-Banner {
    Write-Host ""
    Write-ColorOutput "====================================" "Cyan"
    Write-ColorOutput "      Caro Installer for Windows    " "Cyan"
    Write-ColorOutput "====================================" "Cyan"
    Write-Host ""
}

function Get-Platform {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        "X64" { return "windows-amd64" }
        "Arm64" { return "windows-arm64" }
        default {
            Write-ColorOutput "Unsupported architecture: $arch" "Red"
            exit 1
        }
    }
}

function Get-LatestVersion {
    Write-ColorOutput "Fetching latest version..." "Blue"

    try {
        $releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
        $release = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing
        $version = $release.tag_name -replace '^v', ''
        Write-ColorOutput "Latest version: $version" "Green"
        return $version
    }
    catch {
        Write-ColorOutput "Failed to fetch latest version: $_" "Red"
        exit 1
    }
}

function Test-ExistingInstallation {
    $existingPath = Get-Command $binaryName -ErrorAction SilentlyContinue
    if ($existingPath) {
        $existingVersion = & $binaryName --version 2>&1 | Select-Object -First 1
        Write-ColorOutput "Found existing installation:" "Yellow"
        Write-ColorOutput "  Path: $($existingPath.Source)" "Yellow"
        Write-ColorOutput "  Version: $existingVersion" "Yellow"
        return $true
    }
    return $false
}

function Install-Caro {
    param([string]$Version)

    $platform = Get-Platform

    # Create install directory
    if (!(Test-Path $installDir)) {
        Write-ColorOutput "Creating install directory: $installDir" "Blue"
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    }

    # Construct download URL
    $assetName = "caro-$Version-$platform.exe"
    $downloadUrl = "https://github.com/$repo/releases/download/v$Version/$assetName"
    $checksumUrl = "$downloadUrl.sha256"
    $targetPath = Join-Path $installDir "caro.exe"

    Write-ColorOutput "Downloading caro v$Version for $platform..." "Blue"
    Write-ColorOutput "  URL: $downloadUrl" "Gray"

    try {
        # Download binary
        $tempFile = [System.IO.Path]::GetTempFileName()
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing

        # Download and verify checksum
        try {
            $checksumContent = (Invoke-WebRequest -Uri $checksumUrl -UseBasicParsing).Content
            $expectedHash = $checksumContent.Split()[0].ToLower()
            $actualHash = (Get-FileHash -Path $tempFile -Algorithm SHA256).Hash.ToLower()

            if ($expectedHash -eq $actualHash) {
                Write-ColorOutput "Checksum verified!" "Green"
            }
            else {
                Write-ColorOutput "Warning: Checksum mismatch" "Yellow"
                Write-ColorOutput "  Expected: $expectedHash" "Yellow"
                Write-ColorOutput "  Actual:   $actualHash" "Yellow"
            }
        }
        catch {
            Write-ColorOutput "Could not verify checksum (file may not exist for this release)" "Yellow"
        }

        # Move to final location
        Move-Item -Path $tempFile -Destination $targetPath -Force

        Write-ColorOutput "Installed to: $targetPath" "Green"
    }
    catch {
        Write-ColorOutput "Download failed: $_" "Red"
        if (Test-Path $tempFile) { Remove-Item $tempFile -Force }
        exit 1
    }
}

function Add-ToPath {
    # Check if already in PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")

    if ($userPath -like "*$installDir*") {
        Write-ColorOutput "Install directory already in PATH" "Green"
        return
    }

    Write-ColorOutput "Adding $installDir to user PATH..." "Blue"

    try {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        Write-ColorOutput "PATH updated successfully" "Green"

        # Update current session
        $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")
    }
    catch {
        Write-ColorOutput "Failed to update PATH automatically" "Yellow"
        Write-ColorOutput "Please add this directory to your PATH manually:" "Yellow"
        Write-ColorOutput "  $installDir" "Cyan"
    }
}

function Test-Installation {
    Write-ColorOutput "Verifying installation..." "Blue"

    $caroPath = Join-Path $installDir "caro.exe"

    if (Test-Path $caroPath) {
        try {
            $version = & $caroPath --version 2>&1 | Select-Object -First 1
            Write-ColorOutput "Installation successful!" "Green"
            Write-ColorOutput "  $version" "Green"
            return $true
        }
        catch {
            Write-ColorOutput "Binary exists but failed to execute: $_" "Red"
            return $false
        }
    }

    Write-ColorOutput "Installation verification failed" "Red"
    return $false
}

function Show-NextSteps {
    Write-Host ""
    Write-ColorOutput "====================================" "Green"
    Write-ColorOutput "     Installation Complete!         " "Green"
    Write-ColorOutput "====================================" "Green"
    Write-Host ""

    Write-ColorOutput "Next steps:" "Cyan"
    Write-Host ""

    Write-ColorOutput "1. Open a NEW PowerShell window (to refresh PATH)" "Yellow"
    Write-Host ""

    Write-ColorOutput "2. Try it out:" "Yellow"
    Write-ColorOutput '   caro "list all files in this directory"' "Green"
    Write-Host ""

    Write-ColorOutput "3. Get help:" "Yellow"
    Write-ColorOutput "   caro --help" "Green"
    Write-Host ""

    Write-ColorOutput "Documentation:" "Cyan"
    Write-ColorOutput "   https://caro.sh" "Blue"
    Write-ColorOutput "   https://github.com/$repo" "Blue"
    Write-Host ""

    Write-ColorOutput "Setup Guide:" "Cyan"
    Write-ColorOutput "   https://github.com/$repo/blob/main/docs/WINDOWS_SETUP.md" "Blue"
    Write-Host ""

    # Tip about shell detection
    Write-ColorOutput "Tip: Caro auto-detects PowerShell and generates native commands." "Magenta"
    Write-ColorOutput "     Use --shell to force a different shell (bash, cmd, powershell)" "Magenta"
    Write-Host ""
}

# Main installation flow
function Main {
    Write-Banner

    # Check for existing installation
    $hasExisting = Test-ExistingInstallation

    if ($hasExisting) {
        $response = Read-Host "Would you like to reinstall/update? (Y/n)"
        if ($response -match '^[Nn]') {
            Write-ColorOutput "Installation cancelled" "Yellow"
            exit 0
        }
    }

    # Get latest version
    $version = Get-LatestVersion

    # Install
    Install-Caro -Version $version

    # Add to PATH
    Add-ToPath

    # Verify
    if (Test-Installation) {
        Show-NextSteps
    }
    else {
        Write-ColorOutput "Installation may have issues. Please check the setup guide:" "Yellow"
        Write-ColorOutput "   https://github.com/$repo/blob/main/docs/WINDOWS_SETUP.md" "Blue"
        exit 1
    }
}

# Run main
Main
