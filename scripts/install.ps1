$ErrorActionPreference = "Stop"

$Repo = "requla11/fish"
$InstallDir = if ($env:FISH_INSTALL_DIR) { $env:FISH_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "fish\bin" }
$BinName = "fish.exe"

Write-Host "==> Installing Fish Build Orchestration System..." -ForegroundColor Cyan

$Arch = if ([IntPtr]::Size -eq 8) { "x86_64" } else { "i686" }
$Target = "$Arch-pc-windows-msvc"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$Downloaded = $false
$ReleaseUrl = "https://api.github.com/repos/$Repo/releases/latest"

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13
    $Release = Invoke-RestMethod -Uri $ReleaseUrl -UseBasicParsing -ErrorAction SilentlyContinue
    if ($Release -and $Release.tag_name) {
        $Tag = $Release.tag_name
        $ArchiveName = "fish-$Tag-$Target.zip"
        $ArchiveUrl = "https://github.com/$Repo/releases/download/$Tag/$ArchiveName"

        Write-Host "==> Downloading Fish $Tag for $Target..." -ForegroundColor Cyan
        $TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
        $ZipPath = Join-Path $TempDir $ArchiveName

        Invoke-WebRequest -Uri $ArchiveUrl -OutFile $ZipPath -UseBasicParsing
        Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

        $Candidate = Join-Path $TempDir "fish-$Tag-$Target\$BinName"
        if (-not (Test-Path $Candidate)) {
            $Candidate = Join-Path $TempDir $BinName
        }

        if (Test-Path $Candidate) {
            Copy-Item -Path $Candidate -Destination (Join-Path $InstallDir $BinName) -Force
            $Downloaded = $true
        }

        Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
} catch {
    $Downloaded = $false
}

if (-not $Downloaded) {
    Write-Host "==> Prebuilt binary not found on GitHub Releases." -ForegroundColor Yellow
    $Cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($Cargo) {
        Write-Host "==> Building and installing Fish from GitHub source via Cargo..." -ForegroundColor Cyan
        $FishRoot = Join-Path $env:LOCALAPPDATA "fish"
        & cargo install --git "https://github.com/$Repo.git" fish-cli --bin fish --root $FishRoot
        $Downloaded = $true
    } else {
        Write-Error "Failed to download prebuilt binary and 'cargo' is not installed. Please install Rust or download a release from https://github.com/$Repo/releases"
        exit 1
    }
}

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -split ";" -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "==> Added $InstallDir to User PATH environment variable." -ForegroundColor Green
}

Write-Host ""
Write-Host "==> Fish installed successfully to $(Join-Path $InstallDir $BinName)!" -ForegroundColor Green
Write-Host "==> Restart your terminal or run: `fish --version` to get started." -ForegroundColor Cyan
