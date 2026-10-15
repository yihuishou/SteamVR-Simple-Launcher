# Windows 10 兼容问题：STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)

本文档记录 SteamVR-Simple-Launcher 从 windows-reactor git 版本升级到 crates.io `0.100.0` 后，在 Windows 10 上启动失败的原因、现象与修复方案。

## 现象

在 Windows 10 22H2（10.0.19045）上运行 `cargo run` 或 `cargo run --release`，进程在加载期立即退出：

```
error: run failed: exit code: 0xc0000139 (STATUS_ENTRYPOINT_NOT_FOUND)
```

系统错误框提示（WER 未上报时事件日志无记录）：

```
steamvr_launcher.exe - 无法找到入口
无法定位程序输入点 TryCreatePackageDependency 于动态链接库 ...steamvr_launcher.exe 上。
```

关键特征：**进程在 `main()` 执行前就死了**，窗口从未显示。旧版（git commit `cf58d5c`）在同一台机器上能正常跑。

## 原因

1. **windows-reactor 0.100.0 静态早期绑定了两个 Win11 专属 AppModel API**
   `windows-reactor 0.100.0` 通过 `windows-core 0.100.0` 的 `windows_core::link!` 宏，在 `src/native/winui/bindings.rs` 里以**静态导入**方式绑定：
   - `TryCreatePackageDependency`
   - `AddPackageDependency`

   这两个函数的用途是把 `Microsoft.WindowsAppRuntime.2` 框架包加入进程包图——即**框架依赖（framework-dependent）引导**的标准做法。

2. **这两个函数是 Windows 11（10.0.22000）才有的**
   微软文档明确：`TryCreatePackageDependency` / `AddPackageDependency` 的 Minimum supported client 为 **Windows 11（introduced in 10.0.22000.0）**。Windows 10 任何版本（包括 22H2）的 `api-ms-win-appmodel-runtime-l1-1-5.dll` 都没有这两个导出，也没有任何可再发行组件能补上。

3. **Windows 加载器在 `main()` 前解析静态导入表**
   这两个函数名进了 exe 的 PE 导入表（IAT）。Windows 10 加载器在进程启动、`main()` 执行之前就要解析所有静态导入，找不到入口点就直接以 `0xc0000139` 退出——连自包含分支都进不去。

4. **运行时本身支持 Win10，不支持的只是 Reactor 的引导方式**
   机器上装的 Windows App Runtime 自己的 MSIX 清单写的是 `<TargetDeviceFamily Name="Windows.Universal" MinVersion="10.0.17763.0" />`，即**运行时支持 Windows 10 1809+**。问题不在运行时，而在 Reactor 走的那条框架依赖引导路径只有 Windows 11 24H1+ 走得通。

5. **上游问题**
   - 破坏性 commit `5419f50` 引入该问题：[microsoft/windows-rs#4866](https://github.com/microsoft/windows-rs/issues/4866)（"commit 5419f50 breaks framework-dependent apps on Windows builds below 22000"，同错 0xc0000139）
   - 上游修复 PR [microsoft/windows-rs#4873](https://github.com/microsoft/windows-rs/pull/4873)（改成运行时解析/动态加载）**已关闭未合并**
   - 相关：[microsoft/windows-rs#4692](https://github.com/microsoft/windows-rs/issues/4692)

## 修复方案

采用「**自包含部署 + `/DELAYLOAD` 延迟加载**」两手组合，缺一不可。

### 1) 自包含部署（原本已有）

`build.rs` 调用 `windows_reactor_setup::as_self_contained()`，编译时把 Windows App Runtime（2.4.0）铺到 `target\` 并按自包含标记嵌入清单。设置界面只用 exe 旁边这份运行时，不再走那条只有 Win11 走得通的框架依赖引导。

> 但**只换部署方式不够**——IAT 里仍留着那两个函数名，Windows 10 会在加载期就拒绝启动。

### 2) `/DELAYLOAD` 延迟加载（本次新增）

在 `build.rs` 的 `as_self_contained()` 之后，针对 MSVC 目标追加两行链接器参数：

```rust
if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
    println!("cargo:rustc-link-arg-bins=delayimp.lib");
    println!("cargo:rustc-link-arg-bins=/DELAYLOAD:api-ms-win-appmodel-runtime-l1-1-5.dll");
}
```

作用：把 `TryCreatePackageDependency` / `AddPackageDependency` 从导入表（IAT）挪进**延迟加载描述符**。这样 Windows 10 加载器在加载期不再因为找不到入口点而拒绝启动；而在自包含部署下，这两个函数**根本不会被调用**，所以延迟加载描述符也不会触发。

### 为什么守卫用 `msvc` 而不是 `windows`

`/DELAYLOAD` 和 `delayimp.lib` 是 **MSVC 链接器专属**参数。若交叉编译到 `windows-gnu` 目标，塞 MSVC 参数会直接构建失败。因此守卫条件用 `CARGO_CFG_TARGET_ENV == "msvc"`（而非 `CARGO_CFG_TARGET_OS == "windows"`）更精确。

### 参考实现

[qingjian-team/qingjian PR #18](https://github.com/qingjian-team/qingjian/pull/18)（2026-09-13 合并）采用**完全相同**的两手修法：`as_self_contained()` + `/DELAYLOAD:api-ms-win-appmodel-runtime-l1-1-5.dll` + `delayimp.lib`，根因定位与上游 issue 引用也一致。差异仅在守卫条件（它用 `windows`、本文用 `msvc`）和分发方式（它随安装包带运行时、本文用 build.rs 嵌入 exe）。

## 验证

- 重建后 exe 存活超过 8 秒 = 窗口已显示，不再在加载期退出（修复前 0 秒即死）
- `dumpbin /imports` 确认 `TryCreatePackageDependency` / `AddPackageDependency` 已从静态导入表移入延迟加载段
- 反证：去掉两行 `/DELAYLOAD` 重新编译，同一台 Win10 机器会重新以 0xc0000139 在加载期失败

## 代价与影响

- exe 多依赖 `delayimp.lib`（MSVC 运行时自带，无额外体积）
- 自包含部署使安装包/分发体积增加约 56 MB 运行时文件（LZMA2 压缩后更小），换来不依赖机器是否安装框架包
- 升级 `windows-reactor` / `windows-reactor-setup` 时，需照新版 crate 的 `assets/runtime.txt` 核对 `build.rs` 的 `EMBED_FILES` 清单

## 相关文件

| 文件 | 作用 |
|------|------|
| `build.rs` | 自包含部署 + `/DELAYLOAD` 链接参数（MSVC 守卫） |
| `Cargo.toml` | `windows-reactor = "0.100"`、`windows-reactor-setup = "0.100"`（build-dep） |
| `src/main.rs` | 运行时释放 + windows-reactor 引导 |
