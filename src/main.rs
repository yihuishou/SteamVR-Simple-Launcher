#![windows_subsystem = "windows"]

mod app;
mod shortcut_manager;
mod steam_language;
mod steam_path;

// 包含构建时生成的嵌入式运行时数据
include!(concat!(env!("OUT_DIR"), "/runtime_data.rs"));

/// 嵌入的运行时数据目录名
const RUNTIME_DIR_NAME: &str = "runtime";

fn main() {
    // 第一步：释放嵌入的运行时文件到 AppData
    let runtime_dir = extract_runtime();

    // 第二步：设置 DLL 搜索路径（必须在 Windows App SDK 初始化之前）
    set_dll_directory(&runtime_dir);

    // 第三步：启动 Windows Reactor 应用
    windows_reactor::App::new()
        .title("SteamVR 启动器")
        .inner_size(500.0, 520.0)
        .inner_constraints(windows_reactor::InnerConstraints {
            min_width: Some(500.0),
            max_width: Some(500.0),
            min_height: Some(520.0),
            max_height: Some(520.0),
            ..Default::default()
        })
        .render(app::steam_vr_launcher)
        .expect("应用启动失败");
}

/// 将嵌入的运行时文件释放到 %APPDATA%\SteamVRLauncher\runtime\
/// 返回 runtime 目录路径
fn extract_runtime() -> std::path::PathBuf {
    use std::fs;

    // 获取 AppData\Roaming 路径
    let appdata = dirs::data_dir().expect("无法获取 AppData 目录");
    let runtime_dir = appdata.join("SteamVRLauncher").join(RUNTIME_DIR_NAME);

    // 检查是否已经释放过（用第一个文件作为标记）
    let marker_file = runtime_dir.join(RUNTIME_FILES[0].0);
    if marker_file.exists() {
        return runtime_dir;
    }

    // 首次运行，释放所有文件
    fs::create_dir_all(&runtime_dir).expect("无法创建运行时目录");

    for (name, data) in RUNTIME_FILES {
        let file_path = runtime_dir.join(name);

        // 创建子目录（如 zh-CN、en-us）
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("无法创建子目录");
        }

        fs::write(&file_path, data).expect(&format!("无法写入文件: {}", name));
    }

    runtime_dir
}

/// 设置 DLL 搜索路径
/// 使用 Windows API SetDllDirectoryW 将运行时目录添加到搜索路径
fn set_dll_directory(path: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // 使用 raw FFI 调用 SetDllDirectoryW
    extern "system" {
        fn SetDllDirectoryW(lpcwstr: *const u16) -> i32;
    }

    unsafe {
        SetDllDirectoryW(wide.as_ptr());
    }
}
