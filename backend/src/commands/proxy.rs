use crate::config::*;
use crate::toml_helper::*;

#[tauri::command]
pub fn list_proxies(server_id: String) -> Result<Vec<ProxyItem>, String> {
    let config = read_server_file(&server_id)?;
    Ok(config.proxies.unwrap_or_default())
}

#[tauri::command]
pub fn create_proxy(server_id: String, proxy: ProxyItem) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.get_or_insert_with(Vec::new);
    if proxies.iter().any(|p| p.name == proxy.name) {
        return Err(format!("代理 '{}' 已存在", proxy.name));
    }
    proxies.push(proxy);
    write_server_file(&server_id, &config)
}

#[tauri::command]
pub fn update_proxy(server_id: String, old_name: String, proxy: ProxyItem) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.as_mut()
        .ok_or_else(|| "服务器没有代理配置".to_string())?;

    if old_name != proxy.name && proxies.iter().any(|p| p.name == proxy.name) {
        return Err(format!("代理名称 '{}' 已被使用", proxy.name));
    }

    if let Some(p) = proxies.iter_mut().find(|p| p.name == old_name) {
        *p = proxy;
    } else {
        return Err(format!("代理 '{}' 不存在", old_name));
    }
    write_server_file(&server_id, &config)
}

#[tauri::command]
pub fn delete_proxy(server_id: String, name: String) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.as_mut()
        .ok_or_else(|| "服务器没有代理配置".to_string())?;
    let len_before = proxies.len();
    proxies.retain(|p| p.name != name);
    if proxies.len() == len_before {
        return Err(format!("代理 '{}' 不存在", name));
    }
    write_server_file(&server_id, &config)
}

#[tauri::command]
pub fn reorder_proxies(server_id: String, names: Vec<String>) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.as_mut()
        .ok_or_else(|| "服务器没有代理配置".to_string())?;
    let mut ordered = Vec::with_capacity(names.len());
    for name in &names {
        let pos = proxies.iter().position(|p| p.name == *name)
            .ok_or_else(|| format!("代理 '{}' 不存在", name))?;
        ordered.push(proxies.remove(pos));
    }
    *proxies = ordered;
    write_server_file(&server_id, &config)
}
