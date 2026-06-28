# 后端联调看板
BACKEND_VERSION: V11
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V11）
- **错误消息智能摘要**：`summarize_frpc_error()` 模式匹配已知 frpc 错误并抽取变量
  - 固定摘要：`Login to server failed`、`Connection refused` 等
  - 动态提取：`Unknown config field "xxx"`、`Proxy name "xxx" already in use`、`Port 8080 already in use` 等
  - 所有可能的 error_message 值（含占位符 `{name}` `{port}` `{detail}` `{reason}`）已列在 `api_spec.json` 的 `events[0].errorCodes.values` 中
  - 未知错误保底显示原始行（截断 120 字符），无输出时为 `null`
