use std::sync::OnceLock;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Table, ArrayOfTables, value};

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

// ── TOML file structures ──

#[derive(Debug)]
struct AuthConfig {
    method: Option<String>,
    token: Option<String>,
}

#[derive(Debug)]
struct LogConfig {
    to: Option<String>,
    level: Option<String>,
    max_days: Option<i32>,
}

#[derive(Debug)]
struct FrpcConfigFile {
    title: String,
    enable: bool,
    sort: i32,
    server_addr: String,
    server_port: u16,
    auth: Option<AuthConfig>,
    log: Option<LogConfig>,
    proxies: Option<Vec<ProxyItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProxyItem {
    name: String,
    desc: Option<String>,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(rename = "type")]
    proxy_type: String,
    #[serde(rename = "localIP")]
    local_ip: Option<String>,
    #[serde(rename = "localPort")]
    local_port: u16,
    #[serde(rename = "customDomains")]
    custom_domains: Option<Vec<String>>,
    locations: Option<Vec<String>>,
}

fn default_true() -> bool { true }

// ── API structures ──

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    id: String,
    title: String,
    enable: bool,
    sort: i32,
    #[serde(rename = "serverAddr")]
    server_addr: String,
    #[serde(rename = "serverPort")]
    server_port: u16,
    auth: Option<AuthConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateServerInput {
    title: String,
    enable: bool,
    sort: i32,
    #[serde(rename = "serverAddr")]
    server_addr: String,
    #[serde(rename = "serverPort")]
    server_port: u16,
    auth: Option<AuthConfig>,
}

// AuthConfig serde for IPC
impl Serialize for AuthConfig {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AuthConfig", 2)?;
        s.serialize_field("method", &self.method)?;
        s.serialize_field("token", &self.token)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for AuthConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct AuthConfigHelper {
            method: Option<String>,
            token: Option<String>,
        }
        let h = AuthConfigHelper::deserialize(deserializer)?;
        Ok(AuthConfig { method: h.method, token: h.token })
    }
}

// ── Helper functions ──

fn get_executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn get_config_dir() -> &'static PathBuf {
    CONFIG_DIR.get().expect("get_config_dir called before setup")
}

fn server_path(id: &str) -> PathBuf {
    get_config_dir().join(format!("frpc.{}.toml", id))
}

fn id_from_filename(filename: &str) -> Option<String> {
    filename
        .strip_suffix(".toml")
        .and_then(|s| s.strip_prefix("frpc."))
        .map(|s| s.to_string())
}

// ── TOML read (DOM API) ──

/// Parse # @title / # @enable / # @sort from raw file content
fn parse_meta_from_raw(content: &str) -> (Option<String>, Option<bool>, Option<i32>) {
    let mut title = None;
    let mut enable = None;
    let mut sort = None;

    // Only scan lines before the first non-comment, non-blank line (the TOML body)
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("# @title ") {
            title = Some(v.trim().to_string());
        } else if let Some(v) = trimmed.strip_prefix("# @enable ") {
            enable = Some(v.trim().parse().unwrap_or(true));
        } else if let Some(v) = trimmed.strip_prefix("# @sort ") {
            sort = v.trim().parse::<i32>().ok();
        } else if !trimmed.starts_with('#') {
            // Hit first non-comment line: stop scanning @ lines
            break;
        }
    }

    (title, enable, sort)
}

/// Extract metadata from V2 TOML keys (backward compat fallback)
fn extract_meta_from_keys(doc: &DocumentMut) -> (Option<String>, Option<bool>, Option<i32>) {
    let title = doc.get("title").and_then(|v| v.as_str()).map(String::from);
    let enable = doc.get("enable").and_then(|v| v.as_bool());
    let sort = doc.get("sort").and_then(|v| v.as_integer()).map(|i| i as i32);
    (title, enable, sort)
}

/// Extract array of strings from a TOML array value
fn extract_string_array(table: &Table, key: &str) -> Option<Vec<String>> {
    table.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect())
}

