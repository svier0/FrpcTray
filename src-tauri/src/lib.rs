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

// 获取可执行文件所在目录
fn get_executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

// 获取同级目录下的 TOML 配置文件列表
#[tauri::command]
fn list_toml_files() -> Result<Vec<String>, String> {
    let exec_dir = get_executable_dir();

    let mut toml_files = Vec::new();

    if let Ok(entries) = fs::read_dir(&exec_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "toml" {
                        if let Some(file_name) = path.file_name() {
                            toml_files.push(file_name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(toml_files)
}

// 读取指定的 TOML 配置文件
#[tauri::command]
fn read_toml_file(filename: String) -> Result<String, String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    if !file_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    if !file_path.is_file() {
        return Err("路径不是文件".to_string());
    }

    fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))
}

// 写入 TOML 配置文件
#[tauri::command]
fn write_toml_file(filename: String, content: String) -> Result<(), String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    fs::write(&file_path, content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// 在指定 TOML 文件中添加代理配置
#[tauri::command]
fn add_proxy_to_toml(filename: String, proxy: ProxyConfig) -> Result<(), String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    if !file_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 读取现有配置
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 解析 TOML
    let mut config: FrpcConfig = toml::from_str(&content)
        .map_err(|e| format!("解析 TOML 失败: {}", e))?;

    // 确保 proxies 数组存在
    if config.proxies.is_none() {
        config.proxies = Some(Vec::new());
    }

    // 添加代理配置
    config.proxies.as_mut().unwrap().push(proxy);

    // 写回文件
    let new_content = toml::to_string(&config)
        .map_err(|e| format!("序列化 TOML 失败: {}", e))?;

    fs::write(&file_path, new_content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// 更新指定 TOML 文件中的代理配置
#[tauri::command]
fn update_proxy_in_toml(filename: String, proxy_id: String, updates: ProxyConfig) -> Result<(), String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    if !file_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 读取现有配置
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 解析 TOML
    let mut config: FrpcConfig = toml::from_str(&content)
        .map_err(|e| format!("解析 TOML 失败: {}", e))?;

    // 更新代理配置
    if let Some(proxies) = config.proxies.as_mut() {
        for proxy in proxies {
            if proxy.id == proxy_id {
                *proxy = updates;
                break;
            }
        }
    } else {
        return Err("配置文件中不存在代理配置".to_string());
    }

    // 写回文件
    let new_content = toml::to_string(&config)
        .map_err(|e| format!("序列化 TOML 失败: {}", e))?;

    fs::write(&file_path, new_content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// 删除指定 TOML 文件中的代理配置
#[tauri::command]
fn delete_proxy_from_toml(filename: String, proxy_id: String) -> Result<(), String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    if !file_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 读取现有配置
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 解析 TOML
    let mut config: FrpcConfig = toml::from_str(&content)
        .map_err(|e| format!("解析 TOML 失败: {}", e))?;

    // 删除代理配置
    if let Some(proxies) = config.proxies.as_mut() {
        proxies.retain(|proxy| proxy.id != proxy_id);
    } else {
        return Err("配置文件中不存在代理配置".to_string());
    }

    // 写回文件
    let new_content = toml::to_string(&config)
        .map_err(|e| format!("序列化 TOML 失败: {}", e))?;

    fs::write(&file_path, new_content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// 获取指定 TOML 文件中的所有代理配置
#[tauri::command]
fn get_proxies_from_toml(filename: String) -> Result<Vec<ProxyConfig>, String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    if !file_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 读取现有配置
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 解析 TOML
    let config: FrpcConfig = toml::from_str(&content)
        .map_err(|e| format!("解析 TOML 失败: {}", e))?;

    Ok(config.proxies.unwrap_or_default())
}

// 创建新的 TOML 配置文件
#[tauri::command]
fn create_toml_file(filename: String, content: Option<String>) -> Result<(), String> {
    let exec_dir = get_executable_dir();
    let file_path = exec_dir.join(filename);

    // 检查文件是否已存在
    if file_path.exists() {
        return Err("配置文件已存在".to_string());
    }

    // 如果没有提供内容，使用默认的 frpc 配置模板
    let default_content = content.unwrap_or_else(|| {
        r#"[server]
addr = "127.0.0.1"
port = 7000
token = "your_token_here"

[log]
level = "info"
file = "frpc.log"
maxSize = 50
maxBackups = 3
maxAge = 7

[[proxies]]
name = "default_proxy"
type = "http"
localPort = 8080
remotePort = 80

[proxies.plugin]
type = "http_proxy""#.to_string()
    });

    fs::write(&file_path, default_content)
        .map_err(|e| format!("创建文件失败: {}", e))
}

// 复制 TOML 配置文件
#[tauri::command]
fn copy_toml_file(source_filename: String, target_filename: String) -> Result<(), String> {
    let exec_dir = get_executable_dir();
    let source_path = exec_dir.join(source_filename);
    let target_path = exec_dir.join(target_filename);

    // 检查源文件是否存在
    if !source_path.exists() {
        return Err("源配置文件不存在".to_string());
    }

    // 检查目标文件是否已存在
    if target_path.exists() {
        return Err("目标配置文件已存在".to_string());
    }

    // 读取源文件内容
    let content = fs::read_to_string(&source_path)
        .map_err(|e| format!("读取源文件失败: {}", e))?;

    // 写入目标文件
    fs::write(&target_path, content)
        .map_err(|e| format!("创建目标文件失败: {}", e))
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
        .invoke_handler(tauri::generate_handler![
            greet,
            list_toml_files,
            read_toml_file,
            write_toml_file,
            create_toml_file,
            copy_toml_file,
            add_proxy_to_toml,
            update_proxy_in_toml,
            delete_proxy_from_toml,
            get_proxies_from_toml
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
