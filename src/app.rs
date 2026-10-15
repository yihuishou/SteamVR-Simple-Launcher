use std::path::Path;
use std::process::Command;
use std::time::Duration;

use windows_reactor::{
    Border, Button, ButtonStyle, ChildrenControl, Color, Component, ComponentContext, ComboBox,
    ContentControl, CornerRadius, FontWeight, LayoutControl, StackPanel, TextBlock, Thickness,
    View, ViewContext, WindowConstraints, WindowVisuals,
};

use crate::shortcut_manager;
use crate::steam_language::{self, LANGUAGES};
use crate::steam_path::{self, load_config, save_config, SteamPaths};

/// 队列消息 — 0.100.0 中所有状态变更都通过消息驱动 update
#[derive(Clone)]
pub enum Message {
    DetectSteam,
    SelectFolder,
    CreateShortcut,
    ApplyLanguage,
    LaunchSteamVR,
    LanguageSelected(usize),
    /// Toast 自动消失（gen 防止旧计时器误清新 Toast）
    ToastExpired(usize),
}

/// 主应用组件
pub struct SteamVrApp {
    steam_paths: Option<SteamPaths>,
    current_lang: String,
    selected_idx: usize,
    toast: Option<(String, bool)>,
    working: bool,
    /// Toast 代际计数，用于 6 秒自动消失计时器
    toast_generation: usize,
}

impl Component for SteamVrApp {
    type Input = ();
    type Message = Message;