/// Extract proxy description from comment before [[proxies]]
fn extract_proxy_desc(table: &Table) -> Option<String> {
    table.decor().prefix()
        .and_then(|raw| raw.as_str())
        .and_then(|s| {
            s.lines()
                .find(|l| {
                    let t = l.trim();
                    t.starts_with('#') && !t.starts_with("# @")
                })
                .map(|l| {
                    l.trim()
                        .strip_prefix("# ")
                        .or_else(|| l.trim().strip_prefix('#'))
                        .unwrap_or("")
                        .trim()
                        .to_string()
                })
                .filter(|d| !d.is_empty())
        })
}

/// Extract proxies from a parsed TOML document
fn extract_proxies(doc: &DocumentMut) -> Option<Vec<ProxyItem>> {
    let arr = doc.get("proxies")?.as_array_of_tables()?;
    let mut proxies = Vec::new();

    for table in arr.iter() {
        let desc = extract_proxy_desc(table);
        let name = table.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let proxy_type = table.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let local_ip = table.get("localIP")
            .and_then(|v| v.as_str())
            .map(String::from);
        let local_port = table.get("localPort")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u16;
        let enabled = table.get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        proxies.push(ProxyItem {
            name,
            desc,
            enabled,
            proxy_type,
            local_ip,
            local_port,
            custom_domains: extract_string_array(table, "customDomains"),
            locations: extract_string_array(table, "locations"),
        });
    }

    if proxies.is_empty() { None } else { Some(proxies) }
}

