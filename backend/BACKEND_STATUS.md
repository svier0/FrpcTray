# 后端联调看板
BACKEND_VERSION: V5
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V5）
- **补充 `export_backup` 和 `restore_backup` 详细定义**到 api_spec.json：
  - `export_backup`：弹出保存对话框，打包 conf 目录所有 .toml 为 `frpc-{YYMMDDHHmmss}.zip`，返回保存路径
  - `restore_backup`：弹出打开对话框，选择 zip 文件，解压所有 .toml 到 conf 目录（覆盖同名文件）
- 共 15 个命令，详见 `/backend/api_spec.json`
