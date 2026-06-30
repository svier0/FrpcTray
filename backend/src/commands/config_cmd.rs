use std::fs;
use std::path::PathBuf;
use crate::config::*;

#[cfg(target_os = "windows")]
fn startup_shortcut_path() -> PathBuf {
    let appdata = std::env::var("APPDATA")
        .unwrap_or_else(|_| r"C:\Users\Default\AppData\Roaming".to_string());
    PathBuf::from(appdata)
        .join(r"Microsoft\Windows\Start Menu\Programs\Startup")
        .join("FrpcTray.lnk")
}

#[cfg(target_os = "macos")]
fn launch_agent_path() -> PathBuf {
    let home = std::env::var("HOME")
        .unwrap_or_else(|_| "/Users/Default".to_string());
    PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join("com.j7yx.svier0.FrpcTray.plist")
}

#[cfg(target_os = "linux")]
fn autostart_desktop_path() -> PathBuf {
    let home = std::env::var("HOME")
        .unwrap_or_else(|_| "/home/default".to_string());
    PathBuf::from(home)
        .join(".config/autostart")
        .join("FrpcTray.desktop")
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let path = startup_shortcut_path();
        if enabled {
            let exe = std::env::current_exe()
                .map_err(|e| format!("无法获取程序路径: {}", e))?;
            let ps = format!(
                "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.Save()",
                path.display(),
                exe.display()
            );
            let out = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps])
                .output()
                .map_err(|e| format!("创建快捷方式失败: {}", e))?;
            if !out.status.success() {
                return Err(format!("创建快捷方式失败: {}", String::from_utf8_lossy(&out.stderr)));
            }
        } else if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("删除快捷方式失败: {}", e))?;
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let path = launch_agent_path();
        if enabled {
            let exe = std::env::current_exe()
                .map_err(|e| format!("无法获取程序路径: {}", e))?;
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.j7yx.svier0.FrpcTray</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                exe.display()
            );
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建 LaunchAgents 目录失败: {}", e))?;
            }
            fs::write(&path, plist)
                .map_err(|e| format!("写入 LaunchAgent plist 失败: {}", e))?;
        } else if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("删除 LaunchAgent plist 失败: {}", e))?;
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let path = autostart_desktop_path();
        if enabled {
            let exe = std::env::current_exe()
                .map_err(|e| format!("无法获取程序路径: {}", e))?;
            let desktop = format!(
                "[Desktop Entry]\nType=Application\nName=FrpcTray\nExec={}\nNoDisplay=true\n",
                exe.display()
            );
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建 autostart 目录失败: {}", e))?;
            }
            fs::write(&path, desktop)
                .map_err(|e| format!("写入 autostart desktop 文件失败: {}", e))?;
        } else if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("删除 autostart desktop 文件失败: {}", e))?;
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = enabled;
        Err("当前平台不支持开机自启".to_string())
    }
}

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
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    let dir = get_config_dir();
    let path = dir.join("config.toml");

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    set_autostart(config.autostart)?;

    Ok(())
}
