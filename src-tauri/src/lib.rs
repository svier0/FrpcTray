// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrpcConfig {
    pub server_addr: Option<String>,
    pub server_port: Option<u16>,
    pub token: Option<String>,
    pub log: Option<LogConfig>,
    pub http_proxy: Option<HttpProxyConfig>,
    pub https_proxy: Option<HttpsProxyConfig>,
    pub [toml::as_array()]: Option<Vec<ProxyConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: Option<String>,
    pub file: Option<String>,
    pub max_size: Option<u64>,
    pub max_backups: Option<u32>,
    pub max_age: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpProxyConfig {
    pub enabled: Option<bool>,
    pub addr: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpsProxyConfig {
    pub enabled: Option<bool>,
    pub addr: Option<String>,
    pub port: Option<u16>,
}
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use toml;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show = MenuItemBuilder::with_id("show", "显示主界面").build(app)?;
            let light = MenuItemBuilder::with_id("light", "轻量模式").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&light)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&quit)
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "light" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
