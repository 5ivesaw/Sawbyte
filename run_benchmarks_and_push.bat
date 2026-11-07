@echo off
REM ==============================================================================
REM Sawbyte v0.5.2 — Local 16-Server Benchmark Runner, Doc Auto-Updater & Git Push
REM ==============================================================================
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo ========================================================================
echo   SAWBYTE v0.5.2 — RUNNING ALL 16 CRACKED SERVER BENCHMARKS ^& PUSHING
echo ========================================================================

echo [1/5] Running Rust edition, let-chain ^& lockfile safeguard...
python tools\fix_rust_edition_and_let_chains.py

where cargo >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [2/5] Formatting ^& building Sawbyte native client...
    cargo fmt --all
    cargo build --workspace --release
    cargo build --release --manifest-path native\Cargo.toml
)

set MODE=%1
if "%MODE%"=="" set MODE=auto
echo [3/5] Running all 16 Cracked Server Benchmarks (mode=%MODE%) and updating README.md + Website...
python benchmarks\run_all_16_servers.py --mode %MODE% --update-docs

if exist .git (
    echo [5/5] Committing and pushing updated benchmark results to GitHub...
    git add README.md docs\BENCHMARKS.md docs\index.html benchmarks\ tools\ Cargo.toml Cargo.lock rustfmt.toml native\rustfmt.toml .github\workflows\
    git commit -m "feat(v0.5.2): fix Rust 2024 edition/let-chains, add 16-server cracked benchmarks & sensory vision"
    git push
)

echo ========================================================================
echo   ALL 16 SERVER BENCHMARKS ^& DOC UPDATES COMPLETE!
echo ========================================================================
