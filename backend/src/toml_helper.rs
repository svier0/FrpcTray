use std::fs;
use toml_edit::{DocumentMut, Item, Table, value};
use crate::config::*;

/// Parse # @title / # @enable / # @sort from raw file content
pub fn parse_meta_from_raw(content: &str) -> (Option<String>, Option<bool>, Option<i32>) {
    let mut title = None;
    let mut enable = None;
    let mut sort = None;

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
            break;
        }
    }

    (title, enable, sort)
}

pub fn extract_meta_from_keys(doc: &DocumentMut) -> (Option<String>, Option<bool>, Option<i32>) {
    let title = doc.get("title").and_then(|v| v.as_str()).map(String::from);
    let enable = doc.get("enable").and_then(|v| v.as_bool());
    let sort = doc.get("sort").and_then(|v| v.as_integer()).map(|i| i as i32);
    (title, enable, sort)
}

pub fn extract_string_array(table: &Table, key: &str) -> Option<Vec<String>> {
    table.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect())
}

pub fn extract_proxy_desc(table: &Table) -> Option<String> {
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

pub fn extract_proxies(doc: &DocumentMut) -> Option<Vec<ProxyItem>> {
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

pub fn read_server_file(id: &str) -> Result<FrpcConfigFile, String> {
    let path = server_path(id);
    if !path.exists() {
        return Err(format!("服务器 '{}' 不存在", id));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let (ctitle, cenable, csort) = parse_meta_from_raw(&content);

    let doc: DocumentMut = content.parse()
        .map_err(|e| format!("解析 TOML 失败: {}", e))?;

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

pub fn serialize_proxies(proxies: &[ProxyItem]) -> String {
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

pub fn strip_proxies_section(toml: &str) -> String {
    let mut result = String::new();
    let mut in_proxies = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[proxies]]" {
            in_proxies = true;
            continue;
        }
        if in_proxies {
            if trimmed.starts_with("[[") || trimmed.starts_with('[') {
                in_proxies = false;
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

pub fn set_or_insert(table: &mut Table, key: &str, val: Item) {
    if let Some(item) = table.get_mut(key) {
        *item = val;
    } else {
        table.insert(key, val);
    }
}

pub fn set_or_remove_str(table: &mut Table, key: &str, val: Option<&str>) {
    match val {
        Some(v) => set_or_insert(table, key, value(v)),
        None => { table.remove(key); }
    }
}

pub fn update_meta_comments(doc: &mut DocumentMut, config: &FrpcConfigFile) {
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

pub fn update_server_fields(doc: &mut DocumentMut, config: &FrpcConfigFile) {
    set_or_insert(doc, "serverAddr", value(&config.server_addr));
    set_or_insert(doc, "serverPort", value(config.server_port as i64));

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

pub fn write_server_file(id: &str, config: &FrpcConfigFile) -> Result<(), String> {
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
    doc.retain(|key, _| key != "proxies");

    let mut output = doc.to_string();
    output = strip_proxies_section(&output);
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
