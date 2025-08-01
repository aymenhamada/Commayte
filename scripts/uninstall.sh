#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🗑️  Commayte Uninstall Script${NC}"
echo -e "${BLUE}============================${NC}"

# Confirm uninstallation
echo -e "${YELLOW}⚠️  This will remove Commayte and stop the Ollama container.${NC}"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}❌ Uninstallation cancelled${NC}"
    exit 0
fi

# Remove Commayte binary
echo -e "${YELLOW}🗑️  Removing Commayte binary...${NC}"
if [ -f "/usr/local/bin/commayte" ]; then
    sudo rm /usr/local/bin/commayte
    echo -e "${GREEN}✅ Commayte binary removed${NC}"
else
    echo -e "${YELLOW}⚠️  Commayte binary not found${NC}"
fi

# Remove configuration
echo -e "${YELLOW}🗑️  Removing configuration...${NC}"
if [ -d "$HOME/.config/commayte" ]; then
    rm -rf "$HOME/.config/commayte"
    echo -e "${GREEN}✅ Configuration removed${NC}"
else
    echo -e "${YELLOW}⚠️  Configuration directory not found${NC}"
fi

# Remove PATH configuration
echo -e "${YELLOW}🗑️  Cleaning up PATH configuration...${NC}"
SHELL_CONFIG=""
if [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [[ "$SHELL" == *"bash"* ]]; then
    SHELL_CONFIG="$HOME/.bashrc"
fi

if [ -n "$SHELL_CONFIG" ] && [ -f "$SHELL_CONFIG" ]; then
    # Remove Commayte PATH lines
    if grep -q "Commayte PATH configuration" "$SHELL_CONFIG"; then
        # Create a temporary file without the Commayte lines
        grep -v -A 1 -B 1 "Commayte PATH configuration" "$SHELL_CONFIG" > "${SHELL_CONFIG}.tmp"
        mv "${SHELL_CONFIG}.tmp" "$SHELL_CONFIG"
        echo -e "${GREEN}✅ PATH configuration cleaned up${NC}"
    else
        echo -e "${YELLOW}⚠️  No Commayte PATH configuration found${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Could not determine shell config file${NC}"
fi

# Stop Ollama service
echo -e "${YELLOW}🛑 Stopping Ollama service...${NC}"
if command -v ollama &> /dev/null; then
    # Kill any running Ollama processes
    pkill -f ollama 2>/dev/null || true
    echo -e "${GREEN}✅ Ollama service stopped${NC}"
else
    echo -e "${YELLOW}⚠️  Ollama not found${NC}"
fi

# Remove Ollama installation
echo -e "${YELLOW}🗑️  Removing Ollama...${NC}"
read -p "Do you want to remove Ollama completely? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Remove Ollama binary
    if [ -f "/usr/local/bin/ollama" ]; then
        sudo rm /usr/local/bin/ollama
        echo -e "${GREEN}✅ Ollama binary removed${NC}"
    fi
    
    # Remove Ollama data directory
    if [ -d "$HOME/.ollama" ]; then
        rm -rf "$HOME/.ollama"
        echo -e "${GREEN}✅ Ollama data removed${NC}"
    fi
else
    echo -e "${BLUE}ℹ️  Ollama preserved${NC}"
fi

echo -e "${GREEN}🎉 Uninstallation complete!${NC}" 