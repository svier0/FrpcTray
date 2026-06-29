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
    failed_errors: Mutex<HashMap<String, Option<String>>>,
}

impl FrpcManager {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            failed_errors: Mutex::new(HashMap::new()),
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
    let b = s.as_bytes();
    if b.len() >= 19
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && (b[4] == b'-' || b[4] == b'/')
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && (b[7] == b'-' || b[7] == b'/')
        && b[8..10].iter().all(|c| c.is_ascii_digit())
        && b[10] == b' '
        && b[11..13].iter().all(|c| c.is_ascii_digit())
        && b[13] == b':'
        && b[14..16].iter().all(|c| c.is_ascii_digit())
        && b[16] == b':'
        && b[17..19].iter().all(|c| c.is_ascii_digit())
    {
        let mut end = 19;
        if end < b.len() && b[end] == b'.' {
            end += 1;
            while end < b.len() && b[end].is_ascii_digit() {
                end += 1;
            }
        }
        if end < b.len() {
            return s[end..].trim();
        }
        return "";
    }
    s
}

fn strip_log_prefix(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut pos = 0;
    while pos < bytes.len() && bytes[pos] == b'[' {
        if let Some(end) = s[pos..].find(']') {
            pos += end + 1;
            while pos < bytes.len() && bytes[pos] == b' ' {
                pos += 1;
            }
        } else {
            break;
        }
    }
    s[pos..].trim()
}

fn summarize_frpc_error(raw: &str) -> Option<String> {
    let line = strip_log_prefix(strip_timestamp(raw));
    let lower = line.to_lowercase();

    // --- Simple fixed mappings ---

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

    // service is stopped
    if lower.contains("service") && lower.contains("is stopped") {
        return Some("Service stopped".to_string());
    }

    // Info/warning level log lines that didn't match any specific error pattern
    if lower.starts_with("[i]") || lower.starts_with("[w]") {
        return None;
    }

    None
}

fn read_log_tail(server_id: &str) -> Option<String> {
    use std::io::{BufReader, Read, Seek, SeekFrom};

    let path = get_config_dir()
        .parent()?
        .join("log")
        .join(format!("frpc.{}.log", server_id));

    let file = std::fs::File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();
    let read_size = std::cmp::min(file_len, 4096);
    let mut reader = BufReader::new(file);
    if read_size < file_len {
        reader.seek(SeekFrom::End(-(read_size as i64))).ok()?;
    }

    let mut content = String::new();
    reader.read_to_string(&mut content).ok()?;

    content.lines()
        .filter(|l| !l.trim().is_empty())
        .last()
        .map(|s| s.to_string())
}

fn log_tail_contains(server_id: &str, pattern: &str) -> bool {
    use std::io::{BufReader, Read, Seek, SeekFrom};

    let path = match get_config_dir().parent() {
        Some(p) => p.join("log").join(format!("frpc.{}.log", server_id)),
        None => return false,
    };

    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let file_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return false,
    };

    let read_size = std::cmp::min(file_len, 4096);
    let mut reader = BufReader::new(file);
    if read_size < file_len {
        let _ = reader.seek(SeekFrom::End(-(read_size as i64)));
    }

    let mut content = String::new();
    if reader.read_to_string(&mut content).is_err() {
        return false;
    }

    content.to_lowercase().contains(pattern)
}

