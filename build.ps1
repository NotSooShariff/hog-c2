# Hog C2 Build Script for Windows
# This script builds a customized version with your Notion API credentials

Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "   Hog C2 - Custom Build Script" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host ""

# Check if .env file exists
if (-Not (Test-Path ".env")) {
    Write-Host "[!] .env file not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please create a .env file with your Notion credentials:" -ForegroundColor Yellow
    Write-Host "  1. Copy .env.example to .env" -ForegroundColor Yellow
    Write-Host "  2. Edit .env and add your NOTION_API_SECRET" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Example:" -ForegroundColor Gray
    Write-Host "  Copy-Item .env.example .env" -ForegroundColor Gray
    Write-Host ""
    exit 1
}

Write-Host "[✓] Found .env file" -ForegroundColor Green

# Load environment variables from .env
Get-Content .env | ForEach-Object {
    if ($_ -match '^\s*([^#][^=]+)=(.+)$') {
        $name = $matches[1].Trim()
        $value = $matches[2].Trim()
        [Environment]::SetEnvironmentVariable($name, $value, "Process")
        Write-Host "[✓] Loaded: $name" -ForegroundColor Green
    }
}

# Verify required variables
$notionSecret = $env:NOTION_API_SECRET
if ([string]::IsNullOrEmpty($notionSecret) -or $notionSecret -eq "secret_YOUR_NOTION_TOKEN_HERE") {
    Write-Host ""
    Write-Host "[!] ERROR: NOTION_API_SECRET not configured in .env file!" -ForegroundColor Red
    Write-Host "Please edit .env and add your real Notion API token." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "[✓] Configuration validated" -ForegroundColor Green
Write-Host "    Database: $env:NOTION_DATABASE_NAME" -ForegroundColor Gray
Write-Host "    Token: $($notionSecret.Substring(0, [Math]::Min(15, $notionSecret.Length)))..." -ForegroundColor Gray
Write-Host ""

# Check for required tools
Write-Host "Checking prerequisites..." -ForegroundColor Cyan

$hasNode = Get-Command node -ErrorAction SilentlyContinue
if (-Not $hasNode) {
    Write-Host "[!] Node.js not found. Please install from https://nodejs.org/" -ForegroundColor Red
    exit 1
}
Write-Host "[✓] Node.js: $(node --version)" -ForegroundColor Green

$hasRust = Get-Command cargo -ErrorAction SilentlyContinue
if (-Not $hasRust) {
    Write-Host "[!] Rust not found. Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}
Write-Host "[✓] Rust: $(rustc --version)" -ForegroundColor Green
Write-Host ""

# Install dependencies
Write-Host "Installing dependencies..." -ForegroundColor Cyan
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "[!] Failed to install dependencies" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Build the application
Write-Host "Building application..." -ForegroundColor Cyan
Write-Host "This may take several minutes..." -ForegroundColor Yellow
Write-Host ""

npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "[!] Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=======================================" -ForegroundColor Green
Write-Host "   Build Complete!" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Green
Write-Host ""
Write-Host "Your installers are ready:" -ForegroundColor Cyan
Write-Host "  Location: src-tauri\target\release\bundle\" -ForegroundColor White
Write-Host ""
Write-Host "Available installers:" -ForegroundColor Cyan
Get-ChildItem "src-tauri\target\release\bundle" -Recurse -Include *.exe,*.msi | ForEach-Object {
    Write-Host "  - $($_.Name)" -ForegroundColor White
}
Write-Host ""
Write-Host "The application is pre-configured with your Notion credentials" -ForegroundColor Yellow
Write-Host "and will auto-start on Windows boot." -ForegroundColor Yellow
Write-Host ""