/// Read a server file and return full config (frpc-native fields + tray metadata)
fn read_server_file(id: &str) -> Result<FrpcConfigFile, String> {
    let path = server_path(id);
    if !path.exists() {
        return Err(format!("服务器 '{}' 不存在", id));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // V3: extract metadata from # @ comments in raw content
    let (ctitle, cenable, csort) = parse_meta_from_raw(&content);

    // Parse TOML body
    let doc: DocumentMut = content.parse()
        .map_err(|e| format!("解析 TOML 失败: {}", e))?;

    // V2 fallback: extract metadata from TOML keys
    let (ktitle, kenable, ksort) = extract_meta_from_keys(&doc);
    let title = ctitle.or(ktitle).unwrap_or_default();
    let enable = cenable.or(kenable).unwrap_or(true);
    let sort = csort.or(ksort).unwrap_or(0);

    let server_addr = doc.get("serverAddr")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少 serverAddr".to_string())?
        .to_string();
    let server_port = doc.get("serverPort")
        .and_then(|v| v.as_integer())
        .ok_or_else(|| "缺少 serverPort".to_string())? as u16;

    let auth = doc.get("auth").and_then(|v| v.as_table()).map(|t| AuthConfig {
        method: t.get("method").and_then(|v| v.as_str()).map(String::from),
        token: t.get("token").and_then(|v| v.as_str()).map(String::from),
    });

    let log = doc.get("log").and_then(|v| v.as_table()).map(|t| LogConfig {
        to: t.get("to").and_then(|v| v.as_str()).map(String::from),
        level: t.get("level").and_then(|v| v.as_str()).map(String::from),
        max_days: t.get("maxDays").and_then(|v| v.as_integer()).map(|i| i as i32),
    });

    let proxies = extract_proxies(&doc);

    Ok(FrpcConfigFile {
        title,
        enable,
        sort,
        server_addr,
        server_port,
        auth,
        log,
        proxies,
    })
}

// ── TOML write (DOM API, minimally invasive) ──

/// Build an inline string array (e.g. ["a", "b"])
fn make_string_array(items: &[String]) -> Item {
    let mut arr = toml_edit::Array::new();
    for s in items {
        arr.push(s.as_str());
    }
    value(toml_edit::Value::Array(arr))
}

/// Update or insert a scalar field in a table, preserving Key decor for existing keys.
/// Uses get_mut() which accesses the Item via mutable reference into the BTreeMap,
/// so the Key (and its prefix decor: blank lines, comments) is untouched.
fn set_or_insert(table: &mut Table, key: &str, val: Item) {
    if let Some(item) = table.get_mut(key) {
        *item = val;
    } else {
        table.insert(key, val);
    }
}

/// Update a string-like field in a table (Some = set/update, None = remove)
fn set_or_remove_str(table: &mut Table, key: &str, val: Option<&str>) {
    match val {
        Some(v) => set_or_insert(table, key, value(v)),
        None => { table.remove(key); }
    }
}

/// Update frpc-native fields in-place, preserving unknown keys and formatting
fn update_server_fields(doc: &mut DocumentMut, config: &FrpcConfigFile) {
    // Simple scalar fields: update in-place, preserving decor
    set_or_insert(doc, "serverAddr", value(&config.server_addr));
    set_or_insert(doc, "serverPort", value(config.server_port as i64));

    // Auth: update in-place if exists, create if new, remove if clearing
    match &config.auth {
        Some(a) if a.method.is_some() || a.token.is_some() => {
            match doc.get_mut("auth") {
                Some(Item::Table(t)) => {
                    set_or_remove_str(t, "method", a.method.as_deref());
                    set_or_remove_str(t, "token", a.token.as_deref());
                }
                _ => {
                    doc.remove("auth");
                    let mut t = Table::new();
                    t.decor_mut().set_prefix("\n");
                    if let Some(ref m) = a.method { t.insert("method", value(m)); }
                    if let Some(ref tk) = a.token { t.insert("token", value(tk)); }
                    doc.insert("auth", Item::Table(t));
                }
            }
        }
        _ => { doc.remove("auth"); }
    }

    // Log: update in-place if exists, create if new, remove if clearing
    match &config.log {
        Some(l) if l.to.is_some() || l.level.is_some() || l.max_days.is_some() => {
            match doc.get_mut("log") {
                Some(Item::Table(t)) => {
                    set_or_remove_str(t, "to", l.to.as_deref());
                    set_or_remove_str(t, "level", l.level.as_deref());
                    match l.max_days {
                        Some(md) => set_or_insert(t, "maxDays", value(md as i64)),
                        None => { t.remove("maxDays"); }
                    }
                }
                _ => {
                    doc.remove("log");
                    let mut t = Table::new();
                    t.decor_mut().set_prefix("\n");
                    if let Some(ref to) = l.to { t.insert("to", value(to)); }
                    if let Some(ref level) = l.level { t.insert("level", value(level)); }
                    if let Some(md) = l.max_days { t.insert("maxDays", value(md as i64)); }
                    doc.insert("log", Item::Table(t));
                }
            }
        }
        _ => { doc.remove("log"); }
    }
}

/// Update a single proxy table in-place, preserving unknown fields.
fn update_proxy_table(table: &mut Table, proxy: &ProxyItem) {
    // Update desc comment: strip old plain-comment, keep # @ lines, prepend new desc
    let existing_prefix = table.decor().prefix()
        .and_then(|r| r.as_str())
        .unwrap_or("");
    let filtered: Vec<&str> = existing_prefix
        .lines()
        .filter(|l| {
            let t = l.trim();
            !(t.starts_with('#') && !t.starts_with("# @"))
        })
        .collect();
    let desc_prefix = match proxy.desc {
        Some(ref d) if !d.trim().is_empty() => format!("# {}\n", d.trim()),
        _ => String::new(),
    };
    let new_prefix = if filtered.iter().all(|l| l.trim().is_empty()) {
        desc_prefix
    } else {
        format!("{}{}", filtered.join("\n"), desc_prefix)
    };
    table.decor_mut().set_prefix(new_prefix);

    // Scalar fields — set_or_insert preserves Key decor on existing keys
    set_or_insert(table, "name", value(&proxy.name));
    set_or_insert(table, "type", value(&proxy.proxy_type));
    set_or_insert(table, "enabled", value(proxy.enabled));
    set_or_remove_str(table, "localIP", proxy.local_ip.as_deref());
    set_or_insert(table, "localPort", value(proxy.local_port as i64));

    // Array fields
    match &proxy.custom_domains {
        Some(domains) if !domains.is_empty() => set_or_insert(table, "customDomains", make_string_array(domains)),
        _ => { table.remove("customDomains"); }
    }
    match &proxy.locations {
        Some(locs) if !locs.is_empty() => set_or_insert(table, "locations", make_string_array(locs)),
        _ => { table.remove("locations"); }
    }
}

/// Create a new proxy table from a ProxyItem (no existing table to update)
fn build_proxy_table(proxy: &ProxyItem) -> Table {
    let mut table = Table::new();
    update_proxy_table(&mut table, proxy);
    table
}

/// Update [[proxies]] in-place: match existing by name, add new, remove deleted.
/// Preserves unknown fields in unchanged parts of each proxy table.
fn update_proxies(doc: &mut DocumentMut, config: &FrpcConfigFile) {
    match &config.proxies {
        Some(proxies) if !proxies.is_empty() => {
            // Get or create ArrayOfTables
            let arr: &mut ArrayOfTables = match doc.get_mut("proxies").and_then(|v| v.as_array_of_tables_mut()) {
                Some(arr) => arr,
                None => {
                    doc.remove("proxies");
                    doc.insert("proxies", Item::ArrayOfTables(ArrayOfTables::new()));
                    doc.get_mut("proxies").and_then(|v| v.as_array_of_tables_mut()).unwrap()
                }
            };

            // Remove proxies not in new config (reverse iteration for safe removal)
            let new_names: std::collections::HashSet<&str> = proxies.iter().map(|p| p.name.as_str()).collect();
            let mut i = 0;
            while i < arr.len() {
                let name = arr.get(i).and_then(|t| t.get("name").and_then(|v| v.as_str())).unwrap_or("");
                if !new_names.contains(name) {
                    arr.remove(i);
                } else {
                    i += 1;
                }
            }

            // Update existing, add new
            for proxy in proxies {
                let found = arr.iter_mut().any(|t| {
                    if t.get("name").and_then(|v| v.as_str()) == Some(proxy.name.as_str()) {
                        update_proxy_table(t, proxy);
                        true
                    } else {
                        false
                    }
                });
                if !found {
                    arr.push(build_proxy_table(proxy));
                }
            }
        }
        _ => { doc.remove("proxies"); }
    }
}

/// Update metadata comments (# @title / # @enable / # @sort) on the first key's decor.
/// In toml_edit, top-of-file comments are stored as the prefix of the first KEY
/// (before the key name), NOT the value (between = and value).
fn update_meta_comments(doc: &mut DocumentMut, config: &FrpcConfigFile) {
    let meta = format!(
        "# @title {}\n# @enable {}\n# @sort {}\n",
        config.title, config.enable, config.sort
    );

    // Set on the first KEY's prefix decor (top-of-file comments live before the key line)
    if let Some((mut key, _)) = doc.iter_mut().next() {
        let existing = key.leaf_decor().prefix()
            .and_then(|r| r.as_str())
            .unwrap_or("");

        let filtered: Vec<&str> = existing
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.starts_with("# @title ")
                    && !t.starts_with("# @enable ")
                    && !t.starts_with("# @sort ")
            })
            .collect();

        let new_prefix = if filtered.iter().all(|l| l.trim().is_empty()) {
            meta
        } else {
            format!("{}{}", filtered.join("\n"), meta)
        };

        key.leaf_decor_mut().set_prefix(new_prefix);
    }
}

