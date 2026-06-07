# SteamVR-Simple-Lunch

Rust/windows-reactor GUI 启动器，用于检测 SteamVR 路径、切换 Steam 语言、创建桌面快捷方式、一键启动 SteamVR。

## Architecture

```
main.rs          # 入口，释放运行时文件，启动 windows-reactor 应用
app.rs           # 全部 WinUI UI 渲染（单文件 UI）
steam_path.rs    # 注册表读取 Steam 安装路径，验证 vrstartup.exe
steam_language.rs # 读写 HKCU\Software\Valve\Steam\Language
shortcut_manager.rs # 桌面 .lnk 快捷方式创建，内嵌 ICO 持久化提取
build.rs         # Windows App SDK self-contained 配置，嵌入运行时数据生成
```

单线程 GUI 应用，无网络、无数据库、无多线程。所有 IO 同步阻塞在主线程。

## Key Conventions

- **Windows 优先**：注册表路径硬编码反斜杠，`windows-registry` crate 操作 HKCU/HKLM
- **中文 UI**：WinUI XAML 原生支持 CJK
- **内嵌资源**：构建时通过 `build.rs` 将 Windows App SDK 运行时 DLL 嵌入 exe，运行时释放到 exe 同级目录
- **DLL 加载**：利用 `LOAD_LIBRARY_SEARCH_APPLICATION_DIR` 自动搜索 exe 所在目录，无需额外 API 调用
- **路径安全**：启动前校验 `is_absolute()` 防路径注入
- **防重复点击**：`is_working` 标志锁住所有按钮
- **Toast 通知**：6 秒自消，`app.rs` 内部计时器驱动

## Build

```powershell
cargo run                      # 调试运行
cargo run --release            # 发布运行
cargo test                     # 运行内联单元测试
cargo fmt                      # 格式化代码
```

Release profile: `strip=true`, `opt-level="z"`, `LTO`, 单 codegen unit, `panic="abort"`。

Release profile 配置在 `Cargo.toml` `[profile.release]`。图标资源通过 `build.rs` + `winres` 注入 exe 资源段。

## Dependencies

| Crate | Purpose |
|-------|---------|
| windows-reactor | WinUI/WinAppSDK GUI 框架（git master） |
| windows-core 0.58 | Windows 运行时核心类型 |
| windows-registry 0.6 | Windows 注册表读写 |
| windows-result 0.2 | Windows 运行时错误处理 |
| lnks 0.2 | .lnk 快捷方式创建 |
| rfd 0.15 | 跨平台文件/文件夹选择对话框 |
| dirs 5 | 桌面、配置目录路径获取 |
| winres 0.1 | build-time exe 资源注入 |

Build-dependencies:
| windows-reactor-setup | Windows App SDK self-contained 部署配置（git master） |

## Testing

3 个模块包含 `#[cfg(test)]` 内联测试：`steam_path.rs`、`steam_language.rs`、`shortcut_manager.rs`。测试不依赖外部状态（注册表不存在时 graceful fallback）。

## File Roles

| File | Responsibility |
|------|----------------|
| `main.rs` | 入口、运行时释放、windows-reactor 引导 |
| `app.rs` | 应用状态、UI 布局、所有按钮回调 |
| `steam_path.rs` | Steam 路径检测（注册表 + 文件验证） |
| `steam_language.rs` | Steam 语言读写（注册表） |
| `shortcut_manager.rs` | 桌面快捷方式生命周期管理 |
| `build.rs` | Windows App SDK 配置、运行时数据嵌入、exe 资源注入 |
