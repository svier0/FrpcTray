use std::path::PathBuf;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static BIN_DIR: OnceLock<PathBuf> = OnceLock::new();
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| reqwest::Client::new())
}

pub fn get_executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn get_config_dir() -> &'static PathBuf {
    CONFIG_DIR.get().expect("get_config_dir called before setup")
}

pub fn get_bin_dir() -> &'static PathBuf {
    BIN_DIR.get().expect("get_bin_dir called before setup")
}

pub fn set_config_dir(path: PathBuf) {
    let _ = CONFIG_DIR.set(path);
}

pub fn set_bin_dir(path: PathBuf) {
    let _ = BIN_DIR.set(path);
}

pub fn server_path(id: &str) -> PathBuf {
    get_config_dir().join(format!("frpc.{}.toml", id))
}

pub fn id_from_filename(filename: &str) -> Option<String> {
    filename
        .strip_suffix(".toml")
        .and_then(|s| s.strip_prefix("frpc."))
        .map(|s| s.to_string())
}

// ── TOML file structures ──

#[derive(Debug)]
pub struct AuthConfig {
    pub method: Option<String>,
    pub token: Option<String>,
}

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

#[derive(Debug)]
pub struct LogConfig {
    pub to: Option<String>,
    pub level: Option<String>,
    pub max_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransportConfig {
    pub protocol: Option<String>,
    #[serde(rename = "tcpMux")]
    pub tcp_mux: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub silent_launch: bool,
    #[serde(default)]
    pub auto_run: bool,
    #[serde(default)]
    pub use_github_proxy: bool,
}

fn default_language() -> String {
    "zh-CN".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            theme: default_theme(),
            autostart: false,
            silent_launch: false,
            auto_run: false,
            use_github_proxy: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub can_upgrade: bool,
    pub download_url: String,
}

pub fn read_app_config() -> AppConfig {
    let dir = get_config_dir();
    let path = dir.join("config.toml");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrpcVersionInfo {
    pub current_version: String,
    pub latest_version: String,
    pub can_upgrade: bool,
    pub platform: String,
    pub arch: String,
}

#[derive(Debug)]
pub struct FrpcConfigFile {
    pub title: String,
    pub enable: bool,
    pub sort: i32,
    pub server_addr: String,
    pub server_port: u16,
    pub auth: Option<AuthConfig>,
    pub log: Option<LogConfig>,
    pub transport: Option<TransportConfig>,
    pub tls: Option<TlsConfig>,
    pub proxies: Option<Vec<ProxyItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyItem {
    pub name: String,
    pub desc: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "type")]
    pub proxy_type: String,
    #[serde(rename = "localIP")]
    pub local_ip: Option<String>,
    #[serde(rename = "localPort")]
    pub local_port: u16,
    #[serde(rename = "remotePort")]
    pub remote_port: Option<u16>,
    #[serde(rename = "customDomains")]
    pub custom_domains: Option<Vec<String>>,
    pub locations: Option<Vec<String>>,
}

fn default_true() -> bool { true }

// ── API structures ──

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub id: String,
    pub title: String,
    pub enable: bool,
    pub sort: i32,
    #[serde(rename = "serverAddr")]
    pub server_addr: String,
    #[serde(rename = "serverPort")]
    pub server_port: u16,
    pub auth: Option<AuthConfig>,
    pub transport: Option<TransportConfig>,
    pub tls: Option<TlsConfig>,
}

// ── Frpc running status ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrpcRunningStatus {
    pub server_id: String,
    pub status: String,
    pub pid: Option<u32>,
    pub error_message: Option<String>,
}

#[allow(dead_code)]
impl FrpcRunningStatus {
    pub fn stopped(server_id: &str) -> Self {
        Self {
            server_id: server_id.to_string(),
            status: "stopped".to_string(),
            pid: None,
            error_message: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrpcStatusEvent {
    pub server_id: String,
    pub old_status: String,
    pub new_status: String,
    pub pid: Option<u32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateServerInput {
    pub title: String,
    pub enable: bool,
    pub sort: i32,
    #[serde(rename = "serverAddr")]
    pub server_addr: String,
    #[serde(rename = "serverPort")]
    pub server_port: u16,
    pub auth: Option<AuthConfig>,
    pub transport: Option<TransportConfig>,
    pub tls: Option<TlsConfig>,
}
