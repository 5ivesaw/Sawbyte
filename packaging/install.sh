#!/bin/sh
# Sawbyte Universal CLI, Headless Engine & Launcher System Installer
# Installs 'sawbyte', 'sawbyte-play', 'sawbyte-launcher', and 'block-profile.json' to ~/.local/bin and ~/.sawbyte

set -e

REPO="5ivesaw/Sawbyte"
TAG="v0.5.0"
INSTALL_DIR="$HOME/.local/bin"
SAWBYTE_HOME_DIR="${SAWBYTE_HOME:-$HOME/.sawbyte}"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "[Sawbyte Installer] Initializing Sawbyte $TAG installation..."

mkdir -p "$INSTALL_DIR"
mkdir -p "$SAWBYTE_HOME_DIR/bin"
mkdir -p "$SAWBYTE_HOME_DIR/editions"
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
    cp -f "$SOURCE_BIN" "$SAWBYTE_HOME_DIR/bin/sawbyte"
    SRC_DIR="$(dirname "$SOURCE_BIN")"
    if [ -x "$SRC_DIR/sawbyte-play" ]; then
        cp -f "$SRC_DIR/sawbyte-play" "$INSTALL_DIR/sawbyte-play"
        cp -f "$SRC_DIR/sawbyte-play" "$SAWBYTE_HOME_DIR/bin/sawbyte-play"
        chmod 755 "$INSTALL_DIR/sawbyte-play" "$SAWBYTE_HOME_DIR/bin/sawbyte-play"
    fi
    if [ -f "$SRC_DIR/block-profile.json" ]; then
        cp -f "$SRC_DIR/block-profile.json" "$SAWBYTE_HOME_DIR/block-profile.json"
        cp -f "$SRC_DIR/block-profile.json" "$SAWBYTE_HOME_DIR/bin/block-profile.json"
    fi
else
    echo "[Sawbyte Installer] Detecting platform architecture..."
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"

    case "$OS-$ARCH" in
        linux-x86_64|linux-amd64)
            ARTIFACT="sawbyte-ubuntu-24.04-X64.tar.gz"
            PLAY_ARTIFACT="sawbyte-play-linux-x64.tar.gz"
            ;;
        darwin-arm64|darwin-aarch64)
            ARTIFACT="sawbyte-macos-14-ARM64.tar.gz"
            PLAY_ARTIFACT="sawbyte-play-macos-arm64.tar.gz"
            ;;
        *)
            echo "[Sawbyte Installer] Error: Unsupported system architecture $OS-$ARCH."
            echo "Please download the binary manually from https://github.com/$REPO/releases/latest"
            exit 1
            ;;
    esac

    LATEST_URL="https://github.com/$REPO/releases/latest/download/$ARTIFACT"
    FALLBACK_URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"
    TMP_DIR="$(mktemp -d)"
    echo "[Sawbyte Installer] Fetching $ARTIFACT from GitHub Releases..."

    if ! curl -fsSL "$LATEST_URL" -o "$TMP_DIR/sawbyte.tar.gz" 2>/dev/null; then
        curl -fsSL "$FALLBACK_URL" -o "$TMP_DIR/sawbyte.tar.gz"
    fi
    mkdir -p "$TMP_DIR/extracted"
    tar -xzf "$TMP_DIR/sawbyte.tar.gz" -C "$TMP_DIR/extracted"
    cp -f "$TMP_DIR/extracted/sawbyte" "$INSTALL_DIR/sawbyte"
    cp -f "$TMP_DIR/extracted/sawbyte" "$SAWBYTE_HOME_DIR/bin/sawbyte"

    if [ -f "$TMP_DIR/extracted/sawbyte-play" ]; then
        cp -f "$TMP_DIR/extracted/sawbyte-play" "$INSTALL_DIR/sawbyte-play"
        cp -f "$TMP_DIR/extracted/sawbyte-play" "$SAWBYTE_HOME_DIR/bin/sawbyte-play"
        chmod 755 "$INSTALL_DIR/sawbyte-play" "$SAWBYTE_HOME_DIR/bin/sawbyte-play"
    else
        PLAY_URL="https://github.com/$REPO/releases/latest/download/$PLAY_ARTIFACT"
        if curl -fsSL "$PLAY_URL" -o "$TMP_DIR/play.tar.gz" 2>/dev/null; then
            mkdir -p "$TMP_DIR/play_ext"
            tar -xzf "$TMP_DIR/play.tar.gz" -C "$TMP_DIR/play_ext"
            if [ -f "$TMP_DIR/play_ext/sawbyte-play" ]; then
                cp -f "$TMP_DIR/play_ext/sawbyte-play" "$INSTALL_DIR/sawbyte-play"
                cp -f "$TMP_DIR/play_ext/sawbyte-play" "$SAWBYTE_HOME_DIR/bin/sawbyte-play"
                chmod 755 "$INSTALL_DIR/sawbyte-play" "$SAWBYTE_HOME_DIR/bin/sawbyte-play"
            fi
        fi
    fi

    if [ -f "$TMP_DIR/extracted/block-profile.json" ]; then
        cp -f "$TMP_DIR/extracted/block-profile.json" "$SAWBYTE_HOME_DIR/block-profile.json"
        cp -f "$TMP_DIR/extracted/block-profile.json" "$SAWBYTE_HOME_DIR/bin/block-profile.json"
    fi
    if [ -f "$TMP_DIR/extracted/sawbyte.png" ]; then
        cp -f "$TMP_DIR/extracted/sawbyte.png" "$ICON_DIR/sawbyte.png"
    fi
    if [ -f "$TMP_DIR/extracted/sawbyte-launcher" ]; then
        cp -f "$TMP_DIR/extracted/sawbyte-launcher" "$INSTALL_DIR/sawbyte-launcher"
        chmod 755 "$INSTALL_DIR/sawbyte-launcher"
    fi
    rm -rf "$TMP_DIR"
