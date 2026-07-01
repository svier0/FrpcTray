use crate::config::*;
use crate::util::compare_versions;

fn get_app_version() -> String {
    let json_str = include_str!("../../tauri.conf.json");
    let v: serde_json::Value = serde_json::from_str(json_str)
        .expect("无法解析 tauri.conf.json");
    v["version"]
        .as_str()
        .unwrap_or("0.0.0")
        .to_string()
}

fn build_download_url(version: &str) -> String {
    let filename = if cfg!(target_os = "windows") {
        format!("FrpcTray_{}_x64-setup_windows.exe", version)
    } else if cfg!(target_os = "macos") {
        format!("FrpcTray_{}_aarch64_macos.dmg", version)
    } else {
        format!("FrpcTray_{}_amd64_linux.deb", version)
    };
    format!("https://github.com/svier0/FrpcTray/releases/download/v{}/{}", version, filename)
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
    })
}
