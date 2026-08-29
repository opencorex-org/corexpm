# CorexPM Windows PowerShell Installer
# Usage: iwr -useb https://corex.dev/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Version = $env:COREX_VERSION
if (-not $Version) { $Version = "v1.0.0" }

$InstallDir = "$env:USERPROFILE\.corex\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$DownloadUrl = "https://github.com/opencorex-org/corexpm/releases/download/$Version/corexpm-windows-x64.exe"
$TargetPath = "$InstallDir\corexpm.exe"

Write-Host "Downloading CorexPM $Version for windows-x64..." -ForegroundColor Green
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TargetPath

Write-Host "`nCorexPM successfully installed to $TargetPath" -ForegroundColor Green
Write-Host "Add '$InstallDir' to your Environment PATH to run 'corexpm' from any terminal.`n" -ForegroundColor Yellow
