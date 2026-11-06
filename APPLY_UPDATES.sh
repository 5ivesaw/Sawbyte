#!/bin/sh
# One-step script to apply Sawbyte updated files and remove legacy XMCL folders.
# Run this script from inside your Sawbyte repository root after extracting Sawbyte-updated-files.zip.
set -e

echo "[Sawbyte Update] Removing legacy copied XMCL directories from launcher/..."
rm -rf launcher/xmcl-keystone-ui \
       launcher/xmcl-runtime \
       launcher/xmcl-runtime-api \
       launcher/packages \
       launcher/patches \
       launcher/xmcl-electron-app \
       launcher/pnpm-lock.yaml \
       launcher/pnpm-workspace.yaml \
       launcher/tsconfig.json

chmod +x packaging/install.sh packaging/sawbyte-launcher "packaging/Sawbyte Launcher.command" launcher/build.mjs launcher/main/index.cjs 2>/dev/null || true
echo "[Sawbyte Update] All updated files applied and legacy XMCL directories cleaned!"
