#!/bin/bash
set -e

FISH_VERSION="0.2.0"
FISH_REPO="requla11/fish"
INSTALL_DIR="/usr/local/bin"
TEMP_DIR=$(mktemp -d)

echo "🐟 Installing Fish v${FISH_VERSION}..."

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

BINARY_NAME="fish-${OS}-${ARCH}"
DOWNLOAD_URL="https://github.com/${FISH_REPO}/releases/download/v${FISH_VERSION}/${BINARY_NAME}"

echo "📥 Downloading Fish from ${DOWNLOAD_URL}..."

if curl -fsSL -o "${TEMP_DIR}/fish" "${DOWNLOAD_URL}"; then
    echo "✅ Download successful"
else
    echo "⚠️  Pre-built binary not found, building from source..."
    echo "📦 This requires Rust to be installed"
    
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo not found. Please install Rust first:"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    cd "${TEMP_DIR}"
    git clone --depth 1 --branch main "https://github.com/${FISH_REPO}.git" fish-rs
    cd fish
    cargo build --release -p fish-cli
    
    cp target/release/fish "${TEMP_DIR}/fish"
    echo "✅ Build successful"
fi

echo "📝 Installing to ${INSTALL_DIR}..."
if [ -w "${INSTALL_DIR}" ]; then
    mv "${TEMP_DIR}/fish" "${INSTALL_DIR}/fish"
    chmod +x "${INSTALL_DIR}/fish"
else
    echo "⚠️  Sudo required for installation"
    sudo mv "${TEMP_DIR}/fish" "${INSTALL_DIR}/fish"
    sudo chmod +x "${INSTALL_DIR}/fish"
fi

rm -rf "${TEMP_DIR}"

if command -v fish &> /dev/null; then
    echo "✅ Fish installed successfully!"
    echo "🎉 Run 'fish --help' to get started"
    fish --version
else
    echo "❌ Installation failed"
    exit 1
fi
