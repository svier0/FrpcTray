# Rust TOML 配置管理功能说明

## 功能概述

已为 frpc-tray 应用添加了 TOML 配置文件管理功能，可以在同级目录下读取和修改 TOML 配置文件。

## 新增的 Rust 方法

### 1. 获取同级目录下的 TOML 文件列表
```rust
#[tauri::command]
fn list_toml_files() -> Result<Vec<String>, String>
```
**功能**: 列出可执行文件同级目录下所有 .toml 文件
**返回**: 文件名列表或错误信息

### 2. 读取指定的 TOML 配置文件
```rust
#[tauri::command]
fn read_toml_file(filename: String) -> Result<String, String>
```
**功能**: 读取指定 TOML 文件的内容
**参数**: filename - 文件名
**返回**: 文件内容或错误信息

### 3. 写入 TOML 配置文件
```rust
#[tauri::command]
fn write_toml_file(filename: String, content: String) -> Result<(), String>
```
**功能**: 写入内容到指定 TOML 文件
**参数**: 
- filename - 文件名
- content - 文件内容
**返回**: 成功或错误信息

### 4. 在指定 TOML 文件中添加代理配置
```rust
#[tauri::command]
fn add_proxy_to_toml(filename: String, proxy: ProxyConfig) -> Result<(), String>
```
**功能**: 在指定的 TOML 文件中添加新的代理配置
**参数**:
- filename - 文件名
- proxy - 代理配置对象
**返回**: 成功或错误信息

### 5. 更新指定 TOML 文件中的代理配置
```rust
#[tauri::command]
fn update_proxy_in_toml(filename: String, proxy_id: String, updates: ProxyConfig) -> Result<(), String>
```
**功能**: 更新指定 TOML 文件中的代理配置
**参数**:
- filename - 文件名
- proxy_id - 要更新的代理 ID
- updates - 更新的代理配置
**返回**: 成功或错误信息

### 6. 删除指定 TOML 文件中的代理配置
```rust
#[tauri::command]
fn delete_proxy_from_toml(filename: String, proxy_id: String) -> Result<(), String>
```
**功能**: 删除指定 TOML 文件中的代理配置
**参数**:
- filename - 文件名
- proxy_id - 要删除的代理 ID
**返回**: 成功或错误信息

### 7. 获取指定 TOML 文件中的所有代理配置
```rust
#[tauri::command]
fn get_proxies_from_toml(filename: String) -> Result<Vec<ProxyConfig>, String>
```
**功能**: 获取指定 TOML 文件中的所有代理配置
**参数**: filename - 文件名
**返回**: 代理配置列表或错误信息

## 数据结构

### ProxyConfig
```rust
pub struct ProxyConfig {
    pub id: String,        // 代理唯一标识
    pub name: String,      // 代理名称
    pub url: String,       // 代理 URL
    pub enabled: bool,     // 是否启用
    pub icon: Option<String>, // 图标（可选）
    pub color: Option<String>, // 颜色（可选）
}
```

### FrpcConfig
```rust
pub struct FrpcConfig {
    pub server_addr: Option<String>,
    pub server_port: Option<u16>,
    pub token: Option<String>,
    pub log: Option<LogConfig>,
    pub http_proxy: Option<HttpProxyConfig>,
    pub https_proxy: Option<HttpsProxyConfig>,
    pub proxies: Option<Vec<ProxyConfig>>,
}
```

## 使用示例

### 前端调用示例

```javascript
// 获取 TOML 文件列表
const tomlFiles = await invoke('list_toml_files')

// 读取 TOML 文件
const content = await invoke('read_toml_file', { filename: 'frpc.toml' })

// 添加代理配置
const proxy = {
  id: 'proxy1',
  name: 'Web Service',
  url: 'http://example.com',
  enabled: true,
  icon: 'web',
  color: '#3B82F6'
}
await invoke('add_proxy_to_toml', {
  filename: 'frpc.toml',
  proxy: proxy
})

// 获取所有代理
const proxies = await invoke('get_proxies_from_toml', { filename: 'frpc.toml' })

// 更新代理
await invoke('update_proxy_in_toml', {
  filename: 'frpc.toml',
  proxy_id: 'proxy1',
  updates: { enabled: false }
})

// 删除代理
await invoke('delete_proxy_from_toml', {
  filename: 'frpc.toml',
  proxy_id: 'proxy1'
})
```

## 配置文件位置

程序会自动查找可执行文件同级目录下的 .toml 文件，例如：
- Windows: `C:\path\to\frpc\frpc.toml`
- macOS: `/Applications/frpc/frpc.toml`
- Linux: `/usr/local/bin/frpc/frpc.toml`

## 注意事项

1. 确保程序有读写同级目录文件的权限
2. TOML 文件格式必须正确，否则会解析失败
3. 代理配置会被添加到 `[[proxies]]` 数组中
4. 每个代理必须有唯一的 ID
5. 修改后的配置会立即写入文件

## 示例配置文件

参考 `example.toml` 文件，了解 frpc 配置文件的格式和结构。