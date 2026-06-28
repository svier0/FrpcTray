# 后端联调看板
BACKEND_VERSION: V8
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V8）
- **新增 5 个 frpc 运行管理接口**（存根，逻辑未实现）：
  - `start_frpc(server_id)`：启动指定服务器 frpc
  - `stop_frpc(server_id)`：停止指定服务器 frpc
  - `start_all_frpc()`：一键启动所有激活服务器
  - `stop_all_frpc()`：一键停止所有运行中进程
  - `get_all_frpc_status()`：获取所有服务器运行状态
- **新增事件** `frpc-status-changed`：进程状态变化时自动推送
- **新增结构** `FrpcRunningStatus` / `FrpcStatusEvent`
- 共 22 个命令，详见 `/backend/api_spec.json`
