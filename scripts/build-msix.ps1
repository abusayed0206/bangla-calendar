# Build MSIX Package for Windows Store
# Run this script after building the release binary with: cargo build --release

param(
    [string]$Version = "0.0.2.0"
)

$ErrorActionPreference = "Stop"

# Paths
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$MsixPackageDir = Join-Path $ProjectRoot "msix-package"
$ReleasesDir = Join-Path $ProjectRoot "releases"
$TargetExe = Join-Path $ProjectRoot "target\release\bangla-calendar.exe"

# Windows SDK tools
$SdkPath = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64"
$MakePri = Join-Path $SdkPath "makepri.exe"
$MakeAppx = Join-Path $SdkPath "makeappx.exe"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Bangla Calendar MSIX Package Builder" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
if (-not (Test-Path $TargetExe)) {
    Write-Host "ERROR: Release binary not found at $TargetExe" -ForegroundColor Red
    Write-Host "Run 'cargo build --release' first." -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $MakeAppx)) {
    Write-Host "ERROR: Windows SDK tools not found." -ForegroundColor Red
    Write-Host "Install Windows SDK from Visual Studio Installer." -ForegroundColor Yellow
    exit 1
}

# Update version in manifest
Write-Host "Updating version to $Version..." -ForegroundColor Yellow
$ManifestPath = Join-Path $MsixPackageDir "AppxManifest.xml"
$Manifest = Get-Content $ManifestPath -Raw
$Manifest = $Manifest -replace 'Version="[\d.]+"', "Version=`"$Version`""
$Manifest | Set-Content $ManifestPath -NoNewline

# Copy latest binary
Write-Host "Copying release binary..." -ForegroundColor Yellow
Copy-Item $TargetExe $MsixPackageDir -Force

# Regenerate resources.pri
Write-Host "Generating resources.pri..." -ForegroundColor Yellow
$PriConfig = Join-Path $MsixPackageDir "priconfig.xml"
$ResourcesPri = Join-Path $MsixPackageDir "resources.pri"

# Remove old pri files
Get-ChildItem $MsixPackageDir -Filter "*.pri" | Remove-Item -Force

& $MakePri createconfig /cf $PriConfig /dq en-US /o 2>$null
& $MakePri new /pr $MsixPackageDir /cf $PriConfig /of $ResourcesPri /o 2>$null

# Create MSIX package
Write-Host "Creating MSIX package..." -ForegroundColor Yellow
$MsixFile = Join-Path $ReleasesDir "BanglaCalendar_$($Version)_x64.msix"

if (Test-Path $MsixFile) {
    Remove-Item $MsixFile -Force
}

& $MakeAppx pack /d $MsixPackageDir /p $MsixFile /o

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "============================================" -ForegroundColor Green
    Write-Host "  Package created successfully!" -ForegroundColor Green
    Write-Host "============================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Output: $MsixFile" -ForegroundColor Cyan
    Write-Host "Size: $([math]::Round((Get-Item $MsixFile).Length / 1MB, 2)) MB" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "For Windows Store submission:" -ForegroundColor Yellow
    Write-Host "1. Upload this .msix file directly to Partner Center" -ForegroundColor White
    Write-Host "2. Or create .msixupload by zipping the .msix file" -ForegroundColor White
} else {
    Write-Host "ERROR: Package creation failed!" -ForegroundColor Red
    exit 1
}