    fn create(_input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        // 优先使用已保存的配置文件；若不存在再回退到注册表检测
        let steam_paths = load_config().or_else(|| steam_path::detect_steam_path());
        let current_lang =
            steam_language::read_steam_language().unwrap_or_else(|_| "english".to_string());
        let selected_idx = LANGUAGES
            .iter()
            .position(|(_, val)| *val == current_lang.as_str())
            .unwrap_or(0);
        Self {
            steam_paths,
            current_lang,
            selected_idx,
            toast: None,
            working: false,
            toast_generation: 0,
        }
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        let generation_before = self.toast_generation;
        match message {
            Message::DetectSteam => {
                self.working = true;
                let new_paths = steam_path::detect_steam_path();
                if new_paths.is_some() {
                    // 保存检测到的路径到配置文件
                    save_config(new_paths.as_ref().unwrap());
                    self.set_toast("✅ 检测到 SteamVR 路径", true);
                } else {
                    self.set_toast("❌ 未检测到 SteamVR，请手动指定路径", false);
                }
                self.steam_paths = new_paths;
                self.working = false;
            }
            Message::SelectFolder => {
                // 同步弹出文件夹选择框（与旧版行为一致，运行在 UI 线程）
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.apply_manual_path(path.to_string_lossy().to_string());
                }
            }
            Message::CreateShortcut => {
                if let Some(ref paths) = self.steam_paths {
                    self.working = true;
                    let working_dir =
                        shortcut_manager::get_working_dir_from_exe(&paths.steamvr_exe);
                    match shortcut_manager::create_desktop_shortcut(
                        &paths.steamvr_exe,
                        &working_dir,
                    ) {
                        Ok(()) => self.set_toast("✅ 桌面快捷方式创建成功", true),
                        Err(e) => {
                            self.set_toast(&format!("❌ 创建快捷方式失败: {}", e), false)
                        }
                    }
                    self.working = false;
                }
            }
            Message::ApplyLanguage => {
                let lang_value = LANGUAGES[self.selected_idx].1;
                self.working = true;
                match steam_language::write_steam_language(lang_value) {
                    Ok(()) => {
                        self.current_lang = lang_value.to_string();
                        self.set_toast("✅ 语言已更改，需重启 Steam 生效", true);
                    }
                    Err(e) => self.set_toast(&format!("❌ 写入语言失败: {}", e), false),
                }
                self.working = false;
            }
            Message::LaunchSteamVR => {
                if let Some(ref paths) = self.steam_paths {
                    self.working = true;

                    if !Path::new(&paths.steamvr_exe).is_absolute() {
                        self.set_toast("❌ 路径不安全，拒绝启动", false);
                        self.working = false;
                        return;
                    }

                    match Command::new(&paths.steamvr_exe).spawn() {
                        Ok(_) => self.set_toast("✅ SteamVR 启动中...", true),
                        Err(e) => self.set_toast(&format!("❌ 启动失败: {}", e), false),
                    }
                    self.working = false;
                }
            }
            Message::LanguageSelected(index) => {
                self.selected_idx = index;
            }
            Message::ToastExpired(gen) => {
                // 仅当代际匹配时清除，避免旧计时器清掉新 Toast
                if gen == self.toast_generation {
                    self.toast = None;
                }
            }
        }
        // 若本次更新设置了新 Toast，启动 6 秒自动消失计时器
        //（spawn_background 在 Windows 线程池执行，不阻塞 UI；丢弃句柄不会取消任务）
        if self.toast_generation != generation_before {
            let gen = self.toast_generation;
            context.spawn_background(move |_| {
                std::thread::sleep(Duration::from_secs(6));
                Message::ToastExpired(gen)
            });
        }
    }

    fn view(&self, _input: &Self::Input, ctx: &mut ViewContext<Self>) -> View {
        // 窗口标题与固定 500x540 客户区
        //（替代旧版 App::new().title(...).inner_size(...).inner_constraints(...)）
        ctx.window_title("SteamVR 启动器");
        ctx.window_visuals(
            WindowVisuals::new()
                .client_size(500.0, 540.0)
                .constraints(WindowConstraints {
                    min_width: Some(500.0),
                    min_height: Some(540.0),
                    max_width: Some(500.0),
                    max_height: Some(540.0),
                }),
        );

        let has_steam = self.steam_paths.is_some();
        let is_working = self.working;

        // 区域 1: SteamVR 路径
        let path_section = Border::new()
            .border_brush(Color::rgb(80, 80, 80))
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(CornerRadius::uniform(6.0))
            .padding(Thickness::uniform(12.0))
            .margin(Thickness::new(8.0, 8.0, 8.0, 8.0))
            .content(StackPanel::new().children((
                TextBlock::new()
                    .text("SteamVR 路径")
                    .font_size(14.0)
                    .foreground(Color::rgb(0, 0, 0))
                    .font_weight(FontWeight::BOLD)
                    .margin(Thickness::new(0.0, 0.0, 0.0, 6.0)),
                if let Some(ref paths) = self.steam_paths {
                    TextBlock::new()
                        .text(&paths.steamvr_path)
                        .foreground(Color::rgb(80, 200, 120))
                } else {
                    TextBlock::new()
                        .text("❌ 未检测到 SteamVR")
                        .foreground(Color::rgb(220, 80, 80))
                },
                if let Some(ref paths) = self.steam_paths {
                    TextBlock::new().text(format!("SteamVR: {}", paths.steamvr_exe))
                } else {
                    TextBlock::new().text("请手动选择安装路径")
                },
                Button::new()
                    .is_enabled(!is_working)
                    .on_click(ctx.message(Message::DetectSteam))
                    .margin(Thickness::new(0.0, 6.0, 0.0, 6.0))
                    .content(TextBlock::new().text("重新检测")),
                Button::new()
                    .is_enabled(!is_working)
                    .on_click(ctx.message(Message::SelectFolder))
                    .content(TextBlock::new().text("选择 SteamVR 安装路径")),
            )));

        // 区域 2: 桌面快捷方式
        //（旧版按 has_steam 分两支：无 Steam 时按钮无回调且禁用；
        // 新版合并为 is_enabled(has_steam && !is_working)，禁用时回调不会触发，行为等价）
        let shortcut_section = Border::new()
            .border_brush(Color::rgb(80, 80, 80))
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(CornerRadius::uniform(6.0))
            .padding(Thickness::uniform(12.0))
            .margin(Thickness::new(8.0, 8.0, 8.0, 8.0))
            .content(StackPanel::new().children((
                TextBlock::new()
                    .text("桌面快捷方式")
                    .font_size(14.0)
                    .foreground(Color::rgb(0, 0, 0))
                    .font_weight(FontWeight::BOLD)
                    .margin(Thickness::new(0.0, 6.0, 0.0, 6.0)),
                if let Some(ref paths) = self.steam_paths {
                    TextBlock::new().text(format!("目标: {}", paths.steamvr_exe))
                } else {
                    TextBlock::new().text("检测到 SteamVR 后可创建快捷方式")
                },
                Button::new()
                    .is_enabled(has_steam && !is_working)
                    .on_click(ctx.message(Message::CreateShortcut))
                    .margin(Thickness::new(0.0, 6.0, 0.0, 0.0))
                    .content(TextBlock::new().text("创建桌面快捷方式")),
            )));

        // 区域 3: 语言设置
        let lang_names: Vec<String> = LANGUAGES.iter().map(|(name, _)| name.to_string()).collect();
        let lang_section = Border::new()
            .border_brush(Color::rgb(80, 80, 80))
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(CornerRadius::uniform(6.0))
            .padding(Thickness::uniform(12.0))
            .margin(Thickness::new(8.0, 8.0, 8.0, 8.0))
            .content(StackPanel::new().children((
                TextBlock::new()
                    .text("语言设置")
                    .font_size(14.0)
                    .foreground(Color::rgb(0, 0, 0))
                    .font_weight(FontWeight::BOLD)
                    .margin(Thickness::new(0.0, 0.0, 0.0, 6.0)),
                ComboBox::new()
                    .items_source(lang_names)
                    .selected_index(Some(self.selected_idx))
                    .on_selection_changed(ctx.callback(|index: Option<usize>| {
                        Message::LanguageSelected(index.unwrap_or(0))
                    }))
                    .margin(Thickness::new(0.0, 0.0, 0.0, 6.0)),
                Button::new()
                    .is_enabled(!is_working)
                    .on_click(ctx.message(Message::ApplyLanguage))
                    .margin(Thickness::new(0.0, 0.0, 0.0, 6.0))
                    .content(TextBlock::new().text("应用更改")),
                TextBlock::new()
                    .text("⚠️ 需重启 SteamVR 生效")
                    .foreground(Color::rgb(220, 180, 60)),
            )));

        // 区域 4: 启动按钮
        //（新版 Button 没有 foreground/background API，用 Accent 样式近似旧版蓝底白字；
        // 禁用时 WinUI 自动置灰，与旧版灰色外观一致）
        let launch_btn = Button::new()
            .is_enabled(has_steam && !is_working)
            .on_click(ctx.message(Message::LaunchSteamVR))
            .style(if has_steam {
                ButtonStyle::Accent
            } else {
                ButtonStyle::Default
            })
            .margin(Thickness::new(12.0, 6.0, 0.0, 6.0))
            .content(TextBlock::new().text("🚀 启动 SteamVR"));

        // Toast 通知
        let toast_el = if let Some((ref message, success)) = self.toast {
            let color = if success {
                Color::rgb(80, 200, 120)
            } else {
                Color::rgb(220, 80, 80)
            };
            TextBlock::new()
                .text(message)
                .foreground(color)
                .margin(Thickness::new(12.0, 0.0, 0.0, 0.0))
        } else {
            TextBlock::new().text("")
        };

        // 组合所有区域
        StackPanel::new().children((
            path_section,
            shortcut_section,
            lang_section,
            launch_btn,
            toast_el,
        ))
    }
}

