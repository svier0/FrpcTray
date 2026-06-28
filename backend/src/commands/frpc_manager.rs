use std::collections::HashMap;
use tokio::io::AsyncBufReadExt;
use tokio::sync::Mutex;
use tokio::sync::watch;
use tauri::{AppHandle, Emitter, Manager};
use crate::config::*;
use crate::toml_helper::read_server_file;
use super::server::list_servers_impl;

struct ProcessEntry {
    pid: u32,
    status: String,
    kill_tx: watch::Sender<bool>,
}

pub struct FrpcManager {
    processes: Mutex<HashMap<String, ProcessEntry>>,
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

async fn spawn_monitor(app: AppHandle, server_id: String, mut child: tokio::process::Child, mut kill_rx: watch::Receiver<bool>) {
    tokio::spawn(async move {
        use tokio::time::Duration;
        let state = app.state::<FrpcManager>();

        // Consume initial changed() notification (returns immediately)
        let _ = kill_rx.changed().await;

        // Phase 1: read stdout until "login to server success", EOF (process exit), kill, or timeout
        let mut login_ok = false;
        let mut error_line: Option<String> = None;

        eprintln!("[frpc-tray] monitor {} stdout is_some={}", server_id, child.stdout.is_some());
        eprintln!("[frpc-tray] monitor {} stderr is_some={}", server_id, child.stderr.is_some());
        eprintln!("[frpc-tray] monitor {} pid={:?}", server_id, child.id());

        // Take stderr and print to terminal for visibility
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = tokio::io::BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => eprint!("[frpc:err] {}", line),
                    }
                }
            });
        }

        if let Some(stdout) = child.stdout.take() {
            eprintln!("[frpc-tray] monitor {} starting read loop", server_id);
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut line = String::new();
            let deadline = tokio::time::sleep(Duration::from_secs(10));
            tokio::pin!(deadline);

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => {
                                eprintln!("[frpc-tray] monitor {} EOF", server_id);
                                break;
                            }
                            Ok(_) => {
                                let trimmed = line.trim();
                                eprintln!("[frpc-tray] monitor {} line: {:?}", server_id, trimmed);
                                let lower = trimmed.to_lowercase();
                                if lower.contains("login to server success") {
                                    login_ok = true;
                                    break;
                                }
                                // frpc service ... stopped — always the last line, startup phase ended
                                if lower.contains("frpc service") && lower.contains("stopped") {
                                    break;
                                }
                                // Capture the first non-empty line as error cause (appears before "frpc service ... stopped")
                                if error_line.is_none() && !trimmed.is_empty() {
                                    error_line = Some(line.clone());
                                }
                                line.clear();
                            }
                            Err(_) => break,
                        }
                    }
                    _ = kill_rx.changed() => {
                        if *kill_rx.borrow() {
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                            break;
                        }
                    }
                    _ = &mut deadline => {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        break;
                    }
                }
            }
        } else {
            // Console mode (CREATE_NEW_CONSOLE) — can't read output, assume success
            login_ok = true;
        }

        if login_ok {
            // Update status to running
            {
                let mut procs = state.processes.lock().await;
                if let Some(entry) = procs.get_mut(&server_id) {
                    entry.status = "running".to_string();
                }
            }
            emit_status(&app, &server_id, "connecting", "running", child.id(), None).await;

            // Phase 2: wait for exit or kill
            loop {
                tokio::select! {
                    _ = child.wait() => {
                        break;
                    }
                    _ = kill_rx.changed() => {
                        if *kill_rx.borrow() {
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                            break;
                        }
                    }
                }
            }

            let mut procs = state.processes.lock().await;
            if procs.remove(&server_id).is_some() {
                emit_status(&app, &server_id, "running", "stopped", None, None).await;
            }
        } else {
            // Process exited or was killed before login
            let msg = error_line.as_deref()
                .and_then(|l| summarize_frpc_error(l.trim()))
                .or_else(|| error_line.map(|l| {
                    let l = l.trim().to_string();
                    if l.len() > 120 {
                        format!("{}...", &l[..117])
                    } else {
                        l
                    }
                }));

            let mut procs = state.processes.lock().await;
            if procs.remove(&server_id).is_some() {
                emit_status(&app, &server_id, "connecting", "stopped", None, msg).await;
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

    let mut cmd = tokio::process::Command::new(&bin);
    cmd.arg("-c")
       .arg(&config_file)
       .current_dir(get_config_dir())
       .stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());

    let child = cmd.spawn()
        .map_err(|e| format!("启动 frpc 失败: {}", e))?;

    let pid = child.id();
    let (kill_tx, kill_rx) = watch::channel(false);

    {
        let mut procs = state.processes.lock().await;
        procs.insert(server_id.clone(), ProcessEntry {
            pid: pid.expect("child should have a pid"),
            status: "connecting".to_string(),
            kill_tx,
        });
    }

    spawn_monitor(app.clone(), server_id.clone(), child, kill_rx).await;
    emit_status(&app, &server_id, "stopped", "connecting", pid, None).await;

    Ok(())
}

#[tauri::command]
pub async fn stop_frpc(
    server_id: String,
    state: tauri::State<'_, FrpcManager>,
    app: AppHandle,
) -> Result<(), String> {
    let entry = {
        let mut procs = state.processes.lock().await;
        procs.remove(&server_id)
    };

    if let Some(entry) = entry {
        let old_status = entry.status.clone();
        let _ = entry.kill_tx.send(true);
        emit_status(&app, &server_id, &old_status, "stopped", None, None).await;
    }
    Ok(())
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
        if let Some(entry) = procs.get(&s.id) {
            FrpcRunningStatus {
                server_id: s.id.clone(),
                status: entry.status.clone(),
                pid: Some(entry.pid),
                error_message: None,
            }
        } else {
            FrpcRunningStatus::stopped(&s.id)
        }
    }).collect())
}

#[tauri::command]
pub async fn open_log_file(server_id: String) -> Result<(), String> {
    let path = get_config_dir()
        .parent()
        .ok_or_else(|| "无法获取配置目录父路径".to_string())?
        .join("log")
        .join(format!("frpc.{}.log", server_id));

    if !path.exists() {
        return Err(format!("日志文件不存在: {}", path.display()));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }

    Ok(())
}
