@echo off
setlocal
cd /d "%~dp0"
title COD Limassol - Blackjack Research
if exist "artifacts\windows\cod-research.exe" (
    "artifacts\windows\cod-research.exe" interactive
) else if exist "target\release\cod-research.exe" (
    "target\release\cod-research.exe" interactive
) else (
    echo The Windows app has not been downloaded or built yet.
    echo See docs\interactive-cli.md for setup instructions.
    pause
    exit /b 1
)
if errorlevel 1 (
    echo.
    echo The app could not finish. Please copy the message above for troubleshooting.
    pause
)