impl SteamVrApp {
    /// 设置 Toast 并递增代际（计时器靠代际区分新旧 Toast）
    fn set_toast(&mut self, message: &str, success: bool) {
        self.toast_generation += 1;
        self.toast = Some((message.to_string(), success));
    }

    /// 验证并应用手动选择的安装路径（对应旧版 apply_manual_path 回调）
    fn apply_manual_path(&mut self, path: String) {
        self.working = true;

        if !Path::new(&path).is_absolute() {
            self.set_toast("❌ 请选择有效的目录", false);
            self.working = false;
            return;
        }

        let steamvr_exe = format!(
            "{}\\steamapps\\common\\SteamVR\\bin\\win64\\vrstartup.exe",
            path
        );

        if Path::new(&steamvr_exe).exists() {
            let new_paths = Some(SteamPaths {
                steamvr_path: path.clone(),
                steamvr_exe,
            });
            save_config(new_paths.as_ref().unwrap());
            self.steam_paths = new_paths;
            self.set_toast("✅ 路径验证成功", true);
        } else if let Some(found_paths) = steam_path::find_vrstartup_in_dir(&path) {
            save_config(&found_paths);
            self.steam_paths = Some(found_paths);
            self.set_toast("✅ 在子目录中找到 SteamVR", true);
        } else {
            self.set_toast("❌ 未找到 SteamVR，请确认目录正确", false);
        }

        self.working = false;
    }
}
