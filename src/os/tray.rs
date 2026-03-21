use anyhow::Result;
use tracing::{error, info};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

pub struct TrayApp {
    _tray_icon: TrayIcon,
    config_item: MenuItem,
    quit_item: MenuItem,
}

impl TrayApp {
    pub fn new() -> Result<Self> {
        info!("Initializing system tray...");

        // Create a basic 16x16 transparent/solid icon
        let width = 16;
        let height = 16;
        let rgba = vec![200; (width * height * 4) as usize];
        let icon = Icon::from_rgba(rgba, width, height)?;

        let tray_menu = Menu::new();

        let config_item = MenuItem::new("Open Configuration", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let _ = tray_menu.append(&config_item);
        let _ = tray_menu.append(&PredefinedMenuItem::separator());
        let _ = tray_menu.append(&quit_item);

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Antarcticite Router")
            .with_icon(icon)
            .build()?;

        info!("Tray icon built successfully.");

        Ok(Self {
            _tray_icon: tray_icon,
            config_item,
            quit_item,
        })
    }

    pub fn handle_events(&self) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            info!("Menu event received: id={:?}", event.id);
            if event.id == self.config_item.id() {
                info!("Open Configuration clicked.");
                if let Some(config_path) = crate::core::config::get_config_path() {
                    info!("Opening config path: {:?}", config_path);
                    let _ = open::that(config_path);
                } else {
                    error!("Could not determine config path.");
                }
            } else if event.id == self.quit_item.id() {
                info!("Quit clicked.");
                std::process::exit(0);
            }
        }
    }
}
