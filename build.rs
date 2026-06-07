fn main() {
    // 配置 Windows App SDK self-contained 部署
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
}
