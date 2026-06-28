# 后端联调看板
BACKEND_VERSION: V11
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V11）
- **错误消息模式匹配**：已知 frpc 错误映射为简洁英文摘要
  - 所有可能的 error_message 值已列在 `api_spec.json` 的 `events[0].errorCodes.values` 中，前端可据此做 i18n 翻译
  - 未知错误保底显示原始行（截断 120 字符）
  - 无输出时 `error_message` 为 `null`
