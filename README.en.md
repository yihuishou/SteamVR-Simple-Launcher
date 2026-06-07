# SteamVR-Simple-Launcher

[中文](README.md) | English

## ❓ What is this?

SteamVR-Simple-Launcher is a lightweight SteamVR launcher designed for Windows users. It helps you quickly detect SteamVR installation path, switch Steam language, create desktop shortcuts, and launch SteamVR with one click.

## ✨ Features

- 🔍 **Auto Detection**: Automatically reads Steam installation path from Windows registry, no manual configuration needed
- 🌐 **Language Switch**: Supports switching Steam client language (Chinese, English, Japanese, and more)
- 🔗 **Shortcut**: Create desktop shortcut with one click for quick SteamVR access
- 🚀 **One-Click Launch**: Launch SteamVR directly without opening Steam client
- 📦 **Single File Distribution**: Entire app packaged as a single exe file with Windows App SDK runtime embedded
- 💼 **Portable**: Runtime files extracted to exe directory, no system installation required

## 🛠️ Tech Stack

- 🦀 **Rust**: Systems programming language for performance and safety
- 📋 **Windows Reactor**: WinUI/WinAppSDK GUI framework for native Windows experience
- 📋 **Windows Registry**: Registry read/write for detecting Steam installation
- 📁 **Windows App SDK**: Self-contained deployment with runtime embedded in exe
- 🔗 **lnks**: .lnk shortcut creation
- 📂 **rfd**: Cross-platform file/folder selection dialog
- 📂 **dirs**: System directory path retrieval (Desktop, Config, etc.)
- 🏗️ **winres**: Build-time exe resource injection (icon)

## 🚀 Installation

### Option 1: Direct Download (Recommended)

1. Download the latest version from [Releases](https://github.com/yihuishou/SteamVR-Simple-Lunch/releases)
2. Place `steamvr_launcher.exe` in any directory
3. Double-click to run

### Option 2: Build from Source

```powershell
# Ensure Rust toolchain is installed
git clone https://github.com/yihuishou/SteamVR-Simple-Lunch.git
cd SteamVR-Simple-Lunch
cargo build --release
```

Build output is located at `target/release/steamvr_launcher.exe`

## ⚡ Quick Start

1. **Launch App**: Double-click `steamvr_launcher.exe`
2. **Detect SteamVR**: App will automatically detect Steam installation path
   - If detected successfully, shows ✅ SteamVR path detected
   - If detection fails, click "Select SteamVR Installation Path" to manually specify
3. **Switch Language** (Optional): Select desired language from dropdown, click "Apply Language"
4. **Create Shortcut** (Optional): Click "Create Desktop Shortcut"
5. **Launch SteamVR**: Click "Launch SteamVR" button

## 📖 Documentation

See [AGENTS.md](AGENTS.md) for detailed documentation including:
- Project architecture
- Code conventions
- Build guide
- Dependencies
- Testing

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to branch (`git push origin feature/AmazingFeature`)
5. Create a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
