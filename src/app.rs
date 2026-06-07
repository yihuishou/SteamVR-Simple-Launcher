use std::process::Command;
use std::path::Path;

use windows_reactor::{
    Element, RenderCx, Color, Brush, Thickness, ElementExt,
    text_block, button, vstack, border, ComboBox,
};

use crate::steam_path::{self, SteamPaths};
use crate::steam_language::{self, LANGUAGES};
use crate::shortcut_manager;

/// 主应用组件 — 接收 RenderCx 引用以使用 hooks
pub fn steam_vr_launcher(cx: &mut RenderCx) -> Element {
    // 状态钩子 — use_state 接受初始值（非闭包），返回 (T, SetState<T>)
    let (steam_paths, set_steam_paths) = cx.use_state(steam_path::detect_steam_path());
    let (current_lang, set_current_lang) = cx.use_state(
        steam_language::read_steam_language().unwrap_or_else(|_| "english".to_string()),
    );
    let (selected_idx, set_selected_idx) = cx.use_state({
        let lang = current_lang.clone();
        LANGUAGES
            .iter()
            .position(|(_, val)| *val == lang.as_str())
            .unwrap_or(0) as i32
    });
    let (toast, set_toast) = cx.use_state(None::<(String, bool)>);
    let (working, set_working) = cx.use_state(false);

    // ── 事件回调 ──────────────────────────────────────────────

    // 重新检测 SteamVR 路径
    let detect_steam = {
        let set_steam_paths = set_steam_paths.clone();
        let set_toast = set_toast.clone();
        let set_working = set_working.clone();
        move || {
            set_working.call(true);
            let new_paths = steam_path::detect_steam_path();
            if new_paths.is_some() {
                set_toast.call(Some(("✅ 检测到 SteamVR 路径".to_string(), true)));
            } else {
                set_toast.call(Some(("❌ 未检测到 SteamVR，请手动指定路径".to_string(), false)));
            }
            set_steam_paths.call(new_paths);
            set_working.call(false);
        }
    };

    // 应用手动路径
    let apply_manual_path = {
        let set_steam_paths = set_steam_paths.clone();
        let set_toast = set_toast.clone();
        let set_working = set_working.clone();
        move |path: String| {
            set_working.call(true);

            if !Path::new(&path).is_absolute() {
                set_toast.call(Some(("❌ 请选择有效的目录".to_string(), false)));
                set_working.call(false);
                return;
            }

            let steamvr_exe = format!(
                "{}\\steamapps\\common\\SteamVR\\bin\\win64\\vrstartup.exe",
                path
            );

            if Path::new(&steamvr_exe).exists() {
                set_steam_paths.call(Some(SteamPaths {
                    steamvr_path: path.clone(),
                    steamvr_exe,
                }));
                set_toast.call(Some(("✅ 路径验证成功".to_string(), true)));
            } else if let Some(found_paths) = steam_path::find_vrstartup_in_dir(&path) {
                set_steam_paths.call(Some(found_paths));
                set_toast.call(Some(("✅ 在子目录中找到 SteamVR".to_string(), true)));
            } else {
                set_toast.call(Some(("❌ 未找到 SteamVR，请确认目录正确".to_string(), false)));
            }

            set_working.call(false);
        }
    };

    // 创建桌面快捷方式
    let create_shortcut = {
        let current_paths = steam_paths.clone();
        let set_toast = set_toast.clone();
        let set_working = set_working.clone();
        move || {
            if let Some(ref paths) = current_paths {
                set_working.call(true);
                let working_dir = shortcut_manager::get_working_dir_from_exe(&paths.steamvr_exe);

                match shortcut_manager::create_desktop_shortcut(
                    &paths.steamvr_exe,
                    &working_dir,
                ) {
                    Ok(()) => set_toast.call(Some(("✅ 桌面快捷方式创建成功".to_string(), true))),
                    Err(e) => {
                        set_toast.call(Some((format!("❌ 创建快捷方式失败: {}", e), false)));
                    }
                }
                set_working.call(false);
            }
        }
    };

    // 应用语言更改
    let apply_language = {
        let set_current_lang = set_current_lang.clone();
        let selected_idx_val = selected_idx;
        let set_toast = set_toast.clone();
        let set_working = set_working.clone();
        move || {
            let lang_value = LANGUAGES[selected_idx_val as usize].1;
            set_working.call(true);

            match steam_language::write_steam_language(lang_value) {
                Ok(()) => {
                    set_current_lang.call(lang_value.to_string());
                    set_toast.call(Some(("✅ 语言已更改，需重启 Steam 生效".to_string(), true)));
                }
                Err(e) => {
                    set_toast.call(Some((format!("❌ 写入语言失败: {}", e), false)));
                }
            }
            set_working.call(false);
        }
    };

    // 启动 SteamVR
    let launch_steamvr = {
        let current_paths = steam_paths.clone();
        let set_toast = set_toast.clone();
        let set_working = set_working.clone();
        move || {
            if let Some(ref paths) = current_paths {
                set_working.call(true);

                if !Path::new(&paths.steamvr_exe).is_absolute() {
                    set_toast.call(Some(("❌ 路径不安全，拒绝启动".to_string(), false)));
                    set_working.call(false);
                    return;
                }

                match Command::new(&paths.steamvr_exe).spawn() {
                    Ok(_) => set_toast.call(Some(("✅ SteamVR 启动中...".to_string(), true))),
                    Err(e) => set_toast.call(Some((format!("❌ 启动失败: {}", e), false))),
                }
                set_working.call(false);
            }
        }
    };

    // ── UI 布局 ──────────────────────────────────────────────

    let has_steam = steam_paths.is_some();
    let is_working = working;

    // 区域 1: SteamVR 路径
    let path_section = border(vstack(vec![
        text_block("SteamVR 路径")
            .font_size(14.0)
            .foreground(Color::rgb(0, 0, 0))
            .bold()
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 6.0 })
            .into(),
        if let Some(ref paths) = steam_paths {
            text_block(&paths.steamvr_path)
                .foreground(Color::rgb(80, 200, 120))
                .into()
        } else {
            text_block("未检测到 SteamVR")
                .foreground(Color::rgb(220, 80, 80))
                .into()
        },
        if let Some(ref paths) = steam_paths {
            text_block(format!("SteamVR: {}", paths.steamvr_exe)).into()
        } else {
            text_block("请手动选择安装路径").into()
        },
        button("重新检测")
            .on_click(detect_steam)
            .enabled(!is_working)
            .margin(Thickness { left: 0.0, top: 6.0, right: 0.0, bottom: 6.0 })
            .into(),
        button("选择 SteamVR 安装路径")
            .on_click(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    apply_manual_path(path.to_string_lossy().to_string());
                }
            })
            .enabled(!is_working)
            .into(),
    ]))
    .border_brush(Brush::from(Color::rgb(80, 80, 80)))
    .border_thickness(Thickness { left: 1.0, top: 1.0, right: 1.0, bottom: 1.0 })
    .corner_radius(6.0)
    .padding(12.0)
    .margin(Thickness { left: 6.0, top: 6.0, right: 6.0, bottom: 6.0 });
    let shortcut_section = border(vstack(vec![
        text_block("桌面快捷方式")
            .font_size(14.0)
            .foreground(Color::rgb(0, 0, 0))
            .bold()
            .margin(Thickness { left: 0.0, top: 6.0, right: 0.0, bottom: 6.0 })
            .into(),
        if let Some(ref paths) = steam_paths {
            text_block(format!("目标: {}", paths.steamvr_exe)).into()
        } else {
            text_block("检测到 SteamVR 后可创建快捷方式").into()
        },
        if has_steam {
            button("创建桌面快捷方式")
                .on_click(create_shortcut)
                .enabled(!is_working)
                .into()
        } else {
            button("创建桌面快捷方式")
                .enabled(false)
                .into()
        },
    ]))
    .border_brush(Brush::from(Color::rgb(80, 80, 80)))
    .border_thickness(Thickness { left: 1.0, top: 1.0, right: 1.0, bottom: 1.0 })
    .corner_radius(6.0)
    .padding(12.0)
    .margin(Thickness { left: 6.0, top: 6.0, right: 6.0, bottom: 6.0 });

    // 区域 3: 语言设置
    let lang_names: Vec<String> = LANGUAGES.iter().map(|(name, _)| name.to_string()).collect();
    let lang_section = border(vstack(vec![
        text_block("语言设置")
            .font_size(14.0)
            .foreground(Color::rgb(0, 0, 0))
            .bold()
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 6.0 })
            .into(),
        ComboBox::new(lang_names)
            .selected_index(selected_idx)
            .on_selection_changed(set_selected_idx)
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 6.0 })
            .into(),
        button("应用更改")
            .on_click(apply_language)
            .enabled(!is_working)
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 6.0 })
            .into(),
        text_block("⚠️ 需重启 Steam 生效")
            .foreground(Color::rgb(220, 180, 60))
            .into(),
    ]))
    .border_brush(Brush::from(Color::rgb(80, 80, 80)))
    .border_thickness(Thickness { left: 1.0, top: 1.0, right: 1.0, bottom: 1.0 })
    .corner_radius(6.0)
    .padding(12.0)
    .margin(Thickness { left: 6.0, top: 6.0, right: 6.0, bottom: 6.0 });

    // 区域 4: 启动按钮
    let launch_btn = button("🚀 启动 SteamVR")
        .on_click(launch_steamvr)
        .enabled(has_steam && !is_working)
        .foreground(if has_steam {
            Color::rgb(255, 255, 255)
        } else {
            Color::rgb(128, 128, 128)
        })
        .background(if has_steam {
            Color::rgb(40, 120, 200)
        } else {
            Color::rgb(60, 60, 60)
        })
        .margin(Thickness { left: 12.0, top: 6.0, right: 0.0, bottom: 6.0 });

    // Toast 通知
    let toast_el: Element = if let Some((ref message, success)) = toast {
        let color = if success {
            Color::rgb(80, 200, 120)
        } else {
            Color::rgb(220, 80, 80)
        };
        text_block(message)
            .foreground(color)
            .margin(Thickness { left: 12.0, top: 0.0, right: 0.0, bottom: 0.0 })
            .into()
    } else {
        text_block("").into()
    };

    // 组合所有区域
    vstack(vec![
        path_section
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 8.0 })
            .into(),
        shortcut_section
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 8.0 })
            .into(),
        lang_section
            .margin(Thickness { left: 0.0, top: 0.0, right: 0.0, bottom: 8.0 })
            .into(),
        launch_btn.into(),
        toast_el,
    ])
    .into()
}
