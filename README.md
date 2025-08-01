# 🚀 Commayte

AI-powered git commit message generator with a beautiful interactive CLI.

![Status](https://img.shields.io/badge/Status-Ready-green)
![Rust](https://img.shields.io/badge/Rust-1.70+-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

- 🤖 **AI-Powered**: Uses Mistral model for intelligent commit messages
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
- 📥 Download Mistral model
- 📦 Install Commayte binary
- ⚙️ Configure everything automatically

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