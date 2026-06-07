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
        .inner_size(420.0, 680.0)
        .render(app::steam_vr_launcher)
        .expect("应用启动失败");
}
