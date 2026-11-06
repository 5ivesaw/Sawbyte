# Sawbyte Repository Delta Update Instructions (For Local Agent / User)

This archive contains **only the 29 modified and newly created files** for the Sawbyte repository so you do not have to re-download unchanged files.

## Exact Steps for Agent to Apply & Push

1. **Extract this archive directly into the root of the `Sawbyte` repository** (overwriting existing paths).
2. **Delete the legacy copied XMCL directories/files inside `launcher/`**:
   ```bash
   rm -rf launcher/xmcl-keystone-ui \
          launcher/xmcl-runtime \
          launcher/xmcl-runtime-api \
          launcher/packages \
          launcher/patches \
          launcher/xmcl-electron-app \
          launcher/pnpm-lock.yaml \
          launcher/pnpm-workspace.yaml \
          launcher/tsconfig.json
   ```
   *(Or simply run `./APPLY_UPDATES.sh` on Linux/macOS or `APPLY_UPDATES.bat` on Windows, then remove `APPLY_UPDATES.sh`, `APPLY_UPDATES.bat`, and `AGENT_UPDATE_INSTRUCTIONS.md` before committing if desired.)*
3. **Ensure executable permissions on scripts**:
   ```bash
   chmod +x packaging/install.sh packaging/sawbyte-launcher "packaging/Sawbyte Launcher.command" launcher/build.mjs launcher/main/index.cjs
   ```
4. **Stage, commit, and push**:
   ```bash
   git add -A
   git commit -m "Overhaul Sawbyte Launcher, fix release workflows, and add Minimal Mode website with real screenshots"
   git push origin main
   ```

## Files Included in This Delta Archive (29 files)

- `.github/workflows/ci.yml`
- `.github/workflows/player.yml`
- `README.md`
- `assets/screenshots/bot_view.png`
- `assets/screenshots/launcher.png`
- `assets/screenshots/server_manager.png`
- `crates/sawbyte-cli/src/gui.rs`
- `crates/sawbyte-core/src/launcher.rs`
- `docs/app.js`
- `docs/assets/preview_launcher.png`
- `docs/assets/screenshots/bot_view.png`
- `docs/assets/screenshots/launcher.png`
- `docs/assets/screenshots/server_manager.png`
- `docs/index.html`
- `docs/style.css`
- `launcher/README.md`
- `launcher/assets/sawbyte.png`
- `launcher/build.mjs`
- `launcher/main/index.cjs`
- `launcher/package.json`
- `launcher/renderer/index.html`
- `native/src/lib.rs`
- `packaging/Sawbyte Launcher.bat`
- `packaging/Sawbyte Launcher.command`
- `packaging/install.bat`
- `packaging/install.sh`
- `packaging/sawbyte-launcher`
- `sdks/python/pyproject.toml`
- `sdks/typescript/package.json`
