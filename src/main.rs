#![windows_subsystem = "windows"]

mod app;
mod shortcut_manager;
mod steam_language;
mod steam_path;

fn main() {
    // 启动 Windows Reactor 应用
    // App::render 返回 windows_reactor::Result（与 windows_core::Result 是不同类型）
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
