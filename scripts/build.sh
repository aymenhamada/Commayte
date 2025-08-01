#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔨 Commayte Build Script${NC}"
echo -e "${BLUE}=======================${NC}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed. Please install Rust first.${NC}"
    echo -e "${YELLOW}📖 Visit: https://rustup.rs/${NC}"
    exit 1
fi

# Create dist directory
mkdir -p dist

# Build for current platform
echo -e "${YELLOW}📦 Building for current platform...${NC}"
cargo build --release --bin commayte

# Copy binary to dist with platform-specific name
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$OS" = "darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        BINARY="commayte-macos-arm64"
    else
        BINARY="commayte-macos-x86_64"
    fi
elif [ "$OS" = "linux" ]; then
    BINARY="commayte-linux-x86_64"
else
    echo -e "${RED}❌ Unsupported operating system: $OS${NC}"
    exit 1
fi

cp target/release/commayte "dist/$BINARY"
echo -e "${GREEN}✅ Built: dist/$BINARY${NC}"

echo -e "${GREEN}🎉 Build complete!${NC}"
echo -e "${BLUE}💡 Binary is ready for testing: ./dist/$BINARY${NC}" 