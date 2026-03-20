use anyhow::Result;
use tracing::info;
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIcon, Icon,
};

pub struct TrayApp {
    _tray_icon: TrayIcon,
}

impl TrayApp {
    pub fn new() -> Result<Self> {
        info!("Initializing system tray...");

        // Create a basic 16x16 transparent/solid icon
        // For a real app, this should load a PNG/ICO byte slice
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

        let config_item_id = config_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        // Spawn a thread to handle menu events
        std::thread::spawn(move || {
            let menu_channel = tray_icon::menu::MenuEvent::receiver();
            let _tray_channel = tray_icon::TrayIconEvent::receiver();
            
            loop {
                if let Ok(event) = menu_channel.recv() {
                    if event.id == config_item_id {
                        if let Some(config_path) = crate::core::config::get_config_path() {
                            let _ = open::that(config_path);
                        }
                    } else if event.id == quit_item_id {
                        std::process::exit(0);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }
}
