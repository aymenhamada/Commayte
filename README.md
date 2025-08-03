# 🚀 Commayte

AI-powered git commit message generator with a beautiful interactive CLI.

![Status](https://img.shields.io/badge/Status-Ready-green)
![Rust](https://img.shields.io/badge/Rust-1.70+-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

- 🤖 **AI-Powered**: Uses Local AI LLM model for intelligent commit messages
- 🎨 **Beautiful UI**: Interactive CLI with spinners and colors
- 📝 **Conventional Commits**: Follows the conventional commit specification
- 🔄 **Interactive**: Regenerate messages until you're satisfied
- 🚀 **Zero Setup**: Automatic local Ollama installation

## 🚀 Quick Install

**One command to install everything:**

```bash
curl -fsSL https://github.com/aymenhamada/Commayte/releases/latest/download/install.sh | bash
```

That's it! The installer will:
- ✅ Check prerequisites (Git, curl)
- 📦 Install Ollama locally
- 🤖 **Interactive model selection** (Phi3:latest or Mistral)
- 📥 Download your chosen model
- 📦 Install Commayte binary
- ⚙️ Configure everything automatically

**Supported Platforms:**
- 🐧 Linux (x86_64)
- 🍎 macOS (Intel & Apple Silicon)
- 🪟 Windows (x86_64, via Git Bash/WSL)

## 🗑️ Uninstall

**To completely remove Commayte:**

```bash
curl -fsSL https://github.com/aymenhamada/Commayte/releases/latest/download/uninstall.sh | bash
```

The uninstaller will:
- 🗑️ Remove Commayte binary
- ⚙️ Clean up configuration files
- 🛑 Stop Ollama service
- 🧹 Clean up PATH configuration
- 🤖 Optionally remove Ollama completely

### Model Selection

The installer runs in **interactive mode** to let you choose your preferred AI model:
- **Mistral** (default): Better quality, more consuming
- **Phi3:latest**: Good balance of speed and quality

### Manual Configuration

If you prefer a different model or want to change later, you can manually edit the configuration:

```bash
# Edit the config file
nano ~/.config/commayte/config.toml

# Change the model property
model = "your-preferred-model"
```

Then download your chosen model:
```bash
ollama pull your-preferred-model
```

## 📖 Usage

1. **Stage your changes:**
   ```bash
   git add .
   ```

2. **Generate commit message:**
   ```bash
   commayte
   ```

3. **Choose your action:**
   - ✅ Accept and commit
   - 🔄 Regenerate message
   - ❌ Cancel

## 📖 Example

```bash
# Before
git commit -m "fix stuff"

# After
commayte
# Generated: fix(client): resolve authentication token validation
# ✅ Accept and commit
```

## 🛠️ Development

For developers who want to build from source:

```bash
git clone https://github.com/aymenhamada/Commayte.git
cd Commayte
cargo build --release
```

**Note:** End users should use the one-line installer above. Building from source is only for contributors.

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

---

Made with ❤️ by [Aymen Hamada](https://github.com/aymenhamada) 