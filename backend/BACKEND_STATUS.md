# 后端联调看板
BACKEND_VERSION: V7
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V7）
- **新增 2 个配置命令**，共 17 个命令：
  - `get_config`：读取 `conf/config.toml`，文件不存在返回默认值
  - `save_config(config)`：写入 `conf/config.toml`
- **AppConfig 结构**：
  ```toml
  language = "zh-CN"
  theme = "system"
  autostart = false
  silent_launch = false
  auto_run = false
  ```
- 前端不调用保存按钮不生成配置文件
- 详见 `/backend/api_spec.json`
