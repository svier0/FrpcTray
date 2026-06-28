# 后端联调看板
BACKEND_VERSION: V9
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V9）
- **frpc 运行管理已实现**：
  - `start_frpc(server_id)`：启动单个服务器的 frpc（使用 `frpc -c conf/frpc.{id}.toml`）
  - `stop_frpc(server_id)`：停止单个服务器的 frpc
  - `start_all_frpc()`：一键启动所有激活服务器
  - `stop_all_frpc()`：一键停止所有运行中进程
  - `get_all_frpc_status()`：获取所有服务器运行状态
- **进程监控**：后台 tokio task 监听进程退出，自动发送 `frpc-status-changed` 事件
- **状态值**：`running` / `stopped` / `error`
- 共 22 个命令，详见 `/backend/api_spec.json`
