use std::sync::OnceLock;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

// ── TOML file structures ──

#[derive(Debug, Serialize, Deserialize)]
struct AuthConfig {
    method: Option<String>,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogConfig {
    to: Option<String>,
    level: Option<String>,
    #[serde(rename = "maxDays")]
    max_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrpcConfigFile {
    title: String,
    enable: bool,
    sort: i32,
    #[serde(rename = "serverAddr")]
    server_addr: String,
    #[serde(rename = "serverPort")]
    server_port: u16,
    auth: Option<AuthConfig>,
    log: Option<LogConfig>,
    proxies: Option<Vec<ProxyItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProxyItem {
    name: String,
    desc: Option<String>,
    #[serde(rename = "type")]
    proxy_type: String,
    #[serde(rename = "localIP")]
    local_ip: Option<String>,
    #[serde(rename = "localPort")]
    local_port: u16,
    #[serde(rename = "customDomains")]
    custom_domains: Option<Vec<String>>,
    locations: Option<Vec<String>>,
}

// ── API structures ──

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    id: String,
    title: String,
    enable: bool,
    sort: i32,
    #[serde(rename = "serverAddr")]
    server_addr: String,
    #[serde(rename = "serverPort")]
    server_port: u16,
    auth: Option<AuthConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateServerInput {
    title: String,
    enable: bool,
    sort: i32,
    #[serde(rename = "serverAddr")]
    server_addr: String,
    #[serde(rename = "serverPort")]
    server_port: u16,
    auth: Option<AuthConfig>,
}

// ── Helper functions ──

fn get_executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn get_config_dir() -> &'static PathBuf {
    CONFIG_DIR.get().expect("get_config_dir called before setup")
}

fn server_path(id: &str) -> PathBuf {
    get_config_dir().join(format!("frpc.{}.toml", id))
}

fn id_from_filename(filename: &str) -> Option<String> {
    filename
        .strip_suffix(".toml")
        .and_then(|s| s.strip_prefix("frpc."))
        .map(|s| s.to_string())
}

fn read_server_file(id: &str) -> Result<FrpcConfigFile, String> {
    let path = server_path(id);
    if !path.exists() {
        return Err(format!("服务器 '{}' 不存在", id));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("解析 TOML 失败: {}", e))
}

fn write_server_file(id: &str, config: &FrpcConfigFile) -> Result<(), String> {
    let path = server_path(id);
    let content = toml::to_string(config)
        .map_err(|e| format!("序列化 TOML 失败: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// ── Server commands ──

#[tauri::command]
fn list_servers() -> Result<Vec<ServerInfo>, String> {
    let dir = get_config_dir();
    fs::create_dir_all(dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let mut servers = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let id = match id_from_filename(&name) {
                Some(id) => id,
                None => continue,
            };
            if let Ok(config) = read_server_file(&id) {
                servers.push(ServerInfo {
                    id,
                    title: config.title,
                    enable: config.enable,
                    sort: config.sort,
                    server_addr: config.server_addr,
                    server_port: config.server_port,
                    auth: config.auth,
                });
            }
        }
    }

    servers.sort_by_key(|s| s.sort);
    Ok(servers)
}

#[tauri::command]
fn get_server(id: String) -> Result<ServerInfo, String> {
    let config = read_server_file(&id)?;
    Ok(ServerInfo {
        id,
        title: config.title,
        enable: config.enable,
        sort: config.sort,
        server_addr: config.server_addr,
        server_port: config.server_port,
        auth: config.auth,
    })
}

#[tauri::command]
fn create_server(input: CreateServerInput) -> Result<String, String> {
    let dir = get_config_dir();
    fs::create_dir_all(dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    // Auto-generate a single-letter id (a..z)
    let used_ids: std::collections::BTreeSet<String> = if let Ok(entries) = fs::read_dir(dir) {
        entries
            .flatten()
            .filter_map(|e| {
                let name = e.path().file_name()?.to_str()?.to_string();
                id_from_filename(&name)
            })
            .collect()
    } else {
        std::collections::BTreeSet::new()
    };

    let id = ('a'..='z')
        .map(|c| c.to_string())
        .find(|c| !used_ids.contains(c))
        .ok_or_else(|| "没有可用的单字母 ID（a..z 已用完）".to_string())?;

    let path = server_path(&id);
    if path.exists() {
        return Err(format!("服务器 '{}' 已存在", id));
    }

    let config = FrpcConfigFile {
        title: input.title,
        enable: input.enable,
        sort: input.sort,
        server_addr: input.server_addr,
        server_port: input.server_port,
        auth: input.auth,
        log: Some(LogConfig {
            to: Some("frpc.log".to_string()),
            level: Some("info".to_string()),
            max_days: Some(3),
        }),
        proxies: None,
    };
    write_server_file(&id, &config)?;
    Ok(id)
}

#[tauri::command]
fn update_server(server: ServerInfo) -> Result<(), String> {
    let mut config = read_server_file(&server.id)?;
    config.title = server.title;
    config.enable = server.enable;
    config.sort = server.sort;
    config.server_addr = server.server_addr;
    config.server_port = server.server_port;
    config.auth = server.auth;
    write_server_file(&server.id, &config)
}

#[tauri::command]
fn delete_server(id: String) -> Result<(), String> {
    let path = server_path(&id);
    if !path.exists() {
        return Err(format!("服务器 '{}' 不存在", id));
    }
    fs::remove_file(&path)
        .map_err(|e| format!("删除文件失败: {}", e))
}

#[tauri::command]
fn reorder_servers(ids: Vec<String>) -> Result<(), String> {
    for (index, id) in ids.iter().enumerate() {
        let mut config = read_server_file(id)?;
        config.sort = index as i32 + 1;
        write_server_file(id, &config)?;
    }
    Ok(())
}

// ── Proxy commands ──

#[tauri::command]
fn list_proxies(server_id: String) -> Result<Vec<ProxyItem>, String> {
    let config = read_server_file(&server_id)?;
    Ok(config.proxies.unwrap_or_default())
}

#[tauri::command]
fn create_proxy(server_id: String, proxy: ProxyItem) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.get_or_insert_with(Vec::new);
    if proxies.iter().any(|p| p.name == proxy.name) {
        return Err(format!("代理 '{}' 已存在", proxy.name));
    }
    proxies.push(proxy);
    write_server_file(&server_id, &config)
}

#[tauri::command]
fn update_proxy(server_id: String, old_name: String, proxy: ProxyItem) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.as_mut()
        .ok_or_else(|| "服务器没有代理配置".to_string())?;

    if old_name != proxy.name && proxies.iter().any(|p| p.name == proxy.name) {
        return Err(format!("代理名称 '{}' 已被使用", proxy.name));
    }

    if let Some(p) = proxies.iter_mut().find(|p| p.name == old_name) {
        *p = proxy;
    } else {
        return Err(format!("代理 '{}' 不存在", old_name));
    }
    write_server_file(&server_id, &config)
}

#[tauri::command]
fn delete_proxy(server_id: String, name: String) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.as_mut()
        .ok_or_else(|| "服务器没有代理配置".to_string())?;
    let len_before = proxies.len();
    proxies.retain(|p| p.name != name);
    if proxies.len() == len_before {
        return Err(format!("代理 '{}' 不存在", name));
    }
    write_server_file(&server_id, &config)
}

#[tauri::command]
fn reorder_proxies(server_id: String, names: Vec<String>) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.as_mut()
        .ok_or_else(|| "服务器没有代理配置".to_string())?;
    let mut ordered = Vec::with_capacity(names.len());
    for name in &names {
        let pos = proxies.iter().position(|p| p.name == *name)
            .ok_or_else(|| format!("代理 '{}' 不存在", name))?;
        ordered.push(proxies.remove(pos));
    }
    *proxies = ordered;
    write_server_file(&server_id, &config)
}

// ── App entry ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir: PathBuf = if cfg!(target_os = "windows") {
                get_executable_dir().join("conf")
            } else {
                app.path().app_data_dir()
                    .unwrap_or_else(|_| get_executable_dir())
                    .join("conf")
            };
            fs::create_dir_all(&config_dir)
                .expect("无法创建配置目录");
            let _ = CONFIG_DIR.set(config_dir);

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
            list_servers,
            get_server,
            create_server,
            update_server,
            delete_server,
            reorder_servers,
            list_proxies,
            create_proxy,
            update_proxy,
            delete_proxy,
            reorder_proxies,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