async fn spawn_monitor(app: AppHandle, server_id: String, mut child: tokio::process::Child, mut kill_rx: watch::Receiver<bool>) {
    tokio::spawn(async move {
        use tokio::time::Duration;
        let state = app.state::<FrpcManager>();

        let mut login_ok = false;
        let mut error_line: Option<String> = None;

        let mut stdout_reader = child.stdout.take().map(tokio::io::BufReader::new);
        let mut stderr_reader = child.stderr.take().map(tokio::io::BufReader::new);
        let mut stdout_line = String::new();
        let mut stderr_line = String::new();
        let deadline = tokio::time::sleep(Duration::from_secs(30));
        tokio::pin!(deadline);
        let mut log_check = tokio::time::interval(Duration::from_secs(1));

        loop {
            let stdout_fut = async {
                match &mut stdout_reader {
                    Some(r) => r.read_line(&mut stdout_line).await,
                    None => std::future::pending().await,
                }
            };
            let stderr_fut = async {
                match &mut stderr_reader {
                    Some(r) => r.read_line(&mut stderr_line).await,
                    None => std::future::pending().await,
                }
            };
            tokio::select! {
                result = stdout_fut => {
                    match result {
                        Ok(0) => {
                            stdout_reader = None;
                            if stdout_reader.is_none() && stderr_reader.is_none() { break; }
                        }
                        Ok(_) => {
                            let trimmed = stdout_line.trim();
                            let lower = trimmed.to_lowercase();
                            if lower.contains("login to server success") {
                                login_ok = true;
                                break;
                            }
                            if lower.contains("frpc service") && lower.contains("stopped") {
                                break;
                            }
                            if error_line.is_none() && !trimmed.is_empty() {
                                error_line = Some(stdout_line.clone());
                            }
                            stdout_line.clear();
                        }
                        Err(_) => {
                            stdout_reader = None;
                            if stdout_reader.is_none() && stderr_reader.is_none() { break; }
                        }
                    }
                }
                result = stderr_fut => {
                    match result {
                        Ok(0) => {
                            stderr_reader = None;
                            if stdout_reader.is_none() && stderr_reader.is_none() { break; }
                        }
                        Ok(_) => {
                            let trimmed = stderr_line.trim();
                            if error_line.is_none() && !trimmed.is_empty() {
                                error_line = Some(stderr_line.clone());
                            }
                            stderr_line.clear();
                        }
                        Err(_) => {
                            stderr_reader = None;
                            if stdout_reader.is_none() && stderr_reader.is_none() { break; }
                        }
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
                _ = log_check.tick() => {
                    if log_tail_contains(&server_id, "login to server success") {
                        login_ok = true;
                        break;
                    }
                }
            }
        }

        if login_ok {
            // Update status to running and clear any stored failure error
            {
                let mut procs = state.processes.lock().await;
                if let Some(entry) = procs.get_mut(&server_id) {
                    entry.status = "running".to_string();
                }
            }
            {
                let mut errors = state.failed_errors.lock().await;
                errors.remove(&server_id);
            }
            emit_status(&app, &server_id, "connecting", "running", child.id(), None).await;

            // Phase 2: wait for exit or kill
            let mut killed = false;
            let mut exit_status: Option<std::process::ExitStatus> = None;
            loop {
                tokio::select! {
                    result = child.wait() => {
                        exit_status = result.ok();
                        break;
                    }
                    _ = kill_rx.changed() => {
                        if *kill_rx.borrow() {
                            killed = true;
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                            break;
                        }
                    }
                }
            }

            let err_msg = if killed {
                None
            } else {
                match exit_status {
                    Some(status) if status.success() => {
                        read_log_tail(&server_id)
                            .as_deref()
                            .and_then(summarize_frpc_error)
                            .or_else(|| Some("Process exited unexpectedly".to_string()))
                    }
                    Some(status) => {
                        let reason = status.code()
                            .map(|c| format!("Process exited with code {}", c))
                            .unwrap_or_else(|| "Process exited abnormally".to_string());
                        Some(reason)
                    }
                    None => {
                        Some("Process exited unexpectedly".to_string())
                    }
                }
            };

            let mut procs = state.processes.lock().await;
            if procs.remove(&server_id).is_some() {
                drop(procs);
                let mut errors = state.failed_errors.lock().await;
                if let Some(ref m) = err_msg {
                    errors.insert(server_id.clone(), Some(m.clone()));
                } else {
                    errors.remove(&server_id);
                }
                drop(errors);
                emit_status(&app, &server_id, "running", "stopped", None, err_msg).await;
            }
        } else {
            // Process exited or was killed before login
            let msg = error_line.as_deref()
                .and_then(|l| summarize_frpc_error(l.trim()))
                .or_else(|| {
                    // Only pass through error_line if it's not info/warning level
                    let l = error_line.as_deref()?;
                    let stripped = strip_timestamp(l.trim());
                    if stripped.to_lowercase().starts_with("[i]") || stripped.to_lowercase().starts_with("[w]") {
                        return None;
                    }
                    let l = strip_log_prefix(strip_timestamp(l.trim())).to_string();
                    if l.len() > 120 { Some(format!("{}...", &l[..117])) } else { Some(l) }
                })
                .or_else(|| {
                    read_log_tail(&server_id)
                        .as_deref()
                        .and_then(summarize_frpc_error)
                        .or_else(|| read_log_tail(&server_id).map(|s| {
                            let s = strip_log_prefix(strip_timestamp(&s)).trim().to_string();
                            if s.len() > 120 { format!("{}...", &s[..117]) } else { s }
                        }))
                });

            let mut procs = state.processes.lock().await;
            if procs.remove(&server_id).is_some() {
                drop(procs);
                let mut errors = state.failed_errors.lock().await;
                if let Some(ref m) = msg {
                    errors.insert(server_id.clone(), Some(m.clone()));
                } else {
                    errors.remove(&server_id);
                }
                drop(errors);
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

    eprintln!("[frpc-tray] spawned, stdout.is_some={}, stderr.is_some={}", child.stdout.is_some(), child.stderr.is_some());

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

    {
        let mut errors = state.failed_errors.lock().await;
        errors.remove(&server_id);
    }

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
    let errors = state.failed_errors.lock().await;

    Ok(servers.iter().map(|s| {
        if let Some(entry) = procs.get(&s.id) {
            FrpcRunningStatus {
                server_id: s.id.clone(),
                status: entry.status.clone(),
                pid: Some(entry.pid),
                error_message: None,
            }
        } else {
            let err_msg = errors.get(&s.id).and_then(|e| e.clone());
            FrpcRunningStatus {
                server_id: s.id.clone(),
                status: "stopped".to_string(),
                pid: None,
                error_message: err_msg,
            }
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
