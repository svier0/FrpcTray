use std::fs;
use crate::config::*;
use crate::toml_helper::*;

#[tauri::command]
pub fn list_servers() -> Result<Vec<ServerInfo>, String> {
    let dir = get_config_dir();
    fs::create_dir_all(dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let mut servers = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let id = match id_from_filename(&name) {
                Some(id) => id,
                None => continue,
            };
            if let Ok(config) = read_server_file(&id) {
                servers.push(ServerInfo {
                    id,
                    title: config.title,
                    enable: config.enable,
                    sort: config.sort,
                    server_addr: config.server_addr,
                    server_port: config.server_port,
                    auth: config.auth,
                    transport: config.transport,
                    tls: config.tls,
                });
            }
        }
    }

    servers.sort_by_key(|s| s.sort);
    Ok(servers)
}

#[tauri::command]
pub fn get_server(id: String) -> Result<ServerInfo, String> {
    let config = read_server_file(&id)?;
    Ok(ServerInfo {
        id,
        title: config.title,
        enable: config.enable,
        sort: config.sort,
        server_addr: config.server_addr,
        server_port: config.server_port,
        auth: config.auth,
        transport: config.transport,
        tls: config.tls,
    })
}

#[tauri::command]
pub fn create_server(input: CreateServerInput) -> Result<String, String> {
    let dir = get_config_dir();
    fs::create_dir_all(dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    let used_ids: std::collections::BTreeSet<String> = if let Ok(entries) = fs::read_dir(dir) {
        entries
            .flatten()
            .filter_map(|e| {
                let name = e.path().file_name()?.to_str()?.to_string();
                id_from_filename(&name)
            })
            .collect()
    } else {
        std::collections::BTreeSet::new()
    };

    let id = ('a'..='z')
        .map(|c| c.to_string())
        .find(|c| !used_ids.contains(c))
        .ok_or_else(|| "没有可用的单字母 ID（a..z 已用完）".to_string())?;

    let path = server_path(&id);
    if path.exists() {
        return Err(format!("服务器 '{}' 已存在", id));
    }

    let config = FrpcConfigFile {
        title: input.title,
        enable: input.enable,
        sort: input.sort,
        server_addr: input.server_addr,
        server_port: input.server_port,
        auth: input.auth,
        transport: input.transport,
        tls: input.tls,
        log: Some(LogConfig {
            to: Some("frpc.log".to_string()),
            level: Some("info".to_string()),
            max_days: Some(3),
        }),
        proxies: None,
    };
    write_server_file(&id, &config)?;
    Ok(id)
}

#[tauri::command]
pub fn update_server(server: ServerInfo) -> Result<(), String> {
    let mut config = read_server_file(&server.id)?;
    config.title = server.title;
    config.enable = server.enable;
    config.sort = server.sort;
    config.server_addr = server.server_addr;
    config.server_port = server.server_port;
    config.auth = server.auth;
    config.transport = server.transport;
    config.tls = server.tls;
    write_server_file(&server.id, &config)
}

#[tauri::command]
pub fn delete_server(id: String) -> Result<(), String> {
    let path = server_path(&id);
    if !path.exists() {
        return Err(format!("服务器 '{}' 不存在", id));
    }
    fs::remove_file(&path)
        .map_err(|e| format!("删除文件失败: {}", e))
}

#[tauri::command]
pub fn reorder_servers(ids: Vec<String>) -> Result<(), String> {
    for (index, id) in ids.iter().enumerate() {
        let mut config = read_server_file(id)?;
        config.sort = index as i32 + 1;
        write_server_file(id, &config)?;
    }
    Ok(())
}
