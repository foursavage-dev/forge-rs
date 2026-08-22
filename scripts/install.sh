#!/usr/bin/env bash
set -euo pipefail

REPO="requla11/fish"
INSTALL_DIR="${FISH_INSTALL_DIR:-$HOME/.fish/bin}"
BIN_NAME="fish"

main() {
    echo "==> Installing Fish Build Orchestration System..."

    local os arch target
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$os" in
        linux*)
            case "$arch" in
                x86_64|amd64) target="x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) target="aarch64-unknown-linux-gnu" ;;
                *) echo "Error: Unsupported Linux architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        darwin*)
            case "$arch" in
                x86_64|amd64) target="x86_64-apple-darwin" ;;
                aarch64|arm64) target="aarch64-apple-darwin" ;;
                *) echo "Error: Unsupported macOS architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Error: Unsupported OS: $os. Please use install.ps1 for Windows." >&2
            exit 1
            ;;
    esac

    mkdir -p "$INSTALL_DIR"

    local release_url archive_url
    release_url="https://api.github.com/repos/${REPO}/releases/latest"
    
    local version="latest"
    local tag=""
    
    if command -v curl >/dev/null 2>&1; then
        tag="$(curl -sSL "$release_url" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)"
    fi

    local downloaded=0
    if [ -n "$tag" ]; then
        local archive_name="fish-${tag}-${target}.tar.gz"
        archive_url="https://github.com/${REPO}/releases/download/${tag}/${archive_name}"
        
        echo "==> Downloading Fish ${tag} for ${target}..."
        local tmp_dir
        tmp_dir="$(mktemp -d)"
        
        if curl -fsSL "$archive_url" -o "${tmp_dir}/${archive_name}" 2>/dev/null; then
            tar -xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"
            if [ -f "${tmp_dir}/fish-${tag}-${target}/${BIN_NAME}" ]; then
                cp "${tmp_dir}/fish-${tag}-${target}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
                chmod +x "${INSTALL_DIR}/${BIN_NAME}"
                downloaded=1
            elif [ -f "${tmp_dir}/${BIN_NAME}" ]; then
                cp "${tmp_dir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
                chmod +x "${INSTALL_DIR}/${BIN_NAME}"
                downloaded=1
            fi
        fi
        rm -rf "$tmp_dir"
    fi

    if [ "$downloaded" -eq 0 ]; then
        echo "==> Prebuilt binary not found on GitHub Releases."
        if command -v cargo >/dev/null 2>&1; then
            echo "==> Building and installing Fish from GitHub source via Cargo..."
            cargo install --git "https://github.com/${REPO}.git" fish-cli --bin fish --root "$HOME/.fish"
            downloaded=1
        else
            echo "Error: Failed to download prebuilt binary and 'cargo' is not installed." >&2
            echo "Please install Rust (https://rustup.rs) or download a release from https://github.com/${REPO}/releases" >&2
            exit 1
        fi
    fi

    local path_str="export PATH=\"\$HOME/.fish/bin:\$PATH\""
    local updated_shell=0

    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [ -f "$rc" ] && ! grep -q ".fish/bin" "$rc"; then
            echo "" >> "$rc"
            echo "$path_str" >> "$rc"
            updated_shell=1
        fi
    done

    echo ""
    echo "==> Fish installed successfully to ${INSTALL_DIR}/${BIN_NAME}!"
    if [ "$updated_shell" -eq 1 ]; then
        echo "==> Added ~/.fish/bin to PATH in shell configuration files."
        echo "==> Restart your terminal or run: export PATH=\"\$HOME/.fish/bin:\$PATH\""
    fi
    echo "==> Run 'fish --version' or 'fish --help' to get started."
}

main "$@"
