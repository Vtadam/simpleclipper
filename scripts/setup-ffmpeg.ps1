# SimpleClipper — FFmpeg Setup Script
# Run this once before your first build: .\scripts\setup-ffmpeg.ps1

$ErrorActionPreference = "Stop"

$FFMPEG_VERSION = "7.1"
$FFMPEG_BUILD   = "ffmpeg-n7.1-latest-win64-gpl-shared-7.1"
$FFMPEG_URL     = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/$FFMPEG_BUILD.zip"
$FFMPEG_ZIP     = "$env:TEMP\ffmpeg.zip"
$FFMPEG_DIR     = "$PSScriptRoot\..\src-tauri\ffmpeg"
$FFMPEG_EXTRACT = "$env:TEMP\ffmpeg-extract"

Write-Host ""
Write-Host "  SimpleClipper — FFmpeg Setup" -ForegroundColor Cyan
Write-Host "  =============================" -ForegroundColor Cyan
Write-Host ""

# ── Check if already set up ─────────────────────────────────────────────────
if (Test-Path "$FFMPEG_DIR\avcodec-61.dll") {
    Write-Host "  [OK] FFmpeg DLLs already present at src-tauri/ffmpeg/" -ForegroundColor Green
    Write-Host "       Run with -Force to re-download." -ForegroundColor DarkGray
    Write-Host ""
    exit 0
}

# ── Download ─────────────────────────────────────────────────────────────────
Write-Host "  [1/4] Downloading FFmpeg $FFMPEG_VERSION (shared, GPL)..." -ForegroundColor Yellow
Write-Host "        Source: $FFMPEG_URL" -ForegroundColor DarkGray

$ProgressPreference = "SilentlyContinue"
Invoke-WebRequest -Uri $FFMPEG_URL -OutFile $FFMPEG_ZIP -UseBasicParsing

Write-Host "  [OK] Downloaded." -ForegroundColor Green

# ── Extract ──────────────────────────────────────────────────────────────────
Write-Host "  [2/4] Extracting..." -ForegroundColor Yellow

if (Test-Path $FFMPEG_EXTRACT) { Remove-Item $FFMPEG_EXTRACT -Recurse -Force }
Expand-Archive -Path $FFMPEG_ZIP -DestinationPath $FFMPEG_EXTRACT

Write-Host "  [OK] Extracted." -ForegroundColor Green

# ── Copy DLLs ────────────────────────────────────────────────────────────────
Write-Host "  [3/4] Copying DLLs to src-tauri/ffmpeg/..." -ForegroundColor Yellow

$DLL_SOURCE = (Get-ChildItem "$FFMPEG_EXTRACT" -Recurse -Directory -Filter "bin" | Select-Object -First 1).FullName
$REQUIRED_DLLS = @(
    "avcodec-61.dll",
    "avformat-61.dll",
    "avutil-59.dll",
    "swscale-8.dll",
    "swresample-5.dll"
)

New-Item -ItemType Directory -Path $FFMPEG_DIR -Force | Out-Null

$missing = @()
foreach ($dll in $REQUIRED_DLLS) {
    $src = "$DLL_SOURCE\$dll"
    if (Test-Path $src) {
        Copy-Item $src "$FFMPEG_DIR\$dll" -Force
        Write-Host "        Copied: $dll" -ForegroundColor DarkGray
    } else {
        $missing += $dll
    }
}

# Also copy dev headers and libs for compilation
$INCLUDE_SOURCE = (Get-ChildItem "$FFMPEG_EXTRACT" -Recurse -Directory -Filter "include" | Select-Object -First 1).FullName
$LIB_SOURCE     = (Get-ChildItem "$FFMPEG_EXTRACT" -Recurse -Directory -Filter "lib"     | Select-Object -First 1).FullName

if ($INCLUDE_SOURCE) {
    Copy-Item $INCLUDE_SOURCE "$FFMPEG_DIR\include" -Recurse -Force
}
if ($LIB_SOURCE) {
    Copy-Item $LIB_SOURCE "$FFMPEG_DIR\lib" -Recurse -Force
}

if ($missing.Count -gt 0) {
    Write-Host ""
    Write-Host "  [WARN] Some DLLs not found (version mismatch?): $($missing -join ', ')" -ForegroundColor Yellow
    Write-Host "         Check $DLL_SOURCE for the actual filenames and update tauri.conf.json." -ForegroundColor DarkGray
} else {
    Write-Host "  [OK] All DLLs copied." -ForegroundColor Green
}

# ── Set FFMPEG_DIR environment variable ─────────────────────────────────────
Write-Host "  [4/4] Setting FFMPEG_DIR environment variable..." -ForegroundColor Yellow

$absPath = (Resolve-Path $FFMPEG_DIR).Path
[System.Environment]::SetEnvironmentVariable("FFMPEG_DIR", $absPath, "User")
$env:FFMPEG_DIR = $absPath

Write-Host "  [OK] FFMPEG_DIR = $absPath" -ForegroundColor Green

# ── Cleanup ──────────────────────────────────────────────────────────────────
Remove-Item $FFMPEG_ZIP -Force -ErrorAction SilentlyContinue
Remove-Item $FFMPEG_EXTRACT -Recurse -Force -ErrorAction SilentlyContinue

# ── Done ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  Setup complete!" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Next steps:" -ForegroundColor White
Write-Host "    1. Restart your terminal (to pick up FFMPEG_DIR)" -ForegroundColor DarkGray
Write-Host "    2. Run: npm run tauri dev" -ForegroundColor DarkGray
Write-Host "    3. Or build the installer: .\scripts\build-installer.ps1" -ForegroundColor DarkGray
Write-Host ""
