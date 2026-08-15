# Forge Installation Script for Windows
# Usage: irm https://raw.githubusercontent.com/foursavage-dev/forge-rs/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$FORGE_VERSION = "0.1.0"
$FORGE_REPO = "foursavage-dev/forge-rs"
$INSTALL_DIR = "$env:USERPROFILE\.forge\bin"
$TEMP_DIR = Join-Path $env:TEMP "forge-install"

Write-Host "🦀 Installing Forge v${FORGE_VERSION}..." -ForegroundColor Green

# Detect architecture
$ARCH = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "x86" }
$BINARY_NAME = "forge-windows-${ARCH}"
$DOWNLOAD_URL = "https://github.com/${FORGE_REPO}/releases/download/v${FORGE_VERSION}/${BINARY_NAME}.exe"

Write-Host "📥 Downloading Forge from ${DOWNLOAD_URL}..." -ForegroundColor Cyan

# Create temp directory
New-Item -ItemType Directory -Force -Path $TEMP_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null

# Try to download binary
try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile "$TEMP_DIR\forge.exe" -UseBasicParsing
    Write-Host "✅ Download successful" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Pre-built binary not found, building from source..." -ForegroundColor Yellow
    Write-Host "📦 This requires Rust to be installed" -ForegroundColor Yellow
    
    # Check if cargo is available
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Rust/Cargo not found. Please install Rust first:" -ForegroundColor Red
        Write-Host "   irm https://sh.rustup.rs | iex" -ForegroundColor Cyan
        exit 1
    }
    
    # Clone and build
    Push-Location $TEMP_DIR
    git clone --depth 1 --branch main "https://github.com/${FORGE_REPO}.git" forge-rs
    Set-Location forge-rs
    cargo build --release
    
    Copy-Item "target\release\forge.exe" "$TEMP_DIR\forge.exe"
    Write-Host "✅ Build successful" -ForegroundColor Green
    Pop-Location
}

# Install binary
Write-Host "📝 Installing to ${INSTALL_DIR}..." -ForegroundColor Cyan
Move-Item "$TEMP_DIR\forge.exe" "${INSTALL_DIR}\forge.exe" -Force

# Add to PATH if not already there
$PATH_ENV = [Environment]::GetEnvironmentVariable("Path", "User")
if ($PATH_ENV -notlike "*${INSTALL_DIR}*") {
    Write-Host "🔧 Adding ${INSTALL_DIR} to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "${PATH_ENV};${INSTALL_DIR}", "User")
    Write-Host "⚠️  Please restart your terminal to use PATH changes" -ForegroundColor Yellow
}

# Cleanup
Remove-Item -Recurse -Force $TEMP_DIR -ErrorAction SilentlyContinue

# Verify installation
if (Get-Command forge -ErrorAction SilentlyContinue) {
    Write-Host "✅ Forge installed successfully!" -ForegroundColor Green
    Write-Host "🎉 Run 'forge --help' to get started" -ForegroundColor Green
    forge --version
} else {
    Write-Host "❌ Installation failed" -ForegroundColor Red
    Write-Host "You can run forge directly from: ${INSTALL_DIR}\forge.exe" -ForegroundColor Yellow
    exit 1
}
