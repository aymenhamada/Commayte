#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Commayte Installation Script${NC}"
echo -e "${BLUE}==============================${NC}"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo -e "${RED}❌ This script should not be run as root${NC}"
   exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"

if ! command_exists git; then
    echo -e "${RED}❌ Git is not installed. Please install Git first.${NC}"
    exit 1
fi

if ! command_exists curl; then
    echo -e "${RED}❌ curl is not installed. Please install curl first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"

# Install Ollama locally
echo -e "${YELLOW}📦 Installing Ollama locally...${NC}"

# Check if Ollama is already installed
if command_exists ollama; then
    echo -e "${GREEN}✅ Ollama is already installed${NC}"
else
    # Install Ollama based on platform
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    
    if [ "$OS" = "darwin" ]; then
        echo -e "${YELLOW}🍎 Installing Ollama for macOS...${NC}"
        curl -fsSL https://ollama.ai/install.sh | sh
    elif [ "$OS" = "linux" ]; then
        echo -e "${YELLOW}🐧 Installing Ollama for Linux...${NC}"
        curl -fsSL https://ollama.ai/install.sh | sh
    else
        echo -e "${RED}❌ Unsupported operating system: $OS${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Ollama installed successfully${NC}"
fi

# Start Ollama service
echo -e "${YELLOW}🚀 Starting Ollama service...${NC}"

# Check if Ollama is already running
if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Ollama is already running${NC}"
else
    # Start Ollama in background
    ollama serve > /dev/null 2>&1 &
    OLLAMA_PID=$!
    
    # Wait for Ollama to be ready
    echo -e "${YELLOW}⏳ Waiting for Ollama to be ready...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Ollama is ready${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ Ollama failed to start within 30 seconds${NC}"
            kill $OLLAMA_PID 2>/dev/null || true
            exit 1
        fi
        sleep 1
    done
fi

# Ask user for model preference early
echo -e "${YELLOW}⚙️  Model Selection${NC}"
echo -e "${BLUE}🤖 Available models:${NC}"
echo -e "  1. mistral (default - good balance of speed and quality)"
echo -e "  2. phi3:mini (fast and efficient)"

# Check if we're in an interactive terminal
if [ -t 0 ]; then
    read -p "Choose model (1-2) [1]: " model_choice
else
    echo -e "${YELLOW}⚠️  Non-interactive mode detected. Using default model: mistral${NC}"
    model_choice="1"
fi

case $model_choice in
    1|"")
        MODEL="mistral"
        ;;
    2)
        MODEL="phi3:mini"
        ;;
    *)
        MODEL="mistral"
        ;;
esac

# Download selected model
echo -e "${YELLOW}📥 Downloading $MODEL model...${NC}"
if curl -s http://localhost:11434/api/tags | grep -q "$MODEL"; then
    echo -e "${GREEN}✅ $MODEL model is already downloaded${NC}"
else
    echo -e "${YELLOW}⏳ This may take a few minutes...${NC}"
    # Download with a clean progress indicator
    curl -X POST http://localhost:11434/api/pull -d "{\"name\": \"$MODEL\"}" -s 2>/dev/null | while IFS= read -r line; do
        # Parse JSON and show progress
        if echo "$line" | grep -q '"status":"pulling"'; then
            # Extract progress information
            completed=$(echo "$line" | grep -o '"completed":[0-9]*' | cut -d: -f2)
            total=$(echo "$line" | grep -o '"total":[0-9]*' | cut -d: -f2)
            
            if [ -n "$completed" ] && [ -n "$total" ] && [ "$total" -gt 0 ]; then
                # Calculate percentage
                percent=$((completed * 100 / total))
                # Show progress bar
                printf "\r   Downloading $MODEL: [%-50s] %d%%" $(printf "#%.0s" $(seq 1 $((percent/2)))) $percent
            fi
        fi
    done
    echo ""
    echo -e "${GREEN}✅ $MODEL model downloaded successfully${NC}"
fi


# Download Commayte binary
echo -e "${YELLOW}📦 Downloading Commayte binary...${NC}"

# Determine the correct binary for this system
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

# Get latest release URL
REPO="aymenhamada/Commayte"
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    echo -e "${RED}❌ Could not find latest release${NC}"
    echo -e "${YELLOW}💡 Make sure the repository exists and has releases${NC}"
    exit 1
fi

# Download the tar.gz file and extract the correct binary
TAR_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/commayte-$LATEST_RELEASE.tar.gz"

echo -e "${YELLOW}📥 Downloading release archive...${NC}"

