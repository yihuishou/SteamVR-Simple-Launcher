# SteamVR-Simple-Lunch

Rust/eframe GUI 启动器，用于检测 SteamVR 路径、切换 Steam 语言、创建桌面快捷方式、一键启动 SteamVR。

## Architecture

```
main.rs          # 入口，eframe 初始化，CJK 字体加载，窗口图标
app.rs           # SteamVrApp 状态机 + 全部 egui UI 渲染（单文件 UI）
steam_path.rs    # 注册表读取 Steam 安装路径，递归搜索 vrstartup.exe
steam_language.rs # 读写 HKCU\Software\Valve\Steam\Language
shortcut_manager.rs # 桌面 .lnk 快捷方式创建，内嵌 ICO 持久化提取
```

单线程 GUI 应用，无网络、无数据库、无多线程。所有 IO 同步阻塞在主线程。

## Key Conventions

- **Windows 优先**：注册表路径硬编码反斜杠，`winreg` crate 操作 HKCU/HKLM
- **中文 UI**：`main.rs` 加载系统 CJK 字体，全局 1.2x 缩放
- **内嵌资源**：`assets/SteamVRIcon.ico` 编译时通过 `include_bytes!` 嵌入，运行时提取至 `%APPDATA%\SteamVRLauncher\`
- **路径安全**：启动前校验 `is_absolute()` 防路径注入
- **防重复点击**：`is_working` 标志锁住所有按钮
- **Toast 通知**：6 秒自消，`app.rs` 内部计时器驱动

## Build

```powershell
cargo run                      # 调试运行
cargo run --release            # 发布运行
cargo test                     # 运行内联单元测试
```

Release profile: `strip=true`, `opt-level="z"`, `LTO`, 单 codegen unit。

Release profile 配置在 `Cargo.toml` `[profile.release]`。图标资源通过 `build.rs` + `winres` 注入 exe 资源段。

## Dependencies

| Crate | Purpose |
|-------|---------|
| eframe 0.31 | native GUI 框架 |
| egui 0.31 | IMGUI 渲染 |
| winreg 0.52 | Windows 注册表读写 |
| lnks 0.2 | .lnk 快捷方式创建 |
| rfd 0.15 | 跨平台文件/文件夹选择对话框 |
| image 0.25 | .ico 转 .png 供 eframe 窗口图标使用 |
| dirs 5 | 桌面、配置目录路径获取 |
| winres 0.1 | build-time exe 资源注入 |

## Testing

3 个模块包含 `#[cfg(test)]` 内联测试：`steam_path.rs`、`steam_language.rs`、`shortcut_manager.rs`。测试不依赖外部状态（注册表不存在时 graceful fallback）。

## File Roles

| File | Responsibility |
|------|----------------|
| `main.rs` | 入口、字体、图标、eframe 引导 |
| `app.rs` | 应用状态、UI 布局、所有按钮回调 |
| `steam_path.rs` | Steam 路径检测（注册表 + 文件系统） |
| `steam_language.rs` | Steam 语言读写（注册表） |
| `shortcut_manager.rs` | 桌面快捷方式生命周期管理 |
| `build.rs` | Windows exe 资源编译配置 |
| `assets/` | 图标资源（SteamVRIcon.ico） |
