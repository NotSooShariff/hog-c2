#!/bin/bash

# Hog C2 Build Script for Linux/macOS
# This script builds a customized version with your Notion API credentials

set -e

echo "======================================="
echo "   Hog C2 - Custom Build Script"
echo "======================================="
echo ""

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "[!] .env file not found!"
    echo ""
    echo "Please create a .env file with your Notion credentials:"
    echo "  1. Copy .env.example to .env"
    echo "  2. Edit .env and add your NOTION_API_SECRET"
    echo ""
    echo "Example:"
    echo "  cp .env.example .env"
    echo "  nano .env"
    echo ""
    exit 1
fi

echo "[✓] Found .env file"

# Load environment variables from .env
export $(grep -v '^#' .env | xargs)

echo "[✓] Loaded environment variables"

# Verify required variables
if [ -z "$NOTION_API_SECRET" ] || [ "$NOTION_API_SECRET" = "secret_YOUR_NOTION_TOKEN_HERE" ]; then
    echo ""
    echo "[!] ERROR: NOTION_API_SECRET not configured in .env file!"
    echo "Please edit .env and add your real Notion API token."
    exit 1
fi

echo ""
echo "[✓] Configuration validated"
echo "    Database: $NOTION_DATABASE_NAME"
echo "    Token: ${NOTION_API_SECRET:0:15}..."
echo ""

# Check for required tools
echo "Checking prerequisites..."

if ! command -v node &> /dev/null; then
    echo "[!] Node.js not found. Please install from https://nodejs.org/"
    exit 1
fi
echo "[✓] Node.js: $(node --version)"

if ! command -v cargo &> /dev/null; then
    echo "[!] Rust not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "[✓] Rust: $(rustc --version)"
echo ""

# Install dependencies
echo "Installing dependencies..."
npm install

echo ""

# Build the application
echo "Building application..."
echo "This may take several minutes..."
echo ""

npm run tauri build

echo ""
echo "======================================="
echo "   Build Complete!"
echo "======================================="
echo ""
echo "Your installers are ready:"
echo "  Location: src-tauri/target/release/bundle/"
echo ""
echo "Available installers:"
find src-tauri/target/release/bundle -type f \( -name "*.dmg" -o -name "*.AppImage" -o -name "*.deb" \) 2>/dev/null | while read file; do
    echo "  - $(basename "$file")"
done
echo ""
echo "The application is pre-configured with your Notion credentials"
echo "and will auto-start on system boot."
echo ""
