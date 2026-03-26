# SimpleClipper — Installer Build Script
# Produces: src-tauri/target/release/bundle/nsis/SimpleClipper_x.y.z_x64-setup.exe
#           src-tauri/target/release/bundle/msi/SimpleClipper_x.y.z_x64.msi
#
# Usage:
#   .\scripts\build-installer.ps1             # builds both NSIS and MSI
#   .\scripts\build-installer.ps1 -Target nsis
#   .\scripts\build-installer.ps1 -Target msi
#   .\scripts\build-installer.ps1 -SkipChecks

param(
    [ValidateSet("nsis", "msi", "all")]
    [string]$Target = "all",
    [switch]$SkipChecks
)

$ErrorActionPreference = "Stop"
$ROOT = Resolve-Path "$PSScriptRoot\.."

Write-Host ""
Write-Host "  SimpleClipper — Build Installer" -ForegroundColor Cyan
Write-Host "  ================================" -ForegroundColor Cyan
Write-Host ""

# ── Pre-flight checks ────────────────────────────────────────────────────────
if (-not $SkipChecks) {

    # Check Rust
    try {
        $rustVersion = (rustc --version 2>&1)
        Write-Host "  [OK] Rust: $rustVersion" -ForegroundColor Green
    } catch {
        Write-Host "  [ERR] Rust not found. Install from https://rustup.rs/" -ForegroundColor Red
        exit 1
    }

    # Check Node
    try {
        $nodeVersion = (node --version 2>&1)
        Write-Host "  [OK] Node: $nodeVersion" -ForegroundColor Green
    } catch {
        Write-Host "  [ERR] Node.js not found. Install from https://nodejs.org/" -ForegroundColor Red
        exit 1
    }

    # Check FFMPEG_DIR
    if (-not $env:FFMPEG_DIR) {
        Write-Host "  [ERR] FFMPEG_DIR is not set. Run .\scripts\setup-ffmpeg.ps1 first." -ForegroundColor Red
        exit 1
    }
    if (-not (Test-Path "$env:FFMPEG_DIR\avcodec-61.dll")) {
        Write-Host "  [ERR] FFmpeg DLLs not found at FFMPEG_DIR=$env:FFMPEG_DIR" -ForegroundColor Red
        Write-Host "        Run .\scripts\setup-ffmpeg.ps1 to re-download." -ForegroundColor DarkGray
        exit 1
    }
    Write-Host "  [OK] FFmpeg: $env:FFMPEG_DIR" -ForegroundColor Green

    # Check NSIS (for nsis target)
    if ($Target -eq "nsis" -or $Target -eq "all") {
        $pf86 = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::ProgramFilesX86)
        $nsisPath = Join-Path $pf86 "NSIS\makensis.exe"
        if (-not (Test-Path $nsisPath)) {
            Write-Host "  [WARN] NSIS not found. Tauri will attempt to download it automatically." -ForegroundColor Yellow
        } else {
            Write-Host "  [OK] NSIS: $nsisPath" -ForegroundColor Green
        }
    }

    Write-Host ""
}

# ── Install npm dependencies ─────────────────────────────────────────────────
Set-Location $ROOT
Write-Host "  [1/3] Installing npm dependencies..." -ForegroundColor Yellow
npm ci --silent
Write-Host "  [OK] npm dependencies ready." -ForegroundColor Green

# ── Build ────────────────────────────────────────────────────────────────────
Write-Host "  [2/3] Building release bundle (this takes a few minutes)..." -ForegroundColor Yellow
Write-Host ""

$bundleTarget = switch ($Target) {
    "nsis" { "--bundles nsis" }
    "msi"  { "--bundles msi"  }
    "all"  { ""               }
}

$buildCmd = "npm run tauri build -- $bundleTarget".Trim()
Write-Host "        Running: $buildCmd" -ForegroundColor DarkGray
Write-Host ""

Invoke-Expression $buildCmd

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "  [ERR] Build failed. See output above for details." -ForegroundColor Red
    exit 1
}

# ── Report output ────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  [3/3] Locating installer files..." -ForegroundColor Yellow

$bundleDir = "$ROOT\src-tauri\target\release\bundle"
$outputs   = @()

if ($Target -eq "nsis" -or $Target -eq "all") {
    $nsis = Get-ChildItem "$bundleDir\nsis\*-setup.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($nsis) { $outputs += $nsis.FullName }
}
if ($Target -eq "msi" -or $Target -eq "all") {
    $msi = Get-ChildItem "$bundleDir\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($msi) { $outputs += $msi.FullName }
}

Write-Host ""
Write-Host "  Build complete!" -ForegroundColor Cyan
Write-Host ""

if ($outputs.Count -eq 0) {
    Write-Host "  [WARN] No installer files found in $bundleDir" -ForegroundColor Yellow
} else {
    Write-Host "  Output files:" -ForegroundColor White
    foreach ($f in $outputs) {
        $size = [math]::Round((Get-Item $f).Length / 1MB, 1)
        Write-Host "    $f  ($size MB)" -ForegroundColor Green
    }
}

Write-Host ""
