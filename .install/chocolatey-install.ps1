# Chocolatey install script for cmdai
$ErrorActionPreference = 'Stop'

$packageName = 'cmdai'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$version = '0.1.0'
$url64 = "https://github.com/wildcard/cmdai/releases/download/v$version/cmdai-windows-amd64.exe"
$checksum64 = 'PLACEHOLDER_CHECKSUM'
$checksumType64 = 'sha256'

$packageArgs = @{
  packageName   = $packageName
  fileType      = 'exe'
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64= $checksumType64
  validExitCodes= @(0)
  silentArgs    = ''
}

# Download and place the executable
$exePath = Join-Path $toolsDir 'cmdai.exe'
Get-ChocolateyWebFile @packageArgs -FileFullPath $exePath

# Create shim
Install-ChocolateyPath $toolsDir 'Machine'

Write-Host "cmdai has been installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Get started with:" -ForegroundColor Cyan
Write-Host "  cmdai --help" -ForegroundColor Yellow
Write-Host ""
Write-Host "Configuration file location:" -ForegroundColor Cyan
Write-Host "  $env:USERPROFILE\.config\cmdai\config.toml" -ForegroundColor Yellow
