# 后端联调看板
VERSION: V2
ACK_FRONTEND_VERSION: V1

## 📢 最新联调通知
- 已根据前端反馈 V1 接口不匹配，重写为 V2（2026-06-26）
- **数据模型重构**：引入 `ServerInfo`（对应 `conf/frpc.{id}.toml`）和 `ProxyItem`（对应 `[[proxies]]`）
- **11 个命令**：Server CRUD + reorder，Proxy CRUD + reorder
- 字段名与 TOML 完全一致（camelCase），API 见 `/backend/api_spec.json`
