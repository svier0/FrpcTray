use std::fs;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use crate::config::*;

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    let dir = get_config_dir();
    let path = dir.join("config.toml");

    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(config)
}

#[tauri::command]
pub async fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let dir = get_config_dir();
    let path = dir.join("config.toml");

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    let autoload = app.autolaunch();
    let current = autoload.is_enabled().map_err(|e| e.to_string())?;
    if config.autostart && !current {
        autoload.enable().map_err(|e| e.to_string())?;
    } else if !config.autostart && current {
        autoload.disable().map_err(|e| e.to_string())?;
    }

    Ok(())
}
