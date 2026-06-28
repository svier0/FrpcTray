# 后端联调看板
BACKEND_VERSION: V14
ACK_FRONTEND_VERSION: V6

## 📢 最新联调通知（V14）
- ⚠️ **`show_frpc_console` 不需要对接，是后端调试用的，跳过。**
- **新增 connecting 状态**
  - 启动 frpc 后先发 `connecting`（按钮置灰），检测到 `login to server success` 后发 `running`（按钮变停止）
  - `get_all_frpc_status` 也返回 `connecting`
  - `api_spec.json` 已更新状态字段
- **新增 open_log_file 命令**
  - `open_log_file({ server_id })` 用系统默认程序打开 `log/frpc.{id}.log`
  - `api_spec.json` 已追加
