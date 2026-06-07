<div align="center">

# SteamVR-Simple-Launcher

[![English](https://img.shields.io/badge/English-blue)](README_EN.md) [![中文](https://img.shields.io/badge/中文-red)](README.md)

</div>

## ❓ 这是什么？

SteamVR-Simple-Launcher 是一个轻量级的 SteamVR 启动器，专为 Windows 用户设计。它可以帮助你快速检测 SteamVR 安装路径、切换 Steam 语言、创建桌面快捷方式，并一键启动 SteamVR。

## ✨ 有什么特点？

- 🔍 **自动检测**：从 Windows 注册表自动读取 Steam 安装路径，无需手动配置
- 🌐 **语言切换**：支持切换 Steam 客户端语言（中文、英文、日文等多种语言）
- 🔗 **快捷方式**：一键创建桌面快捷方式，方便快速启动 SteamVR
- 🚀 **一键启动**：直接启动 SteamVR，无需打开 Steam 客户端
- 📦 **单文件分发**：整个应用打包为单个 exe 文件，Windows App SDK 运行时自动嵌入
- 💼 **便携式**：运行时文件释放到 exe 同级目录，不依赖系统安装

## 🛠️ 技术栈

- 🦀 **Rust**：系统级编程语言，保证性能和安全性
- 📋 **Windows Reactor**：WinUI/WinAppSDK GUI 框架，原生 Windows 体验
- 📋 **Windows Registry**：注册表读写，检测 Steam 安装路径
- 📁 **Windows App SDK**：Self-contained 部署，运行时嵌入 exe
- 🔗 **lnks**：.lnk 快捷方式创建
- 📂 **rfd**：跨平台文件/文件夹选择对话框
- 📂 **dirs**：系统目录路径获取（桌面、配置等）
- 🏗️ **winres**：build-time exe 资源注入（图标）

## 🚀 如何安装？

### 方式一：直接下载（推荐）

1. 从 [Releases](https://github.com/yihuishou/SteamVR-Simple-Lunch/releases) 下载最新版本
2. 将 `steamvr_launcher.exe` 放到任意目录
3. 双击运行即可

### 方式二：从源码编译

```powershell
# 确保已安装 Rust 工具链
git clone https://github.com/yihuishou/SteamVR-Simple-Launcher.git
cd SteamVR-Simple-Launcher
cargo build --release
```

编译产物位于 `target/release/steamvr_launcher.exe`

## ⚡ 快速使用？

1. **启动应用**：双击 `steamvr_launcher.exe`
2. **检测 SteamVR**：应用会自动检测 Steam 安装路径
   - 如果检测成功，显示 ✅ 检测到 SteamVR 路径
   - 如果检测失败，点击「选择 SteamVR 安装路径」手动指定
3. **切换语言**（可选）：从下拉菜单选择 desired 语言，点击「应用语言」
4. **创建快捷方式**（可选）：点击「创建桌面快捷方式」
5. **启动 SteamVR**：点击「启动 SteamVR」按钮

## 📖 详细文档在哪里？

详见 [AGENTS.md](AGENTS.md)，包含：
- 项目架构说明
- 代码规范
- 构建指南
- 依赖说明
- 测试说明

## 🤝 如何贡献？

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 用什么许可证？

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件