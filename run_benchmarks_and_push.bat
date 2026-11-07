@echo off
REM ==============================================================================
REM Sawbyte v0.5.2 — Interactive Local Live Benchmark Runner & Auto-Publisher
REM ==============================================================================
setlocal enabledelayedexpansion
cd /d "%~dp0"

python "%~dp0run_live_benchmarks.py" %*
