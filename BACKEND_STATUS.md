# 后端联调看板
VERSION: V3
ACK_FRONTEND_VERSION: V2

## 📢 最新联调通知
- ✅ 已处理前端反馈 V2：`create_server` 参数去掉 `id`，后端自动生成单字母 id 并返回（2026-06-26）
- V2：引入 `ServerInfo`（对应 `conf/frpc.{id}.toml`）和 `ProxyItem`（对应 `[[proxies]]`）
- **11 个命令**：Server CRUD + reorder，Proxy CRUD + reorder
- 字段名与 TOML 完全一致（camelCase），API 见 `/backend/api_spec.json`
