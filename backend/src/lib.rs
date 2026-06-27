use std::sync::OnceLock;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Table, value};

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

static BIN_DIR: OnceLock<PathBuf> = OnceLock::new();

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| reqwest::Client::new())
}

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

#[derive(Debug, Serialize, Deserialize)]
struct TransportConfig {
    protocol: Option<String>,
    #[serde(rename = "tcpMux")]
    tcp_mux: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TlsConfig {
    enable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrpcVersionInfo {
    current_version: String,
    latest_version: String,
    can_upgrade: bool,
    platform: String,
    arch: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
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
    transport: Option<TransportConfig>,
    tls: Option<TlsConfig>,
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
    #[serde(rename = "remotePort")]
    remote_port: Option<u16>,
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
    transport: Option<TransportConfig>,
    tls: Option<TlsConfig>,
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
    transport: Option<TransportConfig>,
    tls: Option<TlsConfig>,
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

fn get_bin_dir() -> &'static PathBuf {
    BIN_DIR.get().expect("get_bin_dir called before setup")
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
        let remote_port = table.get("remotePort")
            .and_then(|v| v.as_integer())
            .map(|p| p as u16);

        proxies.push(ProxyItem {
            name,
            desc,
            enabled,
            proxy_type,
            local_ip,
            local_port,
            remote_port,
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

    let transport = doc.get("transport").and_then(|v| v.as_table()).map(|t| TransportConfig {
        protocol: t.get("protocol").and_then(|v| v.as_str()).map(String::from),
        tcp_mux: t.get("tcpMux").and_then(|v| v.as_bool()),
    });

    let tls = doc.get("tls").and_then(|v| v.as_table()).map(|t| TlsConfig {
        enable: t.get("enable").and_then(|v| v.as_bool()),
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
        transport,
        tls,
        proxies,
    })
}

/// Write [[proxies]] as TOML string
fn serialize_proxies(proxies: &[ProxyItem]) -> String {
    let mut out = String::new();
    for (i, proxy) in proxies.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if let Some(ref d) = proxy.desc {
            if !d.trim().is_empty() {
                out.push_str(&format!("# {}\n", d.trim()));
            }
        }
        out.push_str("[[proxies]]\n");
        out.push_str(&format!("name = \"{}\"\n", proxy.name));
        out.push_str(&format!("type = \"{}\"\n", proxy.proxy_type));
        out.push_str(&format!("enabled = {}\n", proxy.enabled));
        if let Some(ref ip) = proxy.local_ip {
            out.push_str(&format!("localIP = \"{}\"\n", ip));
        }
        out.push_str(&format!("localPort = {}\n", proxy.local_port));
        if let Some(ref rp) = proxy.remote_port {
            out.push_str(&format!("remotePort = {}\n", rp));
        }
        if let Some(ref domains) = proxy.custom_domains {
            if !domains.is_empty() {
                out.push_str(&format!("customDomains = [\"{}\"]\n", domains.join("\", \"")));
            }
        }
        if let Some(ref locs) = proxy.locations {
            if !locs.is_empty() {
                out.push_str(&format!("locations = [\"{}\"]\n", locs.join("\", \"")));
            }
        }
    }
    out
}

/// Remove all [[proxies]] blocks from the TOML string.
/// Finds lines starting with [[proxies]] and removes them plus all following
/// lines that are part of the same table (indented or until next [[...]] or [...])
fn strip_proxies_section(toml: &str) -> String {
    let mut result = String::new();
    let mut in_proxies = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[proxies]]" {
            in_proxies = true;
            continue;
        }
        if in_proxies {
            // End of a [[proxies]] block: next [[...]] or [...] header, or blank line
            // followed by a new section, or EOF
            if trimmed.starts_with("[[") || trimmed.starts_with('[') {
                in_proxies = false;
                result.push_str(line);
                result.push('\n');
            }
            // else: skip this line (it's part of a [[proxies]] block)
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

// ── Helper functions for TOML write ──
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

/// Update metadata comments (# @title / # @enable / # @sort) on the first key's decor.
fn update_meta_comments(doc: &mut DocumentMut, config: &FrpcConfigFile) {
    let meta = format!(
        "# @title {}\n# @enable {}\n# @sort {}\n",
        config.title, config.enable, config.sort
    );
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

    // Transport: update in-place if exists, create if new, remove if clearing
    match &config.transport {
        Some(t) if t.protocol.is_some() || t.tcp_mux.is_some() => {
            match doc.get_mut("transport") {
                Some(Item::Table(tbl)) => {
                    set_or_remove_str(tbl, "protocol", t.protocol.as_deref());
                    match t.tcp_mux {
                        Some(mux) => set_or_insert(tbl, "tcpMux", value(mux)),
                        None => { tbl.remove("tcpMux"); }
                    }
                }
                _ => {
                    doc.remove("transport");
                    let mut tbl = Table::new();
                    tbl.decor_mut().set_prefix("\n");
                    if let Some(ref p) = t.protocol { tbl.insert("protocol", value(p)); }
                    if let Some(mux) = t.tcp_mux { tbl.insert("tcpMux", value(mux)); }
                    doc.insert("transport", Item::Table(tbl));
                }
            }
        }
        _ => { doc.remove("transport"); }
    }

    // Tls: update in-place if exists, create if new, remove if clearing
    match &config.tls {
        Some(t) if t.enable.is_some() => {
            match doc.get_mut("tls") {
                Some(Item::Table(tbl)) => {
                    match t.enable {
                        Some(en) => set_or_insert(tbl, "enable", value(en)),
                        None => { tbl.remove("enable"); }
                    }
                }
                _ => {
                    doc.remove("tls");
                    let mut tbl = Table::new();
                    tbl.decor_mut().set_prefix("\n");
                    if let Some(en) = t.enable { tbl.insert("enable", value(en)); }
                    doc.insert("tls", Item::Table(tbl));
                }
            }
        }
        _ => { doc.remove("tls"); }
    }
}

/// Write a server config to file.
/// Uses toml_edit for server-level fields (preserves formatting),
/// string manipulation for [[proxies]] (bypasses toml_edit serialization bug).
fn write_server_file(id: &str, config: &FrpcConfigFile) -> Result<(), String> {
    let path = server_path(id);

    let mut doc = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        let mut doc: DocumentMut = content.parse()
            .map_err(|e| format!("解析 TOML 失败: {}", e))?;
        doc.remove("title");
        doc.remove("enable");
        doc.remove("sort");
        doc
    } else {
        DocumentMut::new()
    };

    update_meta_comments(&mut doc, config);
    update_server_fields(&mut doc, config);
    // Remove old proxies from doc (key-value index)
    doc.retain(|key, _| key != "proxies");

    let mut output = doc.to_string();
    // Strip any lingering [[proxies]] blocks that to_string() may have included
    // (they're in the ordered entry list, not the key-value index)
    output = strip_proxies_section(&output);
    // Append new proxies
    if let Some(ref proxies) = config.proxies {
        if !proxies.is_empty() {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push('\n');
            output.push_str(&serialize_proxies(proxies));
        }
    }
    output.push('\n');

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
        transport: config.transport,
        tls: config.tls,
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
fn update_server(server: ServerInfo) -> Result<(), String> {
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

// ── Frpc version commands ──

#[tauri::command]
async fn get_frpc_version() -> Result<FrpcVersionInfo, String> {
    let current_version = get_current_frpc_version();
    let (latest_version, can_upgrade) = match get_latest_frpc_version().await {
        Ok(v) => {
            let can = compare_versions(&current_version, &v);
            (v, can)
        }
        Err(_) => (String::new(), false),
    };

    Ok(FrpcVersionInfo {
        current_version,
        latest_version,
        can_upgrade,
        platform: get_platform(),
        arch: get_arch(),
    })
}

/// Get current frpc version by running frpc from bin directory
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
            // Parse version from output like "frpc v0.69.1"
            stdout
                .lines()
                .find_map(|line| line.trim().strip_prefix("frpc v").map(String::from))
                .unwrap_or_else(|| "0".to_string())
        }
        _ => "0".to_string(),
    }
}

/// Get latest frpc version from GitHub releases API (with proxy fallback)
async fn get_latest_frpc_version() -> Result<String, String> {
    let client = get_http_client();
    let urls = [
        "https://api.github.com/repos/fatedier/frp/releases/latest",
        "https://gh-proxy.com/https://api.github.com/repos/fatedier/frp/releases/latest",
    ];

    let mut errors = Vec::new();
    for url in &urls {
        match fetch_github_release(client, url).await {
            Ok(version) => return Ok(version),
            Err(e) => errors.push(e),
        }
    }

    Err(format!("获取最新版本失败: {}", errors.join("; ")))
}

async fn fetch_github_release(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let timeout = if url.contains("gh-proxy.com") { 20 } else { 10 };
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

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("{} - 读取响应失败: {}", url, e))?;

