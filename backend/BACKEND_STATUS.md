# 后端联调看板
BACKEND_VERSION: V12
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V12）
- **错误消息智能摘要**（最终方案）：`summarize_frpc_error()` 模式匹配 + 变量抽取
  - 固定摘要：`Login to server failed`、`Connection refused` 等
  - 动态提取（保留变量）：`Unknown config field "{name}"`、`Proxy name "{name}" already in use`、`Port {port} already in use`、`Config parse error: {detail}` 等
  - 完整列表含占位符见 `api_spec.json` → `events[0].errorCodes.values`
  - 未知错误保底显示原始行（截断 120 字符），无输出时为 `null`

- ⚠️ **更新注意**：V11 描述有误（之前说变量错误走原始行回退），实际是**抽取变量**。前端 i18n 翻译请以 `api_spec.json` 的 errorCodes 为准，带 `{xxx}` 的占位符不变。
