#!/bin/sh
# Sawbyte Desktop Launcher Starter for macOS
DIR="$(cd "$(dirname "$0")" && pwd)"
export PATH="$DIR:$PATH"
cd "$DIR"

echo "Starting Sawbyte Graphical Desktop Launcher..."
if [ -d "$DIR/launcher/xmcl-electron-app/dist/mac/Sawbyte Launcher.app" ]; then
    open "$DIR/launcher/xmcl-electron-app/dist/mac/Sawbyte Launcher.app"
elif [ -d "$DIR/launcher/xmcl-electron-app/build/output/mac/Sawbyte Launcher.app" ]; then
    open "$DIR/launcher/xmcl-electron-app/build/output/mac/Sawbyte Launcher.app"
elif [ -d "$DIR/launcher" ] && command -v pnpm >/dev/null 2>&1; then
    cd "$DIR/launcher" && exec pnpm run --prefix=xmcl-electron-app dev "$@"
elif [ -x "$DIR/sawbyte" ]; then
    exec "$DIR/sawbyte" launcher gui "$@"
else
    exec sawbyte launcher gui "$@"
fi