# Download the tar.gz file
if ! curl -L -o "/tmp/commayte-$LATEST_RELEASE.tar.gz" "$TAR_URL"; then
    echo -e "${RED}❌ Failed to download release archive${NC}"
    echo -e "${YELLOW}💡 Check your internet connection and try again${NC}"
    exit 1
fi

# Extract the correct binary
echo -e "${YELLOW}📦 Extracting binary...${NC}"
cd /tmp
tar -xzf "commayte-$LATEST_RELEASE.tar.gz" "$BINARY/$BINARY"

if [ ! -f "/tmp/$BINARY/$BINARY" ]; then
    echo -e "${RED}❌ Failed to extract binary $BINARY/$BINARY${NC}"
    echo -e "${YELLOW}💡 Available binaries in archive:${NC}"
    tar -tzf "commayte-$LATEST_RELEASE.tar.gz" | grep commayte
    exit 1
fi

chmod +x "/tmp/$BINARY/$BINARY"

# Install to /usr/local/bin (requires sudo)
echo -e "${YELLOW}🔧 Installing to /usr/local/bin/commayte...${NC}"
sudo cp "/tmp/$BINARY/$BINARY" /usr/local/bin/commayte
sudo chmod +x /usr/local/bin/commayte

# Clean up
rm -rf "/tmp/$BINARY"
rm "/tmp/commayte-$LATEST_RELEASE.tar.gz"

echo -e "${GREEN}✅ Commayte installed successfully!${NC}"

# Ensure /usr/local/bin is in PATH
echo -e "${YELLOW}🔍 Checking PATH configuration...${NC}"
if [[ ":$PATH:" != *":/usr/local/bin:"* ]]; then
    echo -e "${YELLOW}⚠️  /usr/local/bin not found in PATH${NC}"
    
    # Detect shell and add to appropriate config file
    SHELL_CONFIG=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        SHELL_CONFIG="$HOME/.bashrc"
    fi
    
    if [ -n "$SHELL_CONFIG" ]; then
        echo -e "${YELLOW}📝 Adding /usr/local/bin to PATH in $SHELL_CONFIG${NC}"
        echo "" >> "$SHELL_CONFIG"
        echo "# Commayte PATH configuration" >> "$SHELL_CONFIG"
        echo 'export PATH="/usr/local/bin:$PATH"' >> "$SHELL_CONFIG"
        echo -e "${GREEN}✅ PATH updated. Please restart your terminal or run: source $SHELL_CONFIG${NC}"
    else
        echo -e "${YELLOW}⚠️  Please add /usr/local/bin to your PATH manually${NC}"
        echo -e "${YELLOW}💡 Add this line to your shell config: export PATH=\"/usr/local/bin:\$PATH\"${NC}"
    fi
else
    echo -e "${GREEN}✅ /usr/local/bin is already in PATH${NC}"
fi

# Create configuration directory
mkdir -p ~/.config/commayte

# Create configuration
echo -e "${YELLOW}⚙️  Setting up configuration...${NC}"

# Create the configuration file
cat > ~/.config/commayte/config.toml << EOF
# Commayte Configuration
model = "$MODEL"
EOF

echo -e "${GREEN}✅ Configuration created at ~/.config/commayte/config.toml${NC}"
echo -e "${BLUE}📋 Current settings:${NC}"
echo -e "  Model: $MODEL"

# Pre-load the model for faster first use
echo -e "${YELLOW}🚀 Pre-loading $MODEL model for faster first use...${NC}"
echo -e "${YELLOW}⏳ This may take a moment...${NC}"

# Send a simple request to load the model into memory
curl -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"$MODEL\", \"prompt\": \"hello\", \"stream\": false}" > /dev/null 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ $MODEL model pre-loaded successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Model pre-loading failed, but this won't affect functionality${NC}"
fi

# Test the installation
echo -e "${YELLOW}🧪 Testing installation...${NC}"
if command_exists commayte; then
    echo -e "${GREEN}✅ Commayte is ready to use!${NC}"
    echo -e "${BLUE}💡 Usage: commayte${NC}"
    echo -e "${BLUE}💡 Make sure you have staged changes: git add .${NC}"
else
    echo -e "${YELLOW}⚠️  Commayte not found in current PATH${NC}"
    echo -e "${YELLOW}💡 Try restarting your terminal or run: source ~/.bashrc${NC}"
    echo -e "${YELLOW}💡 Or run directly: /usr/local/bin/commayte${NC}"
fi

echo -e "${GREEN}🎉 Installation complete!${NC}"
echo -e "${BLUE}📖 For more information, visit: https://github.com/aymenhamada/Commayte${NC}" 