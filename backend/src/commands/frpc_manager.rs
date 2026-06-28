use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use crate::config::*;
use crate::toml_helper::read_server_file;
use super::server::list_servers_impl;

pub struct FrpcManager {
    processes: Mutex<HashMap<String, Arc<Mutex<tokio::process::Child>>>>,
}

impl FrpcManager {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }
}

fn frpc_name() -> &'static str {
    if cfg!(target_os = "windows") { "frpc.exe" } else { "frpc" }
}

fn frpc_bin_path() -> std::path::PathBuf {
    get_bin_dir().join(frpc_name())
}

async fn emit_status(app: &AppHandle, server_id: &str, old: &str, new: &str, pid: Option<u32>, err: Option<String>) {
    let _ = app.emit("frpc-status-changed", FrpcStatusEvent {
        server_id: server_id.to_string(),
        old_status: old.to_string(),
        new_status: new.to_string(),
        pid,
        error_message: err,
    });
}

fn extract_quoted_after<'a>(s: &'a str, prefix: &str) -> Option<String> {
    let lower = s.to_lowercase();
    let idx = lower.find(prefix)?;
    let after = s[idx + prefix.len()..].trim();
    let start = after.find('"')?;
    let rest = &after[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_after_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = s.to_lowercase();
    let idx = lower.find(prefix)?;
    Some(&s[idx + prefix.len()..])
}

fn strip_timestamp(s: &str) -> &str {
    let s = s.trim();
    if s.len() > 20 {
        let b = s.as_bytes();
        if b[4] == b'/' && b[7] == b'/' && b[10] == b' '
            && b[13] == b':' && b[16] == b':' && b[19] == b' '
            && b[..4].iter().all(|c| c.is_ascii_digit())
            && b[5..7].iter().all(|c| c.is_ascii_digit())
            && b[8..10].iter().all(|c| c.is_ascii_digit())
        {
            return s[20..].trim();
        }
    }
    s
}

fn summarize_frpc_error(raw: &str) -> Option<String> {
    let line = strip_timestamp(raw);
    let lower = line.to_lowercase();

    // --- Simple fixed mappings ---
    if lower.contains("login to server failed") || lower.contains("login to the server failed") {
        return Some("Login to server failed".to_string());
    }
    if lower.contains("token in login doesn") {
        return Some("Token mismatch".to_string());
    }
    if lower.contains("connection reset by peer") {
        return Some("Connection reset".to_string());
    }
    if lower.contains("network is unreachable") {
        return Some("Network unreachable".to_string());
    }
    if lower.contains("i/o deadline reached") || lower.contains("i/o timeout") {
        return Some("Connection timeout".to_string());
    }
    if lower.contains("recover to server timed out") {
        return Some("Reconnect timeout".to_string());
    }
    if lower.contains("control is closed") {
        return Some("Control channel closed".to_string());
    }
    if lower.contains("permission denied") {
        return Some("Permission denied".to_string());
    }
    if lower.contains("connection refused") {
        return Some("Connection refused".to_string());
    }
    if lower.trim() == "eof" {
        return Some("Connection closed unexpectedly".to_string());
    }

    // --- Variable extraction ---
    // error parsing config: <detail>
    if lower.contains("error parsing config") {
        if let Some(detail) = extract_after_prefix(line, "error parsing config") {
            let detail = detail.trim_start_matches(':').trim();
            if !detail.is_empty() {
                return Some(format!("Config parse error: {}", detail));
            }
        }
        return Some("Config parse error".to_string());
    }

    // unknown field "xxx"
    if lower.contains("unknown field") {
        if let Some(field) = extract_quoted_after(line, "unknown field") {
            return Some(format!("Unknown config field \"{}\"", field));
        }
        // fall through to raw
    }

    // proxy name "xxx" is already in use
    if lower.contains("proxy name") {
        if let Some(name) = extract_quoted_after(line, "proxy name") {
            return Some(format!("Proxy name \"{}\" already in use", name));
        }
    }

    // port already used: <num>
    if lower.contains("port already used") {
        let detail = extract_after_prefix(line, "port already used")
            .map(|s| s.trim_start_matches(':').trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(info) = detail {
            return Some(format!("Port {} already in use", info));
        }
        return Some("Port already in use".to_string());
    }

    // port unavailable
    if lower.contains("port unavailable") {
        let detail = extract_after_prefix(line, "port unavailable")
            .map(|s| s.trim_start_matches(':').trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(info) = detail {
            return Some(format!("Port {} unavailable", info));
        }
        return Some("Port unavailable".to_string());
    }

    // health check failed
    if lower.contains("health check failed") {
        let detail = extract_after_prefix(line, "health check failed")
            .map(|s| s.trim_start_matches(':').trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(info) = detail {
            return Some(format!("Health check failed: {}", info));
        }
        return Some("Health check failed".to_string());
    }

    // tls: failed to verify certificate: <reason>
    if lower.contains("failed to verify certificate") {
        if let Some(reason) = extract_after_prefix(line, "failed to verify certificate") {
            let reason = reason.trim_start_matches(':').trim();
            if !reason.is_empty() {
                return Some(format!("TLS certificate verification failed: {}", reason));
            }
        }
        return Some("TLS certificate verification failed".to_string());
    }

    // tls: <detail>
    if lower.contains("tls:") {
        if let Some(detail) = line.splitn(2, "tls:").nth(1) {
            let detail = detail.trim();
            if !detail.is_empty() {
                return Some(format!("TLS error: {}", detail));
            }
        }
        return Some("TLS error".to_string());
    }

    None
}

async fn spawn_monitor(app: AppHandle, server_id: String, child: Arc<Mutex<tokio::process::Child>>) {
    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let (status, child_output) = {
            let mut child = child.lock().await;
            let status = child.wait().await;

            let mut out_buf = String::new();
            let mut err_buf = String::new();

            if let Some(s) = child.stdout.as_mut() {
                let _ = s.read_to_string(&mut out_buf).await;
            }
            if let Some(s) = child.stderr.as_mut() {
                let _ = s.read_to_string(&mut err_buf).await;
            }

            let combined = if out_buf.trim().is_empty() {
                err_buf
            } else if err_buf.trim().is_empty() {
                out_buf
            } else {
                format!("{}\n{}", out_buf.trim(), err_buf.trim())
            };
            let msg = combined.lines()
                .find(|l| !l.trim().is_empty())
                .map(|l| {
                    let l = l.trim();
                    // Try known error patterns first
                    if let Some(summary) = summarize_frpc_error(l) {
                        return summary;
                    }
                    // Fallback: raw line, truncated to 120 chars
                    if l.len() > 120 {
                        let mut out: String = l.chars().take(117).collect();
                        out.push_str("...");
                        out
                    } else {
                        l.to_string()
                    }
                });

            (status, msg)
        };

        let state = app.state::<FrpcManager>();
        let mut procs = state.processes.lock().await;

        // If already removed by stop_frpc, this was intentional — skip crash event
        if procs.remove(&server_id).is_none() {
            return;
        }

        match status {
            Ok(exit) => {
                let code = exit.code().unwrap_or(-1);
                eprintln!("[frpc-tray] frpc {} 进程退出, code={}", server_id, code);
                if let Some(ref msg) = child_output {
                    eprintln!("[frpc-tray] frpc {} 输出:\n{}", server_id, msg);
                }

                emit_status(&app, &server_id, "running", "stopped", None,
                    child_output).await;
            }
            Err(e) => {
                eprintln!("[frpc-tray] frpc {} 进程等待失败: {}", server_id, e);
                emit_status(&app, &server_id, "running", "error", None,
                    Some(e.to_string())).await;
            }
        }
    });
}

#[tauri::command]
pub async fn start_frpc(
    server_id: String,
    state: tauri::State<'_, FrpcManager>,
    app: AppHandle,
) -> Result<(), String> {
    {
        let procs = state.processes.lock().await;
        if procs.contains_key(&server_id) {
            return Err(format!("服务器 '{}' 已在运行", server_id));
        }
    }

    let config = read_server_file(&server_id)?;
    if !config.enable {
        return Err(format!("服务器 '{}' 未启用", server_id));
    }

    let bin = frpc_bin_path();
    if !bin.exists() {
        return Err("frpc 可执行文件不存在，请先安装或升级内核".to_string());
    }

    let config_file = server_path(&server_id);
    if !config_file.exists() {
        return Err(format!("配置文件不存在: {}", config_file.display()));
    }

    // Ensure log directory exists
    let log_dir = get_config_dir().parent().unwrap().join("log");
    let _ = std::fs::create_dir_all(&log_dir);

    let child = tokio::process::Command::new(&bin)
        .arg("-c")
        .arg(&config_file)
        .current_dir(get_config_dir())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 frpc 失败: {}", e))?;

    let pid = child.id();
    let child = Arc::new(Mutex::new(child));

    {
        let mut procs = state.processes.lock().await;
        procs.insert(server_id.clone(), child.clone());
    }

    spawn_monitor(app.clone(), server_id.clone(), child).await;
    emit_status(&app, &server_id, "stopped", "running", pid, None).await;

    Ok(())
}

#[tauri::command]
pub async fn stop_frpc(
    server_id: String,
    state: tauri::State<'_, FrpcManager>,
    app: AppHandle,
) -> Result<(), String> {
    let child = {
        let mut procs = state.processes.lock().await;
        procs.remove(&server_id)
    };

    match child {
        Some(child) => {
            let mut child = child.lock().await;
            if let Err(e) = child.kill().await {
                return Err(format!("停止 frpc 失败: {}", e));
            }
            drop(child);
            emit_status(&app, &server_id, "running", "stopped", None, None).await;
            Ok(())
        }
        None => Err(format!("服务器 '{}' 未在运行", server_id)),
    }
}

#[tauri::command]
pub async fn start_all_frpc(
    state: tauri::State<'_, FrpcManager>,
    app: AppHandle,
) -> Result<(), String> {
    let servers = list_servers_impl()?;
    let enabled: Vec<String> = servers.iter()
        .filter(|s| s.enable)
        .map(|s| s.id.clone())
        .collect();

    let mut errors = Vec::new();
    for id in enabled {
        if let Err(e) = start_frpc(id.clone(), state.clone(), app.clone()).await {
            errors.push(format!("{}: {}", id, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[tauri::command]
pub async fn stop_all_frpc(
    state: tauri::State<'_, FrpcManager>,
    app: AppHandle,
) -> Result<(), String> {
    let ids: Vec<String> = {
        let procs = state.processes.lock().await;
        procs.keys().cloned().collect()
    };

    let mut errors = Vec::new();
    for id in ids {
        if let Err(e) = stop_frpc(id.clone(), state.clone(), app.clone()).await {
            errors.push(format!("{}: {}", id, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[tauri::command]
pub async fn get_all_frpc_status(
    state: tauri::State<'_, FrpcManager>,
) -> Result<Vec<FrpcRunningStatus>, String> {
    let servers = list_servers_impl()?;
    let procs = state.processes.lock().await;

    Ok(servers.iter().map(|s| {
        if procs.contains_key(&s.id) {
            FrpcRunningStatus {
                server_id: s.id.clone(),
                status: "running".to_string(),
                pid: None, // pid is set via spawn, hard to get here
                error_message: None,
            }
        } else {
            FrpcRunningStatus::stopped(&s.id)
        }
    }).collect())
}
