#!/bin/bash

# Build Script for Linux/macOS
# Usage: ./scripts/build.sh --notion-secret "your_secret" [--target windows|linux|macos] [--config debug|release]

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Default values
TARGET="linux"
CONFIG="release"
DATABASE_NAME="All Clients"
NOTION_SECRET=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --notion-secret)
            NOTION_SECRET="$2"
            shift 2
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --config)
            CONFIG="$2"
            shift 2
            ;;
        --database-name)
            DATABASE_NAME="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 --notion-secret SECRET [OPTIONS]"
            echo ""
            echo "Required:"
            echo "  --notion-secret SECRET    Notion API Secret Token"
            echo ""
            echo "Optional:"
            echo "  --target TARGET           Target platform: windows, linux, macos (default: linux)"
            echo "  --config CONFIG           Build config: debug, release (default: release)"
            echo "  --database-name NAME      Notion database name (default: All Clients)"
            echo "  --help                    Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}ERROR: Unknown option $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Validate required parameters
if [ -z "$NOTION_SECRET" ]; then
    echo -e "${RED}ERROR: --notion-secret is required${NC}"
    echo "Use --help for usage information"
    exit 1
fi

# Validate Notion Secret format
if [[ ! "$NOTION_SECRET" =~ ^secret_ ]]; then
    echo -e "${RED}ERROR: Invalid Notion API Secret format. Must start with 'secret_'${NC}"
    exit 1
fi

echo -e "${CYAN}=====================================${NC}"
echo -e "${CYAN}  Productivity Tracker Build Script${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""

# Set environment variables
export NOTION_API_SECRET="$NOTION_SECRET"
export NOTION_DATABASE_NAME="$DATABASE_NAME"
export APP_NAME="Productivity Tracker"
export APP_VERSION="1.0.0"

echo -e "${GREEN}Configuration:${NC}"
echo -e "  Target: ${WHITE}$TARGET${NC}"
echo -e "  Build: ${WHITE}$CONFIG${NC}"
echo -e "  Database: ${WHITE}$DATABASE_NAME${NC}"
echo -e "  Notion Secret: ${WHITE}${NOTION_SECRET:0:10}...${NC}"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"

if ! command -v npm &> /dev/null; then
    echo -e "${RED}ERROR: npm is not installed or not in PATH${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: Rust/Cargo is not installed or not in PATH${NC}"
    echo -e "${YELLOW}Install Rust from: https://rustup.rs/${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Dependencies OK${NC}"
echo ""

# Install npm dependencies
echo -e "${YELLOW}Installing Node dependencies...${NC}"
npm install
echo -e "${GREEN}✓ Node dependencies installed${NC}"
echo ""

# Build based on configuration
if [ "$CONFIG" = "release" ]; then
    echo -e "${YELLOW}Building application (Release mode)...${NC}"
    echo -e "${WHITE}This may take several minutes due to optimizations...${NC}"

    BUILD_ARGS="tauri build"

    case $TARGET in
        windows)
            echo -e "${YELLOW}WARNING: Cross-compilation to Windows requires additional setup${NC}"
            echo -e "${YELLOW}See: https://tauri.app/v1/guides/building/windows${NC}"
            BUILD_ARGS="$BUILD_ARGS --target x86_64-pc-windows-msvc"
            ;;
        macos)
            if [[ "$OSTYPE" != "darwin"* ]]; then
                echo -e "${YELLOW}WARNING: macOS builds should be done on macOS systems${NC}"
                echo -e "${YELLOW}Cross-compilation may not work properly${NC}"
            fi
            BUILD_ARGS="$BUILD_ARGS --target x86_64-apple-darwin"
            ;;
    esac

    npm run $BUILD_ARGS
else
    echo -e "${YELLOW}Building application (Debug mode)...${NC}"
    npm run tauri build -- --debug
fi

echo ""
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}  Build completed successfully!${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""

# Find and display the output location
OUTPUT_PATH="src-tauri/target/$CONFIG/bundle"
if [ -d "$OUTPUT_PATH" ]; then
    echo -e "${CYAN}Build artifacts location:${NC}"
    echo -e "  ${WHITE}$OUTPUT_PATH${NC}"
    echo ""
    echo -e "${CYAN}Available installers:${NC}"

    # Find installers
    find "$OUTPUT_PATH" -type f \( -name "*.exe" -o -name "*.msi" -o -name "*.deb" -o -name "*.appimage" -o -name "*.dmg" -o -name "*.app" \) | while read -r file; do
        size=$(du -h "$file" | cut -f1)
        echo -e "  ${WHITE}$(basename "$file") ($size)${NC}"
    done
fi

echo ""
echo -e "${CYAN}Next steps:${NC}"
echo -e "  ${WHITE}1. Test the application before distribution${NC}"
echo -e "  ${WHITE}2. The executable is ready for deployment${NC}"
echo -e "  ${WHITE}3. Make sure Notion database is properly configured${NC}"
echo ""
