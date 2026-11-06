@echo off
REM ==============================================================================
REM SAWBYTE FULL OVERHAUL — ONE-CLICK REPLACE & BUILD SCRIPT (Windows 10 / 11)
REM ==============================================================================
setlocal EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
set "TARGET_DIR=%~1"

if "%TARGET_DIR%"=="" (
    set "TARGET_DIR=%SCRIPT_DIR%"
)

echo ========================================================================
echo   SAWBYTE v0.4.0 OVERHAUL — REPLACING ^& BUILDING ALL ENGINE/TAURI FILES
echo ========================================================================
echo   Source Archive Dir : %SCRIPT_DIR%
echo   Target Project Dir : %TARGET_DIR%
echo ========================================================================

if /I not "%SCRIPT_DIR%"=="%TARGET_DIR%" (
    xcopy /E /I /Y "%SCRIPT_DIR%*" "%TARGET_DIR%\" >nul
    echo [1/4] Copied all updated files to %TARGET_DIR%
)

cd /d "%TARGET_DIR%"

where cargo >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [2/4] Compiling Rust Workspace (sawbyte.exe ^& Launcher Backend)...
    cargo build --workspace --release
    echo [3/4] Compiling Standalone Headless ^& Singleplayer Engine (sawbyte-play.exe)...
    cargo build --release --manifest-path native\Cargo.toml

    set "SAWBYTE_HOME=%USERPROFILE%\.sawbyte\bin"
    if not exist "!SAWBYTE_HOME!" mkdir "!SAWBYTE_HOME!"
    if exist "target\release\sawbyte.exe" (
        copy /Y "target\release\sawbyte.exe" "!SAWBYTE_HOME!\sawbyte.exe" >nul
        copy /Y "target\release\sawbyte.exe" "!SAWBYTE_HOME!\sawbyte-launcher.exe" >nul
    )
    if exist "native\target\release\sawbyte-play.exe" (
        copy /Y "native\target\release\sawbyte-play.exe" "!SAWBYTE_HOME!\sawbyte-play.exe" >nul
    )
    if exist "block-profile.json" (
        copy /Y "block-profile.json" "%USERPROFILE%\.sawbyte\block-profile.json" >nul
        copy /Y "block-profile.json" "!SAWBYTE_HOME!\block-profile.json" >nul
    )
    echo [4/4] Installed updated binaries to !SAWBYTE_HOME!
) else (
    echo [!] cargo not found in PATH — source files replaced.
)

echo ========================================================================
echo   Done! Run "sawbyte-launcher.exe" or "sawbyte.exe gui" to launch.
echo ========================================================================
pause
