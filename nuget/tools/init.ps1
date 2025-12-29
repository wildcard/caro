# NuGet package initialization script for caro
# This script is run when the package is installed in Visual Studio

param($installPath, $toolsPath, $package, $project)

Write-Host "caro - Natural Language to Shell Commands"
Write-Host "==========================================="
Write-Host ""
Write-Host "The caro binary has been installed to:"
Write-Host "  $toolsPath"
Write-Host ""
Write-Host "To use caro from the command line, add this directory to your PATH,"
Write-Host "or run directly:"
Write-Host "  & `"$toolsPath\caro.exe`" `"your natural language command`""
Write-Host ""
Write-Host "Documentation: https://github.com/wildcard/caro"