/// Write a server config to file.
/// Parses existing file and makes targeted in-place edits to preserve:
/// - blank lines and formatting
/// - unknown TOML fields not modeled in our structs
/// - inline array format
fn write_server_file(id: &str, config: &FrpcConfigFile) -> Result<(), String> {
    let path = server_path(id);

    let mut doc = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        let mut doc: DocumentMut = content.parse()
            .map_err(|e| format!("解析 TOML 失败: {}", e))?;
        // Migrate: remove old V2 metadata keys
        doc.remove("title");
        doc.remove("enable");
        doc.remove("sort");
        doc
    } else {
        DocumentMut::new()
    };

    // Targeted in-place updates (order matters for formatting)
    update_meta_comments(&mut doc, config);
    update_server_fields(&mut doc, config);
    update_proxies(&mut doc, config);

    let output = doc.to_string();
    fs::write(&path, output)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// ── Server commands ──

#[tauri::command]
fn list_servers() -> Result<Vec<ServerInfo>, String> {
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
                });
            }
        }
    }

    servers.sort_by_key(|s| s.sort);
    Ok(servers)
}

#[tauri::command]
fn get_server(id: String) -> Result<ServerInfo, String> {
    let config = read_server_file(&id)?;
    Ok(ServerInfo {
        id,
        title: config.title,
        enable: config.enable,
        sort: config.sort,
        server_addr: config.server_addr,
        server_port: config.server_port,
        auth: config.auth,
    })
}

