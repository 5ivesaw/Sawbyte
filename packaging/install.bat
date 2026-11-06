@echo off
rem ==============================================================================
rem Sawbyte Universal CLI, Headless Engine & Launcher System Installer for Windows
rem Installs 'sawbyte.exe' & 'sawbyte-play.exe' to %LOCALAPPDATA%\Programs\Sawbyte
rem ==============================================================================

setlocal enabledelayedexpansion

set "TARGET_DIR=%LOCALAPPDATA%\Programs\Sawbyte"
set "SAWBYTE_HOME_DIR=%USERPROFILE%\.sawbyte"
set "BIN_TARGET=%TARGET_DIR%\sawbyte.exe"
set "PLAY_TARGET=%TARGET_DIR%\sawbyte-play.exe"
set "TAG=v0.3.4"
set "LATEST_URL=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-windows-2022-X64.zip"
set "FALLBACK_URL=https://github.com/5ivesaw/Sawbyte/releases/download/%TAG%/sawbyte-windows-2022-X64.zip"

echo [Sawbyte Installer] Initializing Sawbyte %TAG% installation...

if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
if not exist "%SAWBYTE_HOME_DIR%\bin" mkdir "%SAWBYTE_HOME_DIR%\bin"
if not exist "%SAWBYTE_HOME_DIR%\editions" mkdir "%SAWBYTE_HOME_DIR%\editions"

set "SOURCE_BIN="
if exist "%~dp0sawbyte.exe" set "SOURCE_BIN=%~dp0sawbyte.exe"
if exist "%~dp0..\sawbyte.exe" set "SOURCE_BIN=%~dp0..\sawbyte.exe"
if exist ".\sawbyte.exe" set "SOURCE_BIN=.\sawbyte.exe"

if defined SOURCE_BIN (
    echo [Sawbyte Installer] Installing local binary from %SOURCE_BIN%...
    copy /Y "%SOURCE_BIN%" "%BIN_TARGET%" >nul
    copy /Y "%SOURCE_BIN%" "%SAWBYTE_HOME_DIR%\bin\sawbyte.exe" >nul
    if exist "%~dp0sawbyte-play.exe" (
        copy /Y "%~dp0sawbyte-play.exe" "%PLAY_TARGET%" >nul
        copy /Y "%~dp0sawbyte-play.exe" "%SAWBYTE_HOME_DIR%\bin\sawbyte-play.exe" >nul
    )
    if exist "%~dp0block-profile.json" (
        copy /Y "%~dp0block-profile.json" "%TARGET_DIR%\block-profile.json" >nul
        copy /Y "%~dp0block-profile.json" "%SAWBYTE_HOME_DIR%\block-profile.json" >nul
    )
) else (
    echo [Sawbyte Installer] Downloading Windows release package from GitHub Releases...
    set "TEMP_ZIP=%TEMP%\sawbyte-win.zip"
    curl -fsSL "%LATEST_URL%" -o "!TEMP_ZIP!" 2>nul
    if not exist "!TEMP_ZIP!" (
        curl -fsSL "%FALLBACK_URL%" -o "!TEMP_ZIP!"
    )
    powershell -NoProfile -Command "Expand-Archive -Path '!TEMP_ZIP!' -DestinationPath '%TEMP%\sawbyte_extracted' -Force"
    copy /Y "%TEMP%\sawbyte_extracted\sawbyte.exe" "%BIN_TARGET%" >nul
    copy /Y "%TEMP%\sawbyte_extracted\sawbyte.exe" "%SAWBYTE_HOME_DIR%\bin\sawbyte.exe" >nul
    if exist "%TEMP%\sawbyte_extracted\sawbyte-play.exe" (
        copy /Y "%TEMP%\sawbyte_extracted\sawbyte-play.exe" "%PLAY_TARGET%" >nul
        copy /Y "%TEMP%\sawbyte_extracted\sawbyte-play.exe" "%SAWBYTE_HOME_DIR%\bin\sawbyte-play.exe" >nul
    )
    if exist "%TEMP%\sawbyte_extracted\block-profile.json" (
        copy /Y "%TEMP%\sawbyte_extracted\block-profile.json" "%TARGET_DIR%\block-profile.json" >nul
        copy /Y "%TEMP%\sawbyte_extracted\block-profile.json" "%SAWBYTE_HOME_DIR%\block-profile.json" >nul
    )
    del /f /q "!TEMP_ZIP!" 2>nul
    rmdir /s /q "%TEMP%\sawbyte_extracted" 2>nul
)

if not exist "%BIN_TARGET%" (
    echo [Sawbyte Installer] Error: Failed to install sawbyte.exe to %TARGET_DIR%.
    exit /b 1
)

rem Check if TARGET_DIR is already in User PATH
for /f "tokens=2*" %%A in ('reg query "HKCU\Environment" /v PATH 2^>nul') do set "USER_PATH=%%B"
echo !USER_PATH! | find /i "%TARGET_DIR%" >nul
if errorlevel 1 (
    echo [Sawbyte Installer] Adding %TARGET_DIR% to User PATH...
    if defined USER_PATH (
        setx PATH "!USER_PATH!;%TARGET_DIR%" >nul
    ) else (
        setx PATH "%TARGET_DIR%" >nul
    )
    echo [Sawbyte Installer] PATH updated. Please open a new command prompt to use 'sawbyte'.
) else (
    echo [Sawbyte Installer] %TARGET_DIR% is already present in PATH.
)

rem Create desktop launcher shortcut if PowerShell is available
powershell -NoProfile -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut([System.IO.Path]::Combine([System.Environment]::GetFolderPath('Desktop'), 'Sawbyte Launcher.lnk')); $s.TargetPath = '%BIN_TARGET%'; $s.Arguments = 'launcher gui'; $s.Description = 'Sawbyte Minecraft Automation Launcher'; $s.Save()" 2>nul

echo.
echo [Sawbyte Installer] Installation complete!
echo   Executable: %BIN_TARGET%
echo   CLI Usage:  sawbyte --help
echo   Launcher:   sawbyte launcher gui
echo.
pause
