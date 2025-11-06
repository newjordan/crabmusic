#!/bin/bash
# CrabCrust Linux/macOS Installer
# Usage: curl -sSL https://raw.githubusercontent.com/USER/crabcrust/main/install.sh | bash

set -e

echo "🦀 Installing CrabCrust..."
echo ""

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin*)
        if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
            BINARY="crabcrust-macos-arm64"
        else
            BINARY="crabcrust-macos-x64"
        fi
        ;;
    linux*)
        BINARY="crabcrust-linux-x64"
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

URL="https://github.com/USER/crabcrust/releases/latest/download/$BINARY"

# Install directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Download
echo "📥 Downloading from GitHub..."
if command -v curl &> /dev/null; then
    curl -sSL "$URL" -o "$INSTALL_DIR/crabcrust"
elif command -v wget &> /dev/null; then
    wget -q "$URL" -O "$INSTALL_DIR/crabcrust"
else
    echo "❌ Neither curl nor wget found"
    exit 1
fi

chmod +x "$INSTALL_DIR/crabcrust"

# Add to PATH if needed
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "➕ Adding to PATH..."
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.bashrc"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.zshrc" 2>/dev/null || true
        export PATH="$PATH:$INSTALL_DIR"
        ;;
esac

echo ""
echo "✅ CrabCrust installed successfully!"
echo ""
echo "🎮 Try it out:"
echo "   crabcrust demo rocket"
echo ""
echo "🚀 To use with git, add to your shell config:"
echo "   alias git='crabcrust git'"
