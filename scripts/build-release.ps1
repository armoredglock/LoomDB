<#
.SYNOPSIS
    Build and package LoomDB distribution release archives.
#>

$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot\.."
$dist = "$root\dist"
$targetExe = "$root\target\debug\loom.exe"

Write-Host "Building release package..." -ForegroundColor Cyan

# Ensure loom.exe is built
if (-not (Test-Path $targetExe)) {
    Set-Location $root
    cargo build -p loom-cli --release
    $targetExe = "$root\target\release\loom.exe"
}

# Create staging folder for Windows distribution
$stage = "$dist\loom-windows-x64"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage -Force | Out-Null

Copy-Item $targetExe "$stage\loom.exe"
Copy-Item "$root\scripts\install-windows.ps1" "$stage\install-windows.ps1"
Copy-Item "$root\scripts\loom-setup.bat" "$stage\loom-setup.bat"
Copy-Item "$root\studio.html" "$stage\studio.html"

# Readme text
@"
========================================================================
                      🧶 LoomDB v0.1.0 (Windows)
========================================================================

QUICK INSTALLATION:
  Double-click 'loom-setup.bat' or run PowerShell as Administrator:
      powershell -ExecutionPolicy Bypass -File .\install-windows.ps1

PORTABLE CLI USAGE:
  You can run 'loom.exe' directly without installation:
      .\loom.exe repl           # Start interactive terminal REPL
      .\loom.exe server         # Run server & studio on port 8484
      .\loom.exe stats          # Inspect single-file .loom container

Open Source Apache-2.0 / MIT License.
https://github.com/loomdb/loomdb
"@ | Out-File -FilePath "$stage\README.txt" -Encoding utf8

# Create zip archive
$zipPath = "$dist\loom-v0.1.0-windows-x64.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$stage\*" -DestinationPath $zipPath

Write-Host "✅ Created Windows Release: $zipPath" -ForegroundColor Green
