$ErrorActionPreference = "Stop"

$FISH_VERSION = "0.2.0"
$FISH_REPO = "foursavage-dev/forge-rs"
$INSTALL_DIR = "$env:USERPROFILE\.fish\bin"
$TEMP_DIR = Join-Path $env:TEMP "fish-install"

Write-Host "🐟 Installing Fish v${FISH_VERSION}..." -ForegroundColor Green

$ARCH = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "x86" }
$BINARY_NAME = "fish-windows-${ARCH}"
$DOWNLOAD_URL = "https://github.com/${FISH_REPO}/releases/download/v${FISH_VERSION}/${BINARY_NAME}.exe"

Write-Host "📥 Downloading Fish from ${DOWNLOAD_URL}..." -ForegroundColor Cyan

New-Item -ItemType Directory -Force -Path $TEMP_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null

try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile "$TEMP_DIR\fish.exe" -UseBasicParsing
    Write-Host "✅ Download successful" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Pre-built binary not found, building from source..." -ForegroundColor Yellow
    Write-Host "📦 This requires Rust to be installed" -ForegroundColor Yellow
    
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Rust/Cargo not found. Please install Rust first:" -ForegroundColor Red
        Write-Host "   irm https://sh.rustup.rs | iex" -ForegroundColor Cyan
        exit 1
    }
    
    Push-Location $TEMP_DIR
    git clone --depth 1 --branch main "https://github.com/${FISH_REPO}.git" fish-rs
    Set-Location fish-rs
    cargo build --release -p fish-cli
    
    Copy-Item "target\release\fish.exe" "$TEMP_DIR\fish.exe"
    Write-Host "✅ Build successful" -ForegroundColor Green
    Pop-Location
}

Write-Host "📝 Installing to ${INSTALL_DIR}..." -ForegroundColor Cyan
Move-Item "$TEMP_DIR\fish.exe" "${INSTALL_DIR}\fish.exe" -Force

$PATH_ENV = [Environment]::GetEnvironmentVariable("Path", "User")
if ($PATH_ENV -notlike "*${INSTALL_DIR}*") {
    Write-Host "🔧 Adding ${INSTALL_DIR} to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "${PATH_ENV};${INSTALL_DIR}", "User")
    Write-Host "⚠️  Please restart your terminal to use PATH changes" -ForegroundColor Yellow
}

Remove-Item -Recurse -Force $TEMP_DIR -ErrorAction SilentlyContinue

if (Get-Command fish -ErrorAction SilentlyContinue) {
    Write-Host "✅ Fish installed successfully!" -ForegroundColor Green
    Write-Host "🎉 Run 'fish --help' to get started" -ForegroundColor Green
    fish --version
} else {
    Write-Host "❌ Installation failed" -ForegroundColor Red
    Write-Host "You can run fish directly from: ${INSTALL_DIR}\fish.exe" -ForegroundColor Yellow
    exit 1
}
