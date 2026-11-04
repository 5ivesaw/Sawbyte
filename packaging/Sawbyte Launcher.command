#!/bin/sh
# Sawbyte Desktop Launcher Starter for macOS
DIR="$(cd "$(dirname "$0")" && pwd)"
export PATH="$DIR:$PATH"
cd "$DIR"
echo "Starting Sawbyte Graphical Desktop Launcher..."
exec ./sawbyte launcher gui "$@"
