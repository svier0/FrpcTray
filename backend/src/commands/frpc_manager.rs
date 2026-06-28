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
                    let summary = l.split(": ").next().unwrap_or(l).to_string();
                    if summary.len() > 120 {
                        let mut out: String = summary.chars().take(117).collect();
                        out.push_str("...");
                        out
                    } else {
                        summary
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
