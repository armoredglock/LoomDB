<#
.SYNOPSIS
    LoomDB Official Windows Setup & Service Installer
.DESCRIPTION
    Provides step-by-step installation, user agreement confirmation,
    PATH configuration, and background service initialization for LoomDB on Windows.
#>

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$Host.UI.RawUI.WindowTitle = "LoomDB Windows Setup Wizard"

function Show-Header {
    Clear-Host
    Write-Host @"
========================================================================
                      🧶 LoomDB Setup Wizard
        The World's Lightest Multi-Modal, AI-Native Database
========================================================================
"@ -ForegroundColor Cyan
}

# --- STEP 1: SERVICE INFORMATION ---
Show-Header
Write-Host "STEP 1 of 5: Product & Service Information" -ForegroundColor Yellow
Write-Host @"
LoomDB is a next-generation, zero-dependency database engine designed to run 
in a single portable .loom container file.

Key Features & Service Capabilities:
  • Single-File Architecture: Entire DB (Tables, Vectors, WAL) in one file.
  • Multi-Modal Engine: SQL Relational, JSON Docs, AI Vectors, Time-Series.
  • CLI Accessibility: Access your database from any terminal using `loom`.
  • Windows Background Service: Automatic startup on port 8484 with zero idle CPU.
  • Embedded Web Studio: Zero-dependency interactive web query console.
"@
Write-Host ""
Write-Host "Press [Enter] to review the User Agreement..." -ForegroundColor Green
$null = Read-Host

# --- STEP 2: USER AGREEMENT ---
Show-Header
Write-Host "STEP 2 of 5: User License Agreement" -ForegroundColor Yellow
Write-Host @"
LoomDB is free and open-source software distributed under the dual
Apache License, Version 2.0 and the MIT License.

Terms Summary:
  1. Freedom to Use: You are free to use, modify, distribute, and embed
     LoomDB in personal, commercial, and open-source projects at zero cost.
  2. No Warranty: LoomDB is provided 'AS IS' without warranty of any kind.
  3. Attribution: Retain original copyright notices when distributing binaries.

Full legal text is available in the repository LICENSE file.
"@
Write-Host ""
$agreement = Read-Host "Do you accept these terms? (Y/N)"
if ($agreement -notmatch "^[Yy]$") {
    Write-Host "`nSetup aborted. You must accept the agreement to install LoomDB." -ForegroundColor Red
    exit 1
}

# --- STEP 3: INSTALLATION DIRECTORY ---
Show-Header
Write-Host "STEP 3 of 5: Installation Destination" -ForegroundColor Yellow
$defaultInstallDir = "$env:LOCALAPPDATA\LoomDB"
Write-Host "Default destination: $defaultInstallDir"
$customDir = Read-Host "Press [Enter] to use default, or enter custom path"

$installDir = if ([string]::IsNullOrWhiteSpace($customDir)) { $defaultInstallDir } else { $customDir }

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}
Write-Host "Selected destination: $installDir" -ForegroundColor Green

# Locate or extract loom.exe
$sourceExe = "$PSScriptRoot\..\target\debug\loom.exe"
if (-not (Test-Path $sourceExe)) {
    $sourceExe = "$PSScriptRoot\loom.exe"
}

if (Test-Path $sourceExe) {
    Copy-Item $sourceExe -Destination "$installDir\loom.exe" -Force
    Write-Host "Copied executable to $installDir\loom.exe" -ForegroundColor Green
} else {
    Write-Host "Warning: Source binary not found at $sourceExe. Please place loom.exe into $installDir." -ForegroundColor Yellow
}

# --- STEP 4: PATH ENVIRONMENT VARIABLE ---
Show-Header
Write-Host "STEP 4 of 5: Command-Line (CLI) Accessibility" -ForegroundColor Yellow
Write-Host "Adding $installDir to User PATH environment variable..."

$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -split ';' -notcontains $installDir) {
    $newPath = "$userPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$installDir"
    Write-Host "✅ PATH updated. You can now run 'loom' from any PowerShell or Command Prompt!" -ForegroundColor Green
} else {
    Write-Host "✅ Path already configured." -ForegroundColor Green
}

# --- STEP 5: WINDOWS SERVICE SETUP ---
Show-Header
Write-Host "STEP 5 of 5: Windows Background Service Option" -ForegroundColor Yellow
Write-Host "Would you like LoomDB to run automatically as a Windows background service?"
Write-Host "This allows client applications and the Web Studio to connect at 127.0.0.1:8484."
$svcChoice = Read-Host "Install as Windows Service? (Y/N)"

if ($svcChoice -match "^[Yy]$") {
    Write-Host "Registering LoomDB Windows service..." -ForegroundColor Cyan
    & "$installDir\loom.exe" service install --addr "127.0.0.1:8484"
    Write-Host "✅ Service registered." -ForegroundColor Green
} else {
    Write-Host "Skipped service installation. You can start the server manually anytime using: loom server" -ForegroundColor Gray
}

# --- FINISH AND RUN ---
Show-Header
Write-Host @"
========================================================================
                     🎉 Installation Complete!
========================================================================

LoomDB has been successfully installed on your computer.

Quick Commands:
  • loom               Launch the interactive SQL / LoomQL REPL terminal
  • loom server        Run the local standalone server with Web Studio
  • loom studio        Export the zero-dependency Web Studio UI
  • loom stats         Inspect container pages and memory consumption

"@ -ForegroundColor Green

$runNow = Read-Host "Would you like to launch the LoomDB CLI REPL now? (Y/N)"
if ($runNow -match "^[Yy]$") {
    & "$installDir\loom.exe" repl
} else {
    Write-Host "Thank you for using LoomDB! Open any terminal and type 'loom' to begin." -ForegroundColor Cyan
}
