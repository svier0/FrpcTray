use std::io::Write;
use std::path::PathBuf;

use futures_util::StreamExt;
use tauri::Emitter;

use crate::config::*;
use crate::util::compare_versions;

fn detect_install_method() -> String {
    if cfg!(target_os = "windows") {
        if let Ok(exe) = std::env::current_exe() {
            let path = exe.to_string_lossy().to_lowercase();
            if path.contains("\\scoop\\apps\\") {
                return "scoop".to_string();
            }
        }
        if std::env::var("SCOOP").is_ok() {
            return "scoop".to_string();
        }
        "installer".to_string()
    } else {
        "installer".to_string()
    }
}

fn get_app_version() -> String {
    let json_str = include_str!("../../tauri.conf.json");
    let v: serde_json::Value = serde_json::from_str(json_str)
        .expect("无法解析 tauri.conf.json");
    v["version"]
        .as_str()
        .unwrap_or("0.0.0")
        .to_string()
}

fn build_filename(version: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("FrpcTray_{}_x64-setup_windows.exe", version)
    } else if cfg!(target_os = "macos") {
        format!("FrpcTray_{}_aarch64_macos.dmg", version)
    } else {
        format!("FrpcTray_{}_amd64_linux.deb", version)
    }
}

fn build_download_url(version: &str) -> String {
    format!(
        "https://github.com/svier0/FrpcTray/releases/download/v{}/{}",
        version,
        build_filename(version)
    )
}

struct ReleaseInfo {
    version: String,
}

async fn fetch_latest_release(client: &reqwest::Client, url: &str) -> Result<ReleaseInfo, String> {
    let timeout_secs = if url.contains("gh-proxy.com") || url.contains("ghfast.top") { 30 } else { 10 };
    let response = client
        .get(url)
        .header("User-Agent", "FrpcTray/1.0")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|e| format!("{} - {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!("{} - HTTP {}", url, response.status()));
    }

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("{} - 读取响应失败: {}", url, e))?;

    let json: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| format!("{} - 解析 JSON 失败: {}", url, e))?;

    let tag = json["tag_name"]
        .as_str()
        .ok_or_else(|| format!("{} - 响应中缺少 tag_name 字段", url))?;

    let version = tag
        .strip_prefix('v')
        .unwrap_or(tag)
        .to_string();

    Ok(ReleaseInfo { version })
}

#[tauri::command]
pub async fn check_app_update() -> Result<AppUpdateInfo, String> {
    let current_version = get_app_version();
    let cfg = read_app_config();
    let client = get_http_client();

    let mut urls: Vec<String> = Vec::new();
    urls.push("https://api.github.com/repos/svier0/FrpcTray/releases/latest".to_string());
    if cfg.use_github_proxy {
        urls.push("https://gh-proxy.com/https://api.github.com/repos/svier0/FrpcTray/releases/latest".to_string());
        urls.push("https://ghfast.top/https://api.github.com/repos/svier0/FrpcTray/releases/latest".to_string());
    }

    let mut errors = Vec::new();
    let mut latest_version = String::new();
    for url in &urls {
        match fetch_latest_release(client, url).await {
            Ok(info) => {
                latest_version = info.version;
                break;
            }
            Err(e) => errors.push(e),
        }
    }

    let download_url = if !latest_version.is_empty() {
        build_download_url(&latest_version)
    } else {
        String::new()
    };

    if latest_version.is_empty() {
        eprintln!("[FrpcTray] 检查更新失败: {}", errors.join("; "));
    }

    let can_upgrade = if latest_version.is_empty() {
        false
    } else {
        compare_versions(&current_version, &latest_version)
    };

    Ok(AppUpdateInfo {
        current_version,
        latest_version,
        can_upgrade,
        download_url,
        install_method: detect_install_method(),
    })
}

#[derive(Clone, serde::Serialize)]
struct DownloadEvent {
    phase: String,
    progress: f64,
    message: String,
}

