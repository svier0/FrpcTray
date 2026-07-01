# 后端联调看板
BACKEND_VERSION: V18
ACK_FRONTEND_VERSION: V10

## 📢 最新联调通知（V18）
- ✅ **修复：Windows 所有子进程弹出黑框** — 启动/停止 frpc、查版本、开机自启等全部加 `CREATE_NO_WINDOW`
  - 涉及 6 处 spawn：`start_frpc` / `stop_frpc` / `open_log_file` / `get_frpc_version` / 开机自启 / `scoop update`
  - 安装器启动保留窗口（正常行为）