#[tauri::command]
fn create_server(input: CreateServerInput) -> Result<String, String> {
    let dir = get_config_dir();
    fs::create_dir_all(dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    // Auto-generate a single-letter id (a..z)
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
fn update_server(server: ServerInfo) -> Result<(), String> {
    let mut config = read_server_file(&server.id)?;
    config.title = server.title;
    config.enable = server.enable;
    config.sort = server.sort;
    config.server_addr = server.server_addr;
    config.server_port = server.server_port;
    config.auth = server.auth;
    write_server_file(&server.id, &config)
}

#[tauri::command]
fn delete_server(id: String) -> Result<(), String> {
    let path = server_path(&id);
    if !path.exists() {
        return Err(format!("服务器 '{}' 不存在", id));
    }
    fs::remove_file(&path)
        .map_err(|e| format!("删除文件失败: {}", e))
}

#[tauri::command]
fn reorder_servers(ids: Vec<String>) -> Result<(), String> {
    for (index, id) in ids.iter().enumerate() {
        let mut config = read_server_file(id)?;
        config.sort = index as i32 + 1;
        write_server_file(id, &config)?;
    }
    Ok(())
}

// ── Proxy commands ──

#[tauri::command]
fn list_proxies(server_id: String) -> Result<Vec<ProxyItem>, String> {
    let config = read_server_file(&server_id)?;
    Ok(config.proxies.unwrap_or_default())
}

#[tauri::command]
fn create_proxy(server_id: String, proxy: ProxyItem) -> Result<(), String> {
    let mut config = read_server_file(&server_id)?;
    let proxies = config.proxies.get_or_insert_with(Vec::new);
    if proxies.iter().any(|p| p.name == proxy.name) {
        return Err(format!("代理 '{}' 已存在", proxy.name));
    }
    proxies.push(proxy);
    write_server_file(&server_id, &config)
}

#[tauri::command]
fn update_proxy(server_id: String, old_name: String, proxy: ProxyItem) -> Result<(), String> {
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
fn delete_proxy(server_id: String, name: String) -> Result<(), String> {
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
fn reorder_proxies(server_id: String, names: Vec<String>) -> Result<(), String> {
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

// ── App entry ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir: PathBuf = if cfg!(target_os = "windows") {
                get_executable_dir().join("conf")
            } else {
                app.path().app_data_dir()
                    .unwrap_or_else(|_| get_executable_dir())
                    .join("conf")
            };
            fs::create_dir_all(&config_dir)
                .expect("无法创建配置目录");
            let _ = CONFIG_DIR.set(config_dir);

            let show = MenuItemBuilder::with_id("show", "显示主界面").build(app)?;
            let light = MenuItemBuilder::with_id("light", "轻量模式").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&light)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&quit)
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "light" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_servers,
            get_server,
            create_server,
            update_server,
            delete_server,
            reorder_servers,
            list_proxies,
            create_proxy,
            update_proxy,
            delete_proxy,
            reorder_proxies,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}