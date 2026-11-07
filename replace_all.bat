@echo off
REM ==============================================================================
REM SAWBYTE v0.5.2 FULL OVERHAUL — ONE-CLICK REPLACE, ASSIMILATE & BUILD (Windows)
REM ==============================================================================
setlocal EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
set "TARGET_DIR=%~1"

if "%TARGET_DIR%"=="" (
    set "TARGET_DIR=%SCRIPT_DIR%"
)

echo ========================================================================
echo   SAWBYTE v0.5.2 — FULL RUST 26.3 UNIFICATION ^& TAURI LAUNCHER BUILD
echo ========================================================================
echo   Source Archive Dir : %SCRIPT_DIR%
echo   Target Project Dir : %TARGET_DIR%
echo ========================================================================

if /I not "%SCRIPT_DIR%"=="%TARGET_DIR%" (
    xcopy /E /I /Y "%SCRIPT_DIR%*" "%TARGET_DIR%\" >nul
    echo [1/5] Copied all updated v0.5.2 files to %TARGET_DIR%
)

cd /d "%TARGET_DIR%"

where python >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [2/5] Running Sawbyte v0.5.2 Full Rust Engine Assimilator...
    python tools\assimilate_horizon_into_sawbyte.py
)

where cargo >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [3/5] Compiling Rust v0.5.2 Workspace (sawbyte.exe, sawbyte-ctl.exe, sawbyte-prepare.exe)...
    cargo build --workspace --release
    echo [4/5] Compiling Standalone Headless ^& Singleplayer Engine (sawbyte-play.exe)...
    cargo build --release --manifest-path native\Cargo.toml

    set "SAWBYTE_HOME=%USERPROFILE%\.sawbyte\bin"
    if not exist "!SAWBYTE_HOME!" mkdir "!SAWBYTE_HOME!"
    for %%B in (sawbyte.exe sawbyte-ctl.exe sawbyte-prepare.exe) do (
        if exist "target\release\%%B" (
            copy /Y "target\release\%%B" "!SAWBYTE_HOME!\%%B" >nul
        )
    )
    if exist "target\release\sawbyte.exe" (
        copy /Y "target\release\sawbyte.exe" "!SAWBYTE_HOME!\sawbyte-launcher.exe" >nul
    )
    if exist "native\target\release\sawbyte-play.exe" (
        copy /Y "native\target\release\sawbyte-play.exe" "!SAWBYTE_HOME!\sawbyte-play.exe" >nul
    )
    if exist "block-profile.json" (
        copy /Y "block-profile.json" "%USERPROFILE%\.sawbyte\block-profile.json" >nul
        copy /Y "block-profile.json" "!SAWBYTE_HOME!\block-profile.json" >nul
    )
    echo [5/5] Installed updated v0.5.2 binaries to !SAWBYTE_HOME!
) else (
    echo [!] cargo not found in PATH — source files replaced.
)

echo ========================================================================
echo   Sawbyte v0.5.2 Ready! Run "sawbyte-launcher.exe" or "sawbyte.exe gui".
echo ========================================================================
pause
