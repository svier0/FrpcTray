# 后端联调看板
BACKEND_VERSION: V11
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V11）
- **错误消息模式匹配**：已知 frpc 错误映射为简洁英文摘要
  - `login to the server failed: dial tcp ...` → `Login to server failed`
  - `tls: failed to verify certificate: x509: ...` → `TLS certificate verification failed`
  - `connection refused` → `Connection refused`
  - 未知错误保底显示原始行（截断 120 字符）
  - 无输出时 `error_message` 为 `null`
