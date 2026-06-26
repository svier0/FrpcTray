# 后端 Agent 上下文记录

> 此文件仅供后端 Agent 恢复上下文，记录所有未在其他文件中保存的关键信息。

---

## 项目概述
- **项目名称**: frpc-tray
- **技术栈**: Tauri 2.0 + Rust
- **目录结构**: 前端在 `/frontend`, 后端在 `/backend`
- **项目目标**: frpc 桌面端配置管理工具，用户将可执行文件放入 frpc 同级目录使用

---

## 当前开发状态 (截至 2026-06-26)

### 已完成
- ✅ V1 TOML 文件管理 (9 个命令) → 已被 V2 替代
- ✅ V2 API：Server CRUD + reorder, Proxy CRUD + reorder (11 个命令)
- ✅ 数据模型完全匹配前端需求：`ServerInfo`（对应 `conf/frpc.{id}.toml`）、`ProxyItem`（对应 `[[proxies]]`）
- ✅ 字段与示例 `frpc.w.toml` 一致（camelCase，auth 嵌套）
- ✅ 接口规范已写入 `/backend/api_spec.json`（V2）
- ✅ BACKEND_STATUS.md V2 已发布
- ✅ 配置目录跨平台策略：Windows 用 exe 同级 conf/，macOS/Linux 用 app_data_dir()/conf/
- ✅ 移除 bundle.resources，目录运行时自动创建
- ✅ 前端反馈 V2 已处理：`create_server` 参数去掉 `id`，后端自动生成单字母 id 并返回

### 待办事项
- [ ] **TOML 存储方案重构**（不污染 frpc 配置 + 保留注释）
  - `toml` crate → `toml_edit`
  - `title`/`enable`/`sort` → 文件顶部 `# @title` / `# @enable` / `# @sort`
  - `desc` → `[[proxies]]` 上方注释，`toml_edit` 排序时跟随
- [ ] 等待前端确认 V3

### 下一步计划
- TOML 存储方案重构
- 等待前端 ACK V3

---

## 关键决策记录

### 2026-06-26 (TOML 存储方案)
- **决策**: 使用 `toml_edit` 替代 `toml` crate，tray 元数据存为注释
- **原因**: frpc-tray 不应污染 frpc 原生配置。当前 `title`/`enable`/`sort` 作为 TOML key 注入会污染配置，`toml` crate 读写丢失注释
- **方案**: 
  - 换 `toml_edit`（`DocumentMut` 保留注释和格式）
  - `title`/`enable`/`sort` 存为文件顶部 `# @title` / `# @enable` / `# @sort`
  - `desc` = `[[proxies]]` 上方注释，`toml_edit` 保证排序时跟随

### 2026-06-26 (之前)
- **决策**: 配置目录使用跨平台策略
- **原因**: 用户拒绝 Windows AppData 方案，要求 exe 同级目录
- **实现**: Windows 用 `get_executable_dir().join("conf")`，macOS/Linux 用 `app.path().app_data_dir().join("conf")`。通过 `OnceLock` 在 `setup()` 初始化，所有命令通过 `get_config_dir()` 读取

### 2026-06-25
- **决策**: TOML 管理功能使用 `#[tauri::command]` 暴露给前端
- **原因**: 用户要求不改前端，只实现 Rust 方法
- **实现**: 9 个命令已注册到 `invoke_handler`（V1，已替换）

---

## 技术细节备忘

### 文件路径规则
- 路径解析在 `setup()` 中完成，存入 `OnceLock<PathBuf>`
- Windows: `<exe_dir>/conf/`（exe 同级 conf 子目录）
- 其他平台: `app_data_dir()/conf/`
- 目录不存在时自动 `create_dir_all`
- 开发时样本 TOML 放 `target/debug/conf/`

### 数据结构 (V2)
- `ServerInfo`: id, title, enable, sort, serverAddr, serverPort, auth? {method?, token?}
- `ProxyItem`: name, desc?, type, localIP?, localPort, customDomains?, locations?
- `FrpcConfigFile`: 内部 TOML 结构（含 log、proxies），仅 ServerInfo 暴露给前端
- 所有命令返回 `Result<T, String>` 格式

### 关键命令说明
- `update_proxy` 带 `old_name` 参数，支持代理重命名
- `reorder_servers(ids)` 按数组顺序分配 sort=1..N，遍历更新每个文件
- `reorder_proxies(server_id, names)` 重排 `[[proxies]]` 数组顺序
- `update_server` 保留原有 log/proxies，只覆盖 server 级字段

---

## 协作状态
- **当前版本**: V2
- **前端 ACK**: 未确认 (需检查 FRONTEND_STATUS.md)
- **阻塞点**: 无

---

## 下次启动检查清单
1. 读取 `FRONTEND_STATUS.md` 检查 ACK 状态
2. 如果前端已 ACK V2，可以开展下一步开发
3. 如果前端有反馈/bug 报告，优先处理
4. 如无新需求，继续等待或开发新功能