fn emit(app: &tauri::AppHandle, phase: &str, progress: f64, message: &str) {
    let _ = app.emit("update-download-progress", DownloadEvent {
        phase: phase.to_string(),
        progress,
        message: message.to_string(),
    });
}

fn build_download_urls(version: &str, use_proxy: bool) -> Vec<String> {
    let direct = build_download_url(version);
    if use_proxy {
        vec![
            format!("https://gh-proxy.com/{}", direct),
            format!("https://ghfast.top/{}", direct),
            direct,
        ]
    } else {
        vec![direct]
    }
}

#[tauri::command]
pub async fn download_app_update(app: tauri::AppHandle, version: String) -> Result<(), String> {
    if detect_install_method() == "scoop" {
        return scoop_update(&app).await;
    }

    let cfg = read_app_config();
    let client = get_http_client();

    let urls = build_download_urls(&version, cfg.use_github_proxy);
    let filename = build_filename(&version);
    let temp_dir = std::env::temp_dir().join("FrpcTray");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;
    let dest_path = temp_dir.join(&filename);
    let tmp_path = temp_dir.join(format!("{}.tmp", filename));

    emit(&app, "downloading", 0.0, "开始下载...");

    let mut errors = Vec::new();
    let mut success = false;
    for url in &urls {
        let response = client
            .get(url)
            .header("User-Agent", "FrpcTray/1.0")
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    errors.push(format!("{} - HTTP {}", url, resp.status()));
                    continue;
                }

                let total = resp.content_length().unwrap_or(0);
                let mut downloaded: u64 = 0;
                let mut stream = resp.bytes_stream();
                let mut file = std::fs::File::create(&tmp_path)
                    .map_err(|e| format!("创建临时文件失败: {}", e))?;

                let mut last_pct: u32 = 0;
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(|e| format!("下载失败: {}", e))?;
                    file.write_all(&chunk)
                        .map_err(|e| format!("写入临时文件失败: {}", e))?;
                    downloaded += chunk.len() as u64;

                    if total > 0 {
                        let pct = (downloaded as f64 / total as f64 * 100.0) as u32;
                        if pct >= last_pct + 5 || pct == 100 {
                            last_pct = pct;
                            emit(&app, "downloading", pct as f64 / 100.0,
                                &format!("{:.1} MB / {:.1} MB",
                                    downloaded as f64 / 1_048_576.0,
                                    total as f64 / 1_048_576.0));
                        }
                    }
                }

                drop(file);
                std::fs::rename(&tmp_path, &dest_path)
                    .map_err(|e| format!("移动文件失败: {}", e))?;

                success = true;
                break;
            }
            Err(e) => {
                errors.push(format!("{} - {}", url, e));
            }
        }
    }

    if !success {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("下载失败: {}", errors.join("; ")));
    }

    emit(&app, "installing", 1.0, "正在启动安装程序...");

    launch_installer(&dest_path)?;

    emit(&app, "done", 1.0, "安装程序已启动，FrpcTray 即将退出...");

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    app.exit(0);
    Ok(())
}

async fn scoop_update(app: &tauri::AppHandle) -> Result<(), String> {
    emit(app, "installing", 0.0, "正在通过 Scoop 更新...");

    let mut cmd = tokio::process::Command::new("scoop");
    cmd.args(["update", "frpctray"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.as_std_mut().creation_flags(0x08000000);
    }
    let output = cmd.output()
        .await
        .map_err(|e| format!("执行 scoop update 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("scoop update 失败: {} {}", stdout, stderr));
    }

    emit(app, "done", 1.0, "Scoop 更新完成，FrpcTray 即将退出...");

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    app.exit(0);
    Ok(())
}

fn launch_installer(path: &PathBuf) -> Result<(), String> {
    let path_str = path.to_string_lossy().to_string();
    if cfg!(target_os = "windows") {
        std::process::Command::new(&path_str)
            .arg("/S")
            .spawn()
            .map_err(|e| format!("启动安装程序失败: {}", e))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("打开 DMG 失败: {}", e))?;
    } else {
        std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("打开安装包失败: {}", e))?;
    }
    Ok(())
}
