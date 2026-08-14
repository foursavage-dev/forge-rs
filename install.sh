#!/usr/bin/env bash
set -euo pipefail

REPO="requla11/forge-rs"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.forge/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="forge-linux-x86_64" ;;
      aarch64|arm64) TARGET="forge-linux-x86_64" ;;
      *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      x86_64) TARGET="forge-macos-x86_64" ;;
      arm64|aarch64) TARGET="forge-macos-aarch64" ;;
      *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS. For Windows, please run install.ps1 in PowerShell." >&2
    exit 1
    ;;
esac

DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${TARGET}"

echo "Installing Forge (${TARGET}) from ${REPO}..."

mkdir -p "$INSTALL_DIR"
TEMP_FILE="$(mktemp)"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TEMP_FILE" "$DOWNLOAD_URL"
else
  echo "Error: curl or wget is required to download Forge." >&2
  exit 1
fi

chmod +x "$TEMP_FILE"
mv "$TEMP_FILE" "$INSTALL_DIR/forge"

echo "Forge successfully installed to $INSTALL_DIR/forge"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "Please add Forge to your PATH by adding the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
    ;;
esac

echo "Run 'forge --version' or 'forge doctor' to get started! 🦀"
