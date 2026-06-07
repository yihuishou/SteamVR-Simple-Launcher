use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // 配置 Windows App SDK self-contained 部署（会下载运行时到 target 目录）
    windows_reactor_setup::as_self_contained();

    // 设置 Windows 资源（图标）
    #[cfg(windows)]
    {
        use winres::WindowsResource;
        WindowsResource::new()
            .set_icon("assets/SteamVRIcon.ico")
            .compile()
            .expect("Failed to compile Windows resources");
    }

    // 生成嵌入式运行时数据（仅精简语言版本）
    generate_runtime_embed();
}

/// 需要嵌入的文件列表（仅 DLL/PRI + zh-CN/en-us 语言资源）
const EMBED_FILES: &[&str] = &[
    // 运行时 DLL 和资源文件
    "CoreMessagingXP.dll",
    "dcompi.dll",
    "dwmcorei.dll",
    "DwmSceneI.dll",
    "DWriteCore.dll",
    "marshal.dll",
    "Microsoft.DirectManipulation.dll",
    "Microsoft.Graphics.Imaging.dll",
    "Microsoft.InputStateManager.dll",
    "Microsoft.Internal.FrameworkUdk.dll",
    "Microsoft.UI.Composition.OSSupport.dll",
    "Microsoft.UI.dll",
    "Microsoft.UI.Input.dll",
    "Microsoft.UI.pri",
    "Microsoft.UI.Windowing.Core.dll",
    "Microsoft.UI.Windowing.dll",
    "Microsoft.UI.Xaml.Controls.dll",
    "Microsoft.UI.Xaml.Controls.pri",
    "Microsoft.ui.xaml.dll",
    "Microsoft.UI.Xaml.Internal.dll",
    "Microsoft.UI.Xaml.Phone.dll",
    "Microsoft.ui.xaml.resources.19h1.dll",
    "Microsoft.ui.xaml.resources.common.dll",
    "Microsoft.Windows.ApplicationModel.Resources.dll",
    "Microsoft.WindowsAppRuntime.dll",
    "Microsoft.WindowsAppRuntime.pri",
    "MRM.dll",
    "resources.pri",
    "SessionHandleIPCProxyStub.dll",
    "WinUIEdit.dll",
    "wuceffectsi.dll",
    // 中文资源
    "zh-CN\\Microsoft.ui.xaml.dll.mui",
    "zh-CN\\Microsoft.UI.Xaml.Phone.dll.mui",
    // 英文资源
    "en-us\\Microsoft.ui.xaml.dll.mui",
    "en-us\\Microsoft.UI.Xaml.Phone.dll.mui",
];

/// 扫描 target 目录，生成嵌入式运行时数据文件
fn generate_runtime_embed() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap_or(&out_dir);

    // 收集实际存在的文件
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    for file_name in EMBED_FILES {
        let path = target_dir.join(file_name);
        if path.exists() {
            // 使用正斜杠作为 key（运行时用）
            let key = file_name.replace('\\', "/");
            entries.push((key, path));
        } else {
            println!("cargo:warning=嵌入文件不存在: {}", path.display());
        }
    }

    // 生成 runtime_data.rs
    let mut code = String::from(
        "// 自动生成的运行时嵌入数据（勿手动编辑）\n\
         // 由 build.rs 在构建时生成\n\n\
         /// 嵌入的运行时文件列表\n\
         /// 每项为 (相对路径, 文件字节数据)\n\
         pub const RUNTIME_FILES: &[(&str, &[u8])] = &[\n",
    );

    for (key, path) in &entries {
        // 使用 r\"...\" 原始字符串避免反斜杠转义问题
        code.push_str(&format!("    (\"{key}\", include_bytes!(r\"{}\")),\n", path.display()));
    }

    code.push_str("];\n");

    let dest = out_dir.join("runtime_data.rs");
    fs::write(&dest, code).expect("无法写入 runtime_data.rs");

    println!("cargo:rerun-if-changed=build.rs");
}
