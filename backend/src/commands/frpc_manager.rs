use crate::config::*;

#[tauri::command]
pub async fn start_frpc(_server_id: String) -> Result<(), String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub async fn stop_frpc(_server_id: String) -> Result<(), String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub async fn start_all_frpc() -> Result<(), String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub async fn stop_all_frpc() -> Result<(), String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub async fn get_all_frpc_status() -> Result<Vec<FrpcRunningStatus>, String> {
    Ok(Vec::new())
}