fi

chmod 755 "$INSTALL_DIR/sawbyte" "$SAWBYTE_HOME_DIR/bin/sawbyte"

# Ensure default block-profile.json exists in ~/.sawbyte
if [ ! -f "$SAWBYTE_HOME_DIR/block-profile.json" ]; then
    cat << 'EOF' > "$SAWBYTE_HOME_DIR/block-profile.json"
{
  "minecraft": "26.3",
  "air": [0],
  "solid": [1],
  "bottom_slabs": [],
  "top_slabs": [],
  "stairs": [],
  "fences": [],
  "carpets": [],
  "climbable": [],
  "water": [],
  "lava": [],
  "cobwebs": [],
  "soul_sand": [],
  "ice": [],
  "hardness": {
    "1": 1.5
  }
}
EOF
    cp -f "$SAWBYTE_HOME_DIR/block-profile.json" "$SAWBYTE_HOME_DIR/bin/block-profile.json"
fi

# Install launcher runner and Electron app files if available
if [ -n "$SCRIPT_DIR" ] && [ -x "$SCRIPT_DIR/sawbyte-launcher" ]; then
    cp -f "$SCRIPT_DIR/sawbyte-launcher" "$INSTALL_DIR/sawbyte-launcher"
    chmod 755 "$INSTALL_DIR/sawbyte-launcher"
elif [ ! -x "$INSTALL_DIR/sawbyte-launcher" ]; then
    cat << 'EOF' > "$INSTALL_DIR/sawbyte-launcher"
#!/bin/sh
exec sawbyte launcher gui "$@"
EOF
    chmod 755 "$INSTALL_DIR/sawbyte-launcher"
fi

for unpacked_dir in "$SCRIPT_DIR/launcher/dist/linux-unpacked" "$SCRIPT_DIR/launcher/xmcl-electron-app/dist/linux-unpacked"; do
    if [ -n "$SCRIPT_DIR" ] && [ -d "$unpacked_dir" ]; then
        mkdir -p "$INSTALL_DIR/launcher/dist"
        cp -rf "$unpacked_dir" "$INSTALL_DIR/launcher/dist/linux-unpacked"
        break
    fi
done

# Install Desktop Entry and Icon
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/sawbyte.desktop" ]; then
    cp -f "$SCRIPT_DIR/sawbyte.desktop" "$DESKTOP_DIR/sawbyte.desktop"
elif [ -f "./packaging/sawbyte.desktop" ]; then
    cp -f "./packaging/sawbyte.desktop" "$DESKTOP_DIR/sawbyte.desktop"
else
    cat << 'EOF' > "$DESKTOP_DIR/sawbyte.desktop"
[Desktop Entry]
Type=Application
Name=Sawbyte Launcher
Comment=Sawbyte Zero-Graphics Minecraft Automation & Headless Client Launcher
Exec=sawbyte-launcher
Icon=sawbyte
Terminal=false
Categories=Game;Development;Utility;
StartupNotify=true
EOF
fi
chmod 644 "$DESKTOP_DIR/sawbyte.desktop"

for icon_cand in "$SCRIPT_DIR/sawbyte.png" "$SCRIPT_DIR/../assets/sawbyte.png" "./assets/sawbyte.png"; do
    if [ -f "$icon_cand" ]; then
        cp -f "$icon_cand" "$ICON_DIR/sawbyte.png"
        break
    fi
done

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
echo "  Sawbyte Home:    $SAWBYTE_HOME_DIR"
echo "  Desktop entry:   $DESKTOP_DIR/sawbyte.desktop"
echo "  CLI usage:       sawbyte --help"
echo "  Launcher usage:  sawbyte-launcher"
