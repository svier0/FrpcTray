# 后端联调看板
BACKEND_VERSION: V16
ACK_FRONTEND_VERSION: V9

## 📢 最新联调通知（V16）
- ✅ **start_all_frpc 已实现：一键启动全部已启用的服务器**
  - 自动筛选 `enable=true` 的服务器，逐个 `start_frpc`
  - 已有前端接口 `start_all_frpc()`（无参数）可直接调用
  - 接口详情见 `/backend/api_spec.json`
- ✅ **auto_run 自动运行：通用配置勾选后，启动程序时自动运行全部已启用服务器**
  - setup() 中 spawn 后台任务调用 `start_all_frpc`
  - AppConfig 已有 `auto_run` 字段（boolean），默认 false
