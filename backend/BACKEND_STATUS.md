# 后端联调看板
BACKEND_VERSION: V10
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V10）
- **错误消息精简**：进程退出时不再展示 `exit code`，只提取冒号前摘要
  - `login to the server failed: dial tcp ...` → `"login to the server failed"`
  - 无输出时 `error_message` 为 `null`，前端自行显示通用提示
