$ErrorActionPreference = 'Stop'

$Repo = "requla11/forge-rs"
$InstallDir = "$env:USERPROFILE\.forge\bin"
$ExePath = "$InstallDir\forge.exe"
$BinaryName = "forge-windows-x86_64.exe"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$BinaryName"

Write-Host "Installing Forge for Windows from $Repo..." -ForegroundColor Cyan

if (!(Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$TempFile = [System.IO.Path]::GetTempFileName()

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
    Move-Item -Path $TempFile -Destination $ExePath -Force
} catch {
    Write-Error "Failed to download Forge binary: $_"
    exit 1
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
    Write-Host "Added $InstallDir to user PATH." -ForegroundColor Green
}

Write-Host "Forge successfully installed to $ExePath" -ForegroundColor Green
Write-Host "Run 'forge --version' or 'forge doctor' to get started! 🦀" -ForegroundColor Cyan
