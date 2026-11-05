#!/bin/sh
# Sawbyte Universal CLI & Launcher System Installer
# Installs 'sawbyte' to user binary directory (~/.local/bin) and integrates with system PATH.

set -e

REPO="5ivesaw/Sawbyte"
TAG="v0.3.2"
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "[Sawbyte Installer] Initializing installation..."

mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd || echo "")"
SOURCE_BIN=""

if [ -n "$SCRIPT_DIR" ] && [ -x "$SCRIPT_DIR/sawbyte" ]; then
    SOURCE_BIN="$SCRIPT_DIR/sawbyte"
elif [ -n "$SCRIPT_DIR" ] && [ -x "$SCRIPT_DIR/../sawbyte" ]; then
    SOURCE_BIN="$SCRIPT_DIR/../sawbyte"
elif [ -x "./sawbyte" ]; then
    SOURCE_BIN="./sawbyte"
fi

if [ -n "$SOURCE_BIN" ]; then
    echo "[Sawbyte Installer] Installing local binary from $SOURCE_BIN..."
    cp -f "$SOURCE_BIN" "$INSTALL_DIR/sawbyte"
else
    echo "[Sawbyte Installer] Detecting platform architecture..."
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"

    case "$OS-$ARCH" in
        linux-x86_64)
            ARTIFACT="sawbyte-ubuntu-24.04-X64.zip"
            ;;
        darwin-arm64|darwin-aarch64)
            ARTIFACT="sawbyte-macos-14-ARM64.tar.gz"
            ;;
        *)
            echo "[Sawbyte Installer] Error: Unsupported system architecture $OS-$ARCH."
            echo "Please download the binary manually from https://github.com/$REPO/releases"
            exit 1
            ;;
    esac

    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"
    TMP_DIR="$(mktemp -d)"
    echo "[Sawbyte Installer] Fetching $ARTIFACT from GitHub Releases ($TAG)..."

    if [ "$ARTIFACT" = "${ARTIFACT%.tar.gz}" ]; then
        curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/sawbyte.zip"
        unzip -q "$TMP_DIR/sawbyte.zip" -d "$TMP_DIR/extracted"
        cp -f "$TMP_DIR/extracted/sawbyte" "$INSTALL_DIR/sawbyte"
    else
        curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/sawbyte.tar.gz"
        tar -xzf "$TMP_DIR/sawbyte.tar.gz" -C "$TMP_DIR"
        cp -f "$TMP_DIR/sawbyte" "$INSTALL_DIR/sawbyte"
    fi
    rm -rf "$TMP_DIR"
fi

chmod 755 "$INSTALL_DIR/sawbyte"

# Install Desktop Entry and Icon
if [ -f "$SCRIPT_DIR/sawbyte.desktop" ]; then
    cp -f "$SCRIPT_DIR/sawbyte.desktop" "$DESKTOP_DIR/sawbyte.desktop"
elif [ -f "./packaging/sawbyte.desktop" ]; then
    cp -f "./packaging/sawbyte.desktop" "$DESKTOP_DIR/sawbyte.desktop"
else
    cat << 'EOF' > "$DESKTOP_DIR/sawbyte.desktop"
[Desktop Entry]
Type=Application
Name=Sawbyte Launcher
Comment=Sawbyte Zero-Graphics Minecraft Automation & Headless Client Launcher
Exec=sawbyte launcher gui
Icon=sawbyte
Terminal=false
Categories=Game;Development;Utility;
StartupNotify=true
EOF
fi
chmod 644 "$DESKTOP_DIR/sawbyte.desktop"

if [ -f "$SCRIPT_DIR/../assets/sawbyte.png" ]; then
    cp -f "$SCRIPT_DIR/../assets/sawbyte.png" "$ICON_DIR/sawbyte.png"
elif [ -f "./assets/sawbyte.png" ]; then
    cp -f "./assets/sawbyte.png" "$ICON_DIR/sawbyte.png"
fi

# Verify PATH
IN_PATH=0
case ":$PATH:" in
    *":$INSTALL_DIR:"*) IN_PATH=1 ;;
esac

if [ "$IN_PATH" -eq 0 ]; then
    echo "[Sawbyte Installer] Adding $INSTALL_DIR to your shell PATH configuration..."
    if [ -f "$HOME/.bashrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    fi
    if [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
    fi
    echo "[Sawbyte Installer] NOTE: Run 'export PATH=\"\$HOME/.local/bin:\$PATH\"' or restart your terminal."
fi

echo "[Sawbyte Installer] Installation complete!"
echo "  Binary location: $INSTALL_DIR/sawbyte"
echo "  Desktop entry:   $DESKTOP_DIR/sawbyte.desktop"
echo "  CLI usage:       sawbyte --help"
echo "  Launcher usage:  sawbyte launcher gui"
