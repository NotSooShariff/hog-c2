# Build Script for Windows (PowerShell)
# Usage: .\scripts\build.ps1 -NotionSecret "your_secret" [-Target windows|linux|macos]

param(
    [Parameter(Mandatory=$true, HelpMessage="Notion API Secret Token")]
    [string]$NotionSecret,

    [Parameter(Mandatory=$false, HelpMessage="Target platform: windows, linux, or macos")]
    [ValidateSet("windows", "linux", "macos")]
    [string]$Target = "windows",

    [Parameter(Mandatory=$false, HelpMessage="Database name in Notion")]
    [string]$DatabaseName = "All Clients",

    [Parameter(Mandatory=$false, HelpMessage="Build configuration: debug or release")]
    [ValidateSet("debug", "release")]
    [string]$Config = "release"
)

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  Productivity Tracker Build Script" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Validate Notion Secret format
if (-not $NotionSecret.StartsWith("secret_")) {
    Write-Host "ERROR: Invalid Notion API Secret format. Must start with 'secret_'" -ForegroundColor Red
    exit 1
}

# Set environment variables
$env:NOTION_API_SECRET = $NotionSecret
$env:NOTION_DATABASE_NAME = $DatabaseName
$env:APP_NAME = "Productivity Tracker"
$env:APP_VERSION = "1.0.0"

Write-Host "Configuration:" -ForegroundColor Green
Write-Host "  Target: $Target" -ForegroundColor White
Write-Host "  Build: $Config" -ForegroundColor White
Write-Host "  Database: $DatabaseName" -ForegroundColor White
Write-Host "  Notion Secret: $($NotionSecret.Substring(0, 10))..." -ForegroundColor White
Write-Host ""

# Check if npm is installed
Write-Host "Checking dependencies..." -ForegroundColor Yellow
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: npm is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Rust/Cargo is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "✓ Dependencies OK" -ForegroundColor Green
Write-Host ""

# Install npm dependencies
Write-Host "Installing Node dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to install npm dependencies" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Node dependencies installed" -ForegroundColor Green
Write-Host ""

# Build based on configuration
if ($Config -eq "release") {
    Write-Host "Building application (Release mode)..." -ForegroundColor Yellow
    Write-Host "This may take several minutes due to optimizations..." -ForegroundColor Gray

    # Set target for cross-compilation if needed
    $buildArgs = @("tauri", "build")

    switch ($Target) {
        "linux" {
            Write-Host "WARNING: Cross-compilation to Linux from Windows requires additional setup" -ForegroundColor Yellow
            Write-Host "See: https://tauri.app/v1/guides/building/linux" -ForegroundColor Yellow
            $buildArgs += "--target", "x86_64-unknown-linux-gnu"
        }
        "macos" {
            Write-Host "WARNING: Cross-compilation to macOS from Windows is not directly supported" -ForegroundColor Yellow
            Write-Host "macOS builds should be done on macOS systems" -ForegroundColor Yellow
            exit 1
        }
    }

    npm run $buildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Building application (Debug mode)..." -ForegroundColor Yellow
    npm run tauri build -- --debug
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "  Build completed successfully!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host ""

# Find and display the output location
$outputPath = "src-tauri\target\$Config\bundle"
if (Test-Path $outputPath) {
    Write-Host "Build artifacts location:" -ForegroundColor Cyan
    Write-Host "  $outputPath" -ForegroundColor White
    Write-Host ""
    Write-Host "Available installers:" -ForegroundColor Cyan
    Get-ChildItem -Path $outputPath -Recurse -Include *.exe, *.msi, *.deb, *.appimage, *.dmg | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        Write-Host "  $($_.Name) ($size MB)" -ForegroundColor White
    }
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Test the application before distribution" -ForegroundColor White
Write-Host "  2. The executable is ready for deployment" -ForegroundColor White
Write-Host "  3. Make sure Notion database is properly configured" -ForegroundColor White
Write-Host ""
