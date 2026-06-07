#![windows_subsystem = "windows"]

mod app;
mod shortcut_manager;
mod steam_language;
mod steam_path;

// 包含构建时生成的嵌入式运行时数据
include!(concat!(env!("OUT_DIR"), "/runtime_data.rs"));

fn main() {
    // 第一步：释放嵌入的运行时文件到 exe 同级目录
    extract_runtime();

    // 第二步：启动 Windows Reactor 应用
    // DLL 搜索：LOAD_LIBRARY_SEARCH_APPLICATION_DIR 会自动搜索 exe 所在目录
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

/// 将嵌入的运行时文件释放到 exe 同级目录
fn extract_runtime() {
    use std::fs;

    // 获取 exe 所在目录
    let exe_dir = std::env::current_exe()
        .expect("无法获取可执行文件路径")
        .parent()
        .expect("无法获取可执行文件目录")
        .to_path_buf();

    // 检查是否已经释放过（用第一个文件作为标记）
    let marker_file = exe_dir.join(RUNTIME_FILES[0].0);
    if marker_file.exists() {
        return;
    }

    // 首次运行，释放所有文件
    for (name, data) in RUNTIME_FILES {
        let file_path = exe_dir.join(name);

        // 创建子目录（如 zh-CN、en-us）
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("无法创建子目录");
        }

        fs::write(&file_path, data).expect(&format!("无法写入文件: {}", name));
    }
}
