# CrabCrust Windows Installer
# Usage: iwr -useb https://raw.githubusercontent.com/USER/crabcrust/main/install.ps1 | iex

$ErrorActionPreference = 'Stop'

Write-Host "🦀 Installing CrabCrust..." -ForegroundColor Cyan
Write-Host ""

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
$url = "https://github.com/USER/crabcrust/releases/latest/download/crabcrust-windows-$arch.exe"

# Create install directory
$installDir = "$env:LOCALAPPDATA\crabcrust"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$exePath = "$installDir\crabcrust.exe"

# Download
Write-Host "📥 Downloading from GitHub..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $url -OutFile $exePath -UseBasicParsing
} catch {
    Write-Host "❌ Download failed: $_" -ForegroundColor Red
    exit 1
}

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "➕ Adding to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$userPath;$installDir",
        "User"
    )
    $env:Path = "$env:Path;$installDir"
}

# Test installation
Write-Host ""
Write-Host "✅ CrabCrust installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "🎮 Try it out:" -ForegroundColor Cyan
Write-Host "   crabcrust demo rocket" -ForegroundColor White
Write-Host ""
Write-Host "🚀 To use with git, add to your PowerShell profile:" -ForegroundColor Cyan
Write-Host "   function git { crabcrust git `$args }" -ForegroundColor White
Write-Host ""
Write-Host "📝 Edit profile: notepad `$PROFILE" -ForegroundColor Yellow
