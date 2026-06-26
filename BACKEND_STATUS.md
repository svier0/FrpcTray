# 后端联调看板
VERSION: V3
ACK_FRONTEND_VERSION: V3

## 📢 最新联调通知
- ✅ V3 TOML 存储方案重构完成（2026-06-26）
  - `toml` crate → `toml_edit` 0.22（纯 DOM API）
  - `title`/`enable`/`sort` → 文件顶部 `# @title` / `# @enable` / `# @sort` 注释
  - `desc` → `[[proxies]]` 上方注释
  - V2 文件自动迁移：读取兼容注释+key 双路径，下次写入自动升级为 V3
  - 11 个命令 API 完全不变，前端无需修改
- ✅ 已处理前端反馈 V2：`create_server` 参数去掉 `id`，后端自动生成单字母 id 并返回（2026-06-26）
- V2：引入 `ServerInfo`（对应 `conf/frpc.{id}.toml`）和 `ProxyItem`（对应 `[[proxies]]`）
- **11 个命令**：Server CRUD + reorder，Proxy CRUD + reorder
- 字段名与 TOML 完全一致（camelCase），API 见 `backend/api_spec.json`

## 前端留言
- **需要 ProxyItem 添加 enabled 字段**：代理项需要一个 enabled 字段来控制启用/禁用状态，用于在 UI 上显示开关
