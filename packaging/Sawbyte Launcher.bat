@echo off
rem ====================================================================
rem   Sawbyte Native Desktop Launcher + Companion CLI Terminal (Windows)
rem ====================================================================
cd /d "%~dp0"
set "PATH=%~dp0;%USERPROFILE%\.sawbyte\bin;%LOCALAPPDATA%\Programs\Sawbyte;%PATH%"
title Sawbyte Desktop Launcher ^& Interactive CLI Companion

if exist "sawbyte-launcher.exe" (
    "sawbyte-launcher.exe" %*
) else if exist "sawbyte.exe" (
    "sawbyte.exe" launcher gui %*
) else (
    sawbyte launcher gui %*
)
