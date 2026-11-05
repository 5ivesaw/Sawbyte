@echo off
rem ====================================================================
rem   Sawbyte Desktop Launcher Starter for Windows
rem ====================================================================
cd /d "%~dp0"
echo Starting Sawbyte Graphical Desktop Launcher...

if exist "launcher\xmcl-electron-app\dist\win-unpacked\sawbyte-launcher.exe" (
    start "" "launcher\xmcl-electron-app\dist\win-unpacked\sawbyte-launcher.exe" %*
) else if exist "sawbyte-launcher-bin.exe" (
    start "" "sawbyte-launcher-bin.exe" %*
) else if exist "launcher" (
    where pnpm >nul 2>&1
    if not errorlevel 1 (
        cd launcher && pnpm run --prefix=xmcl-electron-app dev %*
    ) else (
        start "" sawbyte.exe launcher gui %*
    )
) else (
    start "" sawbyte.exe launcher gui %*
)