    let release: GithubRelease = serde_json::from_slice(&body)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    // Strip 'v' prefix from tag_name
    Ok(release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name).to_string())
}

/// Compare versions, return true if latest > current
/// Handles formats like "0.69.1", "0.2.0-preview", "0.69.2"
fn compare_versions(current: &str, latest: &str) -> bool {
    if current == "0" {
        return true; // Not installed, can upgrade
    }

    // Parse version parts (ignore preview suffix)
    fn parse_version(v: &str) -> (u32, u32, u32) {
        let main = v.split('-').next().unwrap_or(v);
        let parts: Vec<&str> = main.split('.').collect();
        let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    }

    let (c_major, c_minor, c_patch) = parse_version(current);
    let (l_major, l_minor, l_patch) = parse_version(latest);

    (l_major, l_minor, l_patch) > (c_major, c_minor, c_patch)
}

/// Get platform name for download URL
fn get_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else {
        "linux".to_string()
    }
}

/// Get architecture name for download URL
fn get_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "amd64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "arm64".to_string()
    } else if cfg!(target_arch = "x86") {
        "386".to_string()
    } else if cfg!(target_arch = "arm") {
        "arm".to_string()
    } else {
        "amd64".to_string() // default
    }
}

// ── Frpc upgrade command ──

async fn download_zip(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let timeout = if url.contains("gh-proxy.com") || url.contains("ghfast.top") { 60 } else { 30 };
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
async fn upgrade_frpc(version: String) -> Result<(), String> {
    let platform = get_platform();
    let arch = get_arch();
    let path = format!("v{version}/frp_{version}_{platform}_{arch}.zip", version = version, platform = platform, arch = arch);
    let urls = [
        format!("https://github.com/fatedier/frp/releases/download/{}", path),
        format!("https://gh-proxy.com/https://github.com/fatedier/frp/releases/download/{}", path),
        format!("https://ghfast.top/https://github.com/fatedier/frp/releases/download/{}", path),
    ];

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

    // Extract zip from memory
    let cursor = std::io::Cursor::new(body.to_vec());
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("解压失败: {}", e))?;

    let frpc_name = if cfg!(target_os = "windows") { "frpc.exe" } else { "frpc" };
    let bin_dir = get_bin_dir();
    fs::create_dir_all(bin_dir)
        .map_err(|e| format!("创建 bin 目录失败: {}", e))?;
    let dest_path = bin_dir.join(frpc_name);

    // Find and extract frpc binary
    let mut found = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {}", e))?;
        let entry_name = entry.name().to_string();

        if entry_name.ends_with(frpc_name) && entry_name.contains("frp_") {
            // Write to temp file first, then rename (atomic on same filesystem)
            let temp_path = bin_dir.join(format!("{}.tmp", frpc_name));
            {
                let mut temp_file = fs::File::create(&temp_path)
                    .map_err(|e| format!("创建临时文件失败: {}", e))?;
                std::io::copy(&mut entry, &mut temp_file)
                    .map_err(|e| format!("写入临时文件失败: {}", e))?;
            }

            // Set executable permission on Unix
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

            let bin_dir: PathBuf = if cfg!(target_os = "windows") {
                get_executable_dir().join("bin")
            } else {
                app.path().app_data_dir()
                    .unwrap_or_else(|_| get_executable_dir())
                    .join("bin")
            };
            fs::create_dir_all(&bin_dir)
                .expect("无法创建 bin 目录");
            let _ = BIN_DIR.set(bin_dir);

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
            get_frpc_version,
            upgrade_frpc,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proxy(name: &str, port: u16) -> ProxyItem {
        ProxyItem {
            name: name.into(),
            desc: None,
            enabled: true,
            proxy_type: "http".into(),
            local_ip: None,
            local_port: port,
            remote_port: None,
            custom_domains: None,
            locations: None,
        }
    }

    #[test]
    fn test_update_proxies_reorder() {
        // Build a TOML doc with proxies in order A, B, C
        let doc_str = r#"
serverAddr = "1.2.3.4"
serverPort = 7000

[[proxies]]
name = "A"
type = "http"
localPort = 80

[[proxies]]
name = "B"
type = "http"
localPort = 81

[[proxies]]
name = "C"
type = "http"
localPort = 82
"#;
        let doc: DocumentMut = doc_str.parse().unwrap();

        // Simulate write_server_file: after toml_edit, strip old proxies, append new
        let mut output = doc.to_string();
        output = strip_proxies_section(&output);
        if !output.ends_with('\n') { output.push('\n'); }
        output.push('\n');
        output.push_str(&serialize_proxies(&[
            make_proxy("C", 82),
            make_proxy("A", 80),
            make_proxy("B", 81),
        ]));
        output.push('\n');

        println!("OUTPUT:\n{}", output);

        // Extract proxy names in order
        let names: Vec<&str> = output
            .lines()
            .filter(|l| l.trim().starts_with("name = "))
            .map(|l| l.trim().trim_start_matches("name = \"").trim_end_matches('"'))
            .collect();

        assert_eq!(names, vec!["C", "A", "B"], "Expected C,A,B but got {:?}", names);
    }
}