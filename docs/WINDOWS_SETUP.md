# Windows Setup Guide

This guide covers installing and using Caro on Windows. Caro is designed to generate **native PowerShell commands** when running on Windows, giving you a seamless experience without requiring WSL.

## Quick Install (Recommended)

### PowerShell Install Script

Run this in PowerShell (as Administrator recommended):

```powershell
# Download and run the installer
irm https://raw.githubusercontent.com/wildcard/caro/main/install.ps1 | iex
```

This will:
- Download the latest Windows binary
- Install to `%LOCALAPPDATA%\caro\bin`
- Add to your user PATH
- Verify the installation

### Manual Installation

If you prefer manual installation or the script doesn't work:

#### Step 1: Download the Binary

Download from [GitHub Releases](https://github.com/wildcard/caro/releases/latest):

```powershell
# Create installation directory
$installDir = "$env:LOCALAPPDATA\caro\bin"
New-Item -ItemType Directory -Force -Path $installDir

# Download latest release (update version as needed)
$version = "1.1.3"
$url = "https://github.com/wildcard/caro/releases/download/v$version/caro-$version-windows-amd64.exe"
Invoke-WebRequest -Uri $url -OutFile "$installDir\caro.exe"
```

#### Step 2: Add to PATH

**Option A: User PATH (no admin required)**
```powershell
# Add to user PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "Added $installDir to user PATH"
}

# Refresh current session
$env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")
```

**Option B: System PATH (requires admin)**
```powershell
# Run PowerShell as Administrator
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($machinePath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$machinePath;$installDir", "Machine")
    Write-Host "Added $installDir to system PATH"
}
```

#### Step 3: Verify Installation

Open a **new** PowerShell window and run:

```powershell
caro --version
# Should output: caro 1.1.3 (...)
```

## Standard Installation Paths

Caro follows Windows conventions for installation:

| Path | Description |
|------|-------------|
| `%LOCALAPPDATA%\caro\bin` | **Recommended** - User-local installation |
| `%USERPROFILE%\.local\bin` | Unix-style path (also supported) |
| `%ProgramFiles%\caro` | System-wide (requires admin) |

The installer uses `%LOCALAPPDATA%\caro\bin` by default, which is:
- `C:\Users\<YourUsername>\AppData\Local\caro\bin`

## Shell Detection

Caro automatically detects your shell and generates appropriate commands:

| Shell | Generated Commands |
|-------|-------------------|
| PowerShell | Native PowerShell cmdlets (`Get-ChildItem`, `Remove-Item`, etc.) |
| CMD | Windows CMD commands (`dir`, `del`, etc.) |
| Git Bash / MSYS2 | POSIX commands (`ls`, `rm`, etc.) |
| WSL | POSIX commands for Linux |

### Example: Same Request, Different Shells

```
User: "list all files in current directory"
```

**PowerShell output:**
```powershell
Get-ChildItem -Force
```

**CMD output:**
```cmd
dir /a
```

**WSL/Git Bash output:**
```bash
ls -la
```

## Using Caro on Windows

### Basic Usage

```powershell
# Generate a command
caro "list all PDF files larger than 10MB"

# Example output (PowerShell):
#   Get-ChildItem -Recurse -Filter "*.pdf" | Where-Object { $_.Length -gt 10MB }
#
# Execute this command? (Y)es / (n)o / (e)dit: y
```

### Force a Specific Shell

If you want commands for a different shell:

```powershell
# Generate bash commands (for use in WSL/Git Bash)
caro --shell bash "find large log files"

# Generate CMD commands
caro --shell cmd "delete temporary files"

# Generate PowerShell commands (default on Windows)
caro --shell powershell "compress folder to zip"
```

### Configuration

Set your preferences:

```powershell
# Set default shell explicitly
caro config set shell powershell

# View current configuration
caro config show

# Set safety level
caro config set safety strict
```

Config file location: `%APPDATA%\caro\config.toml`

## Troubleshooting

### "caro is not recognized as a command"

The binary is not in your PATH. Either:

1. **Run from the download location:**
   ```powershell
   & "C:\Users\YourName\Downloads\caro-1.1.3-windows-amd64.exe" --version
   ```

2. **Add to PATH** (see Step 2 above)

3. **Reinstall using the install script**

### Commands fail with "WSL has no installed distributions"

This happens when Caro generates POSIX commands (`ls`, `rm`, etc.) on native Windows. Solutions:

1. **Force PowerShell commands:**
   ```powershell
   caro --shell powershell "list files"
   ```

2. **Set PowerShell as default:**
   ```powershell
   caro config set shell powershell
   ```

3. **Install WSL** if you prefer POSIX commands:
   ```powershell
   wsl --install
   ```

### Checksum Verification

Verify the download integrity:

```powershell
$version = "1.1.3"
$binary = "$env:LOCALAPPDATA\caro\bin\caro.exe"
$checksumUrl = "https://github.com/wildcard/caro/releases/download/v$version/caro-$version-windows-amd64.exe.sha256"

# Download expected checksum
$expected = (Invoke-WebRequest -Uri $checksumUrl).Content.Split()[0]

# Calculate actual checksum
$actual = (Get-FileHash -Path $binary -Algorithm SHA256).Hash.ToLower()

if ($expected -eq $actual) {
    Write-Host "Checksum verified!" -ForegroundColor Green
} else {
    Write-Host "Checksum mismatch!" -ForegroundColor Red
}
```

### Antivirus False Positives

Some antivirus software may flag the binary. This is a false positive. You can:

1. Add an exception for `%LOCALAPPDATA%\caro\bin\caro.exe`
2. Download from [GitHub Releases](https://github.com/wildcard/caro/releases) and verify the checksum

## Using Caro with WSL

If you primarily work in WSL, you have two options:

### Option 1: Install Caro Inside WSL (Recommended)

```bash
# Inside WSL terminal
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
```

This gives you native Linux Caro that generates POSIX commands.

### Option 2: Use Windows Caro with WSL Shell

```powershell
# Force bash shell output from Windows Caro
caro --shell bash "find files modified today"
```

Then copy the command into your WSL terminal.

## Using Caro with Git Bash / MSYS2

Caro auto-detects Git Bash and generates appropriate commands:

```bash
# In Git Bash
caro "list hidden files"
# Output: ls -la
```

## Uninstallation

### Remove Binary and Config

```powershell
# Remove binary
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\caro"

# Remove configuration
Remove-Item -Recurse -Force "$env:APPDATA\caro"

# Remove from PATH (user)
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$newPath = ($userPath -split ';' | Where-Object { $_ -notlike "*caro*" }) -join ';'
[Environment]::SetEnvironmentVariable("Path", $newPath, "User")
```

### If Installed via Cargo

```powershell
cargo uninstall caro
```

## Building from Source

### Prerequisites

1. **Install Rust:** Download from [rustup.rs](https://rustup.rs)
2. **Install Visual Studio Build Tools:** Required for compiling native dependencies
   - Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Select "Desktop development with C++"

### Build

```powershell
# Clone repository
git clone https://github.com/wildcard/caro.git
cd caro

# Build release binary
cargo build --release

# Binary location
.\target\release\caro.exe --version

# Optional: Install to cargo bin
cargo install --path .
```

## Terminal Recommendations

For the best Caro experience on Windows, we recommend:

1. **Windows Terminal** - Modern terminal with tabs, themes, and excellent Unicode support
2. **PowerShell 7+** - Cross-platform PowerShell with improved cmdlets
3. **VS Code Integrated Terminal** - Great for development workflows

## Getting Help

- **Documentation:** [caro.sh](https://caro.sh)
- **Issues:** [GitHub Issues](https://github.com/wildcard/caro/issues)
- **Discussions:** [GitHub Discussions](https://github.com/wildcard/caro/discussions)

---

**Last Updated:** 2026-01-17
