@echo off
setlocal
title LoomDB Setup
cls

echo ========================================================
echo                 LoomDB Windows Installer
echo ========================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-windows.ps1"

if %errorlevel% neq 0 (
    echo.
    echo Installation exited or encountered an error.
    pause
)
