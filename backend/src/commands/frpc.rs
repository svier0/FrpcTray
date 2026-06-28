use std::fs;
use std::process::Command;
use crate::config::*;
use crate::util::*;

#[tauri::command]
pub async fn get_frpc_version() -> Result<FrpcVersionInfo, String> {
    let current_version = get_current_frpc_version();
    let (latest_version, can_upgrade) = match get_latest_frpc_version().await {
        Ok(v) => {
            let can = compare_versions(&current_version, &v);
            (v, can)
        }
        Err(e) => {
            eprintln!("[frpc-tray] {}", e);
            (String::new(), false)
        },
    };

    Ok(FrpcVersionInfo {
        current_version,
        latest_version,
        can_upgrade,
        platform: get_platform(),
        arch: get_arch(),
    })
}

fn get_current_frpc_version() -> String {
    let frpc_path = get_bin_dir().join(if cfg!(target_os = "windows") { "frpc.exe" } else { "frpc" });

    if !frpc_path.exists() {
        return "0".to_string();
    }

    let output = Command::new(&frpc_path)
        .arg("-v")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            parse_frpc_version_output(&stdout)
        }
        _ => "0".to_string(),
    }
}

fn parse_frpc_version_output(output: &str) -> String {
    output
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("frpc v")
                .or_else(|| line.strip_prefix("frpc "))
                .or_else(|| {
                    let first = line.split_whitespace().next()?;
                    if first.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        Some(first)
                    } else {
                        None
                    }
                })
                .map(String::from)
        })
        .unwrap_or_else(|| "0".to_string())
}

async fn get_latest_frpc_version() -> Result<String, String> {
    let client = get_http_client();
    let cfg = crate::config::read_app_config();

    let url = if cfg.use_github_proxy {
        "https://raw.giteeusercontent.com/scoop-installer/Main/raw/master/bucket/frp.json"
    } else {
        "https://raw.githubusercontent.com/ScoopInstaller/Main/refs/heads/master/bucket/frp.json"
    };
    let timeout = if cfg.use_github_proxy { 10 } else { 2 };
    match fetch_scoop_version(client, url, timeout).await {
        Ok(v) => Ok(v),
        Err(e) => {
            let label = if cfg.use_github_proxy { "镜像" } else { "GitHub" };
            Err(format!("从 {} 获取最新版本失败: {}", label, e))
        }
    }
}

async fn fetch_scoop_version(client: &reqwest::Client, url: &str, timeout_secs: u64) -> Result<String, String> {
    let response = client
        .get(url)
        .header("User-Agent", "frpc-tray/1.0")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let scoop_json: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    scoop_json["version"]
        .as_str()
        .map(|v| v.to_string())
        .ok_or_else(|| "版本号字段缺失".to_string())
}

async fn download_zip(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let timeout = if url.contains("gh-proxy.com") || url.contains("ghfast.top") { 60 } else { 2 };
    let response = client
        .get(url)
        .header("User-Agent", "frpc-tray/1.0")
        .timeout(std::time::Duration::from_secs(timeout))
        .send()
        .await
        .map_err(|e| format!("{} - {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!("{} - HTTP {}", url, response.status()));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("{} - 读取响应失败: {}", url, e))
}

#[tauri::command]
pub async fn upgrade_frpc(version: String) -> Result<(), String> {
    let platform = get_platform();
    let arch = get_arch();
    let path = format!("v{version}/frp_{version}_{platform}_{arch}.zip", version = version, platform = platform, arch = arch);
    let cfg = crate::config::read_app_config();

    let urls: Vec<String> = if cfg.use_github_proxy {
        vec![
            format!("https://gh-proxy.com/https://github.com/fatedier/frp/releases/download/{}", path),
            format!("https://ghfast.top/https://github.com/fatedier/frp/releases/download/{}", path),
        ]
    } else {
        vec![
            format!("https://github.com/fatedier/frp/releases/download/{}", path),
        ]
    };

    let client = get_http_client();
    let mut body = None;
    let mut errors = Vec::new();
    for url in &urls {
        match download_zip(client, url).await {
            Ok(b) => { body = Some(b); break; }
            Err(e) => errors.push(e),
        }
    }
    let body = body.ok_or_else(|| format!("下载失败: {}", errors.join("; ")))?;

    let cursor = std::io::Cursor::new(body.to_vec());
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("解压失败: {}", e))?;

    let frpc_name = if cfg!(target_os = "windows") { "frpc.exe" } else { "frpc" };
    let bin_dir = get_bin_dir();
    fs::create_dir_all(bin_dir)
        .map_err(|e| format!("创建 bin 目录失败: {}", e))?;
    let dest_path = bin_dir.join(frpc_name);

    let mut found = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {}", e))?;
        let entry_name = entry.name().to_string();

        if entry_name.ends_with(frpc_name) && entry_name.contains("frp_") {
            let temp_path = bin_dir.join(format!("{}.tmp", frpc_name));
            {
                let mut temp_file = fs::File::create(&temp_path)
                    .map_err(|e| format!("创建临时文件失败: {}", e))?;
                std::io::copy(&mut entry, &mut temp_file)
                    .map_err(|e| format!("写入临时文件失败: {}", e))?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| format!("设置可执行权限失败: {}", e))?;
            }

            fs::rename(&temp_path, &dest_path)
                .map_err(|e| format!("替换 frpc 失败: {}", e))?;

            found = true;
            break;
        }
    }

    if !found {
        return Err(format!("在压缩包中未找到 {}", frpc_name));
    }

    Ok(())
}
