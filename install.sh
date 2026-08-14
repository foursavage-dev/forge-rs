#!/bin/bash
# Forge Installation Script for Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/requla11/forge-rs/main/install.sh | bash

set -e

FORGE_VERSION="0.1.0"
FORGE_REPO="requla11/forge-rs"
INSTALL_DIR="/usr/local/bin"
TEMP_DIR=$(mktemp -d)

echo "🦀 Installing Forge v${FORGE_VERSION}..."

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64)
        ARCH="amd64"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

BINARY_NAME="forge-${OS}-${ARCH}"
DOWNLOAD_URL="https://github.com/${FORGE_REPO}/releases/download/v${FORGE_VERSION}/${BINARY_NAME}"

echo "📥 Downloading Forge from ${DOWNLOAD_URL}..."

# Try to download binary
if curl -fsSL -o "${TEMP_DIR}/forge" "${DOWNLOAD_URL}"; then
    echo "✅ Download successful"
else
    echo "⚠️  Pre-built binary not found, building from source..."
    echo "📦 This requires Rust to be installed"
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo not found. Please install Rust first:"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Clone and build
    cd "${TEMP_DIR}"
    git clone --depth 1 --branch main "https://github.com/${FORGE_REPO}.git" forge-rs
    cd forge-rs
    cargo build --release
    
    cp target/release/forge "${TEMP_DIR}/forge"
    echo "✅ Build successful"
fi

# Install binary
echo "📝 Installing to ${INSTALL_DIR}..."
if [ -w "${INSTALL_DIR}" ]; then
    mv "${TEMP_DIR}/forge" "${INSTALL_DIR}/forge"
    chmod +x "${INSTALL_DIR}/forge"
else
    echo "⚠️  Sudo required for installation"
    sudo mv "${TEMP_DIR}/forge" "${INSTALL_DIR}/forge"
    sudo chmod +x "${INSTALL_DIR}/forge"
fi

# Cleanup
rm -rf "${TEMP_DIR}"

# Verify installation
if command -v forge &> /dev/null; then
    echo "✅ Forge installed successfully!"
    echo "🎉 Run 'forge --help' to get started"
    forge --version
else
    echo "❌ Installation failed"
    exit 1
fi
