@echo off
echo [Sawbyte Update] Removing legacy copied XMCL directories from launcher\...
for %%D in (xmcl-keystone-ui xmcl-runtime xmcl-runtime-api packages patches xmcl-electron-app) do (
    if exist "launcher\%%D" rmdir /s /q "launcher\%%D"
)
for %%F in (pnpm-lock.yaml pnpm-workspace.yaml tsconfig.json) do (
    if exist "launcher\%%F" del /f /q "launcher\%%F"
)
echo [Sawbyte Update] All updated files applied and legacy XMCL directories cleaned!
